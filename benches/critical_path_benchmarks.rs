/// Gas/CPU benchmarks for critical paths not covered by `gas_benchmarks.rs`:
/// alliance founding, ship energy consume/recharge, crafting, and the
/// emergency pause path. Run via `cargo test --benches` (see
/// docs/GAS_OPTIMIZATION_GUIDE.md for the established convention — `harness
/// = false` keeps these out of `cargo bench`'s libtest bench harness while
/// `cargo test --benches` still executes the `// #[test]` fns below).
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Bytes, Env, String, Vec};
use stellar_nebula_nomad::{
    consume_energy, craft, found_alliance, initialize_admins, mint_ship, pause_contract,
    recharge_energy,
    recipes::{set_recipe, Recipe},
};

/// Alliance/guild founding: hot path for the social layer, touches 5 storage
/// writes (alliance record, count, membership, treasury, contribution).
// #[test]
fn bench_found_alliance() {
    let env = Env::default();
    env.mock_all_auths();
    let founder = Address::generate(&env);
    let name = String::from_str(&env, "Star Reavers");

    env.budget().reset_unlimited();
    let _alliance_id = found_alliance(&env, founder, name).unwrap();

    let cpu_insns = env.budget().cpu_instruction_cost();
    let mem_bytes = env.budget().memory_bytes_cost();

    println!("Found Alliance:");
    println!("  CPU instructions: {}", cpu_insns);
    println!("  Memory bytes: {}", mem_bytes);

    assert!(cpu_insns < 1_500_000, "Found alliance exceeds CPU target");
}

/// Energy consume/recharge: called on nearly every scan/harvest/combat
/// action, so it needs to stay cheap.
// #[test]
fn bench_energy_consume_and_recharge() {
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

    env.budget().reset_unlimited();
    recharge_energy(&env, ship.id, 200).unwrap();
    let recharge_cpu = env.budget().cpu_instruction_cost();

    println!("Energy Consume/Recharge:");
    println!("  Consume CPU instructions: {}", consume_cpu);
    println!("  Recharge CPU instructions: {}", recharge_cpu);

    assert!(consume_cpu < 500_000, "Energy consume exceeds CPU target");
    assert!(recharge_cpu < 500_000, "Energy recharge exceeds CPU target");
}

/// Crafting: recipe lookup + resource check/consume + mastery bookkeeping +
/// output mint, one of the most storage-heavy player actions.
// #[test]
fn bench_craft() {
    let env = Env::default();
    env.mock_all_auths();
    let player = Address::generate(&env);

    // Zero-input, unlocked (rarity 1), level-0 recipe so `craft` succeeds
    // without extra resource-minting setup — this isolates the crafting-path
    // overhead itself rather than resource funding.
    set_recipe(
        &env,
        &Recipe {
            id: 1,
            inputs: Vec::new(&env),
            output: (symbol_short!("essence"), 10),
            rarity: 1,
            required_level: 0,
        },
    );

    env.budget().reset_unlimited();
    craft(env.clone(), player, 1).unwrap();

    let cpu_insns = env.budget().cpu_instruction_cost();
    let mem_bytes = env.budget().memory_bytes_cost();

    println!("Craft:");
    println!("  CPU instructions: {}", cpu_insns);
    println!("  Memory bytes: {}", mem_bytes);

    assert!(cpu_insns < 1_500_000, "Craft exceeds CPU target");
}

/// Emergency pause: a safety-critical path that must stay cheap enough to
/// execute reliably even under network congestion.
// #[test]
fn bench_emergency_pause() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let mut admins = Vec::new(&env);
    admins.push_back(admin.clone());
    initialize_admins(&env, admins).unwrap();

    env.budget().reset_unlimited();
    pause_contract(&env, &admin).unwrap();

    let cpu_insns = env.budget().cpu_instruction_cost();
    let mem_bytes = env.budget().memory_bytes_cost();

    println!("Emergency Pause:");
    println!("  CPU instructions: {}", cpu_insns);
    println!("  Memory bytes: {}", mem_bytes);

    assert!(cpu_insns < 300_000, "Emergency pause exceeds CPU target");
}
