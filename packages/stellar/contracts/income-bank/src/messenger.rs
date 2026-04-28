use soroban_sdk::{contractclient, Address, Bytes, Env, Error, String};

#[contractclient(name = "MessengerClient")]
#[allow(dead_code)]
pub trait MessengerTrait {
    fn estimate_fee(env: Env, destination_chain: String) -> i128;

    fn send(
        env: Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        external_id: String,
        tier: u32,
        user_evm_address: Bytes,
        nonce: u64,
    ) -> Result<(), Error>;
}
