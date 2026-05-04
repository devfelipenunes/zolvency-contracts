use soroban_sdk::{contracterror, contracttype, BytesN, String, Bytes};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotAuthorized = 2,
    SoulAlreadyExists = 3,
    NotInitialized = 4,
    CounterOverflow = 5,
    SoulNotFound = 6,
    InvalidRecoverySignature = 7,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    Relayer,
    TotalSouls,
    SoulById(u32),                 // SoulID -> SoulData
    SoulByPasskey(BytesN<65>),     // Passkey PubKey -> SoulID
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SoulData {
    pub id: u32,
    pub passkey: BytesN<65>,           // secp256r1 pubkey
    pub recovery_pubkey: BytesN<65>,   // secp256r1 pubkey for recovery
    pub minted_at: u64,
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
