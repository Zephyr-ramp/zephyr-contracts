//! Storage access and TTL management.
//!
//! Every write extends the TTL of what it touched, so an active escrow is never
//! archived before the user can refund it.

use soroban_sdk::{BytesN, Env};

use crate::error::Error;
use crate::types::{Config, DataKey, Escrow};

/// About 5 seconds per ledger.
pub const DAY_IN_LEDGERS: u32 = 17_280;

/// Instance storage (config + contract code) is kept alive for 30 days after
/// every call, and topped up once it drops below 29 days.
pub const INSTANCE_EXTEND_TO: u32 = 30 * DAY_IN_LEDGERS;
pub const INSTANCE_THRESHOLD: u32 = INSTANCE_EXTEND_TO - DAY_IN_LEDGERS;

/// How long an escrow entry stays live after it expires or settles. It keeps the
/// record readable for indexers and keeps `tx_id` reserved against reuse.
pub const ESCROW_GRACE_LEDGERS: u32 = 30 * DAY_IN_LEDGERS;

pub fn extend_instance(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_THRESHOLD, INSTANCE_EXTEND_TO);
}

pub fn has_config(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Config)
}

pub fn read_config(env: &Env) -> Result<Config, Error> {
    env.storage()
        .instance()
        .get(&DataKey::Config)
        .ok_or(Error::NotInitialized)
}

pub fn write_config(env: &Env, config: &Config) {
    env.storage().instance().set(&DataKey::Config, config);
    extend_instance(env);
}

pub fn has_escrow(env: &Env, tx_id: &BytesN<32>) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Escrow(tx_id.clone()))
}

pub fn read_escrow(env: &Env, tx_id: &BytesN<32>) -> Result<Escrow, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Escrow(tx_id.clone()))
        .ok_or(Error::NotFound)
}

pub fn write_escrow(env: &Env, tx_id: &BytesN<32>, escrow: &Escrow) {
    let key = DataKey::Escrow(tx_id.clone());
    let storage = env.storage().persistent();
    storage.set(&key, escrow);

    // Keep the entry alive until `expires_ledger + grace`, capped at the
    // network's maximum TTL.
    let now = env.ledger().sequence();
    let until_expiry = escrow.expires_ledger.saturating_sub(now);
    let extend_to = until_expiry
        .saturating_add(ESCROW_GRACE_LEDGERS)
        .min(env.storage().max_ttl());
    storage.extend_ttl(&key, extend_to, extend_to);
}
