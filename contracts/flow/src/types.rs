#![allow(clippy::enum_variant_names)]
use soroban_sdk::{contracterror, contracttype, Address, Bytes, BytesN, Env, String};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyHasIdentity = 1,
    NoIdentityFound = 2,
    InvalidNonce = 3,
    InvalidWindow = 4,
    InvalidRevealMode = 5,
    InvalidIncomeValue = 6,
    InvalidIncomeBand = 7,
    InvalidExternalId = 8,
    InvalidCurrency = 9,
    InsufficientPayment = 10,
    NotInitialized = 11,
    NotAdmin = 12,
    TokenNotFound = 13,
    Unauthorized = 14,
    TokenExpired = 15,
    InvalidProofFreshness = 16,
    AlreadyInitialized = 17,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum DataKey {
    Config,
}


#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Ecosystem {
    Evm,
    Cosmos,
    Solana,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct CrossChainParams {
    pub destination_chain: String,
    pub destination_address: String,
    pub user_destination_address: Bytes,
    pub ecosystem: Ecosystem,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RenewalWindow {
    Days30,
    Days60,
    Days90,
}

impl RenewalWindow {
    pub fn to_seconds(&self) -> u64 {
        match self {
            RenewalWindow::Days30 => 30 * 24 * 60 * 60,
            RenewalWindow::Days60 => 60 * 24 * 60 * 60,
            RenewalWindow::Days90 => 90 * 24 * 60 * 60,
        }
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RevealMode {
    Exact,
    Band,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct MintParams {
    pub soul_id: u32,
    pub external_id: String,
    pub income_band: u32,
    pub income_value: Option<i128>,
    pub reveal_mode: RevealMode,
    pub currency: String,
    pub verified_at: u64,
    pub proof_hash: BytesN<32>,
    pub proof_data: Bytes,
    pub window: RenewalWindow,
    pub nonce: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct UpdateParams {
    pub income_band: u32,
    pub income_value: Option<i128>,
    pub reveal_mode: RevealMode,
    pub currency: String,
    pub verified_at: u64,
    pub proof_hash: BytesN<32>,
    pub proof_data: Bytes,
    pub window: RenewalWindow,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IncomeData {
    pub soul_id: u32,
    pub external_id: String,
    pub income_band: u32,
    pub income_value: Option<i128>,
    pub reveal_mode: RevealMode,
    pub currency: String,
    pub verified_at: u64,
    pub proof_hash: BytesN<32>,
    pub proof_data: Bytes,
    pub window: RenewalWindow,
    pub minted_at: u64,
    pub updated_at: u64,
    pub expires_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct Config {
    pub admin: Address,
    pub registry: Address,
    pub fee_token: Address,
    pub access_control: Address,
    pub treasury: Address,
    pub mint_fee_30: i128,
    pub mint_fee_60: i128,
    pub mint_fee_90: i128,
    pub max_proof_age_seconds: u64,
    pub zk_verifier: Option<Address>,
    pub store_proof_data: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub version: String,
    pub data_source: String,
}

pub fn fee_for_window(config: &Config, window: &RenewalWindow) -> i128 {
    match window {
        RenewalWindow::Days30 => config.mint_fee_30,
        RenewalWindow::Days60 => config.mint_fee_60,
        RenewalWindow::Days90 => config.mint_fee_90,
    }
}

pub fn validate_income_fields(
    income_band: u32,
    income_value: &Option<i128>,
    reveal_mode: &RevealMode,
) -> Result<(), Error> {
    if income_band == 0 {
        return Err(Error::InvalidIncomeBand);
    }

    match reveal_mode {
        RevealMode::Exact => {
            if income_value.is_none() {
                return Err(Error::InvalidIncomeValue);
            }
        }
        RevealMode::Band => {
            if income_value.is_some() {
                return Err(Error::InvalidIncomeValue);
            }
        }
    }

    Ok(())
}

pub fn normalize_proof_data(env: &Env, store: bool, data: Bytes) -> Bytes {
    if store {
        data
    } else {
        Bytes::new(env)
    }
}

pub fn validate_proof_freshness(
    now: u64,
    verified_at: u64,
    max_age_seconds: u64,
) -> Result<(), Error> {
    if verified_at == 0 {
        return Err(Error::InvalidProofFreshness);
    }
    if max_age_seconds == 0 {
        return Ok(());
    }
    if now.saturating_sub(verified_at) > max_age_seconds {
        return Err(Error::InvalidProofFreshness);
    }
    Ok(())
}
