#![cfg(test)]

use soroban_sdk::testutils::{Address as _, Ledger, LedgerInfo};
use soroban_sdk::{symbol_short, Address, Env, String};
use stellar_nebula_nomad::dao::{
    self, DaoConfig, DaoError, Proposal, ProposalStatus, VoteDirection,
};

fn setup_env() -> (Env, Address) {
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
    let admin = Address::generate(&env);
    (env, admin)
}

fn advance_ledger(env: &Env, count: u32) {
    let seq = env.ledger().sequence();
    env.ledger().set(LedgerInfo {
        protocol_version: 22,
        sequence_number: seq + count,
        timestamp: env.ledger().timestamp() + (count as u64) * 5,
        network_id: [0u8; 32],
        base_reserve: 10,
        min_temp_entry_ttl: 100,
        min_persistent_entry_ttl: 1_000,
        max_entry_ttl: 10_000,
    });
}

#[test]
fn test_initialize_dao() {
    let (env, admin) = setup_env();
    let staking = Address::generate(&env);
    let governance = Address::generate(&env);

    let result = dao::initialize(
        env.clone(),
        admin.clone(),
        staking,
        governance,
        1000,    // 10% quorum
        100,     // 100 ledger voting period
        50,      // 50 ledger timelock
        500_000, // 500k proposal threshold
    );
    assert!(result.is_ok());

    // Verify cannot initialize twice.
    let result2 = dao::initialize(
        env,
        admin,
        Address::generate(&env),
        Address::generate(&env),
        1000,
        100,
        50,
        500_000,
    );
    assert_eq!(result2, Err(DaoError::AlreadyInitialized));
}

#[test]
fn test_create_proposal() {
    let (env, admin) = setup_env();
    let staking = Address::generate(&env);
    let governance = Address::generate(&env);

    dao::initialize(
        env.clone(),
        admin,
        staking,
        governance,
        1000,
        100,
        50,
        500_000,
    )
    .unwrap();

    let proposer = Address::generate(&env);
    let description = String::from_slice(&env, "Increase rewards");

    let result = dao::create_proposal(env.clone(), proposer.clone(), description.clone());
    assert!(result.is_ok());

    let proposal_id = result.unwrap();
    assert_eq!(proposal_id, 0);

    // Verify proposal was stored.
    let proposal = dao::get_proposal(env.clone(), proposal_id).unwrap();
    assert_eq!(proposal.id, 0);
    assert_eq!(proposal.proposer, proposer);
    assert_eq!(proposal.status, ProposalStatus::Active);

    // Create another proposal and verify ID increments.
    let result2 = dao::create_proposal(env, proposer, description);
    assert_eq!(result2.unwrap(), 1);
}

#[test]
fn test_vote_on_proposal() {
    let (env, admin) = setup_env();
    let staking = Address::generate(&env);
    let governance = Address::generate(&env);

    dao::initialize(
        env.clone(),
        admin,
        staking,
        governance,
        1000,
        100,
        50,
        500_000,
    )
    .unwrap();

    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let description = String::from_slice(&env, "Proposal");

    let proposal_id = dao::create_proposal(env.clone(), proposer, description).unwrap();

    // Vote For.
    let result = dao::vote(env.clone(), voter.clone(), proposal_id, VoteDirection::For);
    assert!(result.is_ok());

    // Verify vote was recorded.
    let vote = dao::get_vote(env.clone(), voter, proposal_id);
    assert!(vote.is_some());
    let v = vote.unwrap();
    assert_eq!(v.direction, VoteDirection::For);

    // Verify proposal vote tally updated.
    let proposal = dao::get_proposal(env, proposal_id).unwrap();
    assert!(proposal.for_votes > 0);
}

