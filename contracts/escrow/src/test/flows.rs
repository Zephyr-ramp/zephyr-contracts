use soroban_sdk::{testutils::Events as _, Event as _};

use super::*;
use crate::events::{Claimed, Locked, Refunded};
use crate::EscrowStatus;

#[test]
fn deposit_locks_funds_and_emits_locked() {
    let s = Setup::new();
    let tx_id = s.tx_id(1);

    s.escrow.deposit(&s.user, &tx_id, &250_0000000, &500);
    let events = s.env.events().all().filter_by_contract(&s.escrow.address);

    let escrow = s.escrow.get_escrow(&tx_id);
    assert_eq!(escrow.user, s.user);
    assert_eq!(escrow.amount, 250_0000000);
    assert_eq!(escrow.created_ledger, START_LEDGER);
    assert_eq!(escrow.expires_ledger, START_LEDGER + 500);
    assert_eq!(escrow.status, EscrowStatus::Locked);

    assert_eq!(s.contract_balance(), 250_0000000);
    assert_eq!(s.token.balance(&s.user), USER_FUNDS - 250_0000000);

    let expected = Locked {
        tx_id,
        user: s.user.clone(),
        amount: 250_0000000,
        expires_ledger: START_LEDGER + 500,
    };
    assert_eq!(
        events,
        std::vec![expected.to_xdr(&s.env, &s.escrow.address)]
    );
}

#[test]
fn claim_pays_anchor_and_emits_claimed() {
    let s = Setup::new();
    let tx_id = s.lock(1, 100_0000000, 500);

    s.escrow.claim(&tx_id);
    let events = s.env.events().all().filter_by_contract(&s.escrow.address);

    assert_eq!(s.escrow.get_escrow(&tx_id).status, EscrowStatus::Claimed);
    assert_eq!(s.token.balance(&s.anchor), 100_0000000);
    assert_eq!(s.contract_balance(), 0);

    let expected = Claimed {
        tx_id,
        anchor: s.anchor.clone(),
        amount: 100_0000000,
    };
    assert_eq!(
        events,
        std::vec![expected.to_xdr(&s.env, &s.escrow.address)]
    );
}

#[test]
fn cancel_refunds_user_and_emits_refunded_by_anchor() {
    let s = Setup::new();
    let tx_id = s.lock(1, 100_0000000, 500);

    s.escrow.cancel(&tx_id);
    let events = s.env.events().all().filter_by_contract(&s.escrow.address);

    assert_eq!(s.escrow.get_escrow(&tx_id).status, EscrowStatus::Refunded);
    assert_eq!(s.token.balance(&s.user), USER_FUNDS);
    assert_eq!(s.contract_balance(), 0);

    let expected = Refunded {
        tx_id,
        user: s.user.clone(),
        amount: 100_0000000,
        by_anchor: true,
    };
    assert_eq!(
        events,
        std::vec![expected.to_xdr(&s.env, &s.escrow.address)]
    );
}

#[test]
fn refund_after_expiry_returns_funds_and_emits_refunded() {
    let s = Setup::new();
    let tx_id = s.lock(1, 100_0000000, 500);
    s.advance(500);

    s.escrow.refund(&tx_id);
    let events = s.env.events().all().filter_by_contract(&s.escrow.address);

    assert_eq!(s.escrow.get_escrow(&tx_id).status, EscrowStatus::Refunded);
    assert_eq!(s.token.balance(&s.user), USER_FUNDS);

    let expected = Refunded {
        tx_id,
        user: s.user.clone(),
        amount: 100_0000000,
        by_anchor: false,
    };
    assert_eq!(
        events,
        std::vec![expected.to_xdr(&s.env, &s.escrow.address)]
    );
}

#[test]
fn escrows_are_independent() {
    let s = Setup::new();
    let a = s.lock(1, 10_0000000, 500);
    let b = s.lock(2, 20_0000000, 1_000);
    let c = s.lock(3, 30_0000000, 100);

    s.escrow.claim(&a);
    s.escrow.cancel(&b);

    assert_eq!(s.escrow.get_escrow(&c).status, EscrowStatus::Locked);
    assert_eq!(s.contract_balance(), 30_0000000);
    assert_eq!(s.token.balance(&s.anchor), 10_0000000);
    assert_eq!(
        s.token.balance(&s.user),
        USER_FUNDS - 10_0000000 - 30_0000000
    );
}

#[test]
fn get_config_returns_initialised_values() {
    let s = Setup::new();
    let config = s.escrow.get_config();
    assert_eq!(config.admin, s.admin);
    assert_eq!(config.anchor, s.anchor);
    assert_eq!(config.token, s.token.address);
    assert_eq!(config.min_timeout_ledgers, MIN_TIMEOUT);
    assert_eq!(config.max_timeout_ledgers, MAX_TIMEOUT);
    assert!(!config.paused);
}

#[test]
fn active_escrow_ttl_outlives_its_expiry() {
    use soroban_sdk::testutils::storage::Persistent as _;

    let s = Setup::new();
    let tx_id = s.lock(1, 1, MAX_TIMEOUT);
    let ttl = s.env.as_contract(&s.escrow.address, || {
        s.env
            .storage()
            .persistent()
            .get_ttl(&crate::DataKey::Escrow(tx_id.clone()))
    });
    assert!(ttl >= MAX_TIMEOUT + crate::storage::ESCROW_GRACE_LEDGERS);
}
