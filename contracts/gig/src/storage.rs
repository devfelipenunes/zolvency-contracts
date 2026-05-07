use soroban_sdk::{Address, Env, Symbol};

use crate::types::{
    Config, DataKey, Error, UberIncomeData,
};

#[allow(dead_code)]
const KEY_CONFIG: &str = "CONFIG";
const KEY_TOKEN_COUNTER: &str = "TOKEN_CTR";

const DAY_IN_LEDGERS: u32 = 17_280;
const THIRTY_DAYS: u32 = 30 * DAY_IN_LEDGERS;
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


pub fn set_token_data(env: &Env, token_id: u64, data: &UberIncomeData) {
    let key = (Symbol::new(env, "TOK"), token_id);
    env.storage().persistent().set(&key, data);
    env.storage()
        .persistent()
        .extend_ttl(&key, ONE_YEAR, ONE_YEAR);
}

pub fn get_token_data(env: &Env, token_id: u64) -> Result<UberIncomeData, Error> {
    let key = (Symbol::new(env, "TOK"), token_id);
    let data: Option<UberIncomeData> = env.storage().persistent().get(&key);
    if let Some(d) = data {
        env.storage()
            .persistent()
            .extend_ttl(&key, ONE_YEAR, ONE_YEAR);
        Ok(d)
    } else {
        Err(Error::TokenNotFound)
    }
}

pub fn set_holder_token(env: &Env, soul_id: u32, token_id: u64) {
    let key = (Symbol::new(env, "HLD"), soul_id);
    env.storage().persistent().set(&key, &token_id);
    env.storage()
        .persistent()
        .extend_ttl(&key, ONE_YEAR, ONE_YEAR);
}

pub fn get_holder_token(env: &Env, soul_id: u32) -> Result<u64, Error> {
    let key = (Symbol::new(env, "HLD"), soul_id);
    let token_id: Option<u64> = env.storage().persistent().get(&key);
    if let Some(id) = token_id {
        env.storage()
            .persistent()
            .extend_ttl(&key, ONE_YEAR, ONE_YEAR);
        Ok(id)
    } else {
        Err(Error::NoIdentityFound)
    }
}

pub fn set_sybil_mapping(env: &Env, external_id: &soroban_sdk::String, token_id: u64) {
    let key = (Symbol::new(env, "SYB"), external_id.clone());
    env.storage().persistent().set(&key, &token_id);
    env.storage()
        .persistent()
        .extend_ttl(&key, ONE_YEAR, ONE_YEAR);
}

pub fn get_admin(env: &Env) -> Result<Address, Error> {
    Ok(get_config(env)?.admin)
}

pub fn get_next_token_id(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&KEY_TOKEN_COUNTER)
        .unwrap_or(1u64)
}

pub fn increment_token_counter(env: &Env) {
    let current = get_next_token_id(env);
    let key = &KEY_TOKEN_COUNTER;
    env.storage().persistent().set(key, &(current + 1));
    env.storage()
        .persistent()
        .extend_ttl(key, ONE_YEAR, ONE_YEAR);
}

pub fn update_token_data(env: &Env, token_id: u64, data: &UberIncomeData) -> Result<(), Error> {
    let key = (Symbol::new(env, "TOK"), token_id);
    if !env.storage().persistent().has(&key) {
        return Err(Error::TokenNotFound);
    }
    env.storage().persistent().set(&key, data);
    env.storage()
        .persistent()
        .extend_ttl(&key, ONE_YEAR, ONE_YEAR);
    Ok(())
}

pub fn set_has_identity(env: &Env, soul_id: u32, has: bool) {
    let key = (Symbol::new(env, "HAS"), soul_id);
    env.storage().persistent().set(&key, &has);
    env.storage()
        .persistent()
        .extend_ttl(&key, ONE_YEAR, ONE_YEAR);
}

pub fn has_identity(env: &Env, soul_id: u32) -> bool {
    let key = (Symbol::new(env, "HAS"), soul_id);
    let has: Option<bool> = env.storage().persistent().get(&key);
    if has.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, ONE_YEAR, ONE_YEAR);
    }
    has.unwrap_or(false)
}

pub fn get_nonce(env: &Env, soul_id: u32) -> u64 {
    let key = (Symbol::new(env, "NON"), soul_id);
    env.storage().temporary().get(&key).unwrap_or(0u64)
}

pub fn increment_nonce(env: &Env, soul_id: u32) {
    let current = get_nonce(env, soul_id);
    let key = (Symbol::new(env, "NON"), soul_id);
    env.storage().temporary().set(&key, &(current + 1));
    env.storage()
        .temporary()
        .extend_ttl(&key, THIRTY_DAYS, THIRTY_DAYS);
}
