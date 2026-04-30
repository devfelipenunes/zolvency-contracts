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

    pub fn send(
        env: Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        external_id: String,
        tier: u32,
        user_evm_address: Bytes,
        nonce: u64,
        token_type: Symbol,
    ) -> Result<(), Error> {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).ok_or(Error::NotInitialized)?;
        admin.require_auth();

        // ── Gating: Authority Check (Simplified) ──
        let gateway: Address = env.storage().instance().get(&DataKey::Gateway).ok_or(Error::NotInitialized)?;
        let gas_service: Address = env.storage().instance().get(&DataKey::GasService).ok_or(Error::NotInitialized)?;
        let gas_token_addr: Address = env.storage().instance().get(&DataKey::GasToken).ok_or(Error::NotInitialized)?;

        // 1. Codificação do Payload (Adicionado Nonce e Token Type)
        let payload = Self::encode_evm_payload(&env, &external_id, tier as u8, &user_evm_address, nonce, token_type);

        // 2. Pagamento de Gás (Axelar Gas Service)
        let gas_token = AxelarGasToken {
            address: gas_token_addr,
            amount: 15_000_000i128,
        };

        let _: Val = env.invoke_contract(
            &gas_service,
            &Symbol::new(&env, "pay_gas"),
            (
                env.current_contract_address(),
                destination_chain.clone(),
                destination_address.clone(),
                payload.clone(),
                caller,
                gas_token,
                Bytes::new(&env),
            )
                .into_val(&env),
        );

        // 3. Chamada do Gateway
        let _: Val = env.invoke_contract(
            &gateway,
            &Symbol::new(&env, "call_contract"),
            (
                env.current_contract_address(),
                destination_chain,
                destination_address,
                payload,
            )
                .into_val(&env),
        );

        Ok(())
    }

    fn encode_evm_payload(env: &Env, external_id: &String, tier: u8, user: &Bytes, nonce: u64, token_type: Symbol) -> Bytes {
        let mut payload = Bytes::new(env);
        
        // 1. External ID (32 bytes)
        let external_id_hash = env.crypto().keccak256(&external_id.clone().to_xdr(env));
        payload.append(&external_id_hash.into());

        // 2. Tier (32 bytes - ABI standard padding)
        let mut tier_bytes = [0u8; 32];
        tier_bytes[31] = tier;
        payload.append(&Bytes::from_array(env, &tier_bytes));

        // 3. User Address (32 bytes - ABI standard padding for address)
        let mut user_bytes = [0u8; 32];
        user.copy_into_slice(&mut user_bytes[12..32]);
        payload.append(&Bytes::from_array(env, &user_bytes));

        // 4. Nonce (32 bytes - ABI standard padding for uint64)
        let mut nonce_bytes = [0u8; 32];
        let n_be = nonce.to_be_bytes();
        nonce_bytes[24..32].copy_from_slice(&n_be);
        payload.append(&Bytes::from_array(env, &nonce_bytes));

        // 5. Token Type (32 bytes - Hash of Symbol)
        let type_hash = env.crypto().keccak256(&token_type.to_xdr(env));
        payload.append(&type_hash.into());

        payload
    }
}

#[cfg(test)]
mod test;
