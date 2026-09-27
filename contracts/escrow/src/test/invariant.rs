//! Property test: whatever sequence of operations runs, the contract's token
//! balance always equals the sum of `Locked` escrows, and no tokens are
//! created or destroyed.

use proptest::prelude::*;
use soroban_sdk::{testutils::Address as _, Address};

use super::*;
use crate::EscrowStatus;

#[derive(Clone, Debug)]
enum Op {
    Deposit {
        user: usize,
        slot: u8,
        amount: i128,
        timeout: u32,
    },
    Claim {
        slot: u8,
    },
    Cancel {
        slot: u8,
    },
    Refund {
        slot: u8,
    },
    Advance {
        ledgers: u32,
    },
    Pause,
    Unpause,
}

const USERS: usize = 3;
const SLOTS: u8 = 8;

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        4 => (0..USERS, 0..SLOTS, -10i128..400_0000000, 0u32..MAX_TIMEOUT + 200)
            .prop_map(|(user, slot, amount, timeout)| Op::Deposit { user, slot, amount, timeout }),
        2 => (0..SLOTS).prop_map(|slot| Op::Claim { slot }),
        1 => (0..SLOTS).prop_map(|slot| Op::Cancel { slot }),
        2 => (0..SLOTS).prop_map(|slot| Op::Refund { slot }),
        2 => (0u32..MAX_TIMEOUT).prop_map(|ledgers| Op::Advance { ledgers }),
        1 => Just(Op::Pause),
        1 => Just(Op::Unpause),
    ]
}

fn cases() -> u32 {
    std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(64)
}

fn sum_locked(s: &Setup) -> i128 {
    (0..SLOTS)
        .filter_map(|n| s.escrow.try_get_escrow(&s.tx_id(n)).ok()?.ok())
        .filter(|e| e.status == EscrowStatus::Locked)
        .map(|e| e.amount)
        .sum()
}

proptest! {
    // 64 cases by default so CI stays fast; set PROPTEST_CASES for a deeper run.
    #![proptest_config(ProptestConfig { cases: cases(), ..ProptestConfig::default() })]

    #[test]
    fn balance_always_equals_sum_of_locked(ops in prop::collection::vec(op(), 1..40)) {
        let s = Setup::new();
        let mut users: std::vec::Vec<Address> = std::vec![s.user.clone()];
        for _ in 1..USERS {
            let u = Address::generate(&s.env);
            s.token_admin.mint(&u, &USER_FUNDS);
            users.push(u);
        }
        let total_supply = USER_FUNDS * USERS as i128;

        for op in ops {
            // Individual operations may legitimately fail (wrong state, bad
            // input, paused...). The invariant must hold either way.
            match op {
                Op::Deposit { user, slot, amount, timeout } => {
                    let _ = s.escrow.try_deposit(&users[user], &s.tx_id(slot), &amount, &timeout);
                }
                Op::Claim { slot } => { let _ = s.escrow.try_claim(&s.tx_id(slot)); }
                Op::Cancel { slot } => { let _ = s.escrow.try_cancel(&s.tx_id(slot)); }
                Op::Refund { slot } => { let _ = s.escrow.try_refund(&s.tx_id(slot)); }
                Op::Advance { ledgers } => s.advance(ledgers),
                Op::Pause => s.escrow.pause(),
                Op::Unpause => s.escrow.unpause(),
            }

            prop_assert_eq!(s.contract_balance(), sum_locked(&s));

            let held: i128 = users.iter().map(|u| s.token.balance(u)).sum::<i128>()
                + s.token.balance(&s.anchor)
                + s.contract_balance();
            prop_assert_eq!(held, total_supply);
        }
    }
}
