#![cfg(test)]

use super::*;
use crate::{Ecosystem, AxelarGasToken};
use soroban_sdk::testutils::Events as _;
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String, Symbol, FromVal};

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
    pub fn pay_gas(
        env: Env,
        _sender: Address,
        _destination_chain: String,
        _destination_address: String,
        _payload: Bytes,
        _refund_address: Address,
        _gas_token: AxelarGasToken, 
        _params: Bytes,
    ) {
        env.events().publish((Symbol::new(&env, "gas_paid"),), ());
    }
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

    let soul_id = 1u32;
    client.send_reputation(&caller, &dest_chain, &dest_addr, &soul_id, &ext_id, &tier, &user_evm, &nonce, &token_type, &Ecosystem::Evm);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_send_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    let adapter_id = env.register(AxelarAdapter, ());
    let client = AxelarAdapterClient::new(&env, &adapter_id);

    let caller = Address::generate(&env);
    let dummy_user = Bytes::from_array(&env, &[0u8; 20]);
    client.send_reputation(&caller, &String::from_str(&env, "x"), &String::from_str(&env, "y"), &0, &String::from_str(&env, "z"), &1, &dummy_user, &0, &Symbol::new(&env, "s"), &Ecosystem::Evm);
}
