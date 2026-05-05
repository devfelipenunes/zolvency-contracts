#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, BytesN, Env};

pub mod nexus_interface;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotAuthorized = 2,
    TokenNotAllowed = 3,
    InvalidPriceSignature = 4,
    PriceTicketExpired = 5,
    NexusRejected = 6,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    NexusContract,
    OraclePubKey,
    ServiceFee,
    NexusFee,
    ZPayTreasury,
    NexusTreasury,
    AllowedToken(Address),
}

#[contract]
pub struct ZPayContract;

#[contractimpl]
impl ZPayContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        nexus_contract: Address,
        oracle_pub_key: BytesN<32>,
        service_fee: i128,
        nexus_fee: i128,
        zpay_treasury: Address,
        nexus_treasury: Address,
    ) -> Result<(), Error> {
        if env.storage().persistent().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::NexusContract, &nexus_contract);
        env.storage().persistent().set(&DataKey::OraclePubKey, &oracle_pub_key);
        env.storage().persistent().set(&DataKey::ServiceFee, &service_fee);
        env.storage().persistent().set(&DataKey::NexusFee, &nexus_fee);
        env.storage().persistent().set(&DataKey::ZPayTreasury, &zpay_treasury);
        env.storage().persistent().set(&DataKey::NexusTreasury, &nexus_treasury);
        Ok(())
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().persistent().get(&DataKey::Admin).unwrap()
    }
}

#[cfg(test)]
mod test;
