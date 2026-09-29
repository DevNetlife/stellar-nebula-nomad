#![cfg(test)]

use soroban_sdk::{symbol_short, Env};
use stellar_nebula_nomad::economics::pricing_algorithm::{
    calculate_ema, clamp_price, check_circuit_breaker, update_market_price, PricingError,
};

#[test]
fn test_ema_calculation() {
    let ema = calculate_ema(100, 150, 20); // 20% alpha
    assert_eq!(ema, 110); // 150 * 0.2 + 100 * 0.8 = 110
}

#[test]
fn test_clamp_price_rate_limit() {
    // 5% max hourly change from 100 is max 105
    let clamped = clamp_price(150, 100, 10, 1000);
    assert_eq!(clamped, 105);
}

#[test]
fn test_circuit_breaker_trigger() {
    // > 20% price move triggers circuit breaker
    let is_triggered = check_circuit_breaker(100, 125);
    assert!(is_triggered);

    let normal_change = check_circuit_breaker(100, 105);
    assert!(!normal_change);
}

#[test]
fn test_dynamic_pricing_flow() {
    let env = Env::default();
    let resource = symbol_short!("ORE");

    let p1 = update_market_price(&env, resource.clone(), 100, 50).unwrap();
    assert_eq!(p1, 100);

    let p2 = update_market_price(&env, resource.clone(), 104, 50).unwrap();
    assert!(p2 > 100 && p2 <= 105);
}
