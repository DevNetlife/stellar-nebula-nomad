//! Fuzz testing for trading module

#![cfg(test)]

use proptest::prelude::*;
use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{symbol_short, Address, Env};
use stellar_nebula_nomad::{
    LimitOrder, NebulaNomadContract, NebulaNomadContractClient, OrderSide, TradingError,
};

fn setup() -> (Env, NebulaNomadContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set(LedgerInfo {
        protocol_version: 22,
        sequence_number: 100,
        timestamp: 1_700_000_000,
        network_id: [0u8; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 100,
        min_persistent_entry_ttl: 1_000,
        max_entry_ttl: 10_000,
    });
    let contract_id = env.register(NebulaNomadContract, ());
    let client = NebulaNomadContractClient::new(&env, &contract_id);
    let trader = Address::generate(&env);
    (env, client, trader)
}

proptest! {
    #[test]
    fn prop_zero_price_rejected(quantity in 1i128..1000i128) {
        let (env, client, trader) = setup();
        let order = LimitOrder {
            id: 0,
            trader: trader.clone(),
            side: OrderSide::Buy,
            resource: symbol_short!("dust"),
            quantity,
            limit_price: 0,
            placed_at: 0,
            is_stop_loss: false,
        };
        let result = client.try_place_limit_order(&trader, &order);
        prop_assert!(matches!(result, Err(Ok(TradingError::InvalidPrice))));
    }

    #[test]
    fn prop_zero_quantity_rejected(price in 1i128..1000i128) {
        let (env, client, trader) = setup();
        let order = LimitOrder {
            id: 0,
            trader: trader.clone(),
            side: OrderSide::Sell,
            resource: symbol_short!("ore"),
            quantity: 0,
            limit_price: price,
            placed_at: 0,
            is_stop_loss: false,
        };
        let result = client.try_place_limit_order(&trader, &order);
        prop_assert!(matches!(result, Err(Ok(TradingError::InvalidQuantity))));
    }

    #[test]
    fn prop_valid_order_succeeds(price in 1i128..10000i128, quantity in 1i128..10000i128) {
        let (env, client, trader) = setup();
        let order = LimitOrder {
            id: 0,
            trader: trader.clone(),
            side: OrderSide::Buy,
            resource: symbol_short!("gas"),
            quantity,
            limit_price: price,
            placed_at: 0,
            is_stop_loss: false,
        };
        let result = client.try_place_limit_order(&trader, &order);
        prop_assert!(result.is_ok());
    }
}

#[test]
fn fuzz_cancel_nonexistent_order() {
    let (_, client, trader) = setup();
    let result = client.try_cancel_limit_order(&trader, &99999u64);
    assert!(matches!(result, Err(Ok(TradingError::OrderNotFound))));
}

#[test]
fn fuzz_order_cap_enforcement() {
    let (env, client, trader) = setup();
    for i in 0..51 {
        let order = LimitOrder {
            id: 0,
            trader: trader.clone(),
            side: OrderSide::Buy,
            resource: symbol_short!("dust"),
            quantity: 100,
            limit_price: 10,
            placed_at: 0,
            is_stop_loss: false,
        };
        let result = client.try_place_limit_order(&trader, &order);
        if i < 50 {
            assert!(result.is_ok());
        } else {
            assert!(matches!(result, Err(Ok(TradingError::OrderCapReached))));
        }
    }
}
