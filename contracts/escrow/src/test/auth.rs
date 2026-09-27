//! Each entry point must require exactly the right signer, and nobody else's
//! signature may substitute for it.

use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, MockAuth, MockAuthInvoke},
    Address, IntoVal, Symbol, Val, Vec,
};

use super::*;

fn invocation(
    s: &Setup,
    contract: &Address,
    fn_name: &str,
    args: Vec<Val>,
) -> AuthorizedInvocation {
    AuthorizedInvocation {
        function: AuthorizedFunction::Contract((
            contract.clone(),
            Symbol::new(&s.env, fn_name),
            args,
        )),
        sub_invocations: std::vec![],
    }
}

/// Authorises `signer` (and only `signer`) for `fn_name(args)` on the escrow.
fn only<'a>(
    s: &'a Setup,
    signer: &'a Address,
    fn_name: &'a str,
    args: Vec<Val>,
) -> std::vec::Vec<MockAuth<'a>> {
    let invoke = std::boxed::Box::leak(std::boxed::Box::new(MockAuthInvoke {
        contract: &s.escrow.address,
        fn_name,
        args,
        sub_invokes: &[],
    }));
    std::vec![MockAuth {
        address: signer,
        invoke,
    }]
}

#[test]
fn deposit_requires_user_auth_covering_the_token_transfer() {
    let s = Setup::new();
    let tx_id = s.tx_id(1);
    s.escrow.deposit(&s.user, &tx_id, &100, &MIN_TIMEOUT);

    let mut expected = invocation(
        &s,
        &s.escrow.address,
        "deposit",
        (&s.user, &tx_id, 100i128, MIN_TIMEOUT).into_val(&s.env),
    );
    expected.sub_invocations = std::vec![invocation(
        &s,
        &s.token.address,
        "transfer",
        (&s.user, &s.escrow.address, 100i128).into_val(&s.env),
    )];
    assert_eq!(s.env.auths(), std::vec![(s.user.clone(), expected)]);
}

#[test]
fn deposit_signed_by_someone_else_fails() {
    let s = Setup::new();
    let tx_id = s.tx_id(1);
    let thief = Address::generate(&s.env);
    let args = (&s.user, &tx_id, 100i128, MIN_TIMEOUT).into_val(&s.env);

    let res = s
        .escrow
        .mock_auths(&only(&s, &thief, "deposit", args))
        .try_deposit(&s.user, &tx_id, &100, &MIN_TIMEOUT);

    assert!(res.is_err());
    assert_eq!(s.token.balance(&s.user), USER_FUNDS);
}

#[test]
fn claim_requires_anchor_auth() {
    let s = Setup::new();
    let tx_id = s.lock(1, 100, MIN_TIMEOUT);
    s.escrow.claim(&tx_id);
    assert_eq!(
        s.env.auths(),
        std::vec![(
            s.anchor.clone(),
            invocation(&s, &s.escrow.address, "claim", (&tx_id,).into_val(&s.env))
        )]
    );
}

#[test]
fn claim_by_non_anchor_fails() {
    let s = Setup::new();
    let tx_id = s.lock(1, 100, MIN_TIMEOUT);
    for signer in [s.user.clone(), s.admin.clone()] {
        let res = s
            .escrow
            .mock_auths(&only(&s, &signer, "claim", (&tx_id,).into_val(&s.env)))
            .try_claim(&tx_id);
        assert!(res.is_err());
    }
    assert_eq!(s.contract_balance(), 100);
}

#[test]
fn cancel_requires_anchor_auth() {
    let s = Setup::new();
    let tx_id = s.lock(1, 100, MIN_TIMEOUT);
    s.escrow.cancel(&tx_id);
    assert_eq!(
        s.env.auths(),
        std::vec![(
            s.anchor.clone(),
            invocation(&s, &s.escrow.address, "cancel", (&tx_id,).into_val(&s.env))
        )]
    );
}

#[test]
fn cancel_by_non_anchor_fails() {
    let s = Setup::new();
    let tx_id = s.lock(1, 100, MIN_TIMEOUT);
    for signer in [s.user.clone(), s.admin.clone()] {
        let res = s
            .escrow
            .mock_auths(&only(&s, &signer, "cancel", (&tx_id,).into_val(&s.env)))
            .try_cancel(&tx_id);
        assert!(res.is_err());
    }
    assert_eq!(s.contract_balance(), 100);
}

#[test]
fn refund_needs_no_auth_and_pays_only_the_user() {
    let s = Setup::new();
    let tx_id = s.lock(1, 100, MIN_TIMEOUT);
    s.advance(MIN_TIMEOUT);

    // No mocked auths at all: a stranger (or a keeper bot) can trigger it.
    s.escrow.mock_auths(&[]).refund(&tx_id);

    assert_eq!(s.env.auths(), std::vec![]);
    assert_eq!(s.token.balance(&s.user), USER_FUNDS);
}

#[test]
fn admin_functions_require_admin_auth() {
    let s = Setup::new();
    let new_anchor = Address::generate(&s.env);

    s.escrow.set_anchor(&new_anchor);
    assert_eq!(
        s.env.auths(),
        std::vec![(
            s.admin.clone(),
            invocation(
                &s,
                &s.escrow.address,
                "set_anchor",
                (&new_anchor,).into_val(&s.env)
            )
        )]
    );

    s.escrow.set_timeouts(&5, &50);
    assert_eq!(
        s.env.auths(),
        std::vec![(
            s.admin.clone(),
            invocation(
                &s,
                &s.escrow.address,
                "set_timeouts",
                (5u32, 50u32).into_val(&s.env)
            )
        )]
    );

    s.escrow.pause();
    assert_eq!(
        s.env.auths(),
        std::vec![(
            s.admin.clone(),
            invocation(&s, &s.escrow.address, "pause", ().into_val(&s.env))
        )]
    );

    s.escrow.unpause();
    assert_eq!(
        s.env.auths(),
        std::vec![(
            s.admin.clone(),
            invocation(&s, &s.escrow.address, "unpause", ().into_val(&s.env))
        )]
    );
}

#[test]
fn admin_functions_reject_non_admin() {
    let s = Setup::new();
    let intruder = Address::generate(&s.env);
    let e = &s.env;

    let res = s
        .escrow
        .mock_auths(&only(&s, &s.anchor, "set_anchor", (&intruder,).into_val(e)))
        .try_set_anchor(&intruder);
    assert!(res.is_err());

    let res = s
        .escrow
        .mock_auths(&only(
            &s,
            &intruder,
            "set_timeouts",
            (1u32, 2u32).into_val(e),
        ))
        .try_set_timeouts(&1, &2);
    assert!(res.is_err());

    let res = s
        .escrow
        .mock_auths(&only(&s, &s.user, "pause", ().into_val(e)))
        .try_pause();
    assert!(res.is_err());

    let res = s
        .escrow
        .mock_auths(&only(&s, &s.anchor, "unpause", ().into_val(e)))
        .try_unpause();
    assert!(res.is_err());

    let hash = BytesN::from_array(e, &[7; 32]);
    let res = s
        .escrow
        .mock_auths(&only(&s, &intruder, "upgrade", (&hash,).into_val(e)))
        .try_upgrade(&hash);
    assert!(res.is_err());

    assert_eq!(s.escrow.get_config().anchor, s.anchor);
    assert!(!s.escrow.get_config().paused);
}
