#![cfg(test)]

use soroban_sdk::testutils::{Address as _, Ledger};
use soroban_sdk::Symbol;
use soroban_sdk::{symbol_short, Address, Bytes, BytesN, Env};
use stellar_nebula_nomad::{
    LimitOrder, NebulaLayout, NebulaNomadContract, NebulaNomadContractClient, OrderSide,
};

fn setup() -> (Env, NebulaNomadContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    // Keep the SDK's default protocol version; only override what the tests
    // depend on.
    env.ledger().with_mut(|li| {
        li.sequence_number = 100;
        li.timestamp = 1_700_000_000;
        li.min_temp_entry_ttl = 100;
        li.min_persistent_entry_ttl = 1000;
        li.max_entry_ttl = 10_000;
    });
    let contract_id = env.register(NebulaNomadContract, ());
    let client = NebulaNomadContractClient::new(&env, &contract_id);
    let player = Address::generate(&env);
    (env, client, player)
}

/// Mint a ship and generate a layout for testing.
fn mint_and_layout(
    env: &Env,
    client: &NebulaNomadContractClient,
    player: &Address,
) -> (u64, NebulaLayout) {
    let metadata = Bytes::from_slice(env, &[0u8; 4]);
    let ship = client.mint_ship(player, &symbol_short!("explorer"), &metadata);
    let seed = BytesN::from_array(env, &[42u8; 32]);
    let layout = client.generate_nebula_layout(&seed, player);
    (ship.id, layout)
}

#[test]
fn test_harvest_and_list_creates_offer() {
    let (env, client, player) = setup();
    let (ship_id, layout) = mint_and_layout(&env, &client, &player);

    let resource = symbol_short!("dust");
    let (harvest, offer) = client.harvest_and_list(&player, &ship_id, &layout, &resource, &100i128);
    assert!(harvest.total_harvested > 0);
    assert!(offer.active);
    assert_eq!(offer.asset_id, resource);
    assert_eq!(offer.min_price, 100);
}

#[test]
fn test_cancel_listing_deactivates_offer() {
    let (env, client, player) = setup();
    let (ship_id, layout) = mint_and_layout(&env, &client, &player);

    let resource = symbol_short!("dust");
    let (_, offer) = client.harvest_and_list(&player, &ship_id, &layout, &resource, &50i128);

    let cancelled = client.cancel_listing(&player, &offer.offer_id);
    assert!(!cancelled.active);
}

#[test]
fn test_cancel_listing_by_non_seller_fails() {
    let (env, client, player) = setup();
    let (ship_id, layout) = mint_and_layout(&env, &client, &player);

    let resource = symbol_short!("dust");
    let (_, offer) = client.harvest_and_list(&player, &ship_id, &layout, &resource, &50i128);
    assert!(
        offer.amount > 0,
        "escrow must be non-empty for this test to mean anything"
    );

    // An unrelated address must not be able to cancel the seller's live offer:
    // the escrow refund is credited to the caller, so allowing this would hand
    // the seller's escrowed units to anyone who noticed the offer.
    let attacker = Address::generate(&env);
    let result = client.try_cancel_listing(&attacker, &offer.offer_id);
    assert!(
        result.is_err(),
        "non-seller must not be able to cancel the offer"
    );

    // The offer must survive the rejected attempt and still be cancellable by
    // its actual seller.
    let cancelled = client.cancel_listing(&player, &offer.offer_id);
    assert!(!cancelled.active);
}

#[test]
fn test_cancel_already_cancelled_fails() {
    let (env, client, player) = setup();
    let (ship_id, layout) = mint_and_layout(&env, &client, &player);

    let resource = symbol_short!("dust");
    let (_, offer) = client.harvest_and_list(&player, &ship_id, &layout, &resource, &50i128);

    // First cancel succeeds
    client.cancel_listing(&player, &offer.offer_id);
    // Second cancel should fail
    let result = client.try_cancel_listing(&player, &offer.offer_id);
    assert!(result.is_err());
}

#[test]
fn test_harvest_and_list_invalid_price_fails() {
    let (env, client, player) = setup();
    let (ship_id, layout) = mint_and_layout(&env, &client, &player);

    let resource = symbol_short!("dust");
    let result = client.try_harvest_and_list(&player, &ship_id, &layout, &resource, &0i128);
    assert!(result.is_err());
}

