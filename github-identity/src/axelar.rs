use soroban_sdk::{Address, Bytes, BytesN, Env, String, Symbol, IntoVal};

pub struct AxelarClient<'a> {
    pub env: &'a Env,
    pub gateway: Address,
    pub gas_service: Address,
}

impl<'a> AxelarClient<'a> {
    pub fn new(env: &'a Env, gateway: Address, gas_service: Address) -> Self {
        Self {
            env,
            gateway,
            gas_service,
        }
    }

    pub fn call_contract(&self, destination_chain: String, destination_address: String, payload: Bytes) {
        self.env.invoke_contract::<()>(
            &self.gateway,
            &Symbol::new(self.env, "call_contract"),
            (destination_chain, destination_address, payload).into_val(self.env),
        );
    }

    pub fn pay_gas(
        &self,
        sender: Address,
        destination_chain: String,
        destination_address: String,
        payload_hash: BytesN<32>,
        gas_token: Address,
        amount: i128,
        params: Bytes,
    ) {
        self.env.invoke_contract::<()>(
            &self.gas_service,
            &Symbol::new(self.env, "pay_gas"),
            (
                sender,
                destination_chain,
                destination_address,
                payload_hash,
                gas_token,
                amount,
                params,
            ).into_val(self.env),
        );
    }
}
