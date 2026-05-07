// contracts/contracts/zpay/src/nexus_interface.rs
use soroban_sdk::{contractclient, Address, Env, Symbol};

#[contractclient(name = "NexusClient")]
pub trait NexusTrait {
    fn verify_authority(
        env: Env,
        mandate_id: u64,
        agent: Address,
        contract: Address,
        function: Symbol,
        transfer_amount: Option<i128>,
    ) -> bool;
}
