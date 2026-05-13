#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, Bytes, Env, Symbol, String
};

// --- TYPES ---

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Ecosystem {
    Evm,
    Cosmos,
    Sui,
    Solana,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct AxelarGasToken {
    pub address: Address,
    pub amount: i128,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
}

// --- MODULES ---

mod logic;
mod storage;

#[cfg(test)]
mod test;

#[contract]
pub struct AxelarAdapter;

#[contractimpl]
impl AxelarAdapter {
    pub fn initialize(
        env: Env,
        admin: Address,
        soul_contract: Address,
        gateway: Address,
        gas_service: Address,
        gas_token: Address,
    ) {
        if storage::get_admin(&env).is_some() {
            return;
        }
        storage::set_admin(&env, &admin);
        storage::set_soul_contract(&env, &soul_contract);
        storage::set_gateway(&env, &gateway);
        storage::set_gas_service(&env, &gas_service);
        storage::set_gas_token(&env, &gas_token);
    }

    pub fn send_reputation(
        env: Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        soul_id: u32,
        external_id: String,
        tier: u32,
        user_evm_address: Bytes,
        nonce: u64,
        token_type: Symbol,
        ecosystem: Ecosystem,
    ) -> Result<(), Error> {
        logic::send_reputation(
            &env,
            caller,
            destination_chain,
            destination_address,
            soul_id,
            external_id,
            tier,
            user_evm_address,
            nonce,
            token_type,
            ecosystem
        )
    }

    pub fn send_will_auth(
        env: Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        will_evm_address: Bytes,
        soul_id: u32,
        permissions: u64,
        expiry: u64,
        ecosystem: Ecosystem,
    ) -> Result<(), Error> {
        logic::send_will_auth(
            &env,
            caller,
            destination_chain,
            destination_address,
            will_evm_address,
            soul_id,
            permissions,
            expiry,
            ecosystem
        )
    }

    pub fn upgrade(env: Env, admin: Address, new_wasm_hash: soroban_sdk::BytesN<32>) -> Result<(), Error> {
        admin.require_auth();
        let stored_admin = storage::get_admin(&env).ok_or(Error::NotInitialized)?;
        if admin != stored_admin {
            return Err(Error::NotInitialized); // Usando NotInitialized como genérico por simplicidade aqui
        }
        env.deployer().update_current_contract_wasm(new_wasm_hash);
        Ok(())
    }
}
