use soroban_sdk::{Address, Bytes, Env, String};

pub trait MessengerTrait {
    fn send_reputation(
        env: Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
    ) -> Result<(), crate::types::Error>;
}
