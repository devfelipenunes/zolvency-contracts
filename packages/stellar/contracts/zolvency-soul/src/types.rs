use soroban_sdk::{contracterror, contracttype, BytesN};

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
