//! Decentralized-exchange harvest and swap integration.
//!
//! The harvest and offer types live in [`crate::resource_minter`] — this module
//! owns the trading paths on top of them, and is re-exported here so callers
//! can keep importing a single module:
//!
//! - **List**: [`harvest_and_list`], [`list_at_market`] and [`list_resource`]
//!   escrow resources from the seller's balance into a [`DexOffer`].
//! - **Buy**: [`buy_offer`] fills all or part of an offer at its listed price.
//! - **Sell**: [`sell_to_order`] fills all or part of a buy-side
//!   [`crate::trading::LimitOrder`] at the order's limit price.
//! - **Browse**: [`get_open_offers`] pages through active offers.
//! - **Cancel**: [`cancel_listing`] returns the unsold escrow.
//!
//! Settlement follows the marketplace convention (see
//! [`crate::nft_marketplace`]): resource units move on-chain, while the price
//! and total cost are recorded in the trading history and published in the
//! `dex/filled` event for the payment rail to settle.

use soroban_sdk::{contracterror, contracttype, symbol_short, Address, Env, Symbol, Vec};

use crate::reentrancy_guard::with_guard;
use crate::resource_minter::{
    credit_resource_balance, dex_offer_count, get_dex_offer, harvest_resources_unguarded,
    next_dex_offer_id, resource_balance, ResourceKey,
};
use crate::trading::{fill_limit_order, push_trade_record, OrderSide, TradeRecord, TradingError};

// Re-exported so callers can depend on this module alone.
pub use crate::resource_minter::{DexOffer, HarvestError, HarvestResult};

/// Cap on open listings a single player may hold, to bound offer-spam.
/// A slot frees up when a listing is cancelled or completely filled.
const MAX_LISTINGS_PER_SESSION: u32 = 5;

/// Maximum offers returned by one [`get_open_offers`] page.
pub const MAX_OFFER_PAGE: u32 = 50;

/// Maximum offer IDs examined by one [`get_open_offers`] call, so a page over
/// a book full of filled/cancelled offers still has a bounded cost.
pub const MAX_OFFER_SCAN: u32 = 200;

#[contracttype]
#[derive(Clone)]
pub enum DexKey {
    /// Number of open listings held by a player.
    SessionListings(Address),
}

/// Result of a [`buy_offer`] or [`sell_to_order`] fill.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DexFill {
    /// Offer ID for [`buy_offer`], limit-order ID for [`sell_to_order`].
    pub id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub asset_id: Symbol,
    /// Units that changed hands.
    pub amount: u32,
    /// Price per unit.
    pub price: i128,
    /// `amount * price`, owed by the buyer to the seller.
    pub total_cost: i128,
    /// Units left on the offer/order after this fill.
    pub remaining: i128,
}

/// One page of [`get_open_offers`].
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OfferPage {
    pub offers: Vec<DexOffer>,
    /// Pass as `start_after` to continue; `None` once the book is exhausted.
    pub next_cursor: Option<u64>,
}

/// Harvest resources from a layout and immediately list one asset on the DEX.
///
/// Combines [`crate::resource_minter::harvest_resources`] with DEX offer
/// creation in a single call: the caller avoids paying for two transactions, and the listed amount is
/// exactly what the harvest yielded (never more, so an offer can never be
/// oversold against the seller's balance).
///
/// Limited to [`MAX_LISTINGS_PER_SESSION`] open listings per player.
///
/// # Errors
/// - [`HarvestError::InvalidPrice`] if `min_price <= 0`.
/// - [`HarvestError::DexFailure`] if the player already hit the listing cap.
/// - [`HarvestError::AssetNotHarvested`] if `resource` was not in the harvest.
/// - plus any error from [`crate::resource_minter::harvest_resources`].
///
/// # Reentrancy
/// Runs under the global reentrancy guard (Issue #472): every check and state
/// effect completes while the lock is held, following checks-effects-
/// interactions, so a nested call into any guarded entry point while this one
/// is in flight is rejected with a `Reentrancy` error.
pub fn harvest_and_list(
    env: &Env,
    player: &Address,
    ship_id: u64,
    layout: &crate::nebula_explorer::NebulaLayout,
    resource: &Symbol,
    min_price: i128,
) -> Result<(HarvestResult, DexOffer), HarvestError> {
    with_guard(env, || {
        harvest_and_list_unguarded(env, player, ship_id, layout, resource, min_price)
    })
}

