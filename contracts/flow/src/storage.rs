use soroban_sdk::{Address, Env};
use crate::{Config, DataKey, Error, FlowIncomeData};

const DAY_IN_LEDGERS: u32 = 17_280;
const ONE_YEAR: u32 = 365 * DAY_IN_LEDGERS;

pub fn set_config(env: &Env, config: &Config) {
    env.storage().instance().set(&DataKey::Config, config);
    extend_instance(env);
}

pub fn get_config(env: &Env) -> Result<Config, Error> {
    let config: Option<Config> = env.storage().instance().get(&DataKey::Config);
    if let Some(c) = config {
        extend_instance(env);
        Ok(c)
    } else {
        Err(Error::NotInitialized)
    }
}

pub fn extend_instance(env: &Env) {
    env.storage().instance().extend_ttl(ONE_YEAR, ONE_YEAR);
}

pub fn set_soul_contract(env: &Env, soul_contract: &Address) {
    env.storage().instance().set(&DataKey::SoulContract, soul_contract);
}

pub fn get_soul_contract(env: &Env) -> Result<Address, Error> {
    env.storage().instance().get(&DataKey::SoulContract).ok_or(Error::NotInitialized)
}

pub fn set_token_data(env: &Env, token_id: u64, data: &FlowIncomeData) {
    let key = DataKey::TokenData(token_id);
    env.storage().persistent().set(&key, data);
    env.storage().persistent().extend_ttl(&key, ONE_YEAR, ONE_YEAR);
}

pub fn get_token_data(env: &Env, token_id: u64) -> Result<FlowIncomeData, Error> {
    let key = DataKey::TokenData(token_id);
    let data: Option<FlowIncomeData> = env.storage().persistent().get(&key);
    if let Some(d) = data {
        env.storage().persistent().extend_ttl(&key, ONE_YEAR, ONE_YEAR);
        Ok(d)
    } else {
        Err(Error::TokenNotFound)
    }
}

pub fn update_token_data(env: &Env, token_id: u64, data: &FlowIncomeData) -> Result<(), Error> {
    let key = DataKey::TokenData(token_id);
    if !env.storage().persistent().has(&key) {
        return Err(Error::TokenNotFound);
    }
    env.storage().persistent().set(&key, data);
    env.storage().persistent().extend_ttl(&key, ONE_YEAR, ONE_YEAR);
    Ok(())
}

pub fn set_holder_token(env: &Env, soul_id: u32, token_id: u64) {
    let key = DataKey::HolderToken(soul_id);
    env.storage().persistent().set(&key, &token_id);
    env.storage().persistent().extend_ttl(&key, ONE_YEAR, ONE_YEAR);
}

pub fn get_holder_token(env: &Env, soul_id: u32) -> Result<u64, Error> {
    let key = DataKey::HolderToken(soul_id);
    let token_id: Option<u64> = env.storage().persistent().get(&key);
    if let Some(id) = token_id {
        env.storage().persistent().extend_ttl(&key, ONE_YEAR, ONE_YEAR);
        Ok(id)
    } else {
        Err(Error::TokenNotFound)
    }
}

pub fn set_sybil_mapping(env: &Env, external_id: &soroban_sdk::String, token_id: u64) {
    let key = DataKey::SybilMapping(external_id.clone());
    env.storage().persistent().set(&key, &token_id);
    env.storage().persistent().extend_ttl(&key, ONE_YEAR, ONE_YEAR);
}

pub fn get_next_token_id(env: &Env) -> u64 {
    env.storage().persistent().get(&DataKey::TokenCounter).unwrap_or(1u64)
}

pub fn increment_token_counter(env: &Env) {
    let current = get_next_token_id(env);
    env.storage().persistent().set(&DataKey::TokenCounter, &(current + 1));
}

pub fn set_has_identity(env: &Env, soul_id: u32, has: bool) {
    let key = DataKey::HasIdentity(soul_id);
    env.storage().persistent().set(&key, &has);
    env.storage().persistent().extend_ttl(&key, ONE_YEAR, ONE_YEAR);
}

pub fn has_identity(env: &Env, soul_id: u32) -> bool {
    let key = DataKey::HasIdentity(soul_id);
    let has: Option<bool> = env.storage().persistent().get(&key);
    has.unwrap_or(false)
}

pub fn get_nonce(env: &Env, soul_id: u32) -> u64 {
    let key = DataKey::Nonce(soul_id);
    env.storage().temporary().get(&key).unwrap_or(0u64)
}

pub fn increment_nonce(env: &Env, soul_id: u32) {
    let current = get_nonce(env, soul_id);
    let key = DataKey::Nonce(soul_id);
    env.storage().temporary().set(&key, &(current + 1));
}

pub fn get_admin(env: &Env) -> Result<Address, Error> {
    Ok(get_config(env)?.admin)
}
