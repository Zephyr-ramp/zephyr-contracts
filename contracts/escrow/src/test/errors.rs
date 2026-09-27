use soroban_sdk::testutils::Address as _;

use super::*;
use crate::Error;

#[test]
fn initialize_twice_fails() {
    let s = Setup::new();
    let res = s
        .escrow
        .try_initialize(&s.admin, &s.anchor, &s.token.address, &1, &2);
    assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn initialize_rejects_bad_timeout_bounds() {
    let s = Setup::uninitialized();
    let init = |min: u32, max: u32| {
        s.escrow
            .try_initialize(&s.admin, &s.anchor, &s.token.address, &min, &max)
    };
    assert_eq!(init(0, 10), Err(Ok(Error::InvalidTimeout)));
    assert_eq!(init(11, 10), Err(Ok(Error::InvalidTimeout)));
    assert_eq!(init(1, u32::MAX), Err(Ok(Error::InvalidTimeout)));
    assert_eq!(init(10, 10), Ok(Ok(())));
}

#[test]
fn calls_before_initialize_fail_with_not_initialized() {
    let s = Setup::uninitialized();
    let tx_id = s.tx_id(1);
    assert_eq!(
        s.escrow.try_deposit(&s.user, &tx_id, &1, &MIN_TIMEOUT),
        Err(Ok(Error::NotInitialized))
    );
    assert_eq!(s.escrow.try_claim(&tx_id), Err(Ok(Error::NotInitialized)));
    assert_eq!(s.escrow.try_cancel(&tx_id), Err(Ok(Error::NotInitialized)));
    assert_eq!(s.escrow.try_refund(&tx_id), Err(Ok(Error::NotInitialized)));
    assert_eq!(s.escrow.try_get_config(), Err(Ok(Error::NotInitialized)));
    assert_eq!(
        s.escrow.try_set_anchor(&s.anchor),
        Err(Ok(Error::NotInitialized))
    );
    assert_eq!(
        s.escrow.try_set_timeouts(&1, &2),
        Err(Ok(Error::NotInitialized))
    );
    assert_eq!(s.escrow.try_pause(), Err(Ok(Error::NotInitialized)));
    assert_eq!(s.escrow.try_unpause(), Err(Ok(Error::NotInitialized)));
}

#[test]
fn deposit_rejects_non_positive_amounts() {
    let s = Setup::new();
    for amount in [0i128, -1, i128::MIN] {
        let res = s
            .escrow
            .try_deposit(&s.user, &s.tx_id(1), &amount, &MIN_TIMEOUT);
        assert_eq!(res, Err(Ok(Error::InvalidAmount)));
    }
    assert_eq!(s.contract_balance(), 0);
}

#[test]
fn deposit_rejects_timeouts_out_of_bounds() {
    let s = Setup::new();
    let dep = |t: u32| s.escrow.try_deposit(&s.user, &s.tx_id(1), &1, &t);
    assert_eq!(dep(MIN_TIMEOUT - 1), Err(Ok(Error::InvalidTimeout)));
    assert_eq!(dep(MAX_TIMEOUT + 1), Err(Ok(Error::InvalidTimeout)));
    assert_eq!(dep(0), Err(Ok(Error::InvalidTimeout)));
}

#[test]
fn deposit_accepts_timeout_bounds_inclusive() {
    let s = Setup::new();
    s.lock(1, 1, MIN_TIMEOUT);
    s.lock(2, 1, MAX_TIMEOUT);
}

#[test]
fn deposit_rejects_duplicate_tx_id_in_every_status() {
    let s = Setup::new();
    let locked = s.lock(1, 1, MIN_TIMEOUT);
    let claimed = s.lock(2, 1, MIN_TIMEOUT);
    let refunded = s.lock(3, 1, MIN_TIMEOUT);
    s.escrow.claim(&claimed);
    s.escrow.cancel(&refunded);

    for tx_id in [locked, claimed, refunded] {
        let res = s.escrow.try_deposit(&s.user, &tx_id, &1, &MIN_TIMEOUT);
        assert_eq!(res, Err(Ok(Error::DuplicateTxId)));
    }
}

#[test]
fn duplicate_tx_id_from_another_user_is_rejected() {
    let s = Setup::new();
    let tx_id = s.lock(1, 1, MIN_TIMEOUT);
    let other = Address::generate(&s.env);
    s.token_admin.mint(&other, &10);
    let res = s.escrow.try_deposit(&other, &tx_id, &1, &MIN_TIMEOUT);
    assert_eq!(res, Err(Ok(Error::DuplicateTxId)));
}

#[test]
fn deposit_while_paused_fails() {
    let s = Setup::new();
    s.escrow.pause();
    let res = s.escrow.try_deposit(&s.user, &s.tx_id(1), &1, &MIN_TIMEOUT);
    assert_eq!(res, Err(Ok(Error::Paused)));
}

#[test]
fn deposit_more_than_balance_fails_and_leaves_no_escrow() {
    let s = Setup::new();
    let res = s
        .escrow
        .try_deposit(&s.user, &s.tx_id(1), &(USER_FUNDS + 1), &MIN_TIMEOUT);
    assert!(res.is_err());
    // The whole invocation rolled back, including the escrow record.
    assert_eq!(
        s.escrow.try_get_escrow(&s.tx_id(1)),
        Err(Ok(Error::NotFound))
    );
}

#[test]
fn unknown_tx_id_is_not_found() {
    let s = Setup::new();
    let missing = s.tx_id(9);
    assert_eq!(s.escrow.try_get_escrow(&missing), Err(Ok(Error::NotFound)));
    assert_eq!(s.escrow.try_claim(&missing), Err(Ok(Error::NotFound)));
    assert_eq!(s.escrow.try_cancel(&missing), Err(Ok(Error::NotFound)));
    assert_eq!(s.escrow.try_refund(&missing), Err(Ok(Error::NotFound)));
}

#[test]
fn double_claim_fails() {
    let s = Setup::new();
    let tx_id = s.lock(1, 100, MIN_TIMEOUT);
    s.escrow.claim(&tx_id);
    assert_eq!(s.escrow.try_claim(&tx_id), Err(Ok(Error::NotLocked)));
    assert_eq!(s.token.balance(&s.anchor), 100);
}

#[test]
fn double_refund_fails() {
    let s = Setup::new();
    let tx_id = s.lock(1, 100, MIN_TIMEOUT);
    s.advance(MIN_TIMEOUT);
    s.escrow.refund(&tx_id);
    assert_eq!(s.escrow.try_refund(&tx_id), Err(Ok(Error::NotLocked)));
    assert_eq!(s.token.balance(&s.user), USER_FUNDS);
}

#[test]
fn settled_escrows_cannot_change_state_again() {
    let s = Setup::new();
    let claimed = s.lock(1, 100, MIN_TIMEOUT);
    let cancelled = s.lock(2, 100, MIN_TIMEOUT);
    s.escrow.claim(&claimed);
    s.escrow.cancel(&cancelled);
    s.advance(MIN_TIMEOUT);

    for tx_id in [claimed, cancelled] {
        assert_eq!(s.escrow.try_claim(&tx_id), Err(Ok(Error::NotLocked)));
        assert_eq!(s.escrow.try_cancel(&tx_id), Err(Ok(Error::NotLocked)));
        assert_eq!(s.escrow.try_refund(&tx_id), Err(Ok(Error::NotLocked)));
    }
}

#[test]
fn set_timeouts_rejects_bad_bounds() {
    let s = Setup::new();
    assert_eq!(
        s.escrow.try_set_timeouts(&0, &5),
        Err(Ok(Error::InvalidTimeout))
    );
    assert_eq!(
        s.escrow.try_set_timeouts(&6, &5),
        Err(Ok(Error::InvalidTimeout))
    );
    assert_eq!(
        s.escrow.try_set_timeouts(&1, &u32::MAX),
        Err(Ok(Error::InvalidTimeout))
    );
}
