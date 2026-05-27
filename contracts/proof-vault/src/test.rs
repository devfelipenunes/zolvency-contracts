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
fn test_inflation_protection() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, user, token, vault) = setup_test(&env);
    
    let deposit_amount = 2000;
    token.mint(&user, &deposit_amount);
    
    vault.deposit(&user, &deposit_amount);
    
    // Total shares should be deposit_amount - MINIMUM_LIQUIDITY
    assert_eq!(vault.get_total_shares(), 1000);
    assert_eq!(vault.get_user_shares(&user), 1000);
}

#[test]
#[should_panic(expected = "Initial deposit too small")]
fn test_deposit_too_small() {
    let env = Env::default();
    env.mock_all_auths();
    let (_, user, token, vault) = setup_test(&env);
    
    let deposit_amount = 500;
    token.mint(&user, &deposit_amount);
    
    vault.deposit(&user, &deposit_amount);
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
    
    // Total balance = 3000
    // Total shares = 1000
    // user2 shares = (1000 * 1000) / 2000 = 500
    assert_eq!(vault.get_user_shares(&user2), 500);
    assert_eq!(vault.get_total_shares(), 1500);
}

#[test]
fn test_consume_credit() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, user, token, vault) = setup_test(&env);
    
    // Initial deposit
    token.mint(&user, &2000);
    vault.deposit(&user, &2000); // 1000 shares
    
    // Consume credit (admin only)
    // total_balance = 2000
    // total_shares = 1000
    // amount = 500 tokens
    // shares_to_burn = (500 * 1000) / 2000 = 250
    vault.consume_credit(&user, &500);
    
    assert_eq!(vault.get_user_shares(&user), 750);
    assert_eq!(vault.get_total_shares(), 750);
}

#[test]
fn test_yield_harvesting() {
    let env = Env::default();
    env.mock_all_auths();
    let (admin, user, token, vault) = setup_test(&env);
    let adapter = Address::generate(&env);
    
    // Initial deposit
    token.mint(&user, &2000);
    vault.deposit(&user, &2000); // 1000 shares, balance = 2000
    
    assert_eq!(vault.get_balance(&user), 2000);
    
    // Set adapter
    vault.set_defi_adapter(&admin, &adapter);
    
    // Delegate liquidity
    vault.delegate_liquidity(&admin, &1000);
    
    // Check balance still same (liquidity moved but still accounted for)
    assert_eq!(vault.get_balance(&user), 2000);
    assert_eq!(vault.get_total_balance(), 2000);
    
    // Harvest yield
    vault.harvest_yield(&adapter, &500);
    
    // Total balance should be 2500 now (2000 + 500)
    // user has 1000 shares out of 1000 total shares
    // balance = (1000 * 2500) / 1000 = 2500
    assert_eq!(vault.get_total_balance(), 2500);
    assert_eq!(vault.get_balance(&user), 2500);
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
