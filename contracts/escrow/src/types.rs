use soroban_sdk::{contracttype, Address, BytesN};

/// Contract configuration, kept in instance storage.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    /// Can change the anchor, timeouts and pause state, and upgrade the contract.
    pub admin: Address,
    /// The only account allowed to `claim` or `cancel` escrows.
    pub anchor: Address,
    /// The token held in escrow (the USDC Stellar Asset Contract).
    pub token: Address,
    /// Shortest timeout a user may choose for a deposit, in ledgers.
    pub min_timeout_ledgers: u32,
    /// Longest timeout a user may choose for a deposit, in ledgers.
    pub max_timeout_ledgers: u32,
    /// When true, `deposit` is rejected. Nothing else is affected.
    pub paused: bool,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EscrowStatus {
    /// Funds are held by the contract.
    Locked,
    /// The anchor paid out fiat and took the funds.
    Claimed,
    /// Funds went back to the user (anchor `cancel` or user `refund`).
    Refunded,
}

/// One withdrawal escrow, kept in persistent storage under its `tx_id`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Escrow {
    pub user: Address,
    /// Amount in the token's smallest unit (stroops for USDC: 7 decimals).
    pub amount: i128,
    pub created_ledger: u32,
    /// From this ledger on, the anchor can no longer claim and anyone can refund.
    pub expires_ledger: u32,
    pub status: EscrowStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Instance storage: [`Config`].
    Config,
    /// Persistent storage: [`Escrow`], keyed by `sha256(zephyr transaction uuid)`.
    Escrow(BytesN<32>),
}
