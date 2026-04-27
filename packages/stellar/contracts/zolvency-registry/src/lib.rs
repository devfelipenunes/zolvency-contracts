#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, IntoVal, Map, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    Signer,
    Tokens,
    Locks(Address),
    Blacklist(Address),
}

#[cfg(test)]
mod test;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenMetadata {
    pub name: soroban_sdk::String,
    pub symbol: soroban_sdk::String,
    pub version: soroban_sdk::String,
    pub data_source: soroban_sdk::String,
}

#[contract]
pub struct ZolvencyRegistry;

#[contractimpl]
impl ZolvencyRegistry {
    pub fn initialize(env: Env, admin: Address, signer: Address) {
        if env.storage().persistent().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::Signer, &signer);
        env.storage()
            .persistent()
            .set(&DataKey::Tokens, &Vec::<Address>::new(&env));
    }

    pub fn register_token(env: Env, admin: Address, token_contract: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("Not admin");
        }

        let mut tokens: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Tokens)
            .unwrap_or(Vec::new(&env));

        // Verifica se já não existe
        for t in tokens.iter() {
            if t == token_contract {
                return;
            }
        }

        tokens.push_back(token_contract);
        env.storage().persistent().set(&DataKey::Tokens, &tokens);
    }

    /// Retorna todos os tokens registrados que um usuário possui e seus dados básicos.
    /// Esta função será usada pesadamente pelo SDK.
    pub fn get_user_reputation(env: Env, user: Address) -> Map<Symbol, u64> {
        if Self::is_blacklisted(env.clone(), user.clone()) {
            return Map::new(&env); // Retorna vazio se estiver na blacklist
        }

        let tokens: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Tokens)
            .unwrap_or(Vec::new(&env));
        let mut reputation = Map::new(&env);

        for token_address in tokens.iter() {
            // Chamada cross-contract para has_identity(user)
            let has: bool = env.invoke_contract(
                &token_address,
                &Symbol::new(&env, "has_identity"),
                Vec::from_array(&env, [user.clone().into_val(&env)]),
            );

            if has {
                // Se o usuário tem o token, pegamos o ID dele
                let token_id: u64 = env.invoke_contract(
                    &token_address,
                    &Symbol::new(&env, "get_user_token"),
                    Vec::from_array(&env, [user.clone().into_val(&env)]),
                );

                // Pegamos o tipo do token via interface padronizada
                let token_type: Symbol = env.invoke_contract(
                    &token_address,
                    &Symbol::new(&env, "get_token_type"),
                    Vec::new(&env),
                );

                reputation.set(token_type, token_id);
            }
        }

        reputation
    }

    pub fn get_signer(env: Env) -> Address {
        env.storage().persistent().get(&DataKey::Signer).unwrap()
    }

    pub fn update_signer(env: Env, admin: Address, new_signer: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("Not admin");
        }
        env.storage()
            .persistent()
            .set(&DataKey::Signer, &new_signer);
    }

    pub fn transfer_admin(env: Env, admin: Address, new_admin: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("Not admin");
        }
        env.storage()
            .persistent()
            .set(&DataKey::PendingAdmin, &new_admin);
    }

    pub fn accept_admin(env: Env, new_admin: Address) {
        new_admin.require_auth();
        let pending_admin: Address = env
            .storage()
            .persistent()
            .get(&DataKey::PendingAdmin)
            .unwrap();
        if new_admin != pending_admin {
            panic!("Not pending admin");
        }
        env.storage().persistent().set(&DataKey::Admin, &new_admin);
        env.storage().persistent().remove(&DataKey::PendingAdmin);
    }

    pub fn lock_reputation(env: Env, caller: Address, user: Address, unlock_timestamp: u64) {
        // Nota: Em produção, 'caller' seria verificado contra uma lista de protocolos autorizados.
        caller.require_auth();

        let key = DataKey::Locks(user.clone());
        env.storage().persistent().set(&key, &unlock_timestamp);
    }

    pub fn is_locked(env: Env, user: Address) -> bool {
        let key = DataKey::Locks(user);
        if let Some(unlock_timestamp) = env.storage().persistent().get::<_, u64>(&key) {
            env.ledger().timestamp() < unlock_timestamp
        } else {
            false
        }
    }

    pub fn apply_slashing(env: Env, admin: Address, user: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("Not admin");
        }
        let key = DataKey::Blacklist(user.clone());
        env.storage().persistent().set(&key, &true);
    }

    pub fn is_blacklisted(env: Env, user: Address) -> bool {
        let key = DataKey::Blacklist(user);
        env.storage().persistent().get(&key).unwrap_or(false)
    }

    pub fn get_token_metadata(env: Env, token_contract: Address) -> TokenMetadata {
        env.invoke_contract(
            &token_contract,
            &Symbol::new(&env, "get_metadata"),
            Vec::new(&env),
        )
    }
}
