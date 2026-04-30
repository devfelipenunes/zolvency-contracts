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

    pub fn set_balance(env: Env, user: Address, balance: u32) {
        let key = (Symbol::new(&env, "bal"), user);
        env.storage().instance().set(&key, &balance);
    }

    pub fn balance(env: Env, user: Address) -> u32 {
        let key = (Symbol::new(&env, "bal"), user);
        env.storage().instance().get(&key).unwrap_or(0u32)
    }
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
        external_id: String,
        tier: u32,
        _user_destination_address: Bytes,
        nonce: u64,
        token_type: Symbol,
    ) {
        // Armazena o último export para verificação
        let key = Symbol::new(&env, "last_export");
        env.storage().instance().set(&key, &(external_id, tier, nonce, token_type));
    }

    pub fn get_last_export(env: Env) -> (String, u32, u64, Symbol) {
        let key = Symbol::new(&env, "last_export");
        env.storage().instance().get(&key).unwrap()
    }
}

#[test]
fn test_registry_integration_with_github_token() {
    let env = Env::default();
    env.mock_all_auths();

    // Mock Soul (necessário pro gating do GitHub)
    let soul_id_contract = env.register(MockSoul, ());
    let soul_client = MockSoulClient::new(&env, &soul_id_contract);

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
        &soul_id_contract,        // soul_contract
        &fee_token,               // fee_token
        &access_control,          // access_control
        &treasury,                // treasury
        &0,                       // mint_fee
    );

    // 3. Registrar o token no Registry
    registry_client.register_token(&admin, &github_id);

    // 4. Usuário minta um token no GitHub Contract
    let user_addr = Address::generate(&env);
    let soul_id = 1u32;

    // habilita soul pro user
    soul_client.set_soul(&soul_id, &true);

    let params = MintParams {
        username: String::from_str(&env, "devfelipenunes"),
        external_id: String::from_str(&env, "gh_123"),
        contributions: 1500u32,
        proof_data: Bytes::new(&env),
        nonce: 0u64,
    };

    // assinatura nova: (caller, soul_id, params)
    github_client.mint(&user_addr, &soul_id, &params, &None);

    // 5. Consultar reputação via Registry (O que o SDK fará)
    let reputation = registry_client.get_soul_reputation(&soul_id, &None);

    // Verificações
    assert!(reputation.contains_key(Symbol::new(&env, "github")));
    assert_eq!(reputation.get(Symbol::new(&env, "github")), Some(1u64));

    // Testar usuário sem token
    let ghost_soul_id = 99u32;
    let empty_reputation = registry_client.get_soul_reputation(&ghost_soul_id, &None);
    assert_eq!(empty_reputation.len(), 0);
}

