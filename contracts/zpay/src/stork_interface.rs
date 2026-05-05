// contracts/contracts/zpay/src/stork_interface.rs
use soroban_sdk::{contractclient, contracttype, BytesN, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemporalNumericValue {
    pub quantized_value: i128,
    pub timestamp: u64,
    pub publisher_merkle_root: BytesN<32>,
}

#[contractclient(name = "StorkClient")]
pub trait StorkOracleTrait {
    fn get_temporal_numeric_value_v1(env: Env, asset_id: BytesN<32>) -> TemporalNumericValue;
}
