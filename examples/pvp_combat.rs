//! # Example: PvP challenges, turn-based combat and ELO
//!
//! Module: `pvp_combat` via `NebulaNomadContract`
//!
//! Shows challenge -> accept -> alternate moves until a winner is found,
//! then reads the updated ELO ratings and combat history.
//!
//! Run:
//! ```text
//! cargo run --example pvp_combat
//! ```

use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env};
use stellar_nebula_nomad::{NebulaNomadContract, NebulaNomadContractClient, INITIAL_ELO};

fn main() {
    let env = Env::default();
    env.mock_all_auths();
    let client = NebulaNomadContractClient::new(&env, &env.register(NebulaNomadContract, ()));

    let admin = Address::generate(&env);
    let orion = Address::generate(&env);
    let vega = Address::generate(&env);
    client.set_pvp_admin(&admin);

    println!("Starting ELO: {INITIAL_ELO}");

    // ── Step 1: Challenge ────────────────────────────────────────────────
    // Stakes are optional (0 = friendly match).
    let challenge_id = client.create_challenge(&orion, &vega, &0i128);
    println!("Challenge #{challenge_id}: Orion -> Vega");

    // ── Step 2: Accept (starts the combat) ───────────────────────────────
    let combat_id = client.accept_challenge(&vega, &challenge_id);
    let combat = client.get_combat(&combat_id);
    println!(
        "Combat #{combat_id} started: HP {} vs {}, first turn: {}",
        combat.player1_hp,
        combat.player2_hp,
        if combat.turn == orion {
            "Orion"
        } else {
            "Vega"
        }
    );

    // ── Step 3: Take turns ───────────────────────────────────────────────
    // Moves: "attack" deals `power` damage and costs power/2 energy.
    // "defend" restores energy. Only the player whose turn it is may move.
    for round in 1..=40 {
        let state = client.get_combat(&combat_id);
        if state.status != symbol_short!("active") {
            break;
        }
        let actor = state.turn.clone();
        let energy = if actor == state.player1 {
            state.player1_energy
        } else {
            state.player2_energy
        };

        // Simple strategy: attack when there's enough energy, else defend.
        let (mv, power) = if energy >= 15 {
            (symbol_short!("attack"), 30u32)
        } else {
            (symbol_short!("defend"), 0u32)
        };

        match client.try_execute_combat_move(&actor, &combat_id, &mv, &power) {
            Ok(Ok(())) => {
                let s = client.get_combat(&combat_id);
                println!(
                    "  round {round:>2}: {:<5} {:?}({power}) -> HP {} / {}",
                    if actor == orion { "Orion" } else { "Vega" },
                    mv,
                    s.player1_hp,
                    s.player2_hp
                );
            }
            Err(Ok(e)) => {
                println!("  round {round:>2}: move rejected: {e:?}");
                break;
            }
            other => {
                println!("  round {round:>2}: unexpected {other:?}");
                break;
            }
        }
    }

    // ── Step 4: Results ──────────────────────────────────────────────────
    let final_state = client.get_combat(&combat_id);
    match &final_state.winner {
        Some(w) if *w == orion => println!("Winner: Orion"),
        Some(_) => println!("Winner: Vega"),
        None => println!("No winner yet (status {:?})", final_state.status),
    }

    for (name, who) in [("Orion", &orion), ("Vega", &vega)] {
        let stats = client.get_combat_stats(who);
        println!(
            "{name:<5} ELO={} W/L/D={}/{}/{}",
            client.get_elo_rating(who),
            stats.wins,
            stats.losses,
            stats.draws
        );
    }

    let history = client.get_player_combat_history(&orion, &10u32);
    println!("Orion has {} recorded combat(s)", history.len());
}