/// Unguarded body of [`harvest_and_list`]; callers must already hold the
/// reentrancy lock (e.g. another guarded entry point composing it).
fn harvest_and_list_unguarded(
    env: &Env,
    player: &Address,
    ship_id: u64,
    layout: &crate::nebula_explorer::NebulaLayout,
    resource: &Symbol,
    min_price: i128,
) -> Result<(HarvestResult, DexOffer), HarvestError> {
    player.require_auth();

    if min_price <= 0 {
        return Err(HarvestError::InvalidPrice);
    }

    ensure_listing_slot(env, player)?;

    // Composes the unguarded harvest: this function already holds the lock,
    // and re-acquiring it would reject our own nested call.
    let harvest_result = harvest_resources_unguarded(env, ship_id, layout)?;

    let mut listed_amount: u32 = 0;
    for i in 0..harvest_result.resources.len() {
        if let Some(hr) = harvest_result.resources.get(i) {
            if hr.asset_id == *resource {
                listed_amount = listed_amount
                    .checked_add(hr.amount)
                    .ok_or(HarvestError::PriceOverflow)?;
            }
        }
    }

    if listed_amount == 0 {
        return Err(HarvestError::AssetNotHarvested);
    }

    // The harvest already credited `player`, so escrow exactly the portion being
    // sold and leave the remainder liquid.
    let offer = escrow_and_list(env, player, resource, listed_amount, min_price)?;

    Ok((harvest_result, offer))
}

/// List `amount` units of `resource` from the seller's existing balance.
///
/// Unlike [`crate::resource_minter::auto_list_on_dex`], which escrows the
/// whole balance, this lists an exact amount and leaves the rest liquid.
/// Counts toward the [`MAX_LISTINGS_PER_SESSION`] open-listing cap.
///
/// # Errors
/// - [`HarvestError::InvalidPrice`] if `min_price <= 0`.
/// - [`HarvestError::InvalidAmount`] if `amount == 0`.
/// - [`HarvestError::InsufficientBalance`] if the seller holds less than `amount`.
/// - [`HarvestError::DexFailure`] if the seller already has the maximum
///   number of open listings.
///
/// # Reentrancy
/// Runs under the global reentrancy guard (Issue #472).
pub fn list_resource(
    env: &Env,
    seller: &Address,
    resource: &Symbol,
    amount: u32,
    min_price: i128,
) -> Result<DexOffer, HarvestError> {
    with_guard(env, || {
        seller.require_auth();
        if min_price <= 0 {
            return Err(HarvestError::InvalidPrice);
        }
        if amount == 0 {
            return Err(HarvestError::InvalidAmount);
        }
        ensure_listing_slot(env, seller)?;
        escrow_and_list(env, seller, resource, amount, min_price)
    })
}

/// Reject the listing if `seller` already holds the maximum open listings.
fn ensure_listing_slot(env: &Env, seller: &Address) -> Result<(), HarvestError> {
    if open_listings(env, seller) >= MAX_LISTINGS_PER_SESSION {
        return Err(HarvestError::DexFailure);
    }
    Ok(())
}

fn open_listings(env: &Env, seller: &Address) -> u32 {
    env.storage()
        .instance()
        .get(&DexKey::SessionListings(seller.clone()))
        .unwrap_or(0)
}

/// Occupy one of `seller`'s listing slots. Also called by
/// [`crate::resource_minter::auto_list_on_dex`], which is not capped but must
/// still be counted so that cancelling its offer frees a slot it really held.
pub(crate) fn note_listing_opened(env: &Env, seller: &Address) {
    env.storage().instance().set(
        &DexKey::SessionListings(seller.clone()),
        &open_listings(env, seller).saturating_add(1),
    );
}

/// Free one of `seller`'s listing slots (offer cancelled or fully filled).
fn release_listing_slot(env: &Env, seller: &Address) {
    let open = open_listings(env, seller);
    env.storage().instance().set(
        &DexKey::SessionListings(seller.clone()),
        &open.saturating_sub(1),
    );
}