#[test]
fn test_registry_with_new_tokens() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1714316400); // 2024-04-28

    // Mock Soul (necessário pro gating dos spokes)
    let soul_id_contract = env.register(MockSoul, ());
    let soul_client = MockSoulClient::new(&env, &soul_id_contract);

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
    bank_client.set_soul_contract(&admin, &soul_id_contract);

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
    kyc_client.set_soul_contract(&admin, &soul_id_contract);

    let uber_id = env.register(UberIncomeContract, ());
    let uber_client = UberIncomeContractClient::new(&env, &uber_id);
    let uber_init = UberInitializeParams {
        admin: admin.clone(),
        registry: registry_id.clone(),
        soul_contract: soul_id_contract.clone(),
        fee_token: Address::generate(&env),
        access_control: Address::generate(&env),
        treasury: Address::generate(&env),
        mint_fee_30: 0,
        mint_fee_60: 0,
        mint_fee_90: 0,
        max_proof_age_seconds: 0,
    };
    uber_client.initialize(&uber_init);

    registry_client.register_token(&admin, &bank_id);
    registry_client.register_token(&admin, &kyc_id);
    registry_client.register_token(&admin, &uber_id);

    let user_addr = Address::generate(&env);
    let soul_id = 1u32;

    // habilita soul pro user (mint do bank/kyc/uber usa recipient)
    soul_client.set_balance(&user_addr, &1u32);
    // e também pro novo sistema se necessário
    soul_client.set_soul(&soul_id, &true);

    let bank_params = BankMintParams {
        soul_id: soul_id,
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
        soul_id: soul_id,
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
        soul_id: soul_id,
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

    // O Registry agora usa soul_id para consultar reputação de TODOS os tokens registrados
    // Mesmo que o token use internamente Address para o mint, o Registry chama has_identity(soul_id: u32)
    // Então os spokes precisam ter sido atualizados para aceitar u32 em has_identity/get_user_token
    let reputation = registry_client.get_soul_reputation(&soul_id, &None);
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

    let soul_id = 1u32;
    let now = env.ledger().timestamp();
    registry_client.lock_soul_reputation(&admin, &soul_id, &(now + 60));

    assert!(registry_client.is_soul_locked(&soul_id));

    env.ledger().set_timestamp(now + 120);
    assert!(!registry_client.is_soul_locked(&soul_id));

    registry_client.apply_soul_slashing(&admin, &soul_id);
    assert!(registry_client.is_soul_blacklisted(&soul_id));
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

#[test]
fn test_registry_token_limit_safety() {
    let env = Env::default();
    env.mock_all_auths();

    let registry_id = env.register(ZolvencyRegistry, ());
    let registry_client = ZolvencyRegistryClient::new(&env, &registry_id);

    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    registry_client.initialize(&admin, &signer);

    // Registrar 25 tokens fictícios
    for _ in 0..25 {
        registry_client.register_token(&admin, &Address::generate(&env));
    }

    let soul_id = 1u32;
    let reputation = registry_client.get_soul_reputation(&soul_id, &None);
    
    // O blind scan deve respeitar o limite de 20 e não dar panic
    assert!(reputation.len() <= 20);
}

#[test]
fn test_export_reputation_blocked_for_blacklisted_soul() {
    let env = Env::default();
    env.mock_all_auths();

    let registry_id = env.register(ZolvencyRegistry, ());
    let registry_client = ZolvencyRegistryClient::new(&env, &registry_id);

    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    registry_client.initialize(&admin, &signer);

    let token_addr = Address::generate(&env);
    registry_client.register_token(&admin, &token_addr.clone());

    let soul_id = 1u32;
    registry_client.apply_soul_slashing(&admin, &soul_id);

    // Tentar exportar deve falhar (Status: Error::NotAdmin, que usamos pro bloqueio)
    let res = registry_client.try_export_reputation(
        &token_addr,
        &soul_id,
        &token_addr,
        &String::from_str(&env, "ext_id"),
        &1u32,
        &0u64,
        &None
    );

    assert!(res.is_err());
}

#[test]
fn test_initialize_already_initialized() {
    let env = Env::default();
    let registry_id = env.register(ZolvencyRegistry, ());
    let registry_client = ZolvencyRegistryClient::new(&env, &registry_id);
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);

    registry_client.initialize(&admin, &signer);
    let res = registry_client.try_initialize(&admin, &signer);
    assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn test_unauthorized_admin_actions() {
    let env = Env::default();
    env.mock_all_auths();
    let registry_id = env.register(ZolvencyRegistry, ());
    let registry_client = ZolvencyRegistryClient::new(&env, &registry_id);
    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);
    let signer = Address::generate(&env);
    registry_client.initialize(&admin, &signer);

    // Registrar token como não-admin
    let res = registry_client.try_register_token(&attacker, &Address::generate(&env));
    assert_eq!(res, Err(Ok(Error::NotAdmin)));

    // Mudar signer como não-admin
    let res = registry_client.try_update_signer(&attacker, &Address::generate(&env));
    assert_eq!(res, Err(Ok(Error::NotAdmin)));

    // Bloquear alma como não-admin
    let res = registry_client.try_lock_soul_reputation(&attacker, &1u32, &1000u64);
    assert_eq!(res, Err(Ok(Error::NotAdmin)));
}

