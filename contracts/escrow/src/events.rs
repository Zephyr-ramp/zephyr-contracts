//! Events emitted by the escrow. The backend indexes `locked`, `claimed` and
//! `refunded` through Soroban RPC `getEvents`, matching on the `tx_id` topic.

use soroban_sdk::{contractevent, Address, BytesN};

/// A user locked funds. Topics: `["locked", tx_id]`.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Locked {
    #[topic]
    pub tx_id: BytesN<32>,
    pub user: Address,
    pub amount: i128,
    pub expires_ledger: u32,
}

/// The anchor claimed the funds after paying out fiat. Topics: `["claimed", tx_id]`.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Claimed {
    #[topic]
    pub tx_id: BytesN<32>,
    pub anchor: Address,
    pub amount: i128,
}

/// Funds went back to the user. `by_anchor` is true for `cancel`, false for
/// `refund`. Topics: `["refunded", tx_id]`.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Refunded {
    #[topic]
    pub tx_id: BytesN<32>,
    pub user: Address,
    pub amount: i128,
    pub by_anchor: bool,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Initialized {
    pub admin: Address,
    pub anchor: Address,
    pub token: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnchorUpdated {
    pub anchor: Address,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeoutsUpdated {
    pub min_timeout_ledgers: u32,
    pub max_timeout_ledgers: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PausedChanged {
    pub paused: bool,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Upgraded {
    pub wasm_hash: BytesN<32>,
}
