//! Boundary conditions around `expires_ledger`: claim is allowed strictly
//! before it, refund at or after it.

use super::*;
use crate::{Error, EscrowStatus};

const TIMEOUT: u32 = 500;
const EXPIRES: u32 = START_LEDGER + TIMEOUT;

#[test]
fn claim_one_ledger_before_expiry_succeeds() {
    let s = Setup::new();
    let tx_id = s.lock(1, 100, TIMEOUT);
    s.set_ledger(EXPIRES - 1);
    s.escrow.claim(&tx_id);
    assert_eq!(s.escrow.get_escrow(&tx_id).status, EscrowStatus::Claimed);
}

#[test]
fn claim_exactly_at_expiry_fails() {
    let s = Setup::new();
    let tx_id = s.lock(1, 100, TIMEOUT);
    s.set_ledger(EXPIRES);
    assert_eq!(s.escrow.try_claim(&tx_id), Err(Ok(Error::Expired)));
}

#[test]
fn claim_one_ledger_after_expiry_fails() {
    let s = Setup::new();
    let tx_id = s.lock(1, 100, TIMEOUT);
    s.set_ledger(EXPIRES + 1);
    assert_eq!(s.escrow.try_claim(&tx_id), Err(Ok(Error::Expired)));
}

#[test]
fn refund_one_ledger_before_expiry_fails() {
    let s = Setup::new();
    let tx_id = s.lock(1, 100, TIMEOUT);
    s.set_ledger(EXPIRES - 1);
    assert_eq!(s.escrow.try_refund(&tx_id), Err(Ok(Error::NotExpired)));
}

#[test]
fn refund_immediately_after_deposit_fails() {
    let s = Setup::new();
    let tx_id = s.lock(1, 100, TIMEOUT);
    assert_eq!(s.escrow.try_refund(&tx_id), Err(Ok(Error::NotExpired)));
}

#[test]
fn refund_exactly_at_expiry_succeeds() {
    let s = Setup::new();
    let tx_id = s.lock(1, 100, TIMEOUT);
    s.set_ledger(EXPIRES);
    s.escrow.refund(&tx_id);
    assert_eq!(s.escrow.get_escrow(&tx_id).status, EscrowStatus::Refunded);
    assert_eq!(s.token.balance(&s.user), USER_FUNDS);
}

#[test]
fn refund_one_ledger_after_expiry_succeeds() {
    let s = Setup::new();
    let tx_id = s.lock(1, 100, TIMEOUT);
    s.set_ledger(EXPIRES + 1);
    s.escrow.refund(&tx_id);
    assert_eq!(s.escrow.get_escrow(&tx_id).status, EscrowStatus::Refunded);
}

#[test]
fn cancel_works_before_and_after_expiry() {
    let s = Setup::new();
    let before = s.lock(1, 100, TIMEOUT);
    let after = s.lock(2, 100, TIMEOUT);
    s.set_ledger(EXPIRES - 1);
    s.escrow.cancel(&before);
    s.set_ledger(EXPIRES + 1);
    s.escrow.cancel(&after);
    assert_eq!(s.token.balance(&s.user), USER_FUNDS);
}

#[test]
fn changing_timeouts_does_not_move_existing_expiry() {
    let s = Setup::new();
    let tx_id = s.lock(1, 100, TIMEOUT);
    s.escrow.set_timeouts(&1, &2);
    assert_eq!(s.escrow.get_escrow(&tx_id).expires_ledger, EXPIRES);
}
