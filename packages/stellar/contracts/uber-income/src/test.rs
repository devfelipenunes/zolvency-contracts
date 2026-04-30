use super::*;
use soroban_sdk::{
	testutils::{Address as _, Ledger as _, Events as _},
	Address, Bytes, BytesN, Env, FromVal, IntoVal, String, Symbol,
};
use crate::types::{
	CrossChainParams, IncomePeriod, MintParams, RenewalWindow, RevealMode, UpdateParams,
};
use zolvency_registry::{ZolvencyRegistry, ZolvencyRegistryClient};

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

#[contract]
pub struct MockAdapter;

#[contractimpl]
impl MockAdapter {
	pub fn send(
		env: Env,
		_caller: Address,
		_dest_chain: String,
		_dest_addr: String,
		_ext_id: String,
		_tier: u32,
		_user_dest: Bytes,
		_nonce: u64,
		_token_type: Symbol,
	) {
		env.events().publish((Symbol::new(&env, "adapter_send"),), ());
	}
}

pub struct TestEnv {
	env: Env,
	client: UberIncomeContractClient<'static>,
	admin: Address,
	registry: Address,
	recipient: Address,
}

fn setup(_store_proof_data: bool) -> TestEnv {
	let env = Env::default();
	env.mock_all_auths();
	env.ledger().set_timestamp(1714316400);

	let admin = Address::generate(&env);
	let registry_id = env.register(ZolvencyRegistry, ());
	let registry_client = ZolvencyRegistryClient::new(&env, &registry_id);
	let signer = Address::generate(&env);
	registry_client.initialize(&admin, &signer);

	let fee_token = Address::generate(&env);
	let access_control = Address::generate(&env);
	let treasury = Address::generate(&env);

	let recipient = Address::generate(&env);

	let soul_contract = env.register(MockSoul, ());
	let soul_client = MockSoulClient::new(&env, &soul_contract);
	soul_client.set_soul(&1u32, &true);

	let contract_id = env.register(UberIncomeContract, ());
	let client = UberIncomeContractClient::new(&env, &contract_id);

	let init_params = crate::types::InitializeParams {
		admin: admin.clone(),
		registry: registry_id.clone(),
		soul_contract,
		fee_token,
		access_control,
		treasury,
		mint_fee_30: 0,
		mint_fee_60: 0,
		mint_fee_90: 0,
		max_proof_age_seconds: 86400,
	};

	client.initialize(&init_params);
	registry_client.register_token(&admin, &contract_id);

	TestEnv {
		env: env.clone(),
		client,
		admin,
		registry: registry_id.clone(),
		recipient,
	}
}

fn mint_params(env: &Env, soul_id: u32) -> MintParams {
	MintParams {
		soul_id,
		external_id: String::from_str(env, "uber_user"),
		income_band: 2,
		income_value: None,
		reveal_mode: RevealMode::Band,
		currency: String::from_str(env, "USD"),
		period: IncomePeriod::Monthly,
		verified_at: env.ledger().timestamp(),
		proof_hash: BytesN::from_array(env, &[4u8; 32]),
		proof_data: Bytes::new(env),
		window: RenewalWindow::Days90,
		nonce: 0,
	}
}

