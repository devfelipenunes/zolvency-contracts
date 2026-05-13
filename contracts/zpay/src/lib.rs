#![no_std]

use soroban_sdk::{contract, contractimpl, contracterror, contracttype, Address, BytesN, Env, Symbol};

// --- TYPES ---

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
    InvalidCurrency = 7,
    Overflow = 8,
    MissingOracleData = 9,
    EscrowNotFound = 10,
    EscrowNotExpired = 11,
    ContractPaused = 12,
    OracleStale = 13,
    MaxRelayerFeeExceeded = 14,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceTicket {
    pub base_currency: Symbol,
    pub price_per_unit: i128,
    pub timestamp: u64,
    pub signature: BytesN<64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowEntry {
    pub root_anchor: Address,
    pub seller: Address,
    pub token: Address,
    pub base_amount: i128,
    pub mandate_id: u64,
    pub timeout_ledger: u32,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    NexusContract,
    OraclePubKey,
    StorkOracle,
    ServiceFeeBps,
    MinServiceFee,
    NexusFeeBps,
    MinNexusFee,
    ZPayTreasury,
    NexusTreasury,
    AllowedToken(Address),
    NextPaymentId,
    Escrow(u64),
    IsPaused,
    MaxStaleness,
    MaxRelayerFeeBps,
    FallbackOracle(Address),
}

// --- MODULES ---

mod storage;
mod logic;
mod interfaces;

#[cfg(test)]
mod test;

pub use crate::interfaces::*;

#[contract]
pub struct ZPayContract;

#[contractimpl]
impl ZPayContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        nexus_contract: Address,
        oracle_pub_key: BytesN<32>,
        stork_oracle: Address,
        service_fee_bps: u32,
        min_service_fee: i128,
        nexus_fee_bps: u32,
        min_nexus_fee: i128,
        zpay_treasury: Address,
        nexus_treasury: Address,
    ) -> Result<(), Error> {
        if storage::get_admin(&env).is_ok() {
            return Err(Error::AlreadyInitialized);
        }
        storage::set_admin(&env, &admin);
        env.storage().persistent().set(&DataKey::NexusContract, &nexus_contract);
        env.storage().persistent().set(&DataKey::OraclePubKey, &oracle_pub_key);
        env.storage().persistent().set(&DataKey::StorkOracle, &stork_oracle);
        env.storage().persistent().set(&DataKey::ServiceFeeBps, &service_fee_bps);
        env.storage().persistent().set(&DataKey::MinServiceFee, &min_service_fee);
        env.storage().persistent().set(&DataKey::NexusFeeBps, &nexus_fee_bps);
        env.storage().persistent().set(&DataKey::MinNexusFee, &min_nexus_fee);
        env.storage().persistent().set(&DataKey::ZPayTreasury, &zpay_treasury);
        env.storage().persistent().set(&DataKey::NexusTreasury, &nexus_treasury);
        env.storage().persistent().set(&DataKey::IsPaused, &false);
        env.storage().persistent().set(&DataKey::MaxStaleness, &3600u64);
        env.storage().persistent().set(&DataKey::MaxRelayerFeeBps, &500u32);
        Ok(())
    }

    pub fn get_admin(env: Env) -> Result<Address, Error> {
        storage::get_admin(&env)
    }

    pub fn propose_new_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), Error> {
        admin.require_auth();
        let current_admin = storage::get_admin(&env)?;
        if admin != current_admin {
            return Err(Error::NotAuthorized);
        }
        env.storage().persistent().set(&DataKey::PendingAdmin, &new_admin);
        Ok(())
    }

    pub fn claim_admin_rights(env: Env, caller: Address) -> Result<(), Error> {
        caller.require_auth();
        let pending: Address = env.storage().persistent().get(&DataKey::PendingAdmin).ok_or(Error::NotAuthorized)?;
        if caller != pending {
            return Err(Error::NotAuthorized);
        }
        storage::set_admin(&env, &caller);
        env.storage().persistent().remove(&DataKey::PendingAdmin);
        Ok(())
    }

    pub fn set_paused(env: Env, admin: Address, paused: bool) -> Result<(), Error> {
        admin.require_auth();
        let stored_admin = storage::get_admin(&env)?;
        if admin != stored_admin {
            return Err(Error::NotAuthorized);
        }
        env.storage().persistent().set(&DataKey::IsPaused, &paused);
        Ok(())
    }

    pub fn set_max_staleness(env: Env, admin: Address, staleness: u64) -> Result<(), Error> {
        admin.require_auth();
        let current_admin = storage::get_admin(&env)?;
        if admin != current_admin {
            return Err(Error::NotAuthorized);
        }
        env.storage().persistent().set(&DataKey::MaxStaleness, &staleness);
        Ok(())
    }

    pub fn set_fallback_oracle(env: Env, admin: Address, token: Address, oracle: Address) -> Result<(), Error> {
        admin.require_auth();
        let current_admin = storage::get_admin(&env)?;
        if admin != current_admin {
            return Err(Error::NotAuthorized);
        }
        env.storage().persistent().set(&DataKey::FallbackOracle(token), &oracle);
        Ok(())
    }

    pub fn add_token(env: Env, admin: Address, token: Address) -> Result<(), Error> {
        admin.require_auth();
        let stored_admin = storage::get_admin(&env)?;
        if admin != stored_admin {
            return Err(Error::NotAuthorized);
        }
        env.storage().persistent().set(&DataKey::AllowedToken(token), &true);
        Ok(())
    }

    pub fn is_paused(env: Env) -> bool {
        storage::is_paused(&env)
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
        oracle_feed_id: Option<BytesN<32>>,
        relayer: Option<Address>,
        relayer_fee: Option<i128>,
    ) -> Result<(), Error> {
        logic::execute_payment(
            &env, agent, root_anchor, seller, token, base_amount, 
            mandate_id, price_ticket, oracle_feed_id, relayer, relayer_fee,
            Symbol::new(&env, "pay")
        )
    }

    pub fn pay_escrow(
        env: Env,
        agent: Address,
        root_anchor: Address,
        seller: Address,
        token: Address,
        base_amount: i128,
        mandate_id: u64,
        price_ticket: Option<PriceTicket>,
        oracle_feed_id: Option<BytesN<32>>,
        timeout_duration: u32,
        relayer: Option<Address>,
        relayer_fee: Option<i128>,
    ) -> Result<u64, Error> {
        logic::create_escrow(
            &env, agent, root_anchor, seller, token, base_amount,
            mandate_id, price_ticket, timeout_duration, relayer, relayer_fee
        )
    }

    pub fn release_escrow(env: Env, caller: Address, payment_id: u64) -> Result<(), Error> {
        logic::release_escrow(&env, caller, payment_id)
    }

    pub fn refund_escrow(env: Env, caller: Address, payment_id: u64) -> Result<(), Error> {
        logic::refund_escrow(&env, caller, payment_id)
    }

    pub fn charge_subscription(
        env: Env,
        seller: Address,
        root_anchor: Address,
        token: Address,
        base_amount: i128,
        mandate_id: u64,
        price_ticket: Option<PriceTicket>,
        oracle_feed_id: Option<BytesN<32>>,
        relayer: Option<Address>,
        relayer_fee: Option<i128>,
    ) -> Result<(), Error> {
        logic::execute_payment(
            &env, seller.clone(), root_anchor, seller, token, base_amount,
            mandate_id, price_ticket, oracle_feed_id, relayer, relayer_fee,
            Symbol::new(&env, "charge")
        )
    }

    pub fn upgrade(env: Env, admin: Address, new_wasm_hash: soroban_sdk::BytesN<32>) -> Result<(), Error> {
        admin.require_auth();
        let stored_admin = storage::get_admin(&env)?;
        if admin != stored_admin {
            return Err(Error::NotAuthorized);
        }
        env.deployer().update_current_contract_wasm(new_wasm_hash);
        Ok(())
    }
}