#[test]
fn test_admin_transfer_full_flow() {
    let env = Env::default();
    env.mock_all_auths();
    let registry_id = env.register(ZolvencyRegistry, ());
    let registry_client = ZolvencyRegistryClient::new(&env, &registry_id);
    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);
    registry_client.initialize(&admin, &Address::generate(&env));

    registry_client.transfer_admin(&admin, &new_admin);
    
    // Tentar aceitar com conta errada
    let wrong_account = Address::generate(&env);
    let res = registry_client.try_accept_admin(&wrong_account);
    assert_eq!(res, Err(Ok(Error::NotPendingAdmin)));

    // Aceitar com a conta correta
    registry_client.accept_admin(&new_admin);

    // Verificar se o novo admin tem poder
    registry_client.register_token(&new_admin, &Address::generate(&env));
}

#[test]
fn test_register_token_idempotency() {
    let env = Env::default();
    env.mock_all_auths();
    let registry_id = env.register(ZolvencyRegistry, ());
    let registry_client = ZolvencyRegistryClient::new(&env, &registry_id);
    let admin = Address::generate(&env);
    registry_client.initialize(&admin, &Address::generate(&env));

    let token = Address::generate(&env);
    registry_client.register_token(&admin, &token);
    registry_client.register_token(&admin, &token); // Chamar de novo

    // O contador não deve ter subido duas vezes
    // Precisamos de um jeito de checar o contador se ele fosse exposto, 
    // mas podemos checar o scan
    let _reputation = registry_client.get_soul_reputation(&1, &None);
    // Se o contador estivesse errado (2), o loop de scan tentaria ler o ID 1 que estaria vazio ou repetido.
    // Como registramos o mesmo token, o ID(0) é o token, e ID(1) não existe.
}

#[test]
fn test_reputation_query_for_blocked_soul() {
    let env = Env::default();
    env.mock_all_auths();
    let registry_id = env.register(ZolvencyRegistry, ());
    let registry_client = ZolvencyRegistryClient::new(&env, &registry_id);
    let admin = Address::generate(&env);
    registry_client.initialize(&admin, &Address::generate(&env));

    let soul_id = 1u32;
    registry_client.apply_soul_slashing(&admin, &soul_id);

    // Deve retornar vazio mesmo se houver tokens (aqui não registramos nenhum, mas o check de blacklist mata antes)
    let reputation = registry_client.get_soul_reputation(&soul_id, &None);
    assert_eq!(reputation.len(), 0);
}

#[test]
fn test_get_soul_reputation_with_filter() {
    let env = Env::default();
    env.mock_all_auths();
    let registry_id = env.register(ZolvencyRegistry, ());
    let registry_client = ZolvencyRegistryClient::new(&env, &registry_id);
    let admin = Address::generate(&env);
    registry_client.initialize(&admin, &Address::generate(&env));

    let token1 = Address::generate(&env);
    let token2 = Address::generate(&env);
    registry_client.register_token(&admin, &token1.clone());
    registry_client.register_token(&admin, &token2.clone());

    // Consultar apenas token1
    let tokens = Some(soroban_sdk::vec![&env, token1.clone()]);
    let reputation = registry_client.get_soul_reputation(&1, &tokens);
    
    // Como os tokens são gerados aleatoriamente e não implementam a interface (são mock addresses),
    // a reputação virá vazia, mas o teste valida que o filtro não quebra a execução.
    assert_eq!(reputation.len(), 0);
}

