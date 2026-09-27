//! # Zephyr withdrawal escrow
//!
//! A user locks USDC for one specific Zephyr withdrawal (`tx_id`). The anchor can
//! only take it with `claim` after paying out fiat, and only before the escrow
//! expires. From `expires_ledger` on, anyone can `refund` it to the user.
//!
//! ```text
//!             deposit            claim (anchor, before expiry)
//!   (none) ───────────▶ Locked ─────────────────────────────▶ Claimed
//!                          │
//!                          │ cancel (anchor, any time)
//!                          │ refund (anyone, at/after expiry)
//!                          ▼
//!                       Refunded
//! ```
//!
//! Pausing blocks new deposits only. It never blocks `claim`, `cancel` or
//! `refund`, so funds can always leave the contract.
#![no_std]

mod error;
mod events;
mod storage;
mod types;

#[cfg(test)]
extern crate std;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, token, Address, BytesN, ContractExecutable, Env};

pub use crate::error::Error;
pub use crate::types::{Config, DataKey, Escrow, EscrowStatus};

use crate::events::{
    AnchorUpdated, Claimed, Initialized, Locked, PausedChanged, Refunded, TimeoutsUpdated, Upgraded,
};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Stores the configuration. Can only be called once.
    ///
    /// `min_timeout`/`max_timeout` bound the `timeout_ledgers` a user may pick
    /// in `deposit`. `token` is the USDC Stellar Asset Contract address.
    pub fn initialize(
        env: Env,
        admin: Address,
        anchor: Address,
        token: Address,
        min_timeout: u32,
        max_timeout: u32,
    ) -> Result<(), Error> {
        if storage::has_config(&env) {
            return Err(Error::AlreadyInitialized);
        }
        validate_timeouts(&env, min_timeout, max_timeout)?;

        storage::write_config(
            &env,
            &Config {
                admin: admin.clone(),
                anchor: anchor.clone(),
                token: token.clone(),
                min_timeout_ledgers: min_timeout,
                max_timeout_ledgers: max_timeout,
                paused: false,
            },
        );
        Initialized {
            admin,
            anchor,
            token,
        }
        .publish(&env);
        Ok(())
    }

    /// Locks `amount` of the configured token from `user` for withdrawal `tx_id`.
    ///
    /// `tx_id` is `sha256(zephyr transaction uuid)`. The escrow expires
    /// `timeout_ledgers` ledgers from now.
    pub fn deposit(
        env: Env,
        user: Address,
        tx_id: BytesN<32>,
        amount: i128,
        timeout_ledgers: u32,
    ) -> Result<(), Error> {
        user.require_auth();
        let config = storage::read_config(&env)?;

        if config.paused {
            return Err(Error::Paused);
        }
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }
        if timeout_ledgers < config.min_timeout_ledgers
            || timeout_ledgers > config.max_timeout_ledgers
        {
            return Err(Error::InvalidTimeout);
        }
        if storage::has_escrow(&env, &tx_id) {
            return Err(Error::DuplicateTxId);
        }

        let now = env.ledger().sequence();
        let expires_ledger = now
            .checked_add(timeout_ledgers)
            .ok_or(Error::InvalidTimeout)?;

        // Effects before interactions: record the escrow, then pull the funds.
        let escrow = Escrow {
            user: user.clone(),
            amount,
            created_ledger: now,
            expires_ledger,
            status: EscrowStatus::Locked,
        };
        storage::write_escrow(&env, &tx_id, &escrow);
        storage::extend_instance(&env);

        token::Client::new(&env, &config.token).transfer(
            &user,
            env.current_contract_address(),
            &amount,
        );

        Locked {
            tx_id,
            user,
            amount,
            expires_ledger,
        }
        .publish(&env);
        Ok(())
    }

    /// Anchor takes the funds after paying out fiat. Only while `Locked` and
    /// strictly before `expires_ledger`.
    pub fn claim(env: Env, tx_id: BytesN<32>) -> Result<(), Error> {
        let config = storage::read_config(&env)?;
        config.anchor.require_auth();

        let mut escrow = locked_escrow(&env, &tx_id)?;
        if env.ledger().sequence() >= escrow.expires_ledger {
            return Err(Error::Expired);
        }

        escrow.status = EscrowStatus::Claimed;
        storage::write_escrow(&env, &tx_id, &escrow);
        storage::extend_instance(&env);

        token::Client::new(&env, &config.token).transfer(
            &env.current_contract_address(),
            &config.anchor,
            &escrow.amount,
        );

        Claimed {
            tx_id,
            anchor: config.anchor,
            amount: escrow.amount,
        }
        .publish(&env);
        Ok(())
    }

    /// Anchor gives up on the withdrawal (e.g. the fiat payout failed) and
    /// returns the funds to the user. Allowed any time while `Locked`.
    pub fn cancel(env: Env, tx_id: BytesN<32>) -> Result<(), Error> {
        let config = storage::read_config(&env)?;
        config.anchor.require_auth();

        let escrow = locked_escrow(&env, &tx_id)?;
        release_to_user(&env, &config, tx_id, escrow, true);
        Ok(())
    }

    /// Returns the funds to the user once the escrow has expired unclaimed.
    /// Anyone may call it (the funds can only go to the original user), and it
    /// works while the contract is paused.
    pub fn refund(env: Env, tx_id: BytesN<32>) -> Result<(), Error> {
        let config = storage::read_config(&env)?;

        let escrow = locked_escrow(&env, &tx_id)?;
        if env.ledger().sequence() < escrow.expires_ledger {
            return Err(Error::NotExpired);
        }
        release_to_user(&env, &config, tx_id, escrow, false);
        Ok(())
    }

    pub fn get_escrow(env: Env, tx_id: BytesN<32>) -> Result<Escrow, Error> {
        storage::read_escrow(&env, &tx_id)
    }

    pub fn get_config(env: Env) -> Result<Config, Error> {
        storage::read_config(&env)
    }

    /// Changes the anchor. Existing `Locked` escrows become claimable/cancellable
    /// by the new anchor only.
    pub fn set_anchor(env: Env, anchor: Address) -> Result<(), Error> {
        let mut config = admin_config(&env)?;
        config.anchor = anchor.clone();
        storage::write_config(&env, &config);
        AnchorUpdated { anchor }.publish(&env);
        Ok(())
    }

    /// Changes the timeout bounds for new deposits. Existing escrows keep their
    /// `expires_ledger`.
    pub fn set_timeouts(env: Env, min_timeout: u32, max_timeout: u32) -> Result<(), Error> {
        let mut config = admin_config(&env)?;
        validate_timeouts(&env, min_timeout, max_timeout)?;
        config.min_timeout_ledgers = min_timeout;
        config.max_timeout_ledgers = max_timeout;
        storage::write_config(&env, &config);
        TimeoutsUpdated {
            min_timeout_ledgers: min_timeout,
            max_timeout_ledgers: max_timeout,
        }
        .publish(&env);
        Ok(())
    }

    /// Blocks new deposits. Claims, cancels and refunds keep working.
    pub fn pause(env: Env) -> Result<(), Error> {
        set_paused(&env, true)
    }

    pub fn unpause(env: Env) -> Result<(), Error> {
        set_paused(&env, false)
    }

    /// Replaces the contract code. Storage (config and escrows) is kept.
    pub fn upgrade(env: Env, wasm_hash: BytesN<32>) -> Result<(), Error> {
        admin_config(&env)?;
        env.deployer()
            .update_current_contract(ContractExecutable::Wasm(wasm_hash.clone()));
        Upgraded { wasm_hash }.publish(&env);
        Ok(())
    }
}

