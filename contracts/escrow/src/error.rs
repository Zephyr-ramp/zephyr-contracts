use soroban_sdk::contracterror;

/// Every failure the escrow can return. Codes are part of the public interface:
/// the backend and frontend map them to user-facing messages, so never renumber.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// `initialize` was already called.
    AlreadyInitialized = 1,
    /// The contract has not been initialised yet.
    NotInitialized = 2,
    /// Amount must be strictly positive.
    InvalidAmount = 3,
    /// Timeout is outside `[min_timeout_ledgers, max_timeout_ledgers]`, or the
    /// timeout bounds themselves are invalid.
    InvalidTimeout = 4,
    /// An escrow with this `tx_id` already exists (in any status).
    DuplicateTxId = 5,
    /// No escrow exists for this `tx_id`.
    NotFound = 6,
    /// The escrow is no longer `Locked` (already claimed or refunded).
    NotLocked = 7,
    /// The escrow reached `expires_ledger`; the anchor can no longer claim it.
    Expired = 8,
    /// The escrow has not reached `expires_ledger`; the user cannot refund yet.
    NotExpired = 9,
    /// New deposits are paused by the admin.
    Paused = 10,
}