/// Move `amount` from the seller's balance into escrow and create the offer.
/// Callers have already authenticated and validated price and listing cap.
fn escrow_and_list(
    env: &Env,
    seller: &Address,
    resource: &Symbol,
    amount: u32,
    min_price: i128,
) -> Result<DexOffer, HarvestError> {
    let balance = resource_balance(env, seller, resource);
    if balance < amount {
        return Err(HarvestError::InsufficientBalance);
    }
    env.storage().instance().set(
        &ResourceKey::ResourceBalance(seller.clone(), resource.clone()),
        &(balance - amount),
    );

    let offer_id = next_dex_offer_id(env)?;
    let offer = DexOffer {
        offer_id,
        seller: seller.clone(),
        asset_id: resource.clone(),
        amount,
        min_price,
        active: true,
    };
    env.storage()
        .instance()
        .set(&ResourceKey::DexOffer(offer_id), &offer);

    note_listing_opened(env, seller);

    env.events().publish(
        (symbol_short!("dex"), symbol_short!("listed")),
        (
            offer_id,
            seller.clone(),
            resource.clone(),
            amount,
            min_price,
        ),
    );

    Ok(offer)
}

/// Buy `amount` units from an active offer at the offer's listed price.
///
/// Partial fills are allowed: the offer keeps the remainder and stays active
/// until it is empty. `max_price` is the buyer's slippage guard — the fill is
/// rejected if the listed price is higher. The escrowed units are credited to
/// the buyer, the trade is appended to the trading history, and `dex/filled`
/// publishes `(offer_id, buyer, seller, amount, price, total_cost)` for
/// settlement.
///
/// # Errors
/// - [`HarvestError::DexFailure`] if the offer is unknown or inactive.
/// - [`HarvestError::SelfTrade`] if `buyer` is the seller.
/// - [`HarvestError::InvalidAmount`] if `amount` is zero or exceeds the offer.
/// - [`HarvestError::SlippageExceeded`] if the listed price exceeds `max_price`.
/// - [`HarvestError::PriceOverflow`] if `amount * price` or the buyer's
///   balance would overflow.
///
/// # Reentrancy
/// Runs under the global reentrancy guard (Issue #472).
pub fn buy_offer(
    env: &Env,
    buyer: &Address,
    offer_id: u64,
    amount: u32,
    max_price: i128,
) -> Result<DexFill, HarvestError> {
    with_guard(env, || {
        buyer.require_auth();

        let mut offer = get_dex_offer(env, offer_id)
            .filter(|o| o.active)
            .ok_or(HarvestError::DexFailure)?;
        if offer.seller == *buyer {
            return Err(HarvestError::SelfTrade);
        }
        if amount == 0 || amount > offer.amount {
            return Err(HarvestError::InvalidAmount);
        }
        if offer.min_price > max_price {
            return Err(HarvestError::SlippageExceeded);
        }
        let total_cost = offer
            .min_price
            .checked_mul(i128::from(amount))
            .ok_or(HarvestError::PriceOverflow)?;

        credit_resource_balance(env, buyer, &offer.asset_id, amount)?;

        offer.amount -= amount;
        if offer.amount == 0 {
            offer.active = false;
            release_listing_slot(env, &offer.seller);
        }
        env.storage()
            .instance()
            .set(&ResourceKey::DexOffer(offer_id), &offer);

        let fill = DexFill {
            id: offer_id,
            buyer: buyer.clone(),
            seller: offer.seller.clone(),
            asset_id: offer.asset_id.clone(),
            amount,
            price: offer.min_price,
            total_cost,
            remaining: i128::from(offer.amount),
        };
        record_fill(env, offer_id, &fill);
        Ok(fill)
    })
}