#[test]
fn test_mint_band_income() {
	let ctx = setup(false);
	let params = mint_params(&ctx.env, 1u32);

	let token_id = ctx.client.mint(&ctx.admin, &params, &None);
	let data = ctx.client.get_token_data(&token_id);

	assert_eq!(token_id, 1);
	assert_eq!(data.income_band, 2);
	assert_eq!(data.reveal_mode, RevealMode::Band);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_mint_exact_without_value_fails() {
	let ctx = setup(false);
	let mut params = mint_params(&ctx.env, 1u32);
	params.reveal_mode = RevealMode::Exact;
	params.income_value = None;

	ctx.client.mint(&ctx.admin, &params, &None);
}

#[test]
fn test_update_while_valid() {
	let ctx = setup(false);
	let params = mint_params(&ctx.env, 1u32);
	let token_id = ctx.client.mint(&ctx.admin, &params, &None);

	let update_params = UpdateParams {
		income_band: 4,
		income_value: Some(9_000),
		reveal_mode: RevealMode::Exact,
		currency: String::from_str(&ctx.env, "USD"),
		period: IncomePeriod::Monthly,
		verified_at: ctx.env.ledger().timestamp(),
		proof_hash: BytesN::from_array(&ctx.env, &[5u8; 32]),
		proof_data: Bytes::from_array(&ctx.env, &[2u8; 2]),
		window: RenewalWindow::Days30,
	};

	ctx.client
	        .update_token(&ctx.admin, &token_id, &update_params, &1u64, &None);
	let data = ctx.client.get_token_data(&token_id);
	assert_eq!(data.income_band, 4);
	assert_eq!(data.reveal_mode, RevealMode::Exact);
}

#[test]
#[should_panic(expected = "Error(Contract, #15)")]
fn test_update_after_expiry_fails() {
	let ctx = setup(false);
	let params = mint_params(&ctx.env, 1u32);
	let token_id = ctx.client.mint(&ctx.admin, &params, &None);

	let current = ctx.env.ledger().timestamp();
	ctx.env
		.ledger()
		.set_timestamp(current + RenewalWindow::Days90.to_seconds() + 1);

	let update_params = UpdateParams {
		income_band: 3,
		income_value: Some(7_000),
		reveal_mode: RevealMode::Exact,
		currency: String::from_str(&ctx.env, "USD"),
		period: IncomePeriod::Monthly,
		verified_at: ctx.env.ledger().timestamp(),
		proof_hash: BytesN::from_array(&ctx.env, &[6u8; 32]),
		proof_data: Bytes::new(&ctx.env),
		window: RenewalWindow::Days30,
	};

	ctx.client
	        .update_token(&ctx.admin, &token_id, &update_params, &1u64, &None);
}

#[test]
fn test_cross_chain_send_event() {
	let ctx = setup(false);
	let adapter_id = ctx.env.register(MockAdapter, ());
	
	let registry_client = ZolvencyRegistryClient::new(&ctx.env, &ctx.registry);
	let interop_config = zolvency_registry::InteropConfig {
		active_protocol: zolvency_registry::InteropProtocol::Axelar,
		adapter_address: adapter_id,
	};
	registry_client.set_interop_config(&ctx.admin, &interop_config);

	let params = mint_params(&ctx.env, 1u32);
	let cc_params = CrossChainParams {
		destination_chain: String::from_str(&ctx.env, "ethereum"),
		destination_address: String::from_str(&ctx.env, "0xabc"),
		user_destination_address: Bytes::from_array(&ctx.env, &[0u8; 20]),
	};

	ctx.client
		.mint(&ctx.admin, &params, &Some(cc_params));

	let events = ctx.env.events().all();
	let has_adapter_event = events.iter().any(|e| {
		e.1.get(0)
			.map(|v| Symbol::from_val(&ctx.env, &v) == Symbol::new(&ctx.env, "adapter_send"))
			.unwrap_or(false)
	});

	assert!(has_adapter_event, "Missing adapter_send event");
}

#[test]
fn test_massive_proof_payload_limit() {
    let ctx = setup(false);
    
    // Simular uma prova ZK razoável para produção (8KB)
	// Como removi o armazenamento do proof_data, o budget agora é apenas para o input
    let mut massive_proof = Bytes::new(&ctx.env);
    for _ in 0..8000 {
        massive_proof.append(&Bytes::from_array(&ctx.env, &[1u8]));
    }

    let mut params = mint_params(&ctx.env, 1u32);
    params.proof_data = massive_proof;

    let res = ctx.client.try_mint(&ctx.admin, &params, &None);
    assert!(res.is_ok());
}

#[test]
#[should_panic(expected = "Error(Contract, #16)")]
fn test_proof_freshness_enforced() {
	let ctx = setup(false);
	let mut params = mint_params(&ctx.env, 1u32);
	params.verified_at = ctx.env.ledger().timestamp() - 100_000;

	ctx.client.mint(&ctx.admin, &params, &None);
}

#[test]
fn test_mint_fee_by_window() {
	let env = Env::default();
	env.mock_all_auths();
	env.ledger().set_timestamp(1714316400);
	
	let admin = Address::generate(&env);
	let registry_id = env.register(ZolvencyRegistry, ());
	let registry_client = ZolvencyRegistryClient::new(&env, &registry_id);
	let signer = Address::generate(&env);
	registry_client.initialize(&admin, &signer);

	let fee_token = env.register_stellar_asset_contract(admin.clone());
	let token_admin = soroban_sdk::token::StellarAssetClient::new(&env, &fee_token);
	token_admin.mint(&admin, &1000i128);
	let treasury = Address::generate(&env);

	let soul_contract = env.register(MockSoul, ());
	let soul_client = MockSoulClient::new(&env, &soul_contract);
	soul_client.set_soul(&1u32, &true);

	let contract_id = env.register(UberIncomeContract, ());
	let client = UberIncomeContractClient::new(&env, &contract_id);

	let init_params = crate::types::InitializeParams {
		admin: admin.clone(),
		registry: registry_id.clone(),
		soul_contract,
		fee_token: fee_token.clone(),
		access_control: Address::generate(&env),
		treasury: treasury.clone(),
		mint_fee_30: 100,
		mint_fee_60: 200,
		mint_fee_90: 300,
		max_proof_age_seconds: 86400,
	};

	client.initialize(&init_params);

	let params = MintParams {
		soul_id: 1,
		external_id: String::from_str(&env, "fee_user"),
		income_band: 2,
		income_value: None,
		reveal_mode: RevealMode::Band,
		currency: String::from_str(&env, "USD"),
		period: IncomePeriod::Monthly,
		verified_at: env.ledger().timestamp(),
		proof_hash: BytesN::from_array(&env, &[7u8; 32]),
		proof_data: Bytes::new(&env),
		window: RenewalWindow::Days90,
		nonce: 0,
	};

	client.mint(&admin, &params, &None);
}

#[test]
fn test_initialize_already_initialized() {
    let ctx = setup(false);
	let init_params = crate::types::InitializeParams {
		admin: ctx.admin.clone(),
		registry: ctx.registry.clone(),
		soul_contract: Address::generate(&ctx.env),
		fee_token: Address::generate(&ctx.env),
		access_control: Address::generate(&ctx.env),
		treasury: Address::generate(&ctx.env),
		mint_fee_30: 0,
		mint_fee_60: 0,
		mint_fee_90: 0,
		max_proof_age_seconds: 0,
	};
    let res = ctx.client.try_initialize(&init_params);
    assert_eq!(res, Err(Ok(crate::types::Error::AlreadyInitialized)));
}

#[test]
fn test_mint_invalid_nonce() {
    let ctx = setup(false);
    let soul_id = 1u32;
    let mut params = mint_params(&ctx.env, soul_id);
    params.nonce = 1; // Nonce errado

    let res = ctx.client.try_mint(&ctx.admin, &params, &None);
    assert_eq!(res, Err(Ok(crate::types::Error::InvalidNonce)));
}
