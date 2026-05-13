#![cfg(test)]

use super::*;

use soroban_sdk::{
    testutils::Address as _, testutils::Ledger as _, Address, Bytes, BytesN, Env, String, Symbol,
};

#[contract]
pub struct MockSoul;

#[contractimpl]
impl MockSoul {
    pub fn set_soul(env: Env, soul_id: u32, exists: bool) {
        let key = (Symbol::new(&env, "soul"), soul_id);
        env.storage().instance().set(&key, &exists);
    }

    pub fn get_soul(env: Env, soul_id: u32) -> Option<bool> {
        let key = (Symbol::new(&env, "soul"), soul_id);
        if env.storage().instance().get(&key).unwrap_or(false) {
            Some(true)
        } else {
            None
        }
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
    let soul_client = MockSoulClient::new(&env, &soul_contract);

    let contract_id = env.register(GithubIdentityContract, ());
    let client = GithubIdentityContractClient::new(&env, &contract_id);

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

fn setup_with_fee(fee: i128) -> (TestEnv, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let soul_contract = env.register(MockSoul, ());
    let soul_client = MockSoulClient::new(&env, &soul_contract);

    let contract_id = env.register(GithubIdentityContract, ());
    let client = GithubIdentityContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let registry = Address::generate(&env);
    
    // Registrar um token de asset real (ou mockado via SDK) para taxas
    let fee_token = env.register_stellar_asset_contract_v2(admin.clone()).address();
    
    let access_control = Address::generate(&env);
    let treasury = Address::generate(&env);

    client.initialize(
        &admin,
        &registry,
        &soul_contract,
        &fee_token,
        &access_control,
        &treasury,
        &fee,
    );

    (TestEnv {
        env,
        client,
        soul_client,
        soul_contract,
    }, admin, fee_token)
}

fn passkey_bytes(env: &Env) -> Bytes {
    Bytes::from_array(env, &[1u8; 65])
}

fn mint_for(ctx: &TestEnv, caller: &Address, soul_id: u32, username: &str, contributions: u32) -> u64 {
    let params = MintParams {
        username: String::from_str(&ctx.env, username),
        external_id: String::from_str(&ctx.env, username),
        contributions,
        proof: ReclaimProof {
            claim_info: ClaimInfo {
                provider: String::from_str(&ctx.env, "github"),
                parameters: String::from_str(&ctx.env, username),
                context: String::from_str(&ctx.env, "soul_id:1"), // Simplificado para teste
            },
            signed_claim: BytesN::from_array(&ctx.env, &[0u8; 32]),
            signatures: soroban_sdk::vec![&ctx.env, BytesN::from_array(&ctx.env, &[0u8; 64])],
            witness_address: BytesN::from_array(&ctx.env, &[0u8; 32]),
        },
        nonce: 0,
    };
    ctx.client.mint(caller, &soul_id, &params, &None)
}

#[test]
fn test_trait_implementation() {
    let ctx = setup();
    assert_eq!(ctx.client.get_token_type(), Symbol::new(&ctx.env, "github"));
    assert_eq!(
        ctx.client.get_source(),
        String::from_str(&ctx.env, "github")
    );



    let md = ctx.client.get_metadata();
    assert_eq!(md.symbol, String::from_str(&ctx.env, "ZOLV-GH"));
}

#[test]
fn test_mint_returns_token_id_one() {
    let ctx = setup();
    let caller = Address::generate(&ctx.env);
    let soul_id = 1u32;

    ctx.soul_client.set_soul(&soul_id, &true);

    let token_id = mint_for(&ctx, &caller, soul_id, "devfelipenunes", 1500);
    assert_eq!(token_id, 1);
}

#[test]
fn test_mint_with_passkey_and_expiry_and_validity() {
    let ctx = setup();
    let caller = Address::generate(&ctx.env);
    let soul_id = 1u32;

    ctx.soul_client.set_soul(&soul_id, &true);

    let params = MintParams {
        username: String::from_str(&ctx.env, "user"),
        external_id: String::from_str(&ctx.env, "ext_id"),
        contributions: 500,
        proof: ReclaimProof {
            claim_info: ClaimInfo {
                provider: String::from_str(&ctx.env, "github"),
                parameters: String::from_str(&ctx.env, "ext_id"),
                context: String::from_str(&ctx.env, "soul_id:1"),
            },
            signed_claim: BytesN::from_array(&ctx.env, &[0u8; 32]),
            signatures: soroban_sdk::vec![&ctx.env, BytesN::from_array(&ctx.env, &[0u8; 64])],
            witness_address: BytesN::from_array(&ctx.env, &[0u8; 32]),
        },
        nonce: 0,
    };

    let token_id = ctx.client.mint(&caller, &soul_id, &params, &None);
    assert_eq!(token_id, 1);
    
    assert!(ctx.client.is_valid(&token_id));
    assert_eq!(ctx.client.get_owner_soul(&token_id), soul_id);
}

#[test]
fn test_mint_requires_soul() {
    let ctx = setup();
    let caller = Address::generate(&ctx.env);
    let soul_id = 1u32;

    let params = MintParams {
        username: String::from_str(&ctx.env, "user"),
        external_id: String::from_str(&ctx.env, "ext_id"),
        contributions: 100,
        proof: ReclaimProof {
            claim_info: ClaimInfo {
                provider: String::from_str(&ctx.env, "github"),
                parameters: String::from_str(&ctx.env, "ext_id"),
                context: String::from_str(&ctx.env, "soul_id:1"),
            },
            signed_claim: BytesN::from_array(&ctx.env, &[0u8; 32]),
            signatures: soroban_sdk::vec![&ctx.env, BytesN::from_array(&ctx.env, &[0u8; 64])],
            witness_address: BytesN::from_array(&ctx.env, &[0u8; 32]),
        },
        nonce: 0,
    };

    let res = ctx.client.try_mint(&caller, &soul_id, &params, &None);
    assert_eq!(res, Err(Ok(Error::Unauthorized)));
}

#[test]
fn test_initialize_already_initialized() {
    let ctx = setup();
    let admin = Address::generate(&ctx.env);
    let res = ctx.client.try_initialize(&admin, &admin, &admin, &admin, &admin, &admin, &0);
    assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn test_mint_invalid_nonce() {
    let ctx = setup();
    let caller = Address::generate(&ctx.env);
    let soul_id = 1u32;
    ctx.soul_client.set_soul(&soul_id, &true);

    let params = MintParams {
        username: String::from_str(&ctx.env, "user"),
        external_id: String::from_str(&ctx.env, "ext_id"),
        contributions: 100,
        proof: ReclaimProof {
            claim_info: ClaimInfo {
                provider: String::from_str(&ctx.env, "github"),
                parameters: String::from_str(&ctx.env, "ext_id"),
                context: String::from_str(&ctx.env, "soul_id:1"),
            },
            signed_claim: BytesN::from_array(&ctx.env, &[0u8; 32]),
            signatures: soroban_sdk::vec![&ctx.env, BytesN::from_array(&ctx.env, &[0u8; 64])],
            witness_address: BytesN::from_array(&ctx.env, &[0u8; 32]),
        },
        nonce: 1, // Nonce errado
    };

    let res = ctx.client.try_mint(&caller, &soul_id, &params, &None);
    assert_eq!(res, Err(Ok(Error::InvalidNonce)));
}

#[test]
fn test_upgrade_requires_admin() {
    // Para testar falha de auth, criamos um env SEM mock_all_auths
    let env = Env::default();
    let contract_id = env.register(GithubIdentityContract, ());
    let client = GithubIdentityContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);
    
    // Inicializa com mock para poder setar o admin
    env.mock_all_auths();
    client.initialize(&admin, &Address::generate(&env), &Address::generate(&env), &Address::generate(&env), &Address::generate(&env), &Address::generate(&env), &0);
    
    // Tenta dar upgrade como atacante SEM mock_all_auths
    let res = client.try_upgrade(&attacker, &BytesN::from_array(&env, &[0u8; 32]));
    assert!(res.is_err());
}
