use soroban_sdk::{Address, Env, BytesN};
use crate::{DataKey, EscrowEntry, Error};

const DAY_IN_LEDGERS: u32 = 17_280;
const ONE_YEAR: u32 = 365 * DAY_IN_LEDGERS;

pub fn extend_persistent(env: &Env, key: &DataKey) {
    env.storage().persistent().extend_ttl(key, ONE_YEAR, ONE_YEAR);
}

pub fn get_admin(env: &Env) -> Result<Address, Error> {
    env.storage().persistent().get(&DataKey::Admin).ok_or(Error::NotAuthorized)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&DataKey::Admin, admin);
    extend_persistent(env, &DataKey::Admin);
}

pub fn get_nexus_contract(env: &Env) -> Result<Address, Error> {
    env.storage().persistent().get(&DataKey::NexusContract).ok_or(Error::NotAuthorized)
}

pub fn get_oracle_pubkey(env: &Env) -> Result<BytesN<32>, Error> {
    env.storage().persistent().get(&DataKey::OraclePubKey).ok_or(Error::MissingOracleData)
}

pub fn get_service_fee_config(env: &Env) -> (u32, i128) {
    let bps = env.storage().persistent().get(&DataKey::ServiceFeeBps).unwrap_or(0);
    let min = env.storage().persistent().get(&DataKey::MinServiceFee).unwrap_or(0);
    (bps, min)
}

pub fn get_nexus_fee_config(env: &Env) -> (u32, i128) {
    let bps = env.storage().persistent().get(&DataKey::NexusFeeBps).unwrap_or(0);
    let min = env.storage().persistent().get(&DataKey::MinNexusFee).unwrap_or(0);
    (bps, min)
}

pub fn get_treasuries(env: &Env) -> (Address, Address) {
    let zpay = env.storage().persistent().get(&DataKey::ZPayTreasury).unwrap();
    let nexus = env.storage().persistent().get(&DataKey::NexusTreasury).unwrap();
    (zpay, nexus)
}

pub fn is_paused(env: &Env) -> bool {
    env.storage().persistent().get(&DataKey::IsPaused).unwrap_or(false)
}

pub fn get_max_staleness(env: &Env) -> u64 {
    env.storage().persistent().get(&DataKey::MaxStaleness).unwrap_or(3600)
}

pub fn get_max_relayer_fee_bps(env: &Env) -> u32 {
    env.storage().persistent().get(&DataKey::MaxRelayerFeeBps).unwrap_or(500)
}

pub fn get_next_payment_id(env: &Env) -> u64 {
    env.storage().persistent().get(&DataKey::NextPaymentId).unwrap_or(1)
}

pub fn increment_next_payment_id(env: &Env) -> u64 {
    let id = get_next_payment_id(env);
    env.storage().persistent().set(&DataKey::NextPaymentId, &(id + 1));
    extend_persistent(env, &DataKey::NextPaymentId);
    id
}

pub fn get_escrow(env: &Env, id: u64) -> Option<EscrowEntry> {
    env.storage().persistent().get(&DataKey::Escrow(id))
}

pub fn set_escrow(env: &Env, id: u64, entry: &EscrowEntry) {
    env.storage().persistent().set(&DataKey::Escrow(id), entry);
    extend_persistent(env, &DataKey::Escrow(id));
}

pub fn remove_escrow(env: &Env, id: u64) {
    env.storage().persistent().remove(&DataKey::Escrow(id));
}

pub fn is_token_allowed(env: &Env, token: &Address) -> bool {
    env.storage().persistent().has(&DataKey::AllowedToken(token.clone()))
}

pub fn get_mandate_vault(env: &Env, mandate_id: u64) -> i128 {
    env.storage().persistent().get(&DataKey::MandateVault(mandate_id)).unwrap_or(0)
}

pub fn set_mandate_vault(env: &Env, mandate_id: u64, amount: i128) {
    env.storage().persistent().set(&DataKey::MandateVault(mandate_id), &amount);
    extend_persistent(env, &DataKey::MandateVault(mandate_id));
}
