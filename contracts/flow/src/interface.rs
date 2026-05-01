use soroban_sdk::{BytesN, Env, String, Symbol};

use crate::types::TokenMetadata;

pub trait ZolvencyTokenTrait {
    fn get_token_type(env: Env) -> Symbol;
    fn get_source(env: Env) -> String;
    fn get_metadata(env: Env) -> TokenMetadata;
    fn is_valid(env: Env, token_id: u64) -> bool;
    fn get_expiry(env: Env, token_id: u64) -> u64;
    fn get_owner_soul(env: Env, token_id: u64) -> u32;
}
