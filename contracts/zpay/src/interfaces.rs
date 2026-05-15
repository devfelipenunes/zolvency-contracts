use soroban_sdk::{contractclient, contracttype, Address, BytesN, Env, Symbol};

// --- STORK ORACLE ---

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

// --- NEXUS ---

#[contractclient(name = "NexusClient")]
pub trait NexusTrait {
    fn verify_authority(
        env: Env,
        mandate_id: u64,
        agent: Address,
        contract: Address,
        function: Symbol,
        transfer_amount: Option<i128>,
        token: Option<Address>,
    ) -> bool;
}

// --- FALLBACK ORACLE ---

#[contractclient(name = "FallbackOracleClient")]
pub trait FallbackOracleTrait {
    fn get_price(env: Env) -> i128;
}