/// Sell `amount` units of the seller's balance into an open buy-side
/// [`crate::trading::LimitOrder`], at the order's limit price.
///
/// The units move from the seller's balance to the order's trader, the order's
/// remaining quantity shrinks (and it is removed once filled), and the trade is
/// recorded exactly like a [`buy_offer`] fill. `min_price` is the seller's
/// slippage guard.
///
/// # Errors
/// - [`HarvestError::OrderUnavailable`] if the order is unknown, sell-side, or
///   for a different resource.
/// - [`HarvestError::SelfTrade`] if `seller` placed the order.
/// - [`HarvestError::InvalidAmount`] if `amount` is zero or exceeds the order.
/// - [`HarvestError::SlippageExceeded`] if the order's price is below `min_price`.
/// - [`HarvestError::InsufficientBalance`] if the seller holds less than `amount`.
///
/// # Reentrancy
/// Runs under the global reentrancy guard (Issue #472).
pub fn sell_to_order(
    env: &Env,
    seller: &Address,
    order_id: u64,
    resource: &Symbol,
    amount: u32,
    min_price: i128,
) -> Result<DexFill, HarvestError> {
    with_guard(env, || {
        seller.require_auth();

        let order = crate::trading::get_limit_order(env, order_id)
            .filter(|o| o.side == OrderSide::Buy && o.resource == *resource)
            .ok_or(HarvestError::OrderUnavailable)?;
        if order.trader == *seller {
            return Err(HarvestError::SelfTrade);
        }
        if amount == 0 || i128::from(amount) > order.quantity {
            return Err(HarvestError::InvalidAmount);
        }
        if order.limit_price < min_price {
            return Err(HarvestError::SlippageExceeded);
        }
        let total_cost = order
            .limit_price
            .checked_mul(i128::from(amount))
            .ok_or(HarvestError::PriceOverflow)?;

        let balance = resource_balance(env, seller, resource);
        if balance < amount {
            return Err(HarvestError::InsufficientBalance);
        }
        env.storage().instance().set(
            &ResourceKey::ResourceBalance(seller.clone(), resource.clone()),
            &(balance - amount),
        );
        credit_resource_balance(env, &order.trader, resource, amount)?;

        fill_limit_order(env, order_id, i128::from(amount)).map_err(|err| match err {
            TradingError::InvalidQuantity => HarvestError::InvalidAmount,
            _ => HarvestError::OrderUnavailable,
        })?;

        let fill = DexFill {
            id: order_id,
            buyer: order.trader.clone(),
            seller: seller.clone(),
            asset_id: resource.clone(),
            amount,
            price: order.limit_price,
            total_cost,
            remaining: order.quantity - i128::from(amount),
        };
        record_fill(env, order_id, &fill);
        Ok(fill)
    })
}

/// Append a fill to the trading history and publish `dex/filled`.
fn record_fill(env: &Env, id: u64, fill: &DexFill) {
    push_trade_record(
        env,
        TradeRecord {
            order_id: id,
            trader: fill.buyer.clone(),
            side: OrderSide::Buy,
            resource: fill.asset_id.clone(),
            quantity: i128::from(fill.amount),
            price: fill.price,
            executed_at: env.ledger().timestamp(),
        },
    );

    env.events().publish(
        (symbol_short!("dex"), symbol_short!("filled")),
        (
            id,
            fill.buyer.clone(),
            fill.seller.clone(),
            fill.amount,
            fill.price,
            fill.total_cost,
        ),
    );
}

/// Page through active offers in ID order, optionally for one resource.
///
/// Starts after offer `start_after` (`0` for the first page) and returns at
/// most `limit` offers (clamped to [`MAX_OFFER_PAGE`]), examining at most
/// [`MAX_OFFER_SCAN`] IDs. Continue with `next_cursor` until it is `None`.
pub fn get_open_offers(
    env: &Env,
    resource: Option<&Symbol>,
    start_after: u64,
    limit: u32,
) -> OfferPage {
    let limit = limit.min(MAX_OFFER_PAGE);
    let last_id = dex_offer_count(env);
    let mut offers = Vec::new(env);
    let mut id = start_after;
    let mut scanned = 0u32;

    while id < last_id && offers.len() < limit && scanned < MAX_OFFER_SCAN {
        id += 1;
        scanned += 1;
        if let Some(offer) = get_dex_offer(env, id) {
            if offer.active && resource.is_none_or(|r| *r == offer.asset_id) {
                offers.push_back(offer);
            }
        }
    }

    OfferPage {
        offers,
        next_cursor: if id < last_id { Some(id) } else { None },
    }
}

/// Cancel an active DEX listing. Only the offer's escrow holder may cancel.
///
/// Refunds the unsold escrowed `amount` back to the seller and frees one of
/// their listing slots. Cancelling an already
/// cancelled, unknown, or someone else's offer returns
/// [`HarvestError::DexFailure`].
///
/// Without the `seller` check below this would be a fund-theft bug: the escrow
/// refund is credited to `caller`, so any address could cancel any live offer
/// and collect the seller's escrowed units.
///
/// # Reentrancy
/// Runs under the global reentrancy guard (Issue #472): every check and state
/// effect completes while the lock is held, following checks-effects-
/// interactions, so a nested call into any guarded entry point while this one
/// is in flight is rejected with a `Reentrancy` error.
pub fn cancel_listing(env: &Env, owner: &Address, offer_id: u64) -> Result<DexOffer, HarvestError> {
    with_guard(env, || cancel_listing_unguarded(env, owner, offer_id))
}

