use soroban_sdk::testutils::Address as _;

use super::*;
use crate::{Error, EscrowStatus};

#[test]
fn set_anchor_moves_claim_rights_and_payouts() {
    let s = Setup::new();
    let tx_id = s.lock(1, 100, MIN_TIMEOUT);
    let new_anchor = Address::generate(&s.env);

    s.escrow.set_anchor(&new_anchor);
    s.escrow.claim(&tx_id);

    assert_eq!(s.escrow.get_config().anchor, new_anchor);
    assert_eq!(s.token.balance(&new_anchor), 100);
    assert_eq!(s.token.balance(&s.anchor), 0);
}

#[test]
fn set_timeouts_changes_bounds_for_new_deposits() {
    let s = Setup::new();
    s.escrow.set_timeouts(&10, &20);
    let dep = |n: u8, t: u32| s.escrow.try_deposit(&s.user, &s.tx_id(n), &1, &t);
    assert_eq!(dep(1, 9), Err(Ok(Error::InvalidTimeout)));
    assert_eq!(dep(2, 21), Err(Ok(Error::InvalidTimeout)));
    assert_eq!(dep(3, 10), Ok(Ok(())));
    assert_eq!(dep(4, 20), Ok(Ok(())));
}

#[test]
fn pause_and_unpause_toggle_deposits() {
    let s = Setup::new();
    s.escrow.pause();
    assert!(s.escrow.get_config().paused);
    assert_eq!(
        s.escrow.try_deposit(&s.user, &s.tx_id(1), &1, &MIN_TIMEOUT),
        Err(Ok(Error::Paused))
    );
    s.escrow.unpause();
    assert!(!s.escrow.get_config().paused);
    s.lock(1, 1, MIN_TIMEOUT);
}

#[test]
fn refund_still_works_while_paused() {
    let s = Setup::new();
    let tx_id = s.lock(1, 100, MIN_TIMEOUT);
    s.escrow.pause();
    s.advance(MIN_TIMEOUT);

    s.escrow.refund(&tx_id);

    assert_eq!(s.escrow.get_escrow(&tx_id).status, EscrowStatus::Refunded);
    assert_eq!(s.token.balance(&s.user), USER_FUNDS);
}

#[test]
fn claim_and_cancel_still_work_while_paused() {
    let s = Setup::new();
    let a = s.lock(1, 100, MIN_TIMEOUT);
    let b = s.lock(2, 100, MIN_TIMEOUT);
    s.escrow.pause();
    s.escrow.claim(&a);
    s.escrow.cancel(&b);
    assert_eq!(s.token.balance(&s.anchor), 100);
    assert_eq!(s.contract_balance(), 0);
}

/// Exercises a real upgrade when the release wasm has been built
/// (`stellar contract build`, which CI runs before the tests). Skipped otherwise.
#[test]
fn upgrade_replaces_code_and_keeps_storage() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../target/wasm32v1-none/release/zephyr_escrow.wasm"
    );
    let Ok(wasm) = std::fs::read(path) else {
        std::eprintln!("skipping upgrade test: {path} not built");
        return;
    };

    let s = Setup::new();
    let tx_id = s.lock(1, 100, MIN_TIMEOUT);
    let hash = s
        .env
        .deployer()
        .upload_contract_wasm(soroban_sdk::Bytes::from_slice(&s.env, &wasm));

    s.escrow.upgrade(&hash);

    // Storage survives the upgrade and the new code serves it.
    assert_eq!(s.escrow.get_escrow(&tx_id).amount, 100);
    assert_eq!(s.escrow.get_config().admin, s.admin);
}
