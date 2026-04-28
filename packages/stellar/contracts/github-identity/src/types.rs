#![allow(clippy::enum_variant_names)]
use soroban_sdk::{contracterror, contracttype, Address, Bytes, BytesN, Env, String};

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
    AxelarConfig,
    LayerZeroConfig,
    InteropConfig,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct AxelarConfig {
    pub gateway: Address,
    pub gas_service: Address,
    pub gas_token: Address,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct LayerZeroConfig {
    pub endpoint: Address,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct MintParams {
    pub contributions: u32,
    pub external_id: String,
    pub nonce: u64,
    pub passkey: Bytes,
    pub passkey_signature: Bytes,
    pub proof_data: Bytes,
    pub username: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GithubData {
    pub contributions: u32,
    pub expires_at: u64,
    pub external_id: String,
    pub minted_at: u64,
    pub passkey: Bytes,
    pub proof_data: Bytes,
    pub tier: Tier,
    pub updated_at: u64,
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

    pub fn to_number(&self) -> u8 {
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
#[derive(Clone)]
pub struct Config {
    pub admin: Address,
    pub registry: Address,
    pub fee_token: Address,
    pub access_control: Address,
    pub treasury: Address,
    pub mint_fee: i128,
    pub zk_verifier: Option<Address>,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct InteropConfig {
    pub active_protocol: InteropProtocol,
    pub adapter_address: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InteropProtocol {
    None,
    Axelar,
    LayerZero,
    Wormhole,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct CrossChainParams {
    pub destination_chain: String,
    pub destination_address: String,
    pub user_destination_address: Bytes,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub version: String,
    pub data_source: String,
}
