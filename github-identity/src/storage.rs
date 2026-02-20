use soroban_sdk::{Address, Env, Symbol};

use crate::types::{Config, Error, GithubData};

const KEY_CONFIG: &str = "CONFIG";
const KEY_TOKEN_COUNTER: &str = "TOKEN_CTR";

const THIRTY_DAYS_IN_LEDGERS: u32 = 518_400;

pub fn set_config(env: &Env, config: &Config) {
    env.storage().persistent().set(&KEY_CONFIG, config);
}

pub fn get_config(env: &Env) -> Result<Config, Error> {
    env.storage()
        .persistent()
        .get(&KEY_CONFIG)
        .ok_or(Error::NotInitialized)
}

pub fn get_admin(env: &Env) -> Result<Address, Error> {
    Ok(get_config(env)?.admin)
}

pub fn get_access_control(env: &Env) -> Result<Address, Error> {
    Ok(get_config(env)?.access_control)
}

pub fn get_treasury(env: &Env) -> Result<Address, Error> {
    Ok(get_config(env)?.treasury)
}

pub fn get_mint_fee(env: &Env) -> i128 {
    get_config(env).map(|c| c.mint_fee).unwrap_or(0)
}

pub fn get_next_token_id(env: &Env) -> u64 {
    env.storage()
        .persistent()
        .get(&KEY_TOKEN_COUNTER)
        .unwrap_or(1u64)
}

pub fn increment_token_counter(env: &Env) {
    let current = get_next_token_id(env);
    env.storage()
        .persistent()
        .set(&KEY_TOKEN_COUNTER, &(current + 1));
}

pub fn set_token_data(env: &Env, token_id: u64, data: &GithubData) {
    let key = (Symbol::new(env, "TOK"), token_id);
    env.storage().persistent().set(&key, data);
}

pub fn get_token_data(env: &Env, token_id: u64) -> Result<GithubData, Error> {
    let key = (Symbol::new(env, "TOK"), token_id);
    env.storage()
        .persistent()
        .get(&key)
        .ok_or(Error::TokenNotFound)
}

pub fn update_token_data(env: &Env, token_id: u64, data: &GithubData) -> Result<(), Error> {
    let key = (Symbol::new(env, "TOK"), token_id);
    if !env.storage().persistent().has(&key) {
        return Err(Error::TokenNotFound);
    }
    env.storage().persistent().set(&key, data);
    Ok(())
}

pub fn set_holder_token(env: &Env, holder: &Address, token_id: u64) {
    let key = (Symbol::new(env, "HLD"), holder.clone());
    env.storage().persistent().set(&key, &token_id);
}

pub fn get_holder_token(env: &Env, holder: &Address) -> Result<u64, Error> {
    let key = (Symbol::new(env, "HLD"), holder.clone());
    env.storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NoIdentityFound)
}

pub fn set_has_identity(env: &Env, holder: &Address, has: bool) {
    let key = (Symbol::new(env, "HAS"), holder.clone());
    env.storage().persistent().set(&key, &has);
}

pub fn has_identity(env: &Env, holder: &Address) -> bool {
    let key = (Symbol::new(env, "HAS"), holder.clone());
    env.storage()
        .persistent()
        .get(&key)
        .unwrap_or(false)
}

pub fn get_nonce(env: &Env, user: &Address) -> u64 {
    let key = (Symbol::new(env, "NON"), user.clone());
    env.storage().temporary().get(&key).unwrap_or(0u64)
}

pub fn increment_nonce(env: &Env, user: &Address) {
    let current = get_nonce(env, user);
    let key = (Symbol::new(env, "NON"), user.clone());
    env.storage()
        .temporary()
        .set(&key, &(current + 1));
    env.storage()
        .temporary()
        .extend_ttl(&key, THIRTY_DAYS_IN_LEDGERS, THIRTY_DAYS_IN_LEDGERS);
}