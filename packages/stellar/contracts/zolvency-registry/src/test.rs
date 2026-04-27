#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env, String, Symbol};

// Importamos o contrato de identidade para usar no teste
mod github_contract {
    soroban_sdk::contractimport!(
        file = "../github-identity/target/wasm32-unknown-unknown/release/github_identity.wasm"
    );
}

#[test]
fn test_registry_integration_with_github_token() {
    let env = Env::default();
    env.mock_all_auths();

    // 1. Deploy do Registry
    let registry_id = env.register(ZolvencyRegistry, ());
    let registry_client = ZolvencyRegistryClient::new(&env, &registry_id);

    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    registry_client.initialize(&admin, &signer);

    // 2. Deploy do Github Identity (Spoke)
    let github_id = env.register(github_contract::WASM, ());
    let github_client = github_contract::Client::new(&env, &github_id);

    github_client.initialize(
        &admin,
        &Address::generate(&env), // registry
        &Address::generate(&env), // fee_token
        &Address::generate(&env), // access_control
        &Address::generate(&env), // treasury
        &0,                       // mint_fee
    );

    // 3. Registrar o token no Registry
    registry_client.register_token(&admin, &github_id);

    // 4. Usuário minta um token no GitHub Contract
    let user = Address::generate(&env);
    let signature = BytesN::from_array(&env, &[0u8; 64]);

    let params = github_contract::MintParams {
        username: String::from_str(&env, "devfelipenunes"),
        external_id: String::from_str(&env, "gh_123"),
        passkey: None,           // Alterado para None
        passkey_signature: None, // Alterado para None
        contributions: 1500u32,
        proof_data: Bytes::new(&env),
        nonce: 0u64,
    };

    github_client.mint(&user, &signature, &params, &None, &None);

    // 5. Consultar reputação via Registry (O que o SDK fará)
    let reputation = registry_client.get_user_reputation(&user);

    // Verificações
    assert!(reputation.contains_key(Symbol::new(&env, "github")));
    assert_eq!(reputation.get(Symbol::new(&env, "github")), Some(1u64));

    // Testar usuário sem token
    let ghost_user = Address::generate(&env);
    let empty_reputation = registry_client.get_user_reputation(&ghost_user);
    assert_eq!(empty_reputation.len(), 0);
}