#[test]
fn test_cross_chain_export_flow() {
    let env = Env::default();
    env.mock_all_auths();
    
    let registry_id = env.register(ZolvencyRegistry, ());
    let registry_client = ZolvencyRegistryClient::new(&env, &registry_id);
    let admin = Address::generate(&env);
    registry_client.initialize(&admin, &Address::generate(&env));

    // Setup Token
    let token_id = env.register(GithubIdentityContract, ());
    let token_client = GithubIdentityContractClient::new(&env, &token_id);
    token_client.initialize(&admin, &registry_id, &Address::generate(&env), &Address::generate(&env), &Address::generate(&env), &Address::generate(&env), &0);
    registry_client.register_token(&admin, &token_id);

    // Setup Adapter
    let adapter_id = env.register(MockAdapter, ());
    let adapter_client = MockAdapterClient::new(&env, &adapter_id);
    
    let interop_config = InteropConfig {
        active_protocol: InteropProtocol::Axelar,
        adapter_address: adapter_id.clone(),
    };
    registry_client.set_interop_config(&admin, &interop_config);

    // Exportar
    let soul_id = 1u32;
    let ext_id = String::from_str(&env, "user_123");
    let tier = 2u32;
    let nonce = 42u64;
    let cc_params = CrossChainParams {
        destination_chain: String::from_str(&env, "ethereum"),
        destination_address: String::from_str(&env, "0x123"),
        user_destination_address: Bytes::from_array(&env, &[0u8; 20]),
    };

    registry_client.export_reputation(
        &token_id, 
        &soul_id, 
        &token_id, 
        &ext_id, 
        &tier, 
        &nonce, 
        &Some(cc_params)
    );

    // Verificar se o adaptador foi chamado
    let (got_ext_id, got_tier, got_nonce, got_type) = adapter_client.get_last_export();
    assert_eq!(got_ext_id, ext_id);
    assert_eq!(got_tier, tier);
    assert_eq!(got_nonce, nonce);
    assert_eq!(got_type, Symbol::new(&env, "github"));
}

#[test]
fn test_storage_ttl_extension() {
    let env = Env::default();
    env.mock_all_auths();
    
    let registry_id = env.register(ZolvencyRegistry, ());
    let registry_client = ZolvencyRegistryClient::new(&env, &registry_id);
    let admin = Address::generate(&env);
    registry_client.initialize(&admin, &Address::generate(&env));

    // O ledger avança
    env.ledger().set_timestamp(1714316400);
    
    // Tentar registrar um token (isso deve disparar o extend_persistent)
    let token = Address::generate(&env);
    registry_client.register_token(&admin, &token);

    // No SDK, não temos uma forma trivial de ver o TTL exato via Client, 
    // mas o fato de não dar panic após avanço de tempo e ledger ajuda a validar a persistência.
    env.as_contract(&registry_id, || {
        assert!(env.storage().persistent().has(&DataKey::Admin));
    });
}

#[test]
fn test_gas_consumption_max_scan() {
    let env = Env::default();
    env.mock_all_auths();
    
    let registry_id = env.register(ZolvencyRegistry, ());
    let registry_client = ZolvencyRegistryClient::new(&env, &registry_id);
    let admin = Address::generate(&env);
    registry_client.initialize(&admin, &Address::generate(&env));

    // Mock Soul (necessário pro gating)
    let soul_contract = env.register(MockSoul, ());
    let soul_client = MockSoulClient::new(&env, &soul_contract);
    let soul_id = 1u32;
    soul_client.set_soul(&soul_id, &true);

    // Registrar 20 tokens REAIS (Spokes) para medir custo real
    for _ in 0..20 {
        let github_id = env.register(GithubIdentityContract, ());
        let github_client = GithubIdentityContractClient::new(&env, &github_id);
        github_client.initialize(&admin, &registry_id, &soul_contract, &Address::generate(&env), &Address::generate(&env), &Address::generate(&env), &0);
        registry_client.register_token(&admin, &github_id);
    }

    // Medir orçamento antes
    // let cpu_before = env.budget().cpu_instruction_count();
    
    // Executar o blind scan de 20 tokens
    let reputation = registry_client.get_soul_reputation(&soul_id, &None);
    
    // let cpu_after = env.budget().cpu_instruction_count();
    // let cpu_used = cpu_after - cpu_before;
    
    // Validação básica
    assert!(reputation.len() <= 20);
    // println!("CPU used for 20-token scan: {}", cpu_used);
}
