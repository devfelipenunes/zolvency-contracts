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

#[contract]
pub struct MockNexus;

#[contractimpl]
impl MockNexus {
    pub fn verify_authority(
        _env: Env,
        _mandate_id: u64,
        _contract: Address,
        _function: Symbol,
        _transfer_amount: Option<i128>,
    ) -> bool {
        true
    }
}

#[test]
fn test_full_pay_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let agent = Address::generate(&env);
    let root_anchor = Address::generate(&env);
    let seller = Address::generate(&env);
    let zpay_treasury = Address::generate(&env);
    let nex_treasury = Address::generate(&env);

    // Register Token
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone()).address();
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);
    
    // Mint tokens to root_anchor
    token_admin_client.mint(&root_anchor, &1_000_000_000);

    // Register Mock Nexus
    let nexus_id = env.register(MockNexus, ());

    // Register ZPay
    let zpay_id = env.register(ZPayContract, ());
    let zpay_client = ZPayContractClient::new(&env, &zpay_id);

    // Initialize ZPay
    zpay_client.initialize(
        &admin,
        &nexus_id,
        &soroban_sdk::BytesN::from_array(&env, &[0; 32]),
        &10_000_000, // 1.0 fee
        &5_000_000,  // 0.5 fee
        &zpay_treasury,
        &nex_treasury
    );

    // Add token to allowlist
    zpay_client.add_token(&admin, &token_id);

    // root_anchor must approve ZPay to spend tokens
    // Note: In Stellar asset contract, we use "approve"
    let token_token_client = soroban_sdk::token::Client::new(&env, &token_id);
    token_token_client.approve(&root_anchor, &zpay_id, &1_000_000_000, &9999);

    // Execute Pay
    zpay_client.pay(
        &agent,
        &root_anchor,
        &seller,
        &token_id,
        &100_000_000, // base amount
        &123,         // mandate id
        &None         // no price ticket
    );

    // Check balances
    assert_eq!(token_token_client.balance(&seller), 100_000_000);
    assert_eq!(token_token_client.balance(&zpay_treasury), 10_000_000);
    assert_eq!(token_token_client.balance(&nex_treasury), 5_000_000);
    assert_eq!(token_token_client.balance(&root_anchor), 1_000_000_000 - 100_000_000 - 10_000_000 - 5_000_000);
}
