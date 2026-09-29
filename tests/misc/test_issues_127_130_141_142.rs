//! Integration tests for Issues #127, #130, #141, #142
//!
//! Note: Soroban contract client methods that return `Result<T,E>` are
//! invoked directly — the test env panics on `Err`. Methods returning `T`
//! directly are called and compared without `.unwrap()`.
#![cfg(test)]

use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{symbol_short, Address, Bytes, BytesN, Env};
use stellar_nebula_nomad::{NebulaNomadContract, NebulaNomadContractClient};

// ── Shared setup ──────────────────────────────────────────────────────────────

fn setup() -> (Env, NebulaNomadContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set(LedgerInfo {
        protocol_version: 22,
        sequence_number: 100,
        timestamp: 1_700_000_000,
        network_id: [0u8; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 100,
        min_persistent_entry_ttl: 1000,
        max_entry_ttl: 10_000,
    });
    let contract_id = env.register(NebulaNomadContract, ());
    let client = NebulaNomadContractClient::new(&env, &contract_id);
    (env, client)
}

fn mint_ship(env: &Env, client: &NebulaNomadContractClient, owner: &Address) -> u64 {
    let metadata = Bytes::from_slice(env, &[0u8; 4]);
    client
        .mint_ship(owner, &symbol_short!("explorer"), &metadata)
        .id
}

// ── #127: Tiered referral rewards ─────────────────────────────────────────────

#[test]
fn test_generate_referral_code_returns_8_bytes() {
    let (env, client) = setup();
    let referrer = Address::generate(&env);
    let code: BytesN<8> = client.generate_referral_code(&referrer);
    assert_eq!(code.len(), 8);
}

#[test]
fn test_generate_referral_code_idempotent() {
    let (env, client) = setup();
    let referrer = Address::generate(&env);
    let code1: BytesN<8> = client.generate_referral_code(&referrer);
    let code2: BytesN<8> = client.generate_referral_code(&referrer);
    assert_eq!(code1, code2);
}

#[test]
fn test_get_referrer_by_code_after_generate() {
    let (env, client) = setup();
    let referrer = Address::generate(&env);
    let code: BytesN<8> = client.generate_referral_code(&referrer);
    // Code maps back to the referrer
    let resolved = client.get_referrer_by_code(&code);
    assert_eq!(resolved, Some(referrer));
}

#[test]
fn test_get_referrer_stats_none_before_activity() {
    let (env, client) = setup();
    let referrer = Address::generate(&env);
    let stats = client.get_referrer_stats(&referrer);
    assert!(stats.is_none());
}

#[test]
fn test_referral_analytics_zero_at_start() {
    let (_env, client) = setup();
    let analytics = client.get_referral_analytics();
    assert_eq!(analytics.total_referrals, 0);
}

#[test]
fn test_referral_leaderboard_starts_empty() {
    let (_env, client) = setup();
    let board = client.get_referral_leaderboard();
    assert_eq!(board.len(), 0);
}

// ── #130: NFT marketplace ─────────────────────────────────────────────────────

#[test]
fn test_list_ship_creates_listing() {
    let (env, client) = setup();
    let seller = Address::generate(&env);
    let ship_id = mint_ship(&env, &client, &seller);
    client.list_ship_for_sale(&seller, &ship_id, &1000);
    let listing = client.get_ship_listing(&ship_id);
    assert!(listing.is_some());
    let l = listing.unwrap();
    assert_eq!(l.ship_id, ship_id);
    assert_eq!(l.price, 1000);
    assert_eq!(l.seller, seller);
}

#[test]
#[should_panic]
fn test_list_ship_twice_panics() {
    let (env, client) = setup();
    let seller = Address::generate(&env);
    let ship_id = mint_ship(&env, &client, &seller);
    client.list_ship_for_sale(&seller, &ship_id, &500);
    client.list_ship_for_sale(&seller, &ship_id, &600); // should panic
}

#[test]
fn test_cancel_listing_removes_it() {
    let (env, client) = setup();
    let seller = Address::generate(&env);
    let ship_id = mint_ship(&env, &client, &seller);
    client.list_ship_for_sale(&seller, &ship_id, &800);
    client.cancel_ship_listing(&seller, &ship_id);
    assert!(client.get_ship_listing(&ship_id).is_none());
}

#[test]
fn test_buy_ship_removes_listing() {
    let (env, client) = setup();
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);
    let ship_id = mint_ship(&env, &client, &seller);
    client.list_ship_for_sale(&seller, &ship_id, &2000);
    client.buy_ship(&buyer, &ship_id);
    assert!(client.get_ship_listing(&ship_id).is_none());
}

