use soroban_sdk::{contractclient, Env};

#[contractclient(name = "FallbackOracleClient")]
pub trait FallbackOracleTrait {
    fn get_price(env: Env) -> i128;
}
