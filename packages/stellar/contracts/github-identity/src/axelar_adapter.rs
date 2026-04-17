use soroban_sdk::{Address, Bytes, Env, String};
use crate::interop::MessengerTrait;
use crate::axelar::AxelarClient;
use crate::types::Error;
use crate::storage;

pub struct AxelarAdapter;

impl MessengerTrait for AxelarAdapter {
    fn send_reputation(
        env: Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
    ) -> Result<(), Error> {
        let axelar_config = storage::get_axelar_config(&env)?;
        
        // Forçar endereços oficiais de 2026 para o teste (como no lib.rs original)
        let gateway_addr = Address::from_string(&String::from_str(&env, "CB2JYOOZPHO43R57TC5PXV22QICKIDC5NKRF62BZG2J6JYFUIQPIAYY3"));
        let gas_service_addr = Address::from_string(&String::from_str(&env, "CCLZOCGHHC6F6JCZHEUP53LDQHRBPPCNRYXOVFZFS3O63OGRC47CKCGV"));

        let axelar_client = AxelarClient::new(&env, gateway_addr, gas_service_addr);

        let gas_amount = 15_000_000i128; // 15 XLM

        // 1. Pagamento de Gás (Versão Amplifier 2026)
        axelar_client.pay_gas(
            caller.clone(),
            destination_chain.clone(),
            destination_address.clone(),
            payload.clone(),
            caller.clone(), // Spender
            axelar_config.gas_token,
            gas_amount,
        );

        // 2. Chamada do Contrato
        axelar_client.call_contract(
            caller,
            destination_chain,
            destination_address,
            payload,
        );

        Ok(())
    }
}
