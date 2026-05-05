#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, BytesN, Env, Symbol};

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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceTicket {
    pub base_currency: Symbol,
    pub price_per_unit: i128, // Scaled by 10^7
    pub timestamp: u64,
    pub signature: soroban_sdk::BytesN<64>,
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

    pub fn add_token(env: Env, admin: Address, token: Address) -> Result<(), Error> {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            return Err(Error::NotAuthorized);
        }
        env.storage().persistent().set(&DataKey::AllowedToken(token), &true);
        Ok(())
    }

    pub fn remove_token(env: Env, admin: Address, token: Address) -> Result<(), Error> {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            return Err(Error::NotAuthorized);
        }
        env.storage().persistent().remove(&DataKey::AllowedToken(token));
        Ok(())
    }

    pub fn is_token_allowed(env: Env, token: Address) -> bool {
        env.storage().persistent().get(&DataKey::AllowedToken(token)).unwrap_or(false)
    }

    pub fn pay(
        env: Env,
        agent: Address,
        root_anchor: Address,
        seller: Address,
        token: Address,
        base_amount: i128,
        mandate_id: u64,
        price_ticket: Option<PriceTicket>,
    ) -> Result<(), Error> {
        agent.require_auth();
        
        if !Self::is_token_allowed(env.clone(), token.clone()) {
            return Err(Error::TokenNotAllowed);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test;
