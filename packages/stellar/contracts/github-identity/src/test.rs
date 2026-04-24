#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, testutils::Events as _, Address, Bytes, BytesN, Env, String, Symbol, FromVal};

// Importamos o Registry para mockar nos testes
mod registry_contract {
    soroban_sdk::contractimport!(
        file = "../zolvency-registry/target/wasm32-unknown-unknown/release/zolvency_registry.wasm"
    );
}

#[contract]
pub struct MockAxelarGateway;

#[contractimpl]
impl MockAxelarGateway {
    pub fn call_contract(env: Env, caller: Address, destination_chain: String, destination_address: String, payload: Bytes) {
        env.events().publish(
            (Symbol::new(&env, "call_contract"),),
            (caller, destination_chain, destination_address, payload),
        );
    }
}

#[contract]
pub struct MockAxelarGasService;

#[contractimpl]
impl MockAxelarGasService {
    pub fn pay_gas(
        env: Env,
        sender: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
        spender: Address,
        token: Address,
        amount: i128,
    ) {
        env.events().publish(
            (Symbol::new(&env, "pay_gas"),),
            (sender, destination_chain, destination_address, payload, spender, token, amount),
        );
    }
}

struct TestEnv {
    env: Env,
    client: GithubIdentityContractClient<'static>,
    admin: Address,
    registry: Address,
    fee_token: Address,
    treasury: Address,
    access_control: Address,
}

fn setup() -> TestEnv {
    let env = Env::default();
    env.mock_all_auths();

    // 1. Deploy do Registry (Necessário para o mint funcionar)
    let registry_id = env.register(registry_contract::WASM, ());
    let registry_client = registry_contract::Client::new(&env, &registry_id);
    
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    registry_client.initialize(&admin, &signer);

    // 2. Deploy do Identity Contract
    let contract_id = env.register(GithubIdentityContract, ());
    let client: GithubIdentityContractClient<'static> =
        unsafe { core::mem::transmute(GithubIdentityContractClient::new(&env, &contract_id)) };

    let fee_token = Address::generate(&env);
    let access_control = Address::generate(&env);
    let treasury = Address::generate(&env);

    client.initialize(&admin, &registry_id, &fee_token, &access_control, &treasury, &0);

    TestEnv {
        env,
        client,
        admin,
        registry: registry_id,
        fee_token,
        treasury,
        access_control,
    }
}

fn stub_signature(env: &Env) -> BytesN<64> {
    BytesN::from_array(env, &[0u8; 64])
}

fn stub_passkey(env: &Env) -> BytesN<65> {
    BytesN::from_array(env, &[0x04; 65])
}

fn stub_passkey_signature(env: &Env) -> BytesN<64> {
    BytesN::from_array(env, &[2u8; 64])
}

fn mint_for(ctx: &TestEnv, user: &Address, username: &str, contributions: u32) -> u64 {
    let params = MintParams {
        username: String::from_str(&ctx.env, username),
        external_id: String::from_str(&ctx.env, username),
        passkey: stub_passkey(&ctx.env),
        passkey_signature: stub_passkey_signature(&ctx.env),
        contributions,
        proof_data: Bytes::new(&ctx.env),
        nonce: ctx.client.get_nonce(user),
    };
    ctx.client.mint(
        user,
        &stub_signature(&ctx.env),
        &params,
        &None,
        &None,
    )
}

#[test]
fn test_initialize_sets_mint_fee() {
    let ctx = setup();
    let mint_fee = 1_000_000i128;

    // Reset para testar novo initialize
    let env = ctx.env;
    let contract_id = env.register(GithubIdentityContract, ());
    let client = GithubIdentityContractClient::new(&env, &contract_id);

    client.initialize(&ctx.admin, &ctx.registry, &ctx.fee_token, &ctx.access_control, &ctx.treasury, &mint_fee);

    assert_eq!(client.get_mint_fee(), mint_fee);
}

#[test]
fn test_trait_implementation() {
    let ctx = setup();
    assert_eq!(ctx.client.get_token_type(), Symbol::new(&ctx.env, "github"));
    assert_eq!(ctx.client.get_source(), String::from_str(&ctx.env, "zk-email"));
}

#[test]
fn test_mint_returns_token_id_one() {
    let ctx = setup();
    let user = Address::generate(&ctx.env);
    let token_id = mint_for(&ctx, &user, "devfelipenunes", 1500);
    assert_eq!(token_id, 1);
}

#[test]
fn test_mint_with_passkey_and_expiry() {
    let ctx = setup();
    let user = Address::generate(&ctx.env);
    let passkey = stub_passkey(&ctx.env);
    
    let params = MintParams {
        username: String::from_str(&ctx.env, "user"),
        external_id: String::from_str(&ctx.env, "ext_id"),
        passkey: passkey.clone(),
        passkey_signature: stub_passkey_signature(&ctx.env),
        contributions: 500,
        proof_data: Bytes::new(&ctx.env),
        nonce: 0,
    };

    let token_id = ctx.client.mint(
        &user,
        &stub_signature(&ctx.env),
        &params,
        &None,
        &None,
    );

    assert_eq!(ctx.client.get_owner_passkey(&token_id), passkey);
    assert!(ctx.client.is_valid(&token_id));
}