#[test]
fn test_burst_limit_enforced() {
    let (env, client, player) = setup();
    let metadata = Bytes::from_slice(&env, &[0u8; 4]);
    let resource = symbol_short!("dust");

    // Create 5 ships, each with its own layout.
    //
    // `generate_nebula_layout` is itself rate-limited to 5 calls / 60 s per
    // address, which would trip at the same iteration count as the DEX cap this
    // test targets. Step past the rate-limit window on each round so the only
    // limit in play is `MAX_LISTINGS_PER_SESSION`.
    for i in 0..5u8 {
        let ship = client.mint_ship(&player, &symbol_short!("explorer"), &metadata);
        let mut seed_bytes = [0u8; 32];
        seed_bytes[0] = i + 1;
        let seed = BytesN::from_array(&env, &seed_bytes);
        let layout = client.generate_nebula_layout(&seed, &player);
        // Should succeed (try_ returns Result)
        let result = client.try_harvest_and_list(&player, &ship.id, &layout, &resource, &10i128);
        assert!(result.is_ok(), "Listing {} should succeed", i);

        env.ledger().with_mut(|li| li.timestamp += 61);
    }

    // 6th listing should fail due to session limit
    let ship6 = client.mint_ship(&player, &symbol_short!("explorer"), &metadata);
    let seed6 = BytesN::from_array(&env, &[99u8; 32]);
    let layout6 = client.generate_nebula_layout(&seed6, &player);
    let result = client.try_harvest_and_list(&player, &ship6.id, &layout6, &resource, &10i128);
    assert!(result.is_err());
}

// ── Full trading cycle (Issue #442) ────────────────────────────────────────

const DUST: Symbol = symbol_short!("dust");

/// Mint a ship for `player`, harvest one layout into their balance, and
/// return the resulting dust balance.
fn fund_with_dust(env: &Env, client: &NebulaNomadContractClient, player: &Address) -> u32 {
    let (ship_id, layout) = mint_and_layout(env, client, player);
    client.harvest_resources(&ship_id, &layout);
    let balance = client.get_resource_balance(player, &DUST);
    assert!(balance >= 4, "fixture layout must yield some dust");
    balance
}

fn buy_order(trader: &Address, quantity: i128, limit_price: i128) -> LimitOrder {
    LimitOrder {
        id: 0,
        trader: trader.clone(),
        side: OrderSide::Buy,
        resource: DUST,
        quantity,
        limit_price,
        placed_at: 0,
        is_stop_loss: false,
    }
}

#[test]
fn test_list_resource_escrows_exact_amount() {
    let (env, client, seller) = setup();
    let balance = fund_with_dust(&env, &client, &seller);

    let offer = client.list_resource(&seller, &DUST, &3, &20i128);
    assert!(offer.active);
    assert_eq!(offer.amount, 3);
    assert_eq!(offer.min_price, 20);
    assert_eq!(client.get_resource_balance(&seller, &DUST), balance - 3);
    assert_eq!(client.get_dex_offer(&offer.offer_id), Some(offer));
}

#[test]
fn test_list_resource_rejects_bad_input() {
    let (env, client, seller) = setup();
    let balance = fund_with_dust(&env, &client, &seller);

    assert!(client
        .try_list_resource(&seller, &DUST, &0, &20i128)
        .is_err());
    assert!(client
        .try_list_resource(&seller, &DUST, &1, &0i128)
        .is_err());
    assert!(client
        .try_list_resource(&seller, &DUST, &(balance + 1), &20i128)
        .is_err());
    // Nothing was escrowed by the failed attempts.
    assert_eq!(client.get_resource_balance(&seller, &DUST), balance);
}

