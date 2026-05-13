use soroban_sdk::{xdr::ToXdr, Address, Bytes, Env, IntoVal, Symbol, Val};
use crate::storage;
use crate::{Ecosystem, Error, AxelarGasToken};

pub fn send_reputation(
    env: &Env,
    caller: Address,
    destination_chain: soroban_sdk::String,
    destination_address: soroban_sdk::String,
    soul_id: u32,
    external_id: soroban_sdk::String,
    tier: u32,
    user_address: Bytes,
    nonce: u64,
    token_type: Symbol,
    ecosystem: Ecosystem,
) -> Result<(), crate::Error> {
    caller.require_auth();

    let mut payload = Bytes::new(env);
    payload.append(&Bytes::from_array(env, &[1u8])); // Type 1

    match ecosystem {
        Ecosystem::Evm => {
            // ABI Encoding (32-byte slots)
            payload.append(&pad_u32(env, soul_id));
            payload.append(&pad_address(env, &user_address));
            payload.append(&env.crypto().keccak256(&external_id.to_xdr(env)).into());
            payload.append(&pad_u32(env, tier));
            payload.append(&pad_u64(env, nonce));
            payload.append(&env.crypto().keccak256(&token_type.to_xdr(env)).into());
        }
        _ => {
            // Compact Encoding (Borsh-friendly)
            payload.append(&Bytes::from_array(env, &soul_id.to_le_bytes()));
            payload.append(&pad_address(env, &user_address));
            payload.append(&env.crypto().keccak256(&external_id.to_xdr(env)).into());
            payload.append(&Bytes::from_array(env, &tier.to_le_bytes()));
            payload.append(&Bytes::from_array(env, &nonce.to_le_bytes()));
            payload.append(&env.crypto().keccak256(&token_type.to_xdr(env)).into());
        }
    }

    call_axelar(env, caller, destination_chain, destination_address, payload)
}

pub fn send_will_auth(
    env: &Env,
    caller: Address,
    destination_chain: soroban_sdk::String,
    destination_address: soroban_sdk::String,
    will_address: Bytes,
    soul_id: u32,
    permissions: u64,
    expiry: u64,
    ecosystem: Ecosystem,
) -> Result<(), crate::Error> {
    caller.require_auth();

    let mut payload = Bytes::new(env);
    payload.append(&Bytes::from_array(env, &[2u8])); // Type 2

    match ecosystem {
        Ecosystem::Evm => {
            // ABI Encoding (32-byte slots)
            payload.append(&pad_address(env, &will_address));
            payload.append(&pad_u32(env, soul_id));
            payload.append(&pad_u64(env, permissions));
            payload.append(&pad_u64(env, expiry));
        }
        _ => {
            // Compact Encoding
            payload.append(&Bytes::from_array(env, &soul_id.to_le_bytes()));
            payload.append(&pad_address(env, &will_address));
            payload.append(&Bytes::from_array(env, &permissions.to_le_bytes()));
            payload.append(&Bytes::from_array(env, &expiry.to_le_bytes()));
        }
    }

    call_axelar(env, caller, destination_chain, destination_address, payload)
}

fn pad_u32(env: &Env, val: u32) -> Bytes {
    let mut b = [0u8; 32];
    let v = val.to_be_bytes(); // ABI uses Big Endian
    b[28..32].copy_from_slice(&v);
    Bytes::from_array(env, &b)
}

fn pad_u64(env: &Env, val: u64) -> Bytes {
    let mut b = [0u8; 32];
    let v = val.to_be_bytes();
    b[24..32].copy_from_slice(&v);
    Bytes::from_array(env, &b)
}

fn pad_address(env: &Env, addr: &Bytes) -> Bytes {
    let mut b = [0u8; 32];
    let len = addr.len() as usize;
    addr.copy_into_slice(&mut b[32 - len..32]);
    Bytes::from_array(env, &b)
}


fn call_axelar(
    env: &Env,
    caller: Address,
    destination_chain: soroban_sdk::String,
    destination_address: soroban_sdk::String,
    payload: Bytes,
) -> Result<(), crate::Error> {
    let gateway = storage::get_gateway(env).ok_or(crate::Error::NotInitialized)?;
    let gas_service = storage::get_gas_service(env).ok_or(crate::Error::NotInitialized)?;
    let gas_token_addr = storage::get_gas_token(env).ok_or(crate::Error::NotInitialized)?;

    let amount = 15_000_000; // 15 XLM fixo para testnet
    
    // Pagamento de Gas
    let gas_token = AxelarGasToken {
        address: gas_token_addr,
        amount,
    };

    let _: Val = env.invoke_contract(
        &gas_service,
        &Symbol::new(env, "pay_gas"),
        (
            caller.clone(),
            destination_chain.clone(),
            destination_address.clone(),
            payload.clone(),
            env.current_contract_address(),
            gas_token,
            Bytes::new(env),
        )
            .into_val(env),
    );

    // Chamada do Contrato
    let _: Val = env.invoke_contract(
        &gateway,
        &Symbol::new(env, "call_contract"),
        (
            env.current_contract_address(),
            destination_chain,
            destination_address,
            payload,
        )
            .into_val(env),
    );

    Ok(())
}
