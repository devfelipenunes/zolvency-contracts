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

    pub fn call_contract(&self, caller: Address, destination_chain: String, destination_address: String, payload: Bytes) {
        self.env.invoke_contract::<()>(
            &self.gateway,
            &Symbol::new(self.env, "call_contract"),
            (caller, destination_chain, destination_address, payload).into_val(self.env),
        );
    }

    pub fn pay_gas(
        &self,
        sender: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
        spender: Address,
        token: Address,
        amount: i128,
    ) {
        self.env.invoke_contract::<()>(
            &self.gas_service,
            &Symbol::new(self.env, "pay_gas"),
            (
                sender,
                destination_chain,
                destination_address,
                payload,
                spender,
                token,
                amount,
            ).into_val(self.env),
        );
    }
}
