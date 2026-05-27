#![cfg(test)]
use super::*;
use soroban_sdk::testutils::{Address as _};
use soroban_sdk::{Address, Env};

fn setup_test(env: &Env) -> (Address, Address, soroban_sdk::token::StellarAssetClient<'static>, ProofVaultContractClient<'static>) {
    let admin = Address::generate(env);
    let user = Address::generate(env);
    let token_admin = Address::generate(env);
    
    let token_address = env.register_stellar_asset_contract(token_admin);
    let token = soroban_sdk::token::StellarAssetClient::new(env, &token_address);
    
    let vault_address = env.register_contract(None, ProofVaultContract);
    let vault = ProofVaultContractClient::new(env, &vault_address);
    
    vault.initialize(&admin, &token_address);
    
    (admin, user, token, vault)
}

#[test]
fn test_deposit_and_balance() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, user, token, vault) = setup_test(&env);
    
    let deposit_amount = 2000;
    token.mint(&user, &deposit_amount);
    
    vault.deposit(&user, &deposit_amount);
    
    // Total principal should be equal to deposit_amount (1:1)
    assert_eq!(vault.get_total_principal(), 2000);
    assert_eq!(vault.get_user_balance(&user), 2000);
    assert_eq!(vault.get_balance(&user), 2000);
}

#[test]
fn test_multiple_deposits() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, user1, token, vault) = setup_test(&env);
    let user2 = Address::generate(&env);
    
    // First deposit
    token.mint(&user1, &2000);
    vault.deposit(&user1, &2000);
    
    // Second deposit
    token.mint(&user2, &1000);
    vault.deposit(&user2, &1000);
    
    // Total principal = 3000
    assert_eq!(vault.get_user_balance(&user1), 2000);
    assert_eq!(vault.get_user_balance(&user2), 1000);
    assert_eq!(vault.get_total_principal(), 3000);
}

#[test]
fn test_consume_credit() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, user, token, vault) = setup_test(&env);
    
    // Initial deposit
    token.mint(&user, &2000);
    vault.deposit(&user, &2000);
    
    // Consume credit (admin only)
    // amount = 500 tokens
    vault.consume_credit(&user, &500);
    
    assert_eq!(vault.get_user_balance(&user), 1500);
    assert_eq!(vault.get_total_principal(), 1500);
}

#[test]
fn test_yield_harvesting_captured_by_protocol() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, user, token, vault) = setup_test(&env);
    let adapter = Address::generate(&env);
    
    // Initial deposit
    token.mint(&user, &2000);
    vault.deposit(&user, &2000);
    
    assert_eq!(vault.get_balance(&user), 2000);
    
    // Set adapter
    vault.set_defi_adapter(&admin, &adapter);
    
    // Delegate liquidity
    vault.delegate_liquidity(&admin, &1000);
    
    // Harvest yield
    vault.harvest_yield(&adapter, &500);
    
    // Total balance should be 2500 now (2000 + 500)
    // BUT user balance remains stable at 2000 (Profit Capture)
    assert_eq!(vault.get_total_balance(), 2500);
    assert_eq!(vault.get_balance(&user), 2000);
}

#[test]
#[should_panic(expected = "Unauthorized: not admin")]
fn test_delegate_liquidity_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, user, token, vault) = setup_test(&env);
    
    token.mint(&user, &2000);
    vault.deposit(&user, &2000);
    
    // User tries to delegate liquidity (should fail)
    vault.delegate_liquidity(&user, &1000);
}

#[test]
#[should_panic(expected = "Unauthorized: not authorized adapter")]
fn test_harvest_yield_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, _user, _token, vault) = setup_test(&env);
    let adapter = Address::generate(&env);
    let unauthorized_adapter = Address::generate(&env);
    
    vault.set_defi_adapter(&admin, &adapter);
    
    // Unauthorized adapter tries to harvest yield (should fail)
    vault.harvest_yield(&unauthorized_adapter, &500);
}

#[test]
fn test_profit_capture_and_withdrawal() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, user, token, vault) = setup_test(&env);
    let adapter = Address::generate(&env);
    let recipient = Address::generate(&env);
    
    // Initial deposit
    token.mint(&user, &2000);
    vault.deposit(&user, &2000);
    
    // Set adapter and delegate
    vault.set_defi_adapter(&admin, &adapter);
    vault.delegate_liquidity(&admin, &1000);
    
    // Harvest yield (profit)
    vault.harvest_yield(&adapter, &500);
    
    // Check profit
    assert_eq!(vault.get_profit(), 500);
    
    // Withdraw profit
    vault.withdraw_profit(&admin, &recipient, &300);
    
    // Check remaining profit and total balance
    assert_eq!(vault.get_profit(), 200);
    assert_eq!(vault.get_total_balance(), 2200);
    
    // Check recipient token balance
    let token_client = soroban_sdk::token::Client::new(&env, &token.address);
    assert_eq!(token_client.balance(&recipient), 300);
}

#[test]
#[should_panic(expected = "Insufficient profit balance")]
fn test_withdraw_profit_insufficient() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, user, token, vault) = setup_test(&env);
    
    token.mint(&user, &1000);
    vault.deposit(&user, &1000);
    
    // No profit generated yet
    vault.withdraw_profit(&admin, &admin, &100);
}