#[test]
fn test_sybil_resistance_mapping() {
    let ctx = setup();
    let user_a = Address::generate(&ctx.env);
    let external_id = String::from_str(&ctx.env, "github_123");

    let params_a = MintParams {
        username: String::from_str(&ctx.env, "alice"),
        external_id: external_id.clone(),
        passkey: stub_passkey(&ctx.env),
        passkey_signature: stub_passkey_signature(&ctx.env),
        contributions: 100,
        proof_data: Bytes::new(&ctx.env),
        nonce: 0,
    };

    let token_id = ctx.client.mint(
        &user_a,
        &stub_signature(&ctx.env),
        &params_a,
        &None,
        &None,
    );

    assert_eq!(token_id, 1);

    let user_b = Address::generate(&ctx.env);
    let params_b = MintParams {
        username: String::from_str(&ctx.env, "bob"),
        external_id: external_id.clone(),
        passkey: stub_passkey(&ctx.env),
        passkey_signature: stub_passkey_signature(&ctx.env),
        contributions: 200,
        proof_data: Bytes::new(&ctx.env),
        nonce: 0,
    };

    let token_id_2 = ctx.client.mint(
        &user_b,
        &stub_signature(&ctx.env),
        &params_b,
        &None,
        &None,
    );

    assert_eq!(token_id_2, 2);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_mint_empty_username_fails() {
    let ctx = setup();
    let user = Address::generate(&ctx.env);
    mint_for(&ctx, &user, "", 1500);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_mint_twice_same_user_fails() {
    let ctx = setup();
    let user = Address::generate(&ctx.env);

    mint_for(&ctx, &user, "user1", 1500);
    mint_for(&ctx, &user, "user1", 1500);
}

#[test]
fn test_nonce_increments_after_mint() {
    let ctx = setup();
    let user = Address::generate(&ctx.env);

    assert_eq!(ctx.client.get_nonce(&user), 0);
    mint_for(&ctx, &user, "user1", 1500);
    assert_eq!(ctx.client.get_nonce(&user), 1);
}

#[test]
fn test_update_token_refreshes_expiry() {
    let ctx = setup();
    let user = Address::generate(&ctx.env);
    let token_id = mint_for(&ctx, &user, "user1", 1500);
    
    let initial_expiry = ctx.client.get_expiry(&token_id);
    ctx.env.ledger().set_timestamp(ctx.env.ledger().timestamp() + 86400);

    ctx.client.update_token(
        &user,
        &token_id,
        &String::from_str(&ctx.env, "user1"),
        &2000u32,
        &Bytes::new(&ctx.env),
        &None,
    );

    let new_expiry = ctx.client.get_expiry(&token_id);
    assert!(new_expiry > initial_expiry);
}

#[test]
fn test_tier_boundaries() {
    assert_eq!(Tier::from_contributions(0), Tier::Novice);
    assert_eq!(Tier::from_contributions(5000), Tier::Singularity);
}

#[test]
fn test_svg_generation() {
    let ctx = setup();
    let user = Address::generate(&ctx.env);
    let token_id = mint_for(&ctx, &user, "dev", 1500);
    let svg = ctx.client.get_token_svg(&token_id);
    assert!(svg.len() > 0);
}

#[test]
fn test_admin_functions() {
    let ctx = setup();
    ctx.client.set_mint_fee(&ctx.admin, &100i128);
    assert_eq!(ctx.client.get_mint_fee(), 100i128);
}

#[contract]
pub struct MockAdapter;

#[contractimpl]
impl MockAdapter {
    pub fn send(
        env: Env,
        _caller: Address,
        _destination_chain: String,
        _destination_address: String,
        _external_id: String,
        _tier: u32,
        _user_evm_address: Bytes,
    ) -> Result<(), crate::types::Error> {
        env.events().publish(
            (Symbol::new(&env, "adapter_send"),),
            (_destination_chain, _destination_address, _external_id, _tier, _user_evm_address),
        );
        Ok(())
    }
}

#[test]
fn test_adapter_push() {
    let ctx = setup();
    let user = Address::generate(&ctx.env);

    // 1. Setup Mock Adapter
    let adapter_id = ctx.env.register(MockAdapter, ());

    // 2. Configure Adapter in GithubIdentity
    ctx.client.set_active_protocol(&ctx.admin, &InteropProtocol::LayerZero, &adapter_id);

    // 3. Mint with CrossChainParams
    let params = MintParams {
        username: String::from_str(&ctx.env, "felipenunes"),
        external_id: String::from_str(&ctx.env, "felipenunes"),
        passkey: stub_passkey(&ctx.env),
        passkey_signature: stub_passkey_signature(&ctx.env),
        contributions: 1500,
        proof_data: Bytes::new(&ctx.env),
        nonce: 0,
    };

    let cc_params = CrossChainParams {
        destination_chain: String::from_str(&ctx.env, "ethereum-sepolia"),
        destination_address: String::from_str(&ctx.env, "0x123"),
        user_destination_address: Bytes::from_array(&ctx.env, &[0u8; 20]),
    };

    ctx.client.mint(
        &user,
        &stub_signature(&ctx.env),
        &params,
        &None,
        &Some(cc_params),
    );

    // 4. Verify Events
    let events = ctx.env.events().all();
    let has_adapter_event = events.iter().any(|e| {
        e.1.get(0).map(|v| Symbol::from_val(&ctx.env, &v) == Symbol::new(&ctx.env, "adapter_send")).unwrap_or(false)
    });

    assert!(has_adapter_event, "Missing adapter_send event");
}
