#![no_std]

use soroban_sdk::{contract, contractimpl, contracterror, contracttype, Address, Bytes, BytesN, Env, String, Symbol, Vec};

// --- TYPES ---

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
pub enum DataKey {
    Config,
    TokenData(u64),
    HolderToken(u32),
    SybilMapping(String),
    TokenCounter,
    HasIdentity(u32),
    Nonce(u32),
    InteropConfig,
    SoulContract,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Ecosystem {
    Evm,
    Cosmos,
    Sui,
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IncomePeriod {
    Monthly,
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
    pub period: IncomePeriod,
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
    pub period: IncomePeriod,
    pub verified_at: u64,
    pub proof_hash: BytesN<32>,
    pub proof_data: Bytes,
    pub window: RenewalWindow,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UberIncomeData {
    pub soul_id: u32,
    pub external_id: String,
    pub income_band: u32,
    pub income_value: Option<i128>,
    pub reveal_mode: RevealMode,
    pub currency: String,
    pub period: IncomePeriod,
    pub verified_at: u64,
    pub proof_hash: BytesN<32>,
    pub window: RenewalWindow,
    pub minted_at: u64,
    pub updated_at: u64,
    pub expires_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct InitializeParams {
    pub admin: Address,
    pub registry: Address,
    pub soul_contract: Address,
    pub fee_token: Address,
    pub access_control: Address,
    pub treasury: Address,
    pub mint_fee_30: i128,
    pub mint_fee_60: i128,
    pub mint_fee_90: i128,
    pub max_proof_age_seconds: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct Config {
    pub admin: Address,
    pub registry: Address,
    pub soul_contract: Address,
    pub fee_token: Address,
    pub access_control: Address,
    pub treasury: Address,
    pub mint_fee_30: i128,
    pub mint_fee_60: i128,
    pub mint_fee_90: i128,
    pub max_proof_age_seconds: u64,
    pub zk_verifier: Option<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub version: String,
    pub data_source: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteropConfig {
    pub adapter_address: Address,
}

// --- MODULES ---

mod storage;
mod logic;

#[cfg(test)]
mod test;

#[contract]
pub struct UberIncomeContract;

#[contractimpl]
impl UberIncomeContract {
    pub fn initialize(env: Env, params: InitializeParams) -> Result<(), Error> {
        if storage::get_config(&env).is_ok() {
            return Err(Error::AlreadyInitialized);
        }
        let config = Config {
            admin: params.admin,
            registry: params.registry,
            soul_contract: params.soul_contract,
            fee_token: params.fee_token,
            access_control: params.access_control,
            treasury: params.treasury,
            mint_fee_30: params.mint_fee_30,
            mint_fee_60: params.mint_fee_60,
            mint_fee_90: params.mint_fee_90,
            max_proof_age_seconds: params.max_proof_age_seconds,
            zk_verifier: None,
        };
        storage::set_config(&env, &config);
        env.storage().persistent().set(&DataKey::SoulContract, &config.soul_contract);
        Ok(())
    }

    pub fn mint(env: Env, caller: Address, params: MintParams, cross_chain: Option<CrossChainParams>) -> Result<u64, Error> {
        logic::mint(&env, caller, params, cross_chain)
    }

    pub fn update_token(
        env: Env,
        admin: Address,
        token_id: u64,
        params: UpdateParams,
        nonce: u64,
        cross_chain: Option<CrossChainParams>,
    ) -> Result<(), Error> {
        logic::update_token(&env, admin, token_id, params, nonce, cross_chain)
    }

    pub fn get_token_data(env: Env, token_id: u64) -> Result<UberIncomeData, Error> {
        storage::get_token_data(&env, token_id)
    }

    pub fn get_holder_token(env: Env, soul_id: u32) -> Result<u64, Error> {
        storage::get_holder_token(&env, soul_id)
    }

    pub fn get_token_type(env: Env) -> Symbol {
        Symbol::new(&env, "gig")
    }

    pub fn get_source(env: Env) -> String {
        String::from_str(&env, "uber")
    }

    pub fn get_metadata(env: Env) -> TokenMetadata {
        TokenMetadata {
            name: String::from_str(&env, "Zolvency Uber Income"),
            symbol: String::from_str(&env, "ZUI"),
            version: String::from_str(&env, "1.0.0"),
            data_source: String::from_str(&env, "uber"),
        }
    }

    pub fn set_interop_config(env: Env, admin: Address, config: InteropConfig) -> Result<(), Error> {
        admin.require_auth();
        if admin != storage::get_admin(&env)? {
            return Err(Error::NotAdmin);
        }
        env.storage().persistent().set(&DataKey::InteropConfig, &config);
        Ok(())
    }

    pub fn upgrade(env: Env, admin: Address, new_wasm_hash: BytesN<32>) -> Result<(), Error> {
        admin.require_auth();
        if admin != storage::get_admin(&env)? {
            return Err(Error::NotAdmin);
        }
        env.deployer().update_current_contract_wasm(new_wasm_hash);
        Ok(())
    }
}