#[test]
fn test_buy_offer_partial_then_full_fill() {
    let (env, client, seller) = setup();
    fund_with_dust(&env, &client, &seller);
    let buyer = Address::generate(&env);

    let offer = client.list_resource(&seller, &DUST, &4, &25i128);

    let first = client.buy_offer(&buyer, &offer.offer_id, &1, &25i128);
    assert_eq!(first.amount, 1);
    assert_eq!(first.price, 25);
    assert_eq!(first.total_cost, 25);
    assert_eq!(first.remaining, 3);
    assert_eq!(first.seller, seller);
    assert_eq!(client.get_resource_balance(&buyer, &DUST), 1);
    assert!(client.get_dex_offer(&offer.offer_id).unwrap().active);

    let second = client.buy_offer(&buyer, &offer.offer_id, &3, &30i128);
    assert_eq!(second.total_cost, 75);
    assert_eq!(second.remaining, 0);
    assert_eq!(client.get_resource_balance(&buyer, &DUST), 4);

    let closed = client.get_dex_offer(&offer.offer_id).unwrap();
    assert!(!closed.active);
    assert_eq!(closed.amount, 0);
    // A filled offer can be neither bought from nor cancelled.
    assert!(client
        .try_buy_offer(&buyer, &offer.offer_id, &1, &25i128)
        .is_err());
    assert!(client.try_cancel_listing(&seller, &offer.offer_id).is_err());

    // Both fills are recorded for price charts.
    let history = client.get_trading_history();
    assert_eq!(history.len(), 2);
    assert_eq!(history.get(1).unwrap().quantity, 3);
    assert_eq!(history.get(1).unwrap().trader, buyer);
}

#[test]
fn test_buy_offer_guards() {
    let (env, client, seller) = setup();
    fund_with_dust(&env, &client, &seller);
    let buyer = Address::generate(&env);
    let offer = client.list_resource(&seller, &DUST, &2, &50i128);

    // Price above the buyer's limit.
    assert!(client
        .try_buy_offer(&buyer, &offer.offer_id, &1, &49i128)
        .is_err());
    // More than the offer holds, or nothing at all.
    assert!(client
        .try_buy_offer(&buyer, &offer.offer_id, &3, &50i128)
        .is_err());
    assert!(client
        .try_buy_offer(&buyer, &offer.offer_id, &0, &50i128)
        .is_err());
    // Seller cannot buy their own offer.
    assert!(client
        .try_buy_offer(&seller, &offer.offer_id, &1, &50i128)
        .is_err());
    // Unknown offer.
    assert!(client.try_buy_offer(&buyer, &9_999, &1, &50i128).is_err());

    // None of the rejected attempts moved anything.
    assert_eq!(client.get_resource_balance(&buyer, &DUST), 0);
    assert_eq!(client.get_dex_offer(&offer.offer_id).unwrap().amount, 2);
}

#[test]
fn test_cancel_after_partial_fill_refunds_remainder() {
    let (env, client, seller) = setup();
    let balance = fund_with_dust(&env, &client, &seller);
    let buyer = Address::generate(&env);

    let offer = client.list_resource(&seller, &DUST, &4, &10i128);
    client.buy_offer(&buyer, &offer.offer_id, &1, &10i128);
    client.cancel_listing(&seller, &offer.offer_id);

    // Seller gets back the 3 unsold units; the sold one stays with the buyer.
    assert_eq!(client.get_resource_balance(&seller, &DUST), balance - 1);
    assert_eq!(client.get_resource_balance(&buyer, &DUST), 1);
}

#[test]
fn test_closed_listings_free_slots() {
    let (env, client, seller) = setup();
    fund_with_dust(&env, &client, &seller);
    let buyer = Address::generate(&env);

    let mut ids = soroban_sdk::Vec::<u64>::new(&env);
    for _ in 0..5 {
        ids.push_back(client.list_resource(&seller, &DUST, &1, &10i128).offer_id);
    }
    // Cap reached.
    assert!(client
        .try_list_resource(&seller, &DUST, &1, &10i128)
        .is_err());

    // A cancellation frees a slot…
    client.cancel_listing(&seller, &ids.get(0).unwrap());
    client.list_resource(&seller, &DUST, &1, &10i128);
    assert!(client
        .try_list_resource(&seller, &DUST, &1, &10i128)
        .is_err());

    // …and so does a complete fill.
    client.buy_offer(&buyer, &ids.get(1).unwrap(), &1, &10i128);
    client.list_resource(&seller, &DUST, &1, &10i128);
}

