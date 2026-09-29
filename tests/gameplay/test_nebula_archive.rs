#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Vec};
use stellar_nebula_nomad::{ArchiveError, NebulaArchive, NebulaLayout};

fn create_mock_layout(env: &Env, size: u32) -> NebulaLayout {
    let anomalies = Vec::new(env);
    let layout_hash = BytesN::from_array(env, &[1u8; 32]);

    NebulaLayout {
        anomalies,
        layout_hash,
        generated_at: env.ledger().timestamp(),
        size,
    }
}

#[test]
fn test_archive_and_replay() {
    let env = Env::default();
    env.mock_all_auths();

    let layout = create_mock_layout(&env, 10);
    let layout_hash = layout.layout_hash.clone();

    let archive_id = stellar_nebula_nomad::NebulaNomadContract::archive_nebula_layout(
        env.clone(),
        layout.clone(),
    )
    .unwrap();

    assert_eq!(archive_id, 1);

    let archived =
        stellar_nebula_nomad::NebulaNomadContract::replay_archive(env.clone(), archive_id).unwrap();

    assert_eq!(archived.archive_id, 1);
    assert_eq!(archived.nebula_hash, layout_hash);
    assert_eq!(archived.layout.size, 10);
}

#[test]
fn test_archive_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let result = stellar_nebula_nomad::NebulaNomadContract::replay_archive(env.clone(), 999);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ArchiveError::ArchiveNotFound);
}

#[test]
fn test_batch_archive_layouts() {
    let env = Env::default();
    env.mock_all_auths();

    let mut layouts = Vec::new(&env);
    for i in 0..5 {
        layouts.push_back(create_mock_layout(&env, i + 1));
    }

    let archive_ids =
        stellar_nebula_nomad::NebulaNomadContract::batch_archive_layouts(env.clone(), layouts)
            .unwrap();

    assert_eq!(archive_ids.len(), 5);
    assert_eq!(archive_ids.get(0).unwrap(), 1);
    assert_eq!(archive_ids.get(4).unwrap(), 5);

    let count = stellar_nebula_nomad::NebulaNomadContract::get_archive_count(env.clone());
    assert_eq!(count, 5);
}

#[test]
fn test_batch_archive_exceeds_limit() {
    let env = Env::default();
    env.mock_all_auths();

    let mut layouts = Vec::new(&env);
    for i in 0..21 {
        layouts.push_back(create_mock_layout(&env, i + 1));
    }

    let result =
        stellar_nebula_nomad::NebulaNomadContract::batch_archive_layouts(env.clone(), layouts);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), ArchiveError::BurstLimitExceeded);
}

#[test]
fn test_get_archive_by_hash() {
    let env = Env::default();
    env.mock_all_auths();

    let layout = create_mock_layout(&env, 15);
    let layout_hash = layout.layout_hash.clone();

    let archive_id =
        stellar_nebula_nomad::NebulaNomadContract::archive_nebula_layout(env.clone(), layout)
            .unwrap();

    let archived =
        stellar_nebula_nomad::NebulaNomadContract::get_archive_by_hash(env.clone(), layout_hash)
            .unwrap();

    assert_eq!(archived.archive_id, archive_id);
    assert_eq!(archived.layout.size, 15);
}

#[test]
fn test_archive_count_increments() {
    let env = Env::default();
    env.mock_all_auths();

    let initial_count = stellar_nebula_nomad::NebulaNomadContract::get_archive_count(env.clone());
    assert_eq!(initial_count, 0);

    for i in 0..3 {
        let layout = create_mock_layout(&env, i + 1);
        stellar_nebula_nomad::NebulaNomadContract::archive_nebula_layout(env.clone(), layout)
            .unwrap();
    }

    let final_count = stellar_nebula_nomad::NebulaNomadContract::get_archive_count(env.clone());
    assert_eq!(final_count, 3);
}
