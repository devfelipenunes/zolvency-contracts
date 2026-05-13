use soroban_sdk::{Env, Address, BytesN};
use crate::{DataKey, SoulData, Error};

const DAY_IN_LEDGERS: u32 = 17_280;
const ONE_YEAR: u32 = 365 * DAY_IN_LEDGERS;

pub fn extend_instance(env: &Env) {
    env.storage().instance().extend_ttl(ONE_YEAR, ONE_YEAR);
}

pub fn extend_persistent(env: &Env, key: &DataKey) {
    env.storage().persistent().extend_ttl(key, ONE_YEAR, ONE_YEAR);
}

pub fn get_admin(env: &Env) -> Result<Address, Error> {
    env.storage().instance().get(&DataKey::Admin).ok_or(Error::NotInitialized)
}

pub fn get_relayer(env: &Env) -> Result<Address, Error> {
    env.storage().instance().get(&DataKey::Relayer).ok_or(Error::NotInitialized)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn set_relayer(env: &Env, relayer: &Address) {
    env.storage().instance().set(&DataKey::Relayer, relayer);
}

pub fn get_total_souls(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::TotalSouls).unwrap_or(0)
}

pub fn increment_total_souls(env: &Env) -> u32 {
    let total = get_total_souls(env) + 1;
    env.storage().instance().set(&DataKey::TotalSouls, &total);
    total
}

pub fn set_soul(env: &Env, soul: &SoulData) {
    let id_key = DataKey::SoulById(soul.id);
    let pk_key = DataKey::SoulByPasskey(soul.passkey.clone());
    let addr_key = DataKey::SoulByAddress(soul.owner.clone());
    
    env.storage().persistent().set(&id_key, soul);
    env.storage().persistent().set(&pk_key, &soul.id);
    env.storage().persistent().set(&addr_key, &soul.id);
    
    extend_persistent(env, &id_key);
    extend_persistent(env, &pk_key);
    extend_persistent(env, &addr_key);
}

pub fn get_soul_id_by_address(env: &Env, address: &Address) -> Option<u32> {
    env.storage().persistent().get(&DataKey::SoulByAddress(address.clone()))
}

pub fn get_soul_id_by_passkey(env: &Env, passkey: &BytesN<65>) -> Option<u32> {
    env.storage().persistent().get(&DataKey::SoulByPasskey(passkey.clone()))
}

pub fn get_soul_by_id(env: &Env, id: u32) -> Option<SoulData> {
    env.storage().persistent().get(&DataKey::SoulById(id))
}

pub fn remove_passkey_mapping(env: &Env, passkey: &BytesN<65>) {
    env.storage().persistent().remove(&DataKey::SoulByPasskey(passkey.clone()));
}
