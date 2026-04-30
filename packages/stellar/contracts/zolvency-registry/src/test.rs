use super::*;
use soroban_sdk::{
    contract, contractimpl,
    testutils::Address as _, testutils::Ledger as _, Address, Bytes, BytesN, Env, String, Symbol,
};

use github_identity::{GithubIdentityContract, GithubIdentityContractClient, MintParams};
use income_bank::{
    IncomeBankContract, IncomeBankContractClient, MintParams as BankMintParams, RenewalWindow,
    RevealMode,
};
use binance_kyc::{
    BinanceKycContract, BinanceKycContractClient, KycLevel, MintParams as KycMintParams,
    RenewalWindow as KycRenewalWindow,
};
use uber_income::{
    IncomePeriod, InitializeParams as UberInitializeParams, MintParams as UberMintParams,
    RenewalWindow as UberRenewalWindow, RevealMode as UberRevealMode, UberIncomeContract,
    UberIncomeContractClient,
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

#[test]
fn test_registry_integration_with_github_token() {
    let env = Env::default();
    env.mock_all_auths();

    // Mock Soul (necessário pro gating do GitHub)
    let soul_id = env.register(MockSoul, ());
    let soul_client = MockSoulClient::new(&env, &soul_id);

    // 1. Deploy do Registry
    let registry_id = env.register(ZolvencyRegistry, ());
    let registry_client = ZolvencyRegistryClient::new(&env, &registry_id);

    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    registry_client.initialize(&admin, &signer);

    // 2. Deploy do Github Identity (Spoke)
    let github_id = env.register(GithubIdentityContract, ());
    let github_client = GithubIdentityContractClient::new(&env, &github_id);

    let fee_token = Address::generate(&env);
    let access_control = Address::generate(&env);
    let treasury = Address::generate(&env);

    github_client.initialize(
        &admin,
        &registry_id,             // registry
        &soul_id,                 // soul_contract
        &fee_token,               // fee_token
        &access_control,          // access_control
        &treasury,                // treasury
        &0,                       // mint_fee
    );

    // 3. Registrar o token no Registry
    registry_client.register_token(&admin, &github_id);

    // 4. Usuário minta um token no GitHub Contract
    let user = Address::generate(&env);

    // habilita soul pro user
    soul_client.set_balance(&user, &1u32);

    let params = MintParams {
        username: String::from_str(&env, "devfelipenunes"),
        external_id: String::from_str(&env, "gh_123"),
        passkey: Bytes::from_array(&env, &[1u8; 65]),
        passkey_signature: Bytes::from_array(&env, &[0u8; 64]),
        contributions: 1500u32,
        proof_data: Bytes::new(&env),
        nonce: 0u64,
    };

    // assinatura nova: (caller, user, params)
    github_client.mint(&user, &user, &params);

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

#[test]
fn test_registry_with_new_tokens() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1714316400); // 2024-04-28

    // Mock Soul (necessário pro gating dos spokes)
    let soul_id = env.register(MockSoul, ());
    let soul_client = MockSoulClient::new(&env, &soul_id);

    let registry_id = env.register(ZolvencyRegistry, ());
    let registry_client = ZolvencyRegistryClient::new(&env, &registry_id);

    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    registry_client.initialize(&admin, &signer);

    let bank_id = env.register(IncomeBankContract, ());
    let bank_client = IncomeBankContractClient::new(&env, &bank_id);
    bank_client.initialize(
        &admin,
        &registry_id,
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
        &0,
        &0,
        &0,
        &0,
        &false,
    );

    // Configura soul gating
    bank_client.set_soul_contract(&admin, &soul_id);

    let kyc_id = env.register(BinanceKycContract, ());
    let kyc_client = BinanceKycContractClient::new(&env, &kyc_id);
    kyc_client.initialize(
        &admin,
        &registry_id,
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
        &0,
        &0,
        &0,
        &false,
    );

    // Configura soul gating
    kyc_client.set_soul_contract(&admin, &soul_id);

    let uber_id = env.register(UberIncomeContract, ());
    let uber_client = UberIncomeContractClient::new(&env, &uber_id);
    let uber_init = UberInitializeParams {
        admin: admin.clone(),
        registry: registry_id.clone(),
        soul_contract: soul_id.clone(),
        fee_token: Address::generate(&env),
        access_control: Address::generate(&env),
        treasury: Address::generate(&env),
        mint_fee_30: 0,
        mint_fee_60: 0,
        mint_fee_90: 0,
        max_proof_age_seconds: 0,
        store_proof_data: false,
    };
    uber_client.initialize(&uber_init);

    registry_client.register_token(&admin, &bank_id);
    registry_client.register_token(&admin, &kyc_id);
    registry_client.register_token(&admin, &uber_id);

    let user = Address::generate(&env);

    // habilita soul pro user (mint do bank/kyc/uber usa recipient)
    soul_client.set_balance(&user, &1u32);

    let bank_params = BankMintParams {
        recipient: user.clone(),
        external_id: String::from_str(&env, "bank_user"),
        income_band: 2,
        income_value: Some(4_000),
        reveal_mode: RevealMode::Exact,
        currency: String::from_str(&env, "USD"),
        verified_at: env.ledger().timestamp(),
        proof_hash: BytesN::from_array(&env, &[1u8; 32]),
        proof_data: Bytes::new(&env),
        window: RenewalWindow::Days30,
        nonce: 0,
    };
    bank_client.mint(&admin, &bank_params, &None);

    let kyc_params = KycMintParams {
        recipient: user.clone(),
        external_id: String::from_str(&env, "binance_user"),
        kyc_level: KycLevel::Advanced,
        country: String::from_str(&env, "BR"),
        verified_at: env.ledger().timestamp(),
        proof_hash: BytesN::from_array(&env, &[2u8; 32]),
        proof_data: Bytes::new(&env),
        window: KycRenewalWindow::Days60,
        nonce: 0,
    };
    kyc_client.mint(&admin, &kyc_params, &None);

    let uber_params = UberMintParams {
        recipient: user.clone(),
        external_id: String::from_str(&env, "uber_user"),
        income_band: 3,
        income_value: Some(5000),
        reveal_mode: UberRevealMode::Exact,
        currency: String::from_str(&env, "USD"),
        period: IncomePeriod::Monthly,
        verified_at: env.ledger().timestamp(),
        proof_hash: BytesN::from_array(&env, &[3u8; 32]),
        proof_data: Bytes::new(&env),
        window: UberRenewalWindow::Days90,
        nonce: 0,
    };
    uber_client.mint(&admin, &uber_params, &None);

    let reputation = registry_client.get_user_reputation(&user);
    assert!(reputation.contains_key(Symbol::new(&env, "bank")));
    assert!(reputation.contains_key(Symbol::new(&env, "binance_kyc")));
    assert!(reputation.contains_key(Symbol::new(&env, "uber")));
}

