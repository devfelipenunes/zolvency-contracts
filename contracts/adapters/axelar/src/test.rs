#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Events as _;
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String, Symbol, vec, FromVal};

#[contract]
pub struct MockGateway;

#[contractimpl]
impl MockGateway {
    pub fn call_contract(env: Env, _caller: Address, _destination_chain: String, _destination_address: String, payload: Bytes) {
        env.events().publish((Symbol::new(&env, "gateway_call"),), (payload,));
    }
}

#[contract]
pub struct MockGasService;

#[contractimpl]
impl MockGasService {
    pub fn pay_gas(env: Env, _sender: Address, _destination_chain: String, _destination_address: String, _payload: Bytes, _execution_gas_limit: Address, _gas_token: AxelarGasToken, _params: Bytes) {
        env.events().publish((Symbol::new(&env, "gas_paid"),), ());
    }
}

#[test]
fn test_encode_evm_payload_format() {
    let env = Env::default();
    let external_id = String::from_str(&env, "gh_123");
    let tier = 2u8;
    let user = Bytes::from_array(&env, &[0xAA; 20]);
    let nonce = 1u64;
    let token_type = Symbol::new(&env, "github");

    let payload = AxelarAdapter::encode_reputation_payload(&env, &external_id, tier, &user, nonce, token_type);

    // Verificações de tamanho
    // 1 (type) + 32 (ext_id hash) + 32 (tier) + 32 (user) + 32 (nonce) + 32 (type hash) = 161 bytes
    assert_eq!(payload.len(), 161);

    // Verificar Tier padding (deve terminar em 02)
    let tier_chunk = payload.slice(33..65);
    let mut expected_tier = [0u8; 32];
    expected_tier[31] = 2;
    assert_eq!(tier_chunk, Bytes::from_array(&env, &expected_tier));

    // Verificar User padding (deve ter 12 zeros seguidos de 20 bytes AA)
    let user_chunk = payload.slice(65..97);
    let mut expected_user = [0u8; 32];
    expected_user[12..32].copy_from_slice(&[0xAA; 20]);
    assert_eq!(user_chunk, Bytes::from_array(&env, &expected_user));

    // Verificar Nonce padding (deve terminar em 01)
    let nonce_chunk = payload.slice(97..129);
    let mut expected_nonce = [0u8; 32];
    expected_nonce[31] = 1;
    assert_eq!(nonce_chunk, Bytes::from_array(&env, &expected_nonce));
}

#[test]
fn test_send_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let soul_contract = Address::generate(&env);
    let gateway_id = env.register(MockGateway, ());
    let gas_service_id = env.register(MockGasService, ());
    let gas_token = Address::generate(&env);

    let adapter_id = env.register(AxelarAdapter, ());
    let client = AxelarAdapterClient::new(&env, &adapter_id);

    client.initialize(&admin, &soul_contract, &gateway_id, &gas_service_id, &gas_token);

    let caller = Address::generate(&env);
    let dest_chain = String::from_str(&env, "ethereum");
    let dest_addr = String::from_str(&env, "0x123");
    let ext_id = String::from_str(&env, "id_1");
    let tier = 1u32;
    let user_evm = Bytes::from_array(&env, &[0xBB; 20]);
    let nonce = 42u64;
    let token_type = Symbol::new(&env, "bank");

    client.send_reputation(&caller, &dest_chain, &dest_addr, &ext_id, &tier, &user_evm, &nonce, &token_type, &Ecosystem::Evm);

    // Verificar eventos do Gateway e Gas
    let events = env.events().all();
    let has_gateway = events.iter().any(|e| e.1.get(0).map(|v| Symbol::from_val(&env, &v) == Symbol::new(&env, "gateway_call")).unwrap_or(false));
    let has_gas = events.iter().any(|e| e.1.get(0).map(|v| Symbol::from_val(&env, &v) == Symbol::new(&env, "gas_paid")).unwrap_or(false));

    assert!(has_gateway);
    assert!(has_gas);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_send_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    let adapter_id = env.register(AxelarAdapter, ());
    let client = AxelarAdapterClient::new(&env, &adapter_id);

    let caller = Address::generate(&env);
    client.send_reputation(&caller, &String::from_str(&env, "x"), &String::from_str(&env, "y"), &String::from_str(&env, "z"), &1, &Bytes::new(&env), &0, &Symbol::new(&env, "s"), &Ecosystem::Evm);
}
