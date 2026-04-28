#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, IntoVal, Map, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    Signer,
    TokenCount,        // Contador de tokens registrados
    TokenId(u32),      // Mapeamento de ID -> Endereço do Contrato
    TokenExists(Address), // Mapeamento de Endereço -> ID (para evitar duplicatas rápidas)
    Locks(Address),
    Blacklist(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotAdmin = 2,
    NotPendingAdmin = 3,
    NotInitialized = 4,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenMetadata {
    pub name: soroban_sdk::String,
    pub symbol: soroban_sdk::String,
    pub version: soroban_sdk::String,
    pub data_source: soroban_sdk::String,
}

const DAY_IN_LEDGERS: u32 = 17_280;
const ONE_YEAR: u32 = 365 * DAY_IN_LEDGERS;

#[contract]
pub struct ZolvencyRegistry;

#[contractimpl]
impl ZolvencyRegistry {
    pub fn initialize(env: Env, admin: Address, signer: Address) -> Result<(), Error> {
        if env.storage().persistent().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::Signer, &signer);
        env.storage().persistent().set(&DataKey::TokenCount, &0u32);
        
        Self::extend_persistent(&env, &DataKey::Admin);
        Self::extend_persistent(&env, &DataKey::Signer);
        Self::extend_persistent(&env, &DataKey::TokenCount);
        
        Ok(())
    }

    pub fn register_token(env: Env, admin: Address, token_contract: Address) -> Result<(), Error> {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).ok_or(Error::NotInitialized)?;
        if admin != stored_admin {
            return Err(Error::NotAdmin);
        }

        // Evita duplicatas usando o mapping TokenExists
        if env.storage().persistent().has(&DataKey::TokenExists(token_contract.clone())) {
            return Ok(());
        }

        let mut count: u32 = env.storage().persistent().get(&DataKey::TokenCount).unwrap_or(0);
        
        env.storage().persistent().set(&DataKey::TokenId(count), &token_contract);
        env.storage().persistent().set(&DataKey::TokenExists(token_contract), &count);
        
        count += 1;
        env.storage().persistent().set(&DataKey::TokenCount, &count);

        Self::extend_persistent(&env, &DataKey::TokenCount);
        
        Ok(())
    }

    pub fn get_user_reputation(env: Env, user: Address) -> Map<Symbol, u64> {
        if Self::is_blacklisted(env.clone(), user.clone()) {
            return Map::new(&env);
        }

        let count: u32 = env.storage().persistent().get(&DataKey::TokenCount).unwrap_or(0);
        let mut reputation = Map::new(&env);

        for i in 0..count {
            if let Some(token_address) = env.storage().persistent().get::<_, Address>(&DataKey::TokenId(i)) {
                // Tenta chamar has_identity(user) de forma segura
                let has_res = env.try_invoke_contract::<bool, soroban_sdk::Error>(
                    &token_address,
                    &Symbol::new(&env, "has_identity"),
                    Vec::from_array(&env, [user.clone().into_val(&env)]),
                );

                if let Ok(Ok(true)) = has_res {
                    // Tenta buscar o token_id
                    let id_res = env.try_invoke_contract::<u64, soroban_sdk::Error>(
                        &token_address,
                        &Symbol::new(&env, "get_user_token"),
                        Vec::from_array(&env, [user.clone().into_val(&env)]),
                    );

                    // Tenta buscar o token_type
                    let type_res = env.try_invoke_contract::<Symbol, soroban_sdk::Error>(
                        &token_address,
                        &Symbol::new(&env, "get_token_type"),
                        Vec::new(&env),
                    );

                    if let (Ok(Ok(token_id)), Ok(Ok(token_type))) = (id_res, type_res) {
                        reputation.set(token_type, token_id);
                    }
                }
            }
        }

        reputation
    }

    pub fn get_signer(env: Env) -> Result<Address, Error> {
        let signer = env.storage().persistent().get(&DataKey::Signer).ok_or(Error::NotInitialized)?;
        Self::extend_persistent(&env, &DataKey::Signer);
        Ok(signer)
    }

    pub fn update_signer(env: Env, admin: Address, new_signer: Address) -> Result<(), Error> {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).ok_or(Error::NotInitialized)?;
        if admin != stored_admin {
            return Err(Error::NotAdmin);
        }
        env.storage().persistent().set(&DataKey::Signer, &new_signer);
        Self::extend_persistent(&env, &DataKey::Signer);
        Ok(())
    }

    pub fn transfer_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), Error> {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).ok_or(Error::NotInitialized)?;
        if admin != stored_admin {
            return Err(Error::NotAdmin);
        }
        env.storage().persistent().set(&DataKey::PendingAdmin, &new_admin);
        Self::extend_persistent(&env, &DataKey::PendingAdmin);
        Ok(())
    }

    pub fn accept_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        new_admin.require_auth();
        let pending_admin: Address = env.storage().persistent().get(&DataKey::PendingAdmin).ok_or(Error::NotPendingAdmin)?;
        if new_admin != pending_admin {
            return Err(Error::NotPendingAdmin);
        }
        env.storage().persistent().set(&DataKey::Admin, &new_admin);
        env.storage().persistent().remove(&DataKey::PendingAdmin);
        Self::extend_persistent(&env, &DataKey::Admin);
        Ok(())
    }

    pub fn lock_reputation(env: Env, caller: Address, user: Address, unlock_timestamp: u64) {
        caller.require_auth();
        let key = DataKey::Locks(user.clone());
        env.storage().persistent().set(&key, &unlock_timestamp);
        Self::extend_persistent(&env, &key);
    }

    pub fn is_locked(env: Env, user: Address) -> bool {
        let key = DataKey::Locks(user);
        if let Some(unlock_timestamp) = env.storage().persistent().get::<_, u64>(&key) {
            Self::extend_persistent(&env, &key);
            env.ledger().timestamp() < unlock_timestamp
        } else {
            false
        }
    }

    pub fn apply_slashing(env: Env, admin: Address, user: Address) -> Result<(), Error> {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).ok_or(Error::NotInitialized)?;
        if admin != stored_admin {
            return Err(Error::NotAdmin);
        }
        let key = DataKey::Blacklist(user.clone());
        env.storage().persistent().set(&key, &true);
        Self::extend_persistent(&env, &key);
        Ok(())
    }

    pub fn is_blacklisted(env: Env, user: Address) -> bool {
        let key = DataKey::Blacklist(user);
        let blacklisted = env.storage().persistent().get(&key).unwrap_or(false);
        if blacklisted {
            Self::extend_persistent(&env, &key);
        }
        blacklisted
    }

    pub fn get_token_metadata(env: Env, token_contract: Address) -> TokenMetadata {
        env.invoke_contract(
            &token_contract,
            &Symbol::new(&env, "get_metadata"),
            Vec::new(&env),
        )
    }

    fn extend_persistent(env: &Env, key: &DataKey) {
        env.storage().persistent().extend_ttl(key, ONE_YEAR, ONE_YEAR);
    }
}

#[cfg(test)]
mod test;