#[test]
#[should_panic]
fn test_buy_unlisted_ship_panics() {
    let (env, client) = setup();
    let buyer = Address::generate(&env);
    client.buy_ship(&buyer, &9999);
}

#[test]
#[should_panic]
fn test_list_ship_zero_price_panics() {
    let (env, client) = setup();
    let seller = Address::generate(&env);
    let ship_id = mint_ship(&env, &client, &seller);
    client.list_ship_for_sale(&seller, &ship_id, &0);
}

#[test]
fn test_get_ship_listing_none_when_not_listed() {
    let (env, client) = setup();
    assert!(client.get_ship_listing(&42).is_none());
}

// ── #141: Limit orders and trading history ────────────────────────────────────

use stellar_nebula_nomad::{LimitOrder, OrderSide, TradeRecord};

fn make_buy_order(env: &Env, trader: &Address) -> LimitOrder {
    LimitOrder {
        id: 0,
        trader: trader.clone(),
        side: OrderSide::Buy,
        resource: symbol_short!("iron"),
        quantity: 100,
        limit_price: 50,
        placed_at: 0,
        is_stop_loss: false,
    }
}

#[test]
fn test_place_limit_order_returns_nonzero_id() {
    let (env, client) = setup();
    let trader = Address::generate(&env);
    let order = make_buy_order(&env, &trader);
    let id: u64 = client.place_limit_order(&trader, &order);
    assert!(id > 0);
}

#[test]
fn test_get_limit_order_after_placement() {
    let (env, client) = setup();
    let trader = Address::generate(&env);
    let order = make_buy_order(&env, &trader);
    let id: u64 = client.place_limit_order(&trader, &order);
    let fetched = client.get_limit_order(&id).unwrap();
    assert_eq!(fetched.quantity, 100);
    assert_eq!(fetched.limit_price, 50);
}

#[test]
fn test_cancel_limit_order_removes_it() {
    let (env, client) = setup();
    let trader = Address::generate(&env);
    let order = make_buy_order(&env, &trader);
    let id: u64 = client.place_limit_order(&trader, &order);
    client.cancel_limit_order(&trader, &id);
    assert!(client.get_limit_order(&id).is_none());
}

#[test]
fn test_get_trader_orders_returns_placed_orders() {
    let (env, client) = setup();
    let trader = Address::generate(&env);
    let o1 = make_buy_order(&env, &trader);
    let mut o2 = make_buy_order(&env, &trader);
    o2.quantity = 200;
    client.place_limit_order(&trader, &o1);
    client.place_limit_order(&trader, &o2);
    let orders = client.get_trader_orders(&trader);
    assert_eq!(orders.len(), 2);
}

#[test]
#[should_panic]
fn test_place_order_zero_price_panics() {
    let (env, client) = setup();
    let trader = Address::generate(&env);
    let mut order = make_buy_order(&env, &trader);
    order.limit_price = 0;
    client.place_limit_order(&trader, &order);
}

#[test]
#[should_panic]
fn test_place_order_zero_quantity_panics() {
    let (env, client) = setup();
    let trader = Address::generate(&env);
    let mut order = make_buy_order(&env, &trader);
    order.quantity = 0;
    client.place_limit_order(&trader, &order);
}

#[test]
fn test_record_trade_appears_in_history() {
    let (env, client) = setup();
    let trader = Address::generate(&env);
    let order = make_buy_order(&env, &trader);
    let id: u64 = client.place_limit_order(&trader, &order);
    let trade = TradeRecord {
        order_id: id,
        trader: trader.clone(),
        side: OrderSide::Buy,
        resource: symbol_short!("iron"),
        quantity: 100,
        price: 50,
        executed_at: 1_700_000_001,
    };
    client.record_trade(&trader, &trade);
    let history = client.get_trading_history();
    assert_eq!(history.len(), 1);
    assert_eq!(history.get(0).unwrap().order_id, id);
}

#[test]
fn test_trading_history_empty_at_start() {
    let (_env, client) = setup();
    let history = client.get_trading_history();
    assert_eq!(history.len(), 0);
}

// ── #142: Event indexer registration and trigger ─────────────────────────────

#[test]
fn test_register_indexer_does_not_panic() {
    let (env, client) = setup();
    let caller = Address::generate(&env);
    // should not panic
    client.register_indexer(&caller, &symbol_short!("nebula"));
}

#[test]
fn test_emit_indexer_event_does_not_panic() {
    let (env, client) = setup();
    let full_payload: BytesN<256> = BytesN::from_array(&env, &[2u8; 256]);
    client.emit_indexer_event(&symbol_short!("scan"), &full_payload);
}
