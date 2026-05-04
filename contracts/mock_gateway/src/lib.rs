#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Address, String, Bytes, Symbol};

#[contract]
pub struct MockGateway;

#[contractimpl]
impl MockGateway {
    pub fn call_contract(env: Env, sender: Address, destination_chain: String, destination_address: String, payload: Bytes) {
        // Just emit an event to prove it was called
        env.events().publish(
            (Symbol::new(&env, "axelar_msg_sent"), sender),
            (destination_chain, destination_address, payload)
        );
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
        env.events().publish((Symbol::new(&env, "gas_paid"),), ());
    }
}
