use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid payload structure")]
    InvalidPayload {},

    #[error("Failed to deserialize SoulID")]
    InvalidSoulID {},

    #[error("Failed to deserialize Address")]
    InvalidAddress {},

    #[error("Failed to deserialize Tier")]
    InvalidTier {},

    #[error("Failed to deserialize Nonce")]
    InvalidNonce {},

    #[error("Failed to deserialize Token Type")]
    InvalidTokenType {},

    #[error("Failed to deserialize Permissions")]
    InvalidPermissions {},

    #[error("Failed to deserialize Expiry")]
    InvalidExpiry {},
}