#[test]
fn test_registry_lock_and_blacklist() {
    let env = Env::default();
    env.mock_all_auths();

    let registry_id = env.register(ZolvencyRegistry, ());
    let registry_client = ZolvencyRegistryClient::new(&env, &registry_id);

    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    registry_client.initialize(&admin, &signer);

    let user = Address::generate(&env);
    let now = env.ledger().timestamp();
    registry_client.lock_reputation(&admin, &user, &(now + 60));

    assert!(registry_client.is_locked(&user));

    env.ledger().set_timestamp(now + 120);
    assert!(!registry_client.is_locked(&user));

    registry_client.apply_slashing(&admin, &user);
    assert!(registry_client.is_blacklisted(&user));
}

#[test]
fn test_registry_get_token_metadata() {
    let env = Env::default();
    env.mock_all_auths();

    // Mock Soul (necessário pro gating do bank se usarmos mint em algum momento)
    let soul_id = env.register(MockSoul, ());

    let registry_id = env.register(ZolvencyRegistry, ());
    let registry_client = ZolvencyRegistryClient::new(&env, &registry_id);

    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    registry_client.initialize(&admin, &signer);

    let bank_id = env.register(IncomeBankContract, ());
    let bank_client = IncomeBankContractClient::new(&env, &bank_id);
    bank_client.initialize(
        &admin,
        &registry_id,
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
        &0,
        &0,
        &0,
        &0,
        &false,
    );

    bank_client.set_soul_contract(&admin, &soul_id);

    let metadata = registry_client.get_token_metadata(&bank_id);
    assert_eq!(metadata.name, String::from_str(&env, "Zolvency Bank Income"));
}
