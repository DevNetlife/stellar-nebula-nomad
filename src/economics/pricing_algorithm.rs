//! Dynamic Market-Based Resource Pricing Algorithm.
//! Implements Exponential Moving Average (EMA), Time-Weighted Average Price (TWAP),
//! bounds damping, and circuit breakers.

use soroban_sdk::{contracterror, contracttype, symbol_short, Env, Symbol, Vec};
use crate::constants::{
    CIRCUIT_BREAKER_THRESHOLD_PERCENT, MAX_HOURLY_PRICE_CHANGE_PERCENT, MAX_PRICE_HISTORY_POINTS,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DynamicPriceState {
    pub current_price: u128,
    pub twap_24h: u128,
    pub ema_price: u128,
    pub min_price_limit: u128,
    pub max_price_limit: u128,
    pub last_updated_at: u64,
    pub is_circuit_broken: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PricePoint {
    pub price: u128,
    pub timestamp: u64,
    pub volume: u128,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum PricingError {
    CircuitBreakerActive = 1,
    InvalidPriceBounds = 2,
    PriceChangeExceedsLimit = 3,
}

pub fn calculate_ema(current_ema: u128, new_trade_price: u128, alpha_percent: u32) -> u128 {
    if current_ema == 0 {
        return new_trade_price;
    }
    let alpha = (alpha_percent as u128).min(100);
    (new_trade_price * alpha + current_ema * (100 - alpha)) / 100
}

pub fn clamp_price(
    proposed_price: u128,
    last_price: u128,
    min_limit: u128,
    max_limit: u128,
) -> u128 {
    let max_allowed_increase = last_price + (last_price * MAX_HOURLY_PRICE_CHANGE_PERCENT as u128 / 100);
    let max_allowed_decrease = last_price.saturating_sub(last_price * MAX_HOURLY_PRICE_CHANGE_PERCENT as u128 / 100);

    let bounded = proposed_price.clamp(max_allowed_decrease, max_allowed_increase);
    bounded.clamp(min_limit, max_limit)
}

pub fn check_circuit_breaker(current_price: u128, new_trade_price: u128) -> bool {
    if current_price == 0 {
        return false;
    }
    let delta = if new_trade_price > current_price {
        new_trade_price - current_price
    } else {
        current_price - new_trade_price
    };

    let percent_change = (delta * 100) / current_price;
    percent_change > CIRCUIT_BREAKER_THRESHOLD_PERCENT as u128
}

pub fn update_market_price(
    env: &Env,
    resource: Symbol,
    trade_price: u128,
    trade_volume: u128,
) -> Result<u128, PricingError> {
    let key = (symbol_short!("PRC_ST"), resource.clone());
    let mut state: DynamicPriceState = env.storage().instance().get(&key).unwrap_or(DynamicPriceState {
        current_price: trade_price,
        twap_24h: trade_price,
        ema_price: trade_price,
        min_price_limit: trade_price / 10,
        max_price_limit: trade_price * 10,
        last_updated_at: env.ledger().timestamp(),
        is_circuit_broken: false,
    });

    if state.is_circuit_broken {
        return Err(PricingError::CircuitBreakerActive);
    }

    if check_circuit_breaker(state.current_price, trade_price) {
        state.is_circuit_broken = true;
        env.storage().instance().set(&key, &state);
        return Err(PricingError::CircuitBreakerActive);
    }

    let new_ema = calculate_ema(state.ema_price, trade_price, 20); // 20% alpha
    let clamped_price = clamp_price(
        new_ema,
        state.current_price,
        state.min_price_limit,
        state.max_price_limit,
    );

    state.ema_price = new_ema;
    state.current_price = clamped_price;
    state.last_updated_at = env.ledger().timestamp();

    env.storage().instance().set(&key, &state);

    // Record price point in history
    let history_key = (symbol_short!("PRC_HST"), resource);
    let mut history: Vec<PricePoint> = env.storage().instance().get(&history_key).unwrap_or(Vec::new(env));
    if history.len() >= MAX_PRICE_HISTORY_POINTS {
        history.pop_front();
    }
    history.push_back(PricePoint {
        price: clamped_price,
        timestamp: env.ledger().timestamp(),
        volume: trade_volume,
    });
    env.storage().instance().set(&history_key, &history);

    Ok(clamped_price)
}
