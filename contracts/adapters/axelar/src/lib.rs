#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, xdr::ToXdr, Address, Bytes, Env, IntoVal,
    String, Symbol, Val,
};

#[contracttype]
#[derive(Clone, Debug)]
pub struct AxelarGasToken {
    pub address: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Ecosystem {
    Evm,
    Cosmos,
    Solana,
}

#[contracttype]
pub enum DataKey {
    Gateway,
    GasService,
    GasToken,
    Admin,
    SoulContract,
}

#[contract]
pub struct AxelarAdapter;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
}

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
        if env.storage().instance().has(&DataKey::Admin) {
            return;
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::SoulContract, &soul_contract);
        env.storage().instance().set(&DataKey::Gateway, &gateway);
        env.storage()
            .instance()
            .set(&DataKey::GasService, &gas_service);
        env.storage().instance().set(&DataKey::GasToken, &gas_token);
    }

    pub fn estimate_fee(_env: Env, _destination_chain: String) -> i128 {
        15_000_000 // 15 XLM fixo para testnet
    }

    pub fn send_reputation(
        env: Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        external_id: String,
        tier: u32,
        user_evm_address: Bytes,
        nonce: u64,
        token_type: Symbol,
        ecosystem: Ecosystem,
    ) -> Result<(), Error> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).ok_or(Error::NotInitialized)?;
        admin.require_auth();

        let payload = match ecosystem {
            Ecosystem::Evm => Self::encode_reputation_payload(&env, &external_id, tier as u8, &user_evm_address, nonce, token_type),
            _ => Self::encode_reputation_payload_borsh(&env, &external_id, tier, &user_evm_address, nonce, token_type),
        };
        Self::call_axelar(&env, caller, destination_chain, destination_address, payload)
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
        let admin: Address = env.storage().instance().get(&DataKey::Admin).ok_or(Error::NotInitialized)?;
        admin.require_auth();

        let payload = match ecosystem {
            Ecosystem::Evm => Self::encode_will_auth_payload(&env, &will_evm_address, soul_id, permissions, expiry),
            _ => Self::encode_will_auth_payload_borsh(&env, &will_evm_address, soul_id, permissions, expiry),
        };
        Self::call_axelar(&env, caller, destination_chain, destination_address, payload)
    }

    fn call_axelar(
        env: &Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
    ) -> Result<(), Error> {
        let gateway: Address = env.storage().instance().get(&DataKey::Gateway).ok_or(Error::NotInitialized)?;
        let gas_service: Address = env.storage().instance().get(&DataKey::GasService).ok_or(Error::NotInitialized)?;
        let gas_token_addr: Address = env.storage().instance().get(&DataKey::GasToken).ok_or(Error::NotInitialized)?;

        // Pagamento de Gás
        let gas_token = AxelarGasToken {
            address: gas_token_addr,
            amount: 15_000_000i128,
        };

        let _: Val = env.invoke_contract(
            &gas_service,
            &Symbol::new(env, "pay_gas"),
            (
                env.current_contract_address(),
                destination_chain.clone(),
                destination_address.clone(),
                payload.clone(),
                caller,
                gas_token,
                Bytes::new(env),
            )
                .into_val(env),
        );

        // Chamada do Gateway
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

    fn encode_reputation_payload(env: &Env, external_id: &String, tier: u8, user: &Bytes, nonce: u64, token_type: Symbol) -> Bytes {
        let mut data = Bytes::new(env);
        data.append(&env.crypto().keccak256(&external_id.clone().to_xdr(env)).into());
        
        let mut tier_bytes = [0u8; 32];
        tier_bytes[31] = tier;
        data.append(&Bytes::from_array(env, &tier_bytes));

        let mut user_bytes = [0u8; 32];
        user.copy_into_slice(&mut user_bytes[12..32]);
        data.append(&Bytes::from_array(env, &user_bytes));

        let mut nonce_bytes = [0u8; 32];
        let n_be = nonce.to_be_bytes();
        nonce_bytes[24..32].copy_from_slice(&n_be);
        data.append(&Bytes::from_array(env, &nonce_bytes));

        data.append(&env.crypto().keccak256(&token_type.to_xdr(env)).into());

        let mut payload = Bytes::new(env);
        payload.append(&Bytes::from_array(env, &[1u8])); // Type 1: Reputation
        payload.append(&data);
        payload
    }

    fn encode_will_auth_payload(env: &Env, will: &Bytes, soul_id: u32, permissions: u64, expiry: u64) -> Bytes {
        let mut data = Bytes::new(env);
        
        // Will Address (32 bytes padded)
        let mut will_bytes = [0u8; 32];
        will.copy_into_slice(&mut will_bytes[12..32]);
        data.append(&Bytes::from_array(env, &will_bytes));

        // Soul ID (32 bytes padded)
        let mut sid_bytes = [0u8; 32];
        let sid_be = soul_id.to_be_bytes();
        sid_bytes[28..32].copy_from_slice(&sid_be);
        data.append(&Bytes::from_array(env, &sid_bytes));

        // Permissions (32 bytes padded)
        let mut perm_bytes = [0u8; 32];
        let p_be = permissions.to_be_bytes();
        perm_bytes[24..32].copy_from_slice(&p_be);
        data.append(&Bytes::from_array(env, &perm_bytes));

        // Expiry (32 bytes padded)
        let mut exp_bytes = [0u8; 32];
        let e_be = expiry.to_be_bytes();
        exp_bytes[24..32].copy_from_slice(&e_be);
        data.append(&Bytes::from_array(env, &exp_bytes));

        let mut payload = Bytes::new(env);
        payload.append(&Bytes::from_array(env, &[2u8])); // Type 2: Will Auth
        payload.append(&data);
        payload
    }

    fn encode_reputation_payload_borsh(env: &Env, external_id: &String, tier: u32, user: &Bytes, nonce: u64, token_type: Symbol) -> Bytes {
        let mut payload = Bytes::new(env);
        payload.append(&Bytes::from_array(env, &[1u8])); // Type 1: Reputation
        
        // Borsh: external_id (as hash), tier (u32), user (32 bytes), nonce (u64), token_type (as hash)
        payload.append(&env.crypto().keccak256(&external_id.clone().to_xdr(env)).into());
        
        let t_le = tier.to_le_bytes();
        payload.append(&Bytes::from_array(env, &t_le));

        let mut user_bytes = [0u8; 32];
        user.copy_into_slice(&mut user_bytes[12..32]); 
        payload.append(&Bytes::from_array(env, &user_bytes));

        let n_le = nonce.to_le_bytes();
        payload.append(&Bytes::from_array(env, &n_le));

        payload.append(&env.crypto().keccak256(&token_type.to_xdr(env)).into());
        
        payload
    }

    fn encode_will_auth_payload_borsh(env: &Env, will: &Bytes, soul_id: u32, permissions: u64, expiry: u64) -> Bytes {
        let mut payload = Bytes::new(env);
        payload.append(&Bytes::from_array(env, &[2u8])); // Type 2: Will Auth
        
        let mut will_bytes = [0u8; 32];
        will.copy_into_slice(&mut will_bytes[12..32]);
        payload.append(&Bytes::from_array(env, &will_bytes));

        let sid_le = soul_id.to_le_bytes();
        payload.append(&Bytes::from_array(env, &sid_le));

        let p_le = permissions.to_le_bytes();
        payload.append(&Bytes::from_array(env, &p_le));

        let e_le = expiry.to_le_bytes();
        payload.append(&Bytes::from_array(env, &e_le));

        payload
    }
}