#[test]
fn test_sell_to_buy_order() {
    let (env, client, seller) = setup();
    let balance = fund_with_dust(&env, &client, &seller);
    let bidder = Address::generate(&env);

    let order_id = client.place_limit_order(&bidder, &buy_order(&bidder, 3, 40));

    let fill = client.sell_to_order(&seller, &order_id, &DUST, &2, &35i128);
    assert_eq!(fill.buyer, bidder);
    assert_eq!(fill.seller, seller);
    assert_eq!(fill.price, 40);
    assert_eq!(fill.total_cost, 80);
    assert_eq!(fill.remaining, 1);
    assert_eq!(client.get_resource_balance(&seller, &DUST), balance - 2);
    assert_eq!(client.get_resource_balance(&bidder, &DUST), 2);
    assert_eq!(client.get_limit_order(&order_id).unwrap().quantity, 1);

    // Filling the rest removes the order from the book and the trader's list.
    client.sell_to_order(&seller, &order_id, &DUST, &1, &40i128);
    assert!(client.get_limit_order(&order_id).is_none());
    assert_eq!(client.get_trader_orders(&bidder).len(), 0);
    assert_eq!(client.get_trading_history().len(), 2);
}

#[test]
fn test_sell_to_order_guards() {
    let (env, client, seller) = setup();
    let balance = fund_with_dust(&env, &client, &seller);
    let bidder = Address::generate(&env);
    let order_id = client.place_limit_order(&bidder, &buy_order(&bidder, 2, 40));

    // Order pays less than the seller will accept.
    assert!(client
        .try_sell_to_order(&seller, &order_id, &DUST, &1, &41i128)
        .is_err());
    // Wrong resource, too much, nothing.
    assert!(client
        .try_sell_to_order(&seller, &order_id, &symbol_short!("drmatt"), &1, &40i128)
        .is_err());
    assert!(client
        .try_sell_to_order(&seller, &order_id, &DUST, &3, &40i128)
        .is_err());
    assert!(client
        .try_sell_to_order(&seller, &order_id, &DUST, &0, &40i128)
        .is_err());
    // A trader cannot fill their own order.
    assert!(client
        .try_sell_to_order(&bidder, &order_id, &DUST, &1, &40i128)
        .is_err());

    // Sell-side orders cannot be sold into.
    let mut ask = buy_order(&bidder, 2, 40);
    ask.side = OrderSide::Sell;
    let ask_id = client.place_limit_order(&bidder, &ask);
    assert!(client
        .try_sell_to_order(&seller, &ask_id, &DUST, &1, &40i128)
        .is_err());

    // Seller without enough balance.
    let broke = Address::generate(&env);
    assert!(client
        .try_sell_to_order(&broke, &order_id, &DUST, &1, &40i128)
        .is_err());

    assert_eq!(client.get_resource_balance(&seller, &DUST), balance);
    assert_eq!(client.get_limit_order(&order_id).unwrap().quantity, 2);
}

#[test]
fn test_get_open_offers_pages_and_filters() {
    let (env, client, seller) = setup();
    fund_with_dust(&env, &client, &seller);
    let buyer = Address::generate(&env);

    let a = client.list_resource(&seller, &DUST, &1, &10i128);
    let b = client.list_resource(&seller, &DUST, &1, &11i128);
    let c = client.list_resource(&seller, &DUST, &1, &12i128);
    // Filled offers drop out of the book.
    client.buy_offer(&buyer, &b.offer_id, &1, &11i128);

    let first = client.get_open_offers(&Some(DUST), &0, &1);
    assert_eq!(first.offers.len(), 1);
    assert_eq!(first.offers.get(0).unwrap(), a);
    let cursor = first.next_cursor.expect("more offers remain");

    let rest = client.get_open_offers(&Some(DUST), &cursor, &10);
    assert_eq!(rest.offers.len(), 1);
    assert_eq!(rest.offers.get(0).unwrap(), c);
    assert_eq!(rest.next_cursor, None);

    // Filtering by a resource nobody listed returns nothing.
    let none = client.get_open_offers(&Some(symbol_short!("drmatt")), &0, &10);
    assert_eq!(none.offers.len(), 0);
    // No filter returns every active offer.
    assert_eq!(client.get_open_offers(&None, &0, &10).offers.len(), 2);
}
