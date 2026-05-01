#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, Env, Symbol,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentAuthData {
    pub permissions: u64,
    pub expiry: u64,
    pub human_owner: Address,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Registry,
    Auth(Address), // Mapping Agent Address -> AuthData
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotAuthorized = 1,
    AlreadyInitialized = 2,
    Expired = 3,
    NonTransferable = 4,
}

#[contract]
pub struct ZolvencyAgentSBT;

#[contractimpl]
impl ZolvencyAgentSBT {
    pub fn initialize(env: Env, admin: Address, registry: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Registry, &registry);
        Ok(())
    }

    pub fn mint(
        env: Env,
        human_owner: Address,
        agent: Address,
        permissions: u64,
        expiry: u64,
    ) -> Result<(), Error> {
        let registry: Address = env.storage().instance().get(&DataKey::Registry).ok_or(Error::NotAuthorized)?;
        registry.require_auth();

        let data = AgentAuthData {
            permissions,
            expiry,
            human_owner,
        };

        env.storage().persistent().set(&DataKey::Auth(agent), &data);
        Ok(())
    }

    pub fn burn(env: Env, caller: Address, agent: Address) -> Result<(), Error> {
        let data: AgentAuthData = env.storage().persistent().get(&DataKey::Auth(agent.clone())).ok_or(Error::NotAuthorized)?;
        
        let registry: Address = env.storage().instance().get(&DataKey::Registry).ok_or(Error::NotAuthorized)?;
        
        // Apenas o humano dono ou o Registry podem queimar o SBT
        if caller != data.human_owner && caller != registry {
            return Err(Error::NotAuthorized);
        }
        caller.require_auth();

        env.storage().persistent().remove(&DataKey::Auth(agent));
        Ok(())
    }

    pub fn get_auth(env: Env, agent: Address) -> Result<AgentAuthData, Error> {
        let data: AgentAuthData = env.storage().persistent().get(&DataKey::Auth(agent)).ok_or(Error::NotAuthorized)?;
        
        if env.ledger().timestamp() > data.expiry {
            return Err(Error::Expired);
        }
        
        Ok(data)
    }

    // Função de "transferência" que sempre falha para garantir Soulboundness
    pub fn transfer(_env: Env, _from: Address, _to: Address, _amount: i128) -> Result<(), Error> {
        Err(Error::NonTransferable)
    }
    
    pub fn name(env: Env) -> Symbol {
        Symbol::new(&env, "Zolvency Agent Sub-SBT")
    }
}
