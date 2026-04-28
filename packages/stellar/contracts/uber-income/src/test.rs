#![cfg(test)]

use super::*;
use soroban_sdk::{
	testutils::Address as _, testutils::Events as _, testutils::Ledger as _, Address, Bytes,
	BytesN, Env, FromVal, String, Symbol,
};

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
        client: UberIncomeContractClient<'static>,
        admin: Address,
        recipient: Address,
}

fn setup(store_proof_data: bool) -> TestEnv {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1714316400);

        let admin = Address::generate(&env);	let registry = Address::generate(&env);
	let fee_token = Address::generate(&env);
	let access_control = Address::generate(&env);
	let treasury = Address::generate(&env);

	let recipient = Address::generate(&env);
	let contract_id = env.register(UberIncomeContract, ());
	let client: UberIncomeContractClient<'static> = unsafe {
		core::mem::transmute(UberIncomeContractClient::new(&env, &contract_id))
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
		&86400,
		&store_proof_data,
	);

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
	let params = mint_params(&ctx.env, &ctx.recipient);

	let token_id = ctx.client.mint(&ctx.admin, &params, &None);
	let data = ctx.client.get_token_data(&token_id);

	assert_eq!(token_id, 1);
	assert_eq!(data.income_band, 2);
	assert_eq!(data.reveal_mode, RevealMode::Band);
	assert!(data.proof_data.is_empty());
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_mint_exact_without_value_fails() {
	let ctx = setup(false);
	let mut params = mint_params(&ctx.env, &ctx.recipient);
	params.reveal_mode = RevealMode::Exact;
	params.income_value = None;

	ctx.client.mint(&ctx.admin, &params, &None);
}

#[test]
fn test_update_while_valid() {
	let ctx = setup(true);
	let params = mint_params(&ctx.env, &ctx.recipient);
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
	assert!(!data.proof_data.is_empty());
}

#[test]
#[should_panic(expected = "Error(Contract, #15)")]
fn test_update_after_expiry_fails() {
	let ctx = setup(false);
	let params = mint_params(&ctx.env, &ctx.recipient);
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
	        .update_token(&ctx.admin, &token_id, &update_params, &1u64, &None);}

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

#[test]
#[should_panic(expected = "Error(Contract, #16)")]
fn test_proof_freshness_enforced() {
	let ctx = setup(false);
	let mut params = mint_params(&ctx.env, &ctx.recipient);
	params.verified_at = ctx.env.ledger().timestamp() - 100_000;

	ctx.client.mint(&ctx.admin, &params, &None);
}
