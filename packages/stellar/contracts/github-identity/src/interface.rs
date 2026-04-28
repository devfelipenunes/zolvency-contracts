use soroban_sdk::{Bytes, Env, String, Symbol};

use crate::types::TokenMetadata;

pub trait ZolvencyTokenTrait {
    /// Retorna o tipo do token (ex: "github", "bank")
    fn get_token_type(env: Env) -> Symbol;

    /// Retorna a fonte da prova (ex: "reclaim", "zk-email")
    fn get_source(env: Env) -> String;

    /// Retorna os metadados do contrato
    fn get_metadata(env: Env) -> TokenMetadata;

    /// Verifica se o token ainda é válido (Business TTL)
    fn is_valid(env: Env, token_id: u64) -> bool;

    /// Retorna o timestamp de expiração (UNIX seconds)
    fn get_expiry(env: Env, token_id: u64) -> u64;

    /// Retorna a chave pública da Passkey vinculada (secp256r1)
    fn get_owner_passkey(env: Env, token_id: u64) -> Bytes;
}
