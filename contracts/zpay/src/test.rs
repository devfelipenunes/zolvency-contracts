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

#[test]
fn test_estimate_total_usd() {
    let env = Env::default();
    let contract_id = env.register(ZPayContract, ());
    let client = ZPayContractClient::new(&env, &contract_id);
    
    client.initialize(
        &Address::generate(&env),
        &Address::generate(&env),
        &soroban_sdk::BytesN::from_array(&env, &[0; 32]),
        &10_000_000, // 1 USD equivalent fee
        &5_000_000,  // 0.5 USD equivalent fee
        &Address::generate(&env),
        &Address::generate(&env)
    );

    // Case 1: No ticket (USD to USD) - Total = base + fees
    let total = client.calculate_usd_impact(&100_000_000, &None);
    assert_eq!(total, 115_000_000); // 100 + 10 + 5

    // Case 2: XLM to USD. Suppose price is 0.1 USD per XLM. Scaled by 10^7 = 1_000_000.
    // If base is 1000 XLM (1_000_000_0000 stroops).
    // Total tokens = 10_000_000_000 (base) + 10_000_000 (srv) + 5_000_000 (nex) = 10_015_000_000
    // Total USD = 10_015_000_000 * 1_000_000 / 10_000_000 = 1_001_500_000 (100.15 USD equivalent scaled by 7)
    let ticket = super::PriceTicket {
        base_currency: soroban_sdk::Symbol::new(&env, "USD"),
        price_per_unit: 1_000_000, // 0.1 USD
        timestamp: env.ledger().timestamp(),
        signature: soroban_sdk::BytesN::from_array(&env, &[0; 64]),
    };
    
    let total_usd = client.calculate_usd_impact(&10_000_000_000, &Some(ticket));
    assert_eq!(total_usd, 1_001_500_000);
}
