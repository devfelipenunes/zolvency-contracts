#![cfg(test)]

use super::*;
use soroban_sdk::{
	testutils::Address as _, testutils::Events as _, testutils::Ledger as _, Address, Bytes,
	BytesN, Env, FromVal, String, Symbol,
};

use zolvency_soul::{ZolvencySoulContract, ZolvencySoulContractClient};

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
                _nonce: u64,
        ) -> Result<(), crate::types::Error> {
                env.events().publish(
                        (Symbol::new(&env, "adapter_send"),),
                        (
                                _destination_chain,
                                _destination_address,
                                _external_id,
                                _tier,
                                _user_evm_address,
                                _nonce,
                        ),
                );
                Ok(())
        }
}

struct TestEnv {
        env: Env,
        client: BinanceKycContractClient<'static>,
        admin: Address,
        recipient: Address,
}

fn setup(store_proof_data: bool) -> TestEnv {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1714316400);

	let admin = Address::generate(&env);
	let registry = Address::generate(&env);
	let fee_token = Address::generate(&env);
	let access_control = Address::generate(&env);
	let treasury = Address::generate(&env);


	let soul_admin = admin.clone();
	let soul_relayer = Address::generate(&env);
	let soul_contract_id = env.register(ZolvencySoulContract, ());
	let soul_client = ZolvencySoulContractClient::new(&env, &soul_contract_id);
	let _ = soul_client.initialize(&soul_admin, &soul_relayer);

	let recipient = Address::generate(&env);
	let passkey = BytesN::from_array(&env, &[0u8; 32]);
	let _ = soul_client.mint(&soul_relayer, &recipient, &passkey);
	let contract_id = env.register(BinanceKycContract, ());
	let client: BinanceKycContractClient<'static> = unsafe {
		core::mem::transmute(BinanceKycContractClient::new(&env, &contract_id))
	};

	client.initialize(
		&admin,
		&registry,
		&fee_token,
		&access_control,
		&treasury,
		&0,
		&0,
		&0,
		&store_proof_data,
	);

	client.set_soul_contract(&admin, &soul_contract_id);

	TestEnv {
		env,
		client,
		admin,
		recipient,
	}
}

fn mint_params(env: &Env, recipient: &Address) -> MintParams {
	MintParams {
		recipient: recipient.clone(),
		external_id: String::from_str(env, "binance_user"),
		kyc_level: KycLevel::Advanced,
		country: String::from_str(env, "BR"),
		verified_at: env.ledger().timestamp(),
		proof_hash: BytesN::from_array(env, &[2u8; 32]),
		proof_data: Bytes::new(env),
		window: RenewalWindow::Days60,
		nonce: 0,
	}
}

#[test]
fn test_mint_kyc() {
	let ctx = setup(false);
	let params = mint_params(&ctx.env, &ctx.recipient);

	let token_id = ctx.client.mint(&ctx.admin, &params, &None);
	let data = ctx.client.get_token_data(&token_id);

	assert_eq!(token_id, 1);
	assert_eq!(data.kyc_level, KycLevel::Advanced);
	assert_eq!(data.country, String::from_str(&ctx.env, "BR"));
	assert!(data.proof_data.is_empty());
}

#[test]
fn test_update_kyc() {
	let ctx = setup(true);
	let params = mint_params(&ctx.env, &ctx.recipient);
	let token_id = ctx.client.mint(&ctx.admin, &params, &None);

	let update_params = UpdateParams {
		kyc_level: KycLevel::Intermediate,
		country: String::from_str(&ctx.env, "US"),
		verified_at: ctx.env.ledger().timestamp(),
		proof_hash: BytesN::from_array(&ctx.env, &[3u8; 32]),
		proof_data: Bytes::from_array(&ctx.env, &[1u8; 4]),
		window: RenewalWindow::Days90,
	};

	ctx.client
	        .update_token(&ctx.admin, &token_id, &update_params, &1u64, &None);
	let data = ctx.client.get_token_data(&token_id);
	assert_eq!(data.kyc_level, KycLevel::Intermediate);
	assert_eq!(data.window, RenewalWindow::Days90);
	assert!(!data.proof_data.is_empty());
}

#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_update_after_expiry_fails() {
	let ctx = setup(false);
	let params = mint_params(&ctx.env, &ctx.recipient);
	let token_id = ctx.client.mint(&ctx.admin, &params, &None);

	let current = ctx.env.ledger().timestamp();
	ctx.env
		.ledger()
		.set_timestamp(current + RenewalWindow::Days60.to_seconds() + 1);

	let update_params = UpdateParams {
		kyc_level: KycLevel::Basic,
		country: String::from_str(&ctx.env, "BR"),
		verified_at: ctx.env.ledger().timestamp(),
		proof_hash: BytesN::from_array(&ctx.env, &[4u8; 32]),
		proof_data: Bytes::new(&ctx.env),
		window: RenewalWindow::Days30,
	};

	ctx.client
	        .update_token(&ctx.admin, &token_id, &update_params, &1u64, &None);}

#[test]
fn test_mint_fee_by_window() {
	let ctx = setup(false);
	ctx.client.set_fees(&ctx.admin, &10, &20, &30);
	assert_eq!(ctx.client.get_mint_fee(&RenewalWindow::Days30), 10);
	assert_eq!(ctx.client.get_mint_fee(&RenewalWindow::Days60), 20);
	assert_eq!(ctx.client.get_mint_fee(&RenewalWindow::Days90), 30);
}

#[test]
fn test_cross_chain_send_event() {
	let ctx = setup(false);
	let adapter_id = ctx.env.register(MockAdapter, ());
	ctx.client
		.set_active_protocol(&ctx.admin, &InteropProtocol::LayerZero, &adapter_id);

	let params = mint_params(&ctx.env, &ctx.recipient);
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
