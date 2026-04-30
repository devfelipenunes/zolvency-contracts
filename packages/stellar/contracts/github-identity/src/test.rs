#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _, testutils::Ledger as _, Address, Bytes, Env, String, Symbol,
};

#[contract]
pub struct MockSoul;

#[contractimpl]
impl MockSoul {
    pub fn set_balance(env: Env, user: Address, balance: u32) {
        let key = (Symbol::new(&env, "bal"), user);
        env.storage().instance().set(&key, &balance);
    }

    pub fn balance(env: Env, user: Address) -> u32 {
        let key = (Symbol::new(&env, "bal"), user);
        env.storage().instance().get(&key).unwrap_or(0u32)
    }
}

struct TestEnv {
    env: Env,
    client: GithubIdentityContractClient<'static>,
    soul_client: MockSoulClient<'static>,
    soul_contract: Address,
}

fn setup() -> TestEnv {
    let env = Env::default();
    env.mock_all_auths();

    let soul_contract = env.register(MockSoul, ());
    let soul_client: MockSoulClient<'static> =
        unsafe { core::mem::transmute(MockSoulClient::new(&env, &soul_contract)) };

    let contract_id = env.register(GithubIdentityContract, ());
    let client: GithubIdentityContractClient<'static> =
        unsafe { core::mem::transmute(GithubIdentityContractClient::new(&env, &contract_id)) };

    let admin = Address::generate(&env);
    let registry = Address::generate(&env);
    let fee_token = Address::generate(&env);
    let access_control = Address::generate(&env);
    let treasury = Address::generate(&env);
    let mint_fee = 0i128;

    client.initialize(
        &admin,
        &registry,
        &soul_contract,
        &fee_token,
        &access_control,
        &treasury,
        &mint_fee,
    );

    TestEnv {
        env,
        client,
        soul_client,
        soul_contract,
    }
}

fn passkey_bytes(env: &Env) -> Bytes {
    Bytes::from_array(env, &[1u8; 65])
}

fn mint_for(ctx: &TestEnv, caller: &Address, user: &Address, username: &str, contributions: u32) -> u64 {
    let params = MintParams {
        username: String::from_str(&ctx.env, username),
        external_id: String::from_str(&ctx.env, username),
        passkey: passkey_bytes(&ctx.env),
        passkey_signature: Bytes::from_array(&ctx.env, &[0u8; 64]),
        contributions,
        proof_data: Bytes::new(&ctx.env),
        nonce: 0,
    };
    ctx.client.mint(caller, user, &params)
}

#[test]
fn test_trait_implementation() {
    let ctx = setup();
    assert_eq!(ctx.client.get_token_type(), Symbol::new(&ctx.env, "github"));
    assert_eq!(
        ctx.client.get_source(),
        String::from_str(&ctx.env, "zk-email")
    );

    let md = ctx.client.get_metadata();
    assert_eq!(md.symbol, String::from_str(&ctx.env, "ZOLV-GH"));
}

#[test]
fn test_mint_returns_token_id_one() {
    let ctx = setup();
    let caller = Address::generate(&ctx.env);
    let user = Address::generate(&ctx.env);

    ctx.soul_client.set_balance(&user, &1u32);

    let token_id = mint_for(&ctx, &caller, &user, "devfelipenunes", 1500);
    assert_eq!(token_id, 1);
}

#[test]
fn test_mint_with_passkey_and_expiry_and_validity() {
    let ctx = setup();
    let caller = Address::generate(&ctx.env);
    let user = Address::generate(&ctx.env);

    ctx.soul_client.set_balance(&user, &1u32);

    let passkey = passkey_bytes(&ctx.env);

    let params = MintParams {
        username: String::from_str(&ctx.env, "user"),
        external_id: String::from_str(&ctx.env, "ext_id"),
        passkey: passkey.clone(),
        passkey_signature: Bytes::from_array(&ctx.env, &[0u8; 64]),
        contributions: 500,
        proof_data: Bytes::new(&ctx.env),
        nonce: 0,
    };

    let token_id = ctx.client.mint(&caller, &user, &params);

    assert_eq!(ctx.client.get_owner_passkey(&token_id), passkey);
    assert!(ctx.client.is_valid(&token_id));

    let expiry = ctx.client.get_expiry(&token_id);
    ctx.env.ledger().set_timestamp(expiry + 1);
    assert!(!ctx.client.is_valid(&token_id));
}

#[test]
#[should_panic(expected = "Unauthorized: No Soul Token detected")]
fn test_mint_requires_soul() {
    let ctx = setup();
    let caller = Address::generate(&ctx.env);
    let user = Address::generate(&ctx.env);

    let params = MintParams {
        username: String::from_str(&ctx.env, "user"),
        external_id: String::from_str(&ctx.env, "ext_id"),
        passkey: passkey_bytes(&ctx.env),
        passkey_signature: Bytes::from_array(&ctx.env, &[0u8; 64]),
        contributions: 100,
        proof_data: Bytes::new(&ctx.env),
        nonce: 0,
    };

    ctx.client.mint(&caller, &user, &params);
}
