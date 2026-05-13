#![no_std]

use soroban_sdk::{contract, contractimpl, contracterror, contracttype, Address, Bytes, BytesN, Env, String, Symbol, Vec};

// --- TYPES ---

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyHasIdentity = 1,
    NoIdentityFound = 2,
    InvalidTier = 3,
    InvalidNonce = 4,
    InvalidSignature = 5,
    InsufficientPayment = 6,
    TransferNotAllowed = 7,
    EmptyUsername = 8,
    NotInitialized = 9,
    NotAdmin = 10,
    TokenNotFound = 11,
    AccessControlError = 12,
    Unauthorized = 13,
    AlreadyInitialized = 14,
    SybilConflict = 15,
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
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimInfo {
    pub provider: String,
    pub parameters: String,
    pub context: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReclaimProof {
    pub claim_info: ClaimInfo,
    pub signed_claim: BytesN<32>,
    pub signatures: Vec<BytesN<64>>,
    pub witness_address: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct MintParams {
    pub contributions: u32,
    pub external_id: String,
    pub nonce: u64,
    pub proof: ReclaimProof,
    pub username: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Tier {
    Novice,
    Pro,
    Architect,
    Legend,
    Singularity,
}

impl Tier {
    pub fn from_contributions(contributions: u32) -> Self {
        match contributions {
            5000.. => Tier::Singularity,
            3000..=4999 => Tier::Legend,
            1000..=2999 => Tier::Architect,
            200..=999 => Tier::Pro,
            _ => Tier::Novice,
        }
    }

    pub fn to_number(&self) -> u32 {
        match self {
            Tier::Novice => 1,
            Tier::Pro => 2,
            Tier::Architect => 3,
            Tier::Legend => 4,
            Tier::Singularity => 5,
        }
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GithubData {
    pub contributions: u32,
    pub expires_at: u64,
    pub external_id: String,
    pub minted_at: u64,
    pub soul_id: u32,
    pub tier: Tier,
    pub updated_at: u64,
    pub username: String,
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
    pub mint_fee: i128,
    pub zk_verifier: Option<Address>,
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
pub struct GithubIdentityContract;

#[contractimpl]
impl GithubIdentityContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        registry: Address,
        soul_contract: Address,
        fee_token: Address,
        access_control: Address,
        treasury: Address,
        mint_fee: i128,
    ) -> Result<(), Error> {
        if storage::get_config(&env).is_ok() {
            return Err(Error::AlreadyInitialized);
        }
        let config = Config {
            admin,
            registry,
            soul_contract,
            fee_token,
            access_control,
            treasury,
            mint_fee,
            zk_verifier: None,
        };
        storage::set_config(&env, &config);
        Ok(())
    }

    pub fn mint(
        env: Env,
        caller: Address,
        soul_id: u32,
        params: MintParams,
        cross_chain: Option<CrossChainParams>,
    ) -> Result<u64, Error> {
        logic::mint(&env, caller, soul_id, params, cross_chain)
    }

    pub fn get_token_data(env: Env, token_id: u64) -> Result<GithubData, Error> {
        storage::get_token_data(&env, token_id)
    }

    pub fn get_holder_token(env: Env, soul_id: u32) -> Result<u64, Error> {
        storage::get_holder_token(&env, soul_id)
    }

    pub fn is_valid(env: Env, token_id: u64) -> bool {
        storage::get_token_data(&env, token_id).is_ok()
    }

    pub fn get_owner_soul(env: Env, token_id: u64) -> u32 {
        storage::get_token_data(&env, token_id).map(|d| d.soul_id).unwrap_or(0)
    }

    pub fn get_token_type(env: Env) -> Symbol {
        Symbol::new(&env, "github")
    }

    pub fn get_source(env: Env) -> String {
        String::from_str(&env, "github")
    }

    pub fn get_metadata(env: Env) -> TokenMetadata {
        TokenMetadata {
            name: String::from_str(&env, "Zolvency Github Reputation"),
            symbol: String::from_str(&env, "ZOLV-GH"),
            version: String::from_str(&env, "1.0.0"),
            data_source: String::from_str(&env, "github"),
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
