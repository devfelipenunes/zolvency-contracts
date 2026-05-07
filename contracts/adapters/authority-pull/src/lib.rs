#![no_std]
use soroban_sdk::{contract, contractevent, contractimpl, Address, Bytes, Env, Error, String};

#[contract]
pub struct AuthorityPullAdapter;

#[contractevent]
pub enum AuthorityPullEvent {
    ReputationExport {
        caller: Address,
        destination_chain: String,
        destination_address: String,
        external_id: String,
        tier: u32,
        user_evm_address: Bytes,
        nonce: u64,
    },
}

#[contractimpl]
impl AuthorityPullAdapter {
    /// No Authority-Pull, não há custo de gás cross-chain na Stellar.
    pub fn estimate_fee(_env: Env, _destination_chain: String) -> i128 {
        0
    }

    /// Apenas emite um evento que o indexador/autoridade captura para assinar.
    pub fn send(
        env: Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        external_id: String,
        tier: u32,
        user_evm_address: Bytes,
        nonce: u64,
    ) -> Result<(), Error> {
        caller.require_auth();

        AuthorityPullEvent::ReputationExport {
            caller,
            destination_chain,
            destination_address,
            external_id,
            tier,
            user_evm_address,
            nonce,
        }
        .publish(&env);

        Ok(())
    }
}

#[cfg(test)]
mod test;
