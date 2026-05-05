#![cfg(test)]
use super::*;
use soroban_sdk::{Address, testutils::Address as _, Env};

#[test]
fn test_initialization() {
    let env = Env::default();
    // Register the contract using the current SDK practice.
    let contract_id = env.register(ZPayContract, ());
    let client = ZPayContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let nexus_contract = Address::generate(&env);
    let oracle_pub_key = soroban_sdk::BytesN::from_array(&env, &[0; 32]);
    let service_fee_amount: i128 = 100_0000; // 0.1 tokens
    let nexus_fee_amount: i128 = 50_0000;    // 0.05 tokens
    let zpay_treasury = Address::generate(&env);
    let nexus_treasury = Address::generate(&env);

    client.initialize(
        &admin,
        &nexus_contract,
        &oracle_pub_key,
        &service_fee_amount,
        &nexus_fee_amount,
        &zpay_treasury,
        &nexus_treasury
    );

    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_allowlist_management() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(ZPayContract, ());
    let client = ZPayContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    
    client.initialize(&admin, &Address::generate(&env), &soroban_sdk::BytesN::from_array(&env, &[0; 32]), &0, &0, &Address::generate(&env), &Address::generate(&env));

    assert_eq!(client.is_token_allowed(&token), false);
    
    client.add_token(&admin, &token);
    assert_eq!(client.is_token_allowed(&token), true);
    
    client.remove_token(&admin, &token);
    assert_eq!(client.is_token_allowed(&token), false);
}