/// Unguarded body of [`cancel_listing`]; callers must already hold the
/// reentrancy lock (e.g. another guarded entry point composing it).
fn cancel_listing_unguarded(
    env: &Env,
    owner: &Address,
    offer_id: u64,
) -> Result<DexOffer, HarvestError> {
    owner.require_auth();

    let mut offer: DexOffer = get_dex_offer(env, offer_id).ok_or(HarvestError::DexFailure)?;

    if !offer.active || offer.seller != *owner {
        return Err(HarvestError::DexFailure);
    }

    offer.active = false;
    env.storage()
        .instance()
        .set(&ResourceKey::DexOffer(offer_id), &offer);
    release_listing_slot(env, owner);

    // Release the escrow.
    let balance_key = ResourceKey::ResourceBalance(owner.clone(), offer.asset_id.clone());
    let balance: u32 = env.storage().instance().get(&balance_key).unwrap_or(0);
    let refunded = balance
        .checked_add(offer.amount)
        .ok_or(HarvestError::PriceOverflow)?;
    env.storage().instance().set(&balance_key, &refunded);

    env.events().publish(
        (symbol_short!("dex"), symbol_short!("canceld")),
        (offer_id, owner.clone()),
    );

    Ok(offer)
}

/// Read a DEX offer by ID.
pub fn get_offer(env: &Env, offer_id: u64) -> Option<DexOffer> {
    get_dex_offer(env, offer_id)
}

// ── Dynamic pricing (Issue #452) ──────────────────────────────────────────────

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DynamicListError {
    /// The harvest or the listing leg failed. The underlying
    /// [`HarvestError`] code is published in the `dlist` event so it stays
    /// diagnosable on-chain.
    HarvestFailed = 1,
    /// No dynamic price exists yet for this resource — nothing has been
    /// observed, so there is nothing to list against.
    PriceUnavailable = 2,
    /// The dynamic price resolved to a non-positive value.
    InvalidPrice = 3,
}

impl From<HarvestError> for DynamicListError {
    fn from(_: HarvestError) -> Self {
        DynamicListError::HarvestFailed
    }
}

impl From<crate::dynamic_pricing::PricingError> for DynamicListError {
    fn from(err: crate::dynamic_pricing::PricingError) -> Self {
        match err {
            crate::dynamic_pricing::PricingError::InvalidPrice => DynamicListError::InvalidPrice,
            _ => DynamicListError::PriceUnavailable,
        }
    }
}

impl crate::error_standard::StandardContractError for DynamicListError {
    fn descriptor(self) -> crate::error_standard::ErrorDescriptor {
        use crate::error_standard::ErrorKind;
        let (kind, retryable) = match self {
            Self::HarvestFailed => (ErrorKind::Conflict, false),
            Self::PriceUnavailable => (ErrorKind::NotFound, true),
            Self::InvalidPrice => (ErrorKind::Validation, false),
        };
        crate::error_standard::ErrorDescriptor {
            module: "dex_integration",
            code: self as u32,
            kind,
            retryable,
        }
    }
}

/// Harvest a resource and list it at the current dynamic market price.
///
/// Same escrow and listing-cap rules as [`harvest_and_list`]; the only
/// difference is that `min_price` comes from
/// [`crate::dynamic_pricing::listing_price`] — the EMA-smoothed, volatility-
/// clamped price — instead of a number the caller types in. That is the point:
/// a hand-typed price is how a listing gets stuck far below market after a
/// spike, and a spammed price is how someone drains the book.
///
/// A player who wants to beat the market can still use [`harvest_and_list`]
/// with an explicit price; this path is the default.
///
/// # Errors
/// - [`DynamicListError::PriceUnavailable`] if the resource has no observed
///   price yet.
/// - [`DynamicListError::HarvestFailed`] for anything [`harvest_and_list`]
///   would reject.
pub fn list_at_market(
    env: &Env,
    player: &Address,
    ship_id: u64,
    layout: &crate::nebula_explorer::NebulaLayout,
    resource: &Symbol,
) -> Result<(HarvestResult, DexOffer, i128), DynamicListError> {
    let price = crate::dynamic_pricing::listing_price(env, resource.clone())?;
    if price <= 0 {
        return Err(DynamicListError::InvalidPrice);
    }

    let result =
        harvest_and_list(env, player, ship_id, layout, resource, price).map_err(|err| {
            env.events().publish(
                (symbol_short!("dlist"), symbol_short!("failed")),
                (resource.clone(), err as u32),
            );
            DynamicListError::HarvestFailed
        })?;

    Ok((result.0, result.1, price))
}