#[test]
fn test_double_vote_rejected() {
    let (env, admin) = setup_env();
    let staking = Address::generate(&env);
    let governance = Address::generate(&env);

    dao::initialize(
        env.clone(),
        admin,
        staking,
        governance,
        1000,
        100,
        50,
        500_000,
    )
    .unwrap();

    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let description = String::from_slice(&env, "Proposal");

    let proposal_id = dao::create_proposal(env.clone(), proposer, description).unwrap();

    // First vote succeeds.
    dao::vote(env.clone(), voter.clone(), proposal_id, VoteDirection::For).unwrap();

    // Second vote should fail.
    let result = dao::vote(env.clone(), voter, proposal_id, VoteDirection::Against);
    assert_eq!(result, Err(DaoError::AlreadyVoted));

    // Verify tally wasn't double-counted.
    let proposal = dao::get_proposal(env, proposal_id).unwrap();
    assert_eq!(proposal.against_votes, 0); // Should still be 0.
}

#[test]
fn test_vote_after_voting_window_rejected() {
    let (env, admin) = setup_env();
    let staking = Address::generate(&env);
    let governance = Address::generate(&env);

    dao::initialize(
        env.clone(),
        admin,
        staking,
        governance,
        1000,
        100, // Voting period
        50,
        500_000,
    )
    .unwrap();

    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let description = String::from_slice(&env, "Proposal");

    let proposal_id = dao::create_proposal(env.clone(), proposer, description).unwrap();

    // Advance past voting window (100 + 1 ledgers).
    advance_ledger(&env, 101);

    // Try to vote. Should fail.
    let result = dao::vote(env.clone(), voter, proposal_id, VoteDirection::For);
    assert_eq!(result, Err(DaoError::VotingNotActive));
}

#[test]
fn test_vote_nonexistent_proposal_rejected() {
    let (env, admin) = setup_env();
    let staking = Address::generate(&env);
    let governance = Address::generate(&env);

    dao::initialize(
        env.clone(),
        admin,
        staking,
        governance,
        1000,
        100,
        50,
        500_000,
    )
    .unwrap();

    let voter = Address::generate(&env);

    let result = dao::vote(
        env,
        voter,
        999, // Non-existent proposal ID
        VoteDirection::For,
    );
    assert_eq!(result, Err(DaoError::ProposalNotFound));
}

#[test]
fn test_execute_proposal_after_timelock() {
    let (env, admin) = setup_env();
    let staking = Address::generate(&env);
    let governance = Address::generate(&env);

    dao::initialize(
        env.clone(),
        admin,
        staking.clone(),
        governance,
        1000,
        100, // Voting period
        50,  // Timelock
        500_000,
    )
    .unwrap();

    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);
    let description = String::from_slice(&env, "Proposal");

    let proposal_id = dao::create_proposal(env.clone(), proposer, description).unwrap();

    // Vote (simulate sufficient power).
    dao::vote(env.clone(), voter, proposal_id, VoteDirection::For).unwrap();

    // Advance past voting window + timelock (100 + 50 + 1 ledgers).
    advance_ledger(&env, 151);

    // Execute proposal.
    let result = dao::execute_proposal(env.clone(), proposal_id);
    assert!(result.is_ok());

    // Verify proposal status is now Executed.
    let proposal = dao::get_proposal(env, proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Executed);
}

#[test]
fn test_execute_proposal_before_timelock_rejected() {
    let (env, admin) = setup_env();
    let staking = Address::generate(&env);
    let governance = Address::generate(&env);

    dao::initialize(
        env.clone(),
        admin,
        staking,
        governance,
        1000,
        100, // Voting period
        50,  // Timelock
        500_000,
    )
    .unwrap();

    let proposer = Address::generate(&env);
    let description = String::from_slice(&env, "Proposal");

    let proposal_id = dao::create_proposal(env.clone(), proposer, description).unwrap();

    // Try to execute before timelock. Should fail.
    let result = dao::execute_proposal(env, proposal_id);
    assert_eq!(result, Err(DaoError::TimelockNotExpired));
}

