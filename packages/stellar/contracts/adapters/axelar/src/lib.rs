#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Bytes, Env, String, Symbol, IntoVal, xdr::ToXdr, Val, Error};

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
}

#[contract]
pub struct AxelarAdapter;

#[contractimpl]
impl AxelarAdapter {
    pub fn initialize(
        env: Env,
        admin: Address,
        gateway: Address,
        gas_service: Address,
        gas_token: Address,
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Gateway, &gateway);
        env.storage().instance().set(&DataKey::GasService, &gas_service);
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
    ) -> Result<(), Error> {
        caller.require_auth();

        let gateway: Address = env.storage().instance().get(&DataKey::Gateway).unwrap();
        let gas_service: Address = env.storage().instance().get(&DataKey::GasService).unwrap();
        let gas_token_addr: Address = env.storage().instance().get(&DataKey::GasToken).unwrap();

        // 1. Codificação do Payload
        let payload = Self::encode_evm_payload(&env, &external_id, tier as u8, &user_evm_address);

        // 2. Pagamento de Gás (Axelar Gas Service)
        let gas_token = AxelarGasToken {
            address: gas_token_addr,
            amount: 15_000_000i128
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
                Bytes::new(&env)
            ).into_val(&env)
        );

        // 3. Chamada do Gateway
        let _: Val = env.invoke_contract(
            &gateway,
            &Symbol::new(&env, "call_contract"),
            (
                env.current_contract_address(),
                destination_chain,
                destination_address,
                payload
            ).into_val(&env)
        );

        Ok(())
    }

    fn encode_evm_payload(env: &Env, external_id: &String, tier: u8, user: &Bytes) -> Bytes {
        let mut payload = Bytes::new(env);
        let external_id_hash = env.crypto().keccak256(&external_id.clone().to_xdr(env));
        payload.append(&external_id_hash.into());
        
        let mut tier_bytes = [0u8; 32];
        tier_bytes[31] = tier;
        payload.append(&Bytes::from_array(env, &tier_bytes));
        
        let mut user_bytes = [0u8; 32];
        user.copy_into_slice(&mut user_bytes[12..32]);
        payload.append(&Bytes::from_array(env, &user_bytes));
        
        payload
    }
}
