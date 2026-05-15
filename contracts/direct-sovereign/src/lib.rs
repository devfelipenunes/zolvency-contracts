#![no_std]

use soroban_sdk::{contract, contractimpl, contracterror, contracttype, Address, Env, Symbol, token, Vec};

mod storage;

#[cfg(test)]
mod test;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotAuthorized = 2,
    NexusRejected = 3,
    SubscriptionNotFound = 4,
    LimitExceeded = 5,
    Expired = 6,
}

pub mod nexus_client {
    use soroban_sdk::{contractclient, Address, Env, Symbol, Vec};

    // We use the EXACT types from nexus to avoid serialization issues
    #[contractclient(name = "NexusClient")]
    pub trait NexusTrait {
        fn initialize(env: Env, admin: Address, signer: Address);
        fn set_soul_contract(env: Env, admin: Address, soul_contract: Address);
        fn issue_mandate_as_admin(
            env: Env,
            root_anchor: Address,
            agent: Address,
            scope: ::nexus::Scope,
            delegation_policy: ::nexus::DelegationPolicy,
            parent_mandate_id: Option<u64>,
        ) -> u64;
        fn verify_authority(
            env: Env,
            mandate_id: u64,
            agent: Address,
            contract: Address,
            function: Symbol,
            amount: Option<i128>,
            token: Option<Address>,
        ) -> bool;
    }
}

#[contract]
pub struct DirectSovereign;

#[contractimpl]
impl DirectSovereign {
    pub fn initialize(env: Env, admin: Address, nexus: Address) -> Result<(), Error> {
        if storage::get_admin(&env).is_some() {
            return Err(Error::AlreadyInitialized);
        }
        storage::set_admin(&env, &admin);
        storage::set_nexus(&env, &nexus);
        Ok(())
    }

    pub fn subscribe(
        env: Env,
        user: Address,
        service_provider: Address,
        token: Address,
        mandate_id: u64,
        monthly_limit: i128,
        duration_months: u32,
    ) -> Result<(), Error> {
        user.require_auth();

        let sub = storage::Subscription {
            user,
            service_provider,
            token,
            monthly_limit,
            current_month_spent: 0,
            last_charge_time: 0,
            start_time: env.ledger().timestamp(),
            duration_months,
            mandate_id,
        };

        storage::set_subscription(&env, mandate_id, &sub);
        Ok(())
    }

    pub fn charge(env: Env, mandate_id: u64, amount: i128) -> Result<(), Error> {
        let mut sub = storage::get_subscription(&env, mandate_id).ok_or(Error::SubscriptionNotFound)?;
        
        sub.service_provider.require_auth();

        let nexus_addr = storage::get_nexus(&env).ok_or(Error::NotAuthorized)?;
        let client = nexus_client::NexusClient::new(&env, &nexus_addr);

        let is_authorized = client.verify_authority(
            &mandate_id,
            &sub.service_provider,
            &env.current_contract_address(),
            &Symbol::new(&env, "charge"),
            &Some(amount),
            &Some(sub.token.clone()),
        );

        if !is_authorized {
            return Err(Error::NexusRejected);
        }

        let current_time = env.ledger().timestamp();
        if current_time > sub.start_time + (sub.duration_months as u64 * 30 * 24 * 60 * 60) {
            return Err(Error::Expired);
        }

        let seconds_in_month = 30 * 24 * 60 * 60;
        if current_time >= sub.last_charge_time + seconds_in_month {
            sub.current_month_spent = 0;
        }

        if sub.current_month_spent + amount > sub.monthly_limit {
            return Err(Error::LimitExceeded);
        }

        let token_client = token::Client::new(&env, &sub.token);
        token_client.transfer_from(
            &env.current_contract_address(),
            &sub.user,
            &sub.service_provider,
            &amount,
        );

        sub.current_month_spent += amount;
        sub.last_charge_time = current_time;
        storage::set_subscription(&env, mandate_id, &sub);

        Ok(())
    }
}