fn validate_timeouts(env: &Env, min_timeout: u32, max_timeout: u32) -> Result<(), Error> {
    // An escrow must outlive its timeout, so the timeout cannot exceed the
    // longest TTL the network allows for a storage entry.
    if min_timeout == 0 || min_timeout > max_timeout || max_timeout > env.storage().max_ttl() {
        return Err(Error::InvalidTimeout);
    }
    Ok(())
}

/// Loads the config and requires the admin's authorisation.
fn admin_config(env: &Env) -> Result<Config, Error> {
    let config = storage::read_config(env)?;
    config.admin.require_auth();
    Ok(config)
}

fn set_paused(env: &Env, paused: bool) -> Result<(), Error> {
    let mut config = admin_config(env)?;
    config.paused = paused;
    storage::write_config(env, &config);
    PausedChanged { paused }.publish(env);
    Ok(())
}

fn locked_escrow(env: &Env, tx_id: &BytesN<32>) -> Result<Escrow, Error> {
    let escrow = storage::read_escrow(env, tx_id)?;
    if escrow.status != EscrowStatus::Locked {
        return Err(Error::NotLocked);
    }
    Ok(escrow)
}

fn release_to_user(
    env: &Env,
    config: &Config,
    tx_id: BytesN<32>,
    mut escrow: Escrow,
    by_anchor: bool,
) {
    escrow.status = EscrowStatus::Refunded;
    storage::write_escrow(env, &tx_id, &escrow);
    storage::extend_instance(env);

    token::Client::new(env, &config.token).transfer(
        &env.current_contract_address(),
        &escrow.user,
        &escrow.amount,
    );

    Refunded {
        tx_id,
        user: escrow.user,
        amount: escrow.amount,
        by_anchor,
    }
    .publish(env);
}
