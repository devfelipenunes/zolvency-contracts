#![no_std]
use soroban_sdk::{contract, contractevent, contractimpl, Env, Address, String, Bytes};

#[contract]
pub struct MockGateway;

#[contractevent]
pub enum MockGatewayEvent {
    AxelarMsgSent {
        sender: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
    },
    GasPaid,
}

#[contractimpl]
impl MockGateway {
    pub fn call_contract(env: Env, sender: Address, destination_chain: String, destination_address: String, payload: Bytes) {
        // Just emit an event to prove it was called
        MockGatewayEvent::AxelarMsgSent {
            sender,
            destination_chain,
            destination_address,
            payload,
        }
        .publish(&env);
    }

    pub fn pay_gas(
        env: Env,
        _sender: Address,
        _destination_chain: String,
        _destination_address: String,
        _payload: Bytes,
        _refund_address: Address,
        _gas_token: soroban_sdk::Val,
        _params: Bytes,
    ) {
        // Mock gas payment success
        MockGatewayEvent::GasPaid.publish(&env);
    }
}
