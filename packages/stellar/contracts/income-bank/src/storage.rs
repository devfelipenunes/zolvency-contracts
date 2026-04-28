use soroban_sdk::{Address, Env, Symbol};

use crate::types::{
    AxelarConfig, Config, DataKey, Error, IncomeData, InteropConfig, LayerZeroConfig,
};

const KEY_CONFIG: &str = "CONFIG";
const KEY_TOKEN_COUNTER: &str = "TOKEN_CTR";

const DAY_IN_LEDGERS: u32 = 17_280;
const THIRTY_DAYS: u32 = 30 * DAY_IN_LEDGERS;
const ONE_YEAR: u32 = 365 * DAY_IN_LEDGERS;

pub fn set_config(env: &Env, config: &Config) {
    let key = &KEY_CONFIG;
    env.storage().persistent().set(key, config);
    env.storage()
        .persistent()
        .extend_ttl(key, ONE_YEAR, ONE_YEAR);
}

pub fn get_config(env: &Env) -> Result<Config, Error> {
    let key = &KEY_CONFIG;
    let config: Option<Config> = env.storage().persistent().get(key);
    if let Some(c) = config {
        env.storage()
            .persistent()
            .extend_ttl(key, ONE_YEAR, ONE_YEAR);
        Ok(c)
    } else {
        Err(Error::NotInitialized)
    }
}

pub fn get_interop_config(env: &Env) -> Result<InteropConfig, Error> {
    env.storage()
        .instance()
        .get(&DataKey::InteropConfig)
        .ok_or(Error::NotInitialized)
}

pub fn set_interop_config(env: &Env, config: &InteropConfig) {
    env.storage()
        .instance()
        .set(&DataKey::InteropConfig, config);
}

pub fn set_axelar_config(env: &Env, config: &AxelarConfig) {
    env.storage().instance().set(&DataKey::AxelarConfig, config);
}

pub fn set_layerzero_config(env: &Env, config: &LayerZeroConfig) {
    env.storage()
        .instance()
        .set(&DataKey::LayerZeroConfig, config);
}

pub fn set_token_data(env: &Env, token_id: u64, data: &IncomeData) {
    let key = (Symbol::new(env, "TOK"), token_id);
    env.storage().persistent().set(&key, data);
    env.storage()
        .persistent()
        .extend_ttl(&key, ONE_YEAR, ONE_YEAR);
}

pub fn get_token_data(env: &Env, token_id: u64) -> Result<IncomeData, Error> {
    let key = (Symbol::new(env, "TOK"), token_id);
    let data: Option<IncomeData> = env.storage().persistent().get(&key);
    if let Some(d) = data {
        env.storage()
            .persistent()
            .extend_ttl(&key, ONE_YEAR, ONE_YEAR);
        Ok(d)
    } else {
        Err(Error::TokenNotFound)
    }
}

pub fn set_holder_token(env: &Env, holder: &Address, token_id: u64) {
    let key = (Symbol::new(env, "HLD"), holder.clone());
    env.storage().persistent().set(&key, &token_id);
    env.storage()
        .persistent()
        .extend_ttl(&key, ONE_YEAR, ONE_YEAR);
}

pub fn get_holder_token(env: &Env, holder: &Address) -> Result<u64, Error> {
    let key = (Symbol::new(env, "HLD"), holder.clone());
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

pub fn update_token_data(env: &Env, token_id: u64, data: &IncomeData) -> Result<(), Error> {
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

pub fn set_has_identity(env: &Env, holder: &Address, has: bool) {
    let key = (Symbol::new(env, "HAS"), holder.clone());
    env.storage().persistent().set(&key, &has);
    env.storage()
        .persistent()
        .extend_ttl(&key, ONE_YEAR, ONE_YEAR);
}

pub fn has_identity(env: &Env, holder: &Address) -> bool {
    let key = (Symbol::new(env, "HAS"), holder.clone());
    let has: Option<bool> = env.storage().persistent().get(&key);
    if has.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, ONE_YEAR, ONE_YEAR);
    }
    has.unwrap_or(false)
}

pub fn get_nonce(env: &Env, user: &Address) -> u64 {
    let key = (Symbol::new(env, "NON"), user.clone());
    env.storage().temporary().get(&key).unwrap_or(0u64)
}

pub fn increment_nonce(env: &Env, user: &Address) {
    let current = get_nonce(env, user);
    let key = (Symbol::new(env, "NON"), user.clone());
    env.storage().temporary().set(&key, &(current + 1));
    env.storage()
        .temporary()
        .extend_ttl(&key, THIRTY_DAYS, THIRTY_DAYS);
}
