//! Test suite for the escrow contract. Everything runs in the in-process
//! Soroban host provided by `soroban-sdk` testutils; no network needed.
//!
//! Amounts are written as `whole_fraction` stroops (e.g. `100_0000000` is 100
//! USDC with 7 decimals), which clippy reads as inconsistent grouping.
#![allow(clippy::inconsistent_digit_grouping)]

mod admin;
mod auth;
mod errors;
mod expiry;
mod flows;
mod invariant;

use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token::{StellarAssetClient, TokenClient},
    Address, BytesN, Env,
};

use crate::{EscrowContract, EscrowContractClient};

pub const MIN_TIMEOUT: u32 = 100;
pub const MAX_TIMEOUT: u32 = 10_000;
pub const START_LEDGER: u32 = 1_000;
/// 1,000 USDC in stroops (7 decimals).
pub const USER_FUNDS: i128 = 1_000_0000000;

pub struct Setup {
    pub env: Env,
    pub escrow: EscrowContractClient<'static>,
    pub token: TokenClient<'static>,
    pub token_admin: StellarAssetClient<'static>,
    pub admin: Address,
    pub anchor: Address,
    pub user: Address,
}

impl Setup {
    /// A deployed, initialised escrow with a funded user. All auths are mocked.
    pub fn new() -> Self {
        let s = Self::uninitialized();
        s.escrow.initialize(
            &s.admin,
            &s.anchor,
            &s.token.address,
            &MIN_TIMEOUT,
            &MAX_TIMEOUT,
        );
        s
    }

    /// Same as [`Setup::new`] but `initialize` has not been called.
    pub fn uninitialized() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_sequence_number(START_LEDGER);

        let issuer = Address::generate(&env);
        let sac = env.register_stellar_asset_contract_v2(issuer);
        let token = TokenClient::new(&env, &sac.address());
        let token_admin = StellarAssetClient::new(&env, &sac.address());

        let admin = Address::generate(&env);
        let anchor = Address::generate(&env);
        let user = Address::generate(&env);
        token_admin.mint(&user, &USER_FUNDS);

        let contract_id = env.register(EscrowContract, ());
        let escrow = EscrowContractClient::new(&env, &contract_id);

        Setup {
            env,
            escrow,
            token,
            token_admin,
            admin,
            anchor,
            user,
        }
    }

    pub fn tx_id(&self, n: u8) -> BytesN<32> {
        BytesN::from_array(&self.env, &[n; 32])
    }

    pub fn advance(&self, ledgers: u32) {
        let seq = self.env.ledger().sequence();
        self.env.ledger().set_sequence_number(seq + ledgers);
    }

    pub fn set_ledger(&self, seq: u32) {
        self.env.ledger().set_sequence_number(seq);
    }

    pub fn contract_balance(&self) -> i128 {
        self.token.balance(&self.escrow.address)
    }

    /// Deposits `amount` from `self.user` with `timeout`, returning the tx_id.
    pub fn lock(&self, n: u8, amount: i128, timeout: u32) -> BytesN<32> {
        let tx_id = self.tx_id(n);
        self.escrow.deposit(&self.user, &tx_id, &amount, &timeout);
        tx_id
    }
}
