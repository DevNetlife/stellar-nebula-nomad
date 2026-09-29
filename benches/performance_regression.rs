/// Performance regression test suite
/// Ensures optimizations don't regress over time
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Bytes, Env, String, Vec};
use stellar_nebula_nomad::*;

const MAX_CPU_NEBULA_GEN: u64 = 1_000_000;
const MAX_CPU_SCAN: u64 = 2_000_000;
const MAX_CPU_HARVEST: u64 = 1_500_000;
const MAX_CPU_MINT: u64 = 800_000;
const MAX_MEM_BYTES: u64 = 100_000;
const MAX_CPU_FOUND_ALLIANCE: u64 = 1_500_000;
const MAX_CPU_ENERGY_OP: u64 = 500_000;
const MAX_CPU_CRAFT: u64 = 1_500_000;
const MAX_CPU_EMERGENCY_PAUSE: u64 = 300_000;

#[test]
fn regression_nebula_generation() {
    let env = Env::default();
    let player = Address::generate(&env);
    let seed = env
        .crypto()
        .sha256(&soroban_sdk::Bytes::from_slice(&env, &[1u8; 32]));

    env.budget().reset_unlimited();
    let _layout = generate_nebula_layout(env.clone(), seed, player);

    let cpu = env.budget().cpu_instruction_cost();
    let mem = env.budget().memory_bytes_cost();

    assert!(
        cpu <= MAX_CPU_NEBULA_GEN,
        "REGRESSION: Nebula gen CPU {} exceeds baseline {}",
        cpu,
        MAX_CPU_NEBULA_GEN
    );
    assert!(
        mem <= MAX_MEM_BYTES,
        "REGRESSION: Memory {} exceeds baseline {}",
        mem,
        MAX_MEM_BYTES
    );
}

#[test]
fn regression_scan_operation() {
    let env = Env::default();
    let player = Address::generate(&env);
    let seed = env
        .crypto()
        .sha256(&soroban_sdk::Bytes::from_slice(&env, &[1u8; 32]));

    env.budget().reset_unlimited();
    let _result = scan_nebula(env.clone(), seed, player);

    let cpu = env.budget().cpu_instruction_cost();

    assert!(
        cpu <= MAX_CPU_SCAN,
        "REGRESSION: Scan CPU {} exceeds baseline {}",
        cpu,
        MAX_CPU_SCAN
    );
}

#[test]
fn regression_mint_ship() {
    let env = Env::default();
    let player = Address::generate(&env);

    env.budget().reset_unlimited();
    let _ship = mint_ship(
        env.clone(),
        player,
        soroban_sdk::symbol_short!("fighter"),
        soroban_sdk::Bytes::new(&env),
    );

    let cpu = env.budget().cpu_instruction_cost();

    assert!(
        cpu <= MAX_CPU_MINT,
        "REGRESSION: Mint CPU {} exceeds baseline {}",
        cpu,
        MAX_CPU_MINT
    );
}

#[test]
fn regression_batch_efficiency() {
    let env = Env::default();
    let player = Address::generate(&env);

    // Single operation
    env.budget().reset_unlimited();
    let _ship1 = mint_ship(
        env.clone(),
        player.clone(),
        soroban_sdk::symbol_short!("fighter"),
        soroban_sdk::Bytes::new(&env),
    );
    let single_cpu = env.budget().cpu_instruction_cost();

    // Batch operation (3 ships)
    let ship_types = soroban_sdk::vec![
        &env,
        soroban_sdk::symbol_short!("fighter"),
        soroban_sdk::symbol_short!("miner"),
        soroban_sdk::symbol_short!("scout"),
    ];

    env.budget().reset_unlimited();
    let _ships = batch_mint_ships(
        env.clone(),
        player,
        ship_types,
        soroban_sdk::Bytes::new(&env),
    );
    let batch_cpu = env.budget().cpu_instruction_cost();

    // Batch should be more efficient than 3x single
    let efficiency_ratio = (batch_cpu as f64) / (single_cpu as f64 * 3.0);

    assert!(
        efficiency_ratio < 0.85,
        "REGRESSION: Batch efficiency {} should be < 0.85",
        efficiency_ratio
    );
}

#[test]
fn regression_storage_bump_cost() {
    let env = Env::default();
    let player = Address::generate(&env);

    let profile_id = initialize_profile(env.clone(), player.clone()).unwrap();

    env.budget().reset_unlimited();
    update_progress(env.clone(), player, profile_id, 1, 100).unwrap();

    let cpu = env.budget().cpu_instruction_cost();

    // Storage operations should be optimized
    assert!(
        cpu <= 500_000,
        "REGRESSION: Storage update CPU {} exceeds 500K",
        cpu
    );
}

#[test]
fn regression_found_alliance() {
    let env = Env::default();
    env.mock_all_auths();
    let founder = Address::generate(&env);

    env.budget().reset_unlimited();
    let _id = found_alliance(&env, founder, String::from_str(&env, "Regression Alliance")).unwrap();

    let cpu = env.budget().cpu_instruction_cost();
    assert!(
        cpu <= MAX_CPU_FOUND_ALLIANCE,
        "REGRESSION: Found alliance CPU {} exceeds baseline {}",
        cpu,
        MAX_CPU_FOUND_ALLIANCE
    );
}

#[test]
fn regression_energy_consume_recharge() {
    let env = Env::default();
    env.mock_all_auths();
    let owner = Address::generate(&env);
    let ship = mint_ship(
        env.clone(),
        owner,
        symbol_short!("fighter"),
        Bytes::new(&env),
    )
    .unwrap();

    env.budget().reset_unlimited();
    consume_energy(&env, ship.id, 500).unwrap();
    let consume_cpu = env.budget().cpu_instruction_cost();
    assert!(
        consume_cpu <= MAX_CPU_ENERGY_OP,
        "REGRESSION: Energy consume CPU {} exceeds baseline {}",
        consume_cpu,
        MAX_CPU_ENERGY_OP
    );

    env.budget().reset_unlimited();
    recharge_energy(&env, ship.id, 200).unwrap();
    let recharge_cpu = env.budget().cpu_instruction_cost();
    assert!(
        recharge_cpu <= MAX_CPU_ENERGY_OP,
        "REGRESSION: Energy recharge CPU {} exceeds baseline {}",
        recharge_cpu,
        MAX_CPU_ENERGY_OP
    );
}

#[test]
fn regression_craft() {
    let env = Env::default();
    env.mock_all_auths();
    let player = Address::generate(&env);

    recipes::set_recipe(
        &env,
        &recipes::Recipe {
            id: 1,
            inputs: Vec::new(&env),
            output: (symbol_short!("essence"), 10),
            rarity: 1,
            required_level: 0,
        },
    );

    env.budget().reset_unlimited();
    craft(env.clone(), player, 1).unwrap();

    let cpu = env.budget().cpu_instruction_cost();
    assert!(
        cpu <= MAX_CPU_CRAFT,
        "REGRESSION: Craft CPU {} exceeds baseline {}",
        cpu,
        MAX_CPU_CRAFT
    );
}

#[test]
fn regression_emergency_pause() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let mut admins = Vec::new(&env);
    admins.push_back(admin.clone());
    initialize_admins(&env, admins).unwrap();

    env.budget().reset_unlimited();
    pause_contract(&env, &admin).unwrap();

    let cpu = env.budget().cpu_instruction_cost();
    assert!(
        cpu <= MAX_CPU_EMERGENCY_PAUSE,
        "REGRESSION: Emergency pause CPU {} exceeds baseline {}",
        cpu,
        MAX_CPU_EMERGENCY_PAUSE
    );
}
