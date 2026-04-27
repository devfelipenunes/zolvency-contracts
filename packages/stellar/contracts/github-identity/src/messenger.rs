use soroban_sdk::{contractclient, Address, Bytes, Env, Error, String};

#[contractclient(name = "MessengerClient")]
#[allow(dead_code)]
pub trait MessengerTrait {
    /// Estima a taxa necessária para o envio cross-chain.
    fn estimate_fee(env: Env, destination_chain: String) -> i128;

    /// Envia a reputação para outra cadeia.
    fn send(
        env: Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        external_id: String,
        tier: u32,
        user_evm_address: Bytes,
    ) -> Result<(), Error>;
}
