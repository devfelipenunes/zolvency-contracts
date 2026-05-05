// contracts/contracts/zpay/src/test.rs
#![cfg(test)]
use super::*;
use soroban_sdk::Env;

#[test]
fn test_hello() {
    let env = Env::default();
    let contract_id = env.register(ZPayContract, ());
    let client = ZPayContractClient::new(&env, &contract_id);
    assert_eq!(client.hello(), 1);
}