#[test]
fn test_execute_proposal_quorum_not_met() {
    let (env, admin) = setup_env();
    let staking = Address::generate(&env);
    let governance = Address::generate(&env);

    dao::initialize(
        env.clone(),
        admin,
        staking,
        governance,
        1000,
        100, // Voting period
        50,  // Timelock
        500_000,
    )
    .unwrap();

    let proposer = Address::generate(&env);
    let description = String::from_slice(&env, "Proposal");

    let proposal_id = dao::create_proposal(env.clone(), proposer, description).unwrap();

    // No votes cast (quorum not met).

    // Advance past voting window + timelock.
    advance_ledger(&env, 151);

    // Execute. Proposal should be marked Failed.
    let result = dao::execute_proposal(env.clone(), proposal_id);
    // This fails because status is not Passed.
    assert_eq!(result, Err(DaoError::InvalidStatus));

    // Verify status is Failed.
    let proposal = dao::get_proposal(env, proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Failed);
}

#[test]
fn test_cancel_proposal() {
    let (env, admin) = setup_env();
    let staking = Address::generate(&env);
    let governance = Address::generate(&env);

    dao::initialize(
        env.clone(),
        admin.clone(),
        staking,
        governance,
        1000,
        100,
        50,
        500_000,
    )
    .unwrap();

    let proposer = Address::generate(&env);
    let description = String::from_slice(&env, "Proposal");

    let proposal_id = dao::create_proposal(env.clone(), proposer.clone(), description).unwrap();

    // Proposer cancels.
    let result = dao::cancel_proposal(env.clone(), proposer, proposal_id);
    assert!(result.is_ok());

    let proposal = dao::get_proposal(env, proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Cancelled);
}

#[test]
fn test_cancel_proposal_by_admin() {
    let (env, admin) = setup_env();
    let staking = Address::generate(&env);
    let governance = Address::generate(&env);

    dao::initialize(
        env.clone(),
        admin.clone(),
        staking,
        governance,
        1000,
        100,
        50,
        500_000,
    )
    .unwrap();

    let proposer = Address::generate(&env);
    let description = String::from_slice(&env, "Proposal");

    let proposal_id = dao::create_proposal(env.clone(), proposer, description).unwrap();

    // Admin cancels (not the proposer).
    let result = dao::cancel_proposal(env.clone(), admin, proposal_id);
    assert!(result.is_ok());

    let proposal = dao::get_proposal(env, proposal_id).unwrap();
    assert_eq!(proposal.status, ProposalStatus::Cancelled);
}

#[test]
fn test_cancel_by_third_party_rejected() {
    let (env, admin) = setup_env();
    let staking = Address::generate(&env);
    let governance = Address::generate(&env);

    dao::initialize(
        env.clone(),
        admin,
        staking,
        governance,
        1000,
        100,
        50,
        500_000,
    )
    .unwrap();

    let proposer = Address::generate(&env);
    let third_party = Address::generate(&env);
    let description = String::from_slice(&env, "Proposal");

    let proposal_id = dao::create_proposal(env.clone(), proposer, description).unwrap();

    // Third party tries to cancel (not proposer or admin).
    let result = dao::cancel_proposal(env, third_party, proposal_id);
    assert_eq!(result, Err(DaoError::NotProposer));
}

#[test]
fn test_treasury_transfer_unauthorized_caller_rejected() {
    let (env, admin) = setup_env();
    let staking = Address::generate(&env);
    let governance = Address::generate(&env);

    dao::initialize(
        env.clone(),
        admin,
        staking,
        governance,
        1000,
        100,
        50,
        500_000,
    )
    .unwrap();

    let recipient = Address::generate(&env);
    let unauthorized = Address::generate(&env);

    // Try to call treasury_transfer as an unauthorized address.
    let result = dao::treasury_transfer(env, unauthorized, recipient, 1_000_000);
    assert_eq!(result, Err(DaoError::UnauthorizedCaller));
}
