use soroban_sdk::{contracttype, Address, Bytes, BytesN, Env, IntoVal, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessagingFee {
    pub native_fee: i128,
    pub lz_token_fee: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessagingReceipt {
    pub guid: BytesN<32>,
    pub nonce: u64,
    pub fee: MessagingFee,
}

pub struct LayerZeroClient<'a> {
    pub env: &'a Env,
    pub endpoint: Address,
}

impl<'a> LayerZeroClient<'a> {
    pub fn new(env: &'a Env, endpoint: Address) -> Self {
        Self { env, endpoint }
    }

    pub fn send(
        &self,
        dst_eid: u32,
        receiver: BytesN<32>,
        message: Bytes,
        options: Bytes,
        fee: MessagingFee,
    ) -> MessagingReceipt {
        self.env.invoke_contract::<MessagingReceipt>(
            &self.endpoint,
            &Symbol::new(self.env, "send"),
            (dst_eid, receiver, message, options, fee).into_val(self.env),
        )
    }

    pub fn quote(
        &self,
        dst_eid: u32,
        receiver: BytesN<32>,
        message: Bytes,
        options: Bytes,
        pay_in_lz_token: bool,
    ) -> MessagingFee {
        self.env.invoke_contract::<MessagingFee>(
            &self.endpoint,
            &Symbol::new(self.env, "quote"),
            (dst_eid, receiver, message, options, pay_in_lz_token).into_val(self.env),
        )
    }
}
