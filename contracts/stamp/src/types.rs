#![allow(clippy::enum_variant_names)]
use soroban_sdk::{contracterror, contracttype, Address, Bytes, BytesN, String};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyHasIdentity = 1,
    NoIdentityFound = 2,
    InvalidNonce = 3,
    InvalidWindow = 4,
    InvalidExternalId = 5,
    InvalidKycLevel = 6,
    InvalidCountry = 7,
    InsufficientPayment = 8,
    NotInitialized = 9,
    NotAdmin = 10,
    TokenNotFound = 11,
    Unauthorized = 12,
    TokenExpired = 13,
    AlreadyInitialized = 14,
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
pub enum KycLevel {
    Basic,
    Intermediate,
    Advanced,
}

impl KycLevel {
    pub fn to_number(&self) -> u32 {
        match self {
            KycLevel::Basic => 1,
            KycLevel::Intermediate => 2,
            KycLevel::Advanced => 3,
        }
    }
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct MintParams {
    pub soul_id: u32,
    pub external_id: String,
    pub kyc_level: KycLevel,
    pub country: String,
    pub verified_at: u64,
    pub proof_hash: BytesN<32>,
    pub proof_data: Bytes,
    pub window: RenewalWindow,
    pub nonce: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct UpdateParams {
    pub kyc_level: KycLevel,
    pub country: String,
    pub verified_at: u64,
    pub proof_hash: BytesN<32>,
    pub proof_data: Bytes,
    pub window: RenewalWindow,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KycData {
    pub soul_id: u32,
    pub external_id: String,
    pub kyc_level: KycLevel,
    pub country: String,
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
