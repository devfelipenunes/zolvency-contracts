#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, IntoVal, Map, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    SoulLocks(u32),
    SoulBlacklist(u32),
    Admin,
    PendingAdmin,
    Signer,
    TokenCount,        // Contador de tokens registrados
    TokenId(u32),      // Mapeamento de ID -> Endereço do Contrato
    TokenExists(Address), // Mapeamento de Endereço -> ID (para evitar duplicatas rápidas)
    AxelarConfig,
    InteropConfig,
    WillAuth(Address), // Mapeamento de Will (EVM/Outro) -> Autorização
    FeeConfig,          // Configuração de taxas (x402)
    Treasury,           // Endereço do Tesouro
    WillContract,       // Endereço do contrato Sub-SBT (Will)
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct FeeConfig {
    pub token: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Scope {
    pub ttl: u64,
    pub contract_allowlist: Option<Vec<Address>>,
    pub function_allowlist: Option<soroban_sdk::Vec<soroban_sdk::String>>,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Mandate {
    pub owner: Address,
    pub soul_id: u32,
    pub agent: Address,
    pub scope: Scope,
    pub can_delegate: bool,
    pub parent_agent: Option<Address>,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ActionContext {
    pub target_contract: Address,
    pub function_name: soroban_sdk::String,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct AxelarConfig {
    pub gateway: Address,
    pub gas_service: Address,
    pub gas_token: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InteropProtocol {
    None,
    Axelar,
    Wormhole,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct InteropConfig {
    pub active_protocol: InteropProtocol,
    pub adapter_address: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Ecosystem {
    Evm,
    Cosmos,
    Solana,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct CrossChainParams {
    pub destination_chain: soroban_sdk::String,
    pub destination_address: soroban_sdk::String,
    pub user_destination_address: soroban_sdk::Bytes,
    pub ecosystem: Ecosystem,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotAdmin = 2,
    NotPendingAdmin = 3,
    NotInitialized = 4,
    SoulBlocked = 5,
    Unauthorized = 6,
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
pub struct Nexus;

#[contractimpl]
impl Nexus {
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

    pub fn get_soul_reputation(env: Env, soul_id: u32, tokens: Option<Vec<Address>>) -> Map<Symbol, u64> {
        if Self::is_soul_blacklisted(env.clone(), soul_id) || Self::is_soul_locked(env.clone(), soul_id) {
            return Map::new(&env);
        }

        let mut reputation = Map::new(&env);

        match tokens {
            Some(token_list) => {
                for token_address in token_list {
                    Self::query_token_reputation(&env, soul_id, &token_address, &mut reputation);
                }
            },
            None => {
                let count: u32 = env.storage().persistent().get(&DataKey::TokenCount).unwrap_or(0);
                let limit = if count > 20 { 20 } else { count };
                
                for i in 0..limit {
                    if let Some(token_address) = env.storage().persistent().get::<_, Address>(&DataKey::TokenId(i)) {
                        Self::query_token_reputation(&env, soul_id, &token_address, &mut reputation);
                    }
                }
            }
        }

        reputation
    }

    fn query_token_reputation(env: &Env, soul_id: u32, token_address: &Address, reputation: &mut Map<Symbol, u64>) {
        let has_res = env.try_invoke_contract::<bool, soroban_sdk::Error>(
            token_address,
            &Symbol::new(env, "has_identity"),
            Vec::from_array(env, [soul_id.into_val(env)]),
        );

        if let Ok(Ok(true)) = has_res {
            let id_res = env.try_invoke_contract::<u64, soroban_sdk::Error>(
                token_address,
                &Symbol::new(env, "get_user_token"),
                Vec::from_array(env, [soul_id.into_val(env)]),
            );

            let type_res = env.try_invoke_contract::<Symbol, soroban_sdk::Error>(
                token_address,
                &Symbol::new(env, "get_token_type"),
                Vec::new(env),
            );

            if let (Ok(Ok(token_id)), Ok(Ok(token_type))) = (id_res, type_res) {
                reputation.set(token_type, token_id);
            }
        }
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

    pub fn lock_soul_reputation(env: Env, admin: Address, soul_id: u32, unlock_timestamp: u64) -> Result<(), Error> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;
        
        let key = DataKey::SoulLocks(soul_id);
        env.storage().persistent().set(&key, &unlock_timestamp);
        Self::extend_persistent(&env, &key);
        Ok(())
    }

    pub fn is_soul_locked(env: Env, soul_id: u32) -> bool {
        let key = DataKey::SoulLocks(soul_id);
        if let Some(unlock_timestamp) = env.storage().persistent().get::<_, u64>(&key) {
            Self::extend_persistent(&env, &key);
            env.ledger().timestamp() < unlock_timestamp
        } else {
            false
        }
    }

    pub fn apply_soul_slashing(env: Env, admin: Address, soul_id: u32) -> Result<(), Error> {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).ok_or(Error::NotInitialized)?;
        if admin != stored_admin {
            return Err(Error::NotAdmin);
        }
        let key = DataKey::SoulBlacklist(soul_id);
        env.storage().persistent().set(&key, &true);
        Self::extend_persistent(&env, &key);
        Ok(())
    }

    pub fn is_soul_blacklisted(env: Env, soul_id: u32) -> bool {
        let key = DataKey::SoulBlacklist(soul_id);
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

    // ── Cross-Chain Logic (Centralized) ──

    pub fn set_interop_config(env: Env, admin: Address, config: InteropConfig) -> Result<(), Error> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;
        env.storage().persistent().set(&DataKey::InteropConfig, &config);
        Ok(())
    }

    pub fn set_axelar_config(env: Env, admin: Address, config: AxelarConfig) -> Result<(), Error> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;
        env.storage().persistent().set(&DataKey::AxelarConfig, &config);
        Ok(())
    }

    pub fn export_reputation(
        env: Env,
        _caller: Address,
        soul_id: u32,
        token_address: Address,
        external_id: soroban_sdk::String,
        tier: u32,
        nonce: u64,
        cross_chain: Option<CrossChainParams>,
    ) -> Result<(), Error> {
        token_address.require_auth();

        // 🛡️ Segurança: Impedir exportação se a alma estiver bloqueada ou na blacklist
        if Self::is_soul_blacklisted(env.clone(), soul_id) || Self::is_soul_locked(env.clone(), soul_id) {
            return Err(Error::SoulBlocked);
        }

        // Apenas tokens registrados podem exportar reputação
        if !env.storage().persistent().has(&DataKey::TokenExists(token_address.clone())) {
            return Err(Error::NotAdmin); 
        }

        // ── x402 Fee Collection ──
        if let Some(fee) = env.storage().persistent().get::<_, FeeConfig>(&DataKey::FeeConfig) {
            let treasury = env.storage().persistent().get::<_, Address>(&DataKey::Treasury).ok_or(Error::NotInitialized)?;
            let token_client = soroban_sdk::token::Client::new(&env, &fee.token);
            token_client.transfer(&_caller, &treasury, &fee.amount);
        }

        // Exporta para o adaptador configurado
        if let Some(cc) = cross_chain {
            if let Some(interop_config) = env.storage().persistent().get::<_, InteropConfig>(&DataKey::InteropConfig) {
                if interop_config.active_protocol != InteropProtocol::None {
                    // Publica um evento interno de exportação
                    env.events().publish(
                        (Symbol::new(&env, "reputation_exported"), soul_id),
                        (token_address.clone(), external_id.clone(), tier, nonce),
                    );

                    // Tenta buscar o token_type
                    let token_type: Symbol = match env.try_invoke_contract::<Symbol, soroban_sdk::Error>(
                        &token_address,
                        &Symbol::new(&env, "get_token_type"),
                        Vec::new(&env),
                    ) {
                        Ok(Ok(s)) => s,
                        _ => Symbol::new(&env, "unknown"),
                    };

                    env.invoke_contract::<()>(
                        &interop_config.adapter_address,
                        &Symbol::new(&env, "send_reputation"),
                        (
                            _caller,
                            cc.destination_chain,
                            cc.destination_address,
                            soul_id, // Add this
                            external_id,
                            tier,
                            cc.user_destination_address,
                            nonce,
                            token_type,
                            cc.ecosystem,
                        )
                            .into_val(&env),
                    );
                }
            }
        }

        Ok(())
    }

    pub fn authorize_will(
        env: Env,
        user: Address,
        will_address: Address,
        scope: Scope,
        can_delegate: bool,
        parent_agent: Option<Address>,
    ) -> Result<(), Error> {
        user.require_auth();
        
        // Handle hierarchical checks
        let owner = if let Some(parent) = parent_agent.clone() {
            let parent_auth: Mandate = env.storage().persistent()
                .get(&DataKey::WillAuth(parent.clone()))
                .ok_or(Error::Unauthorized)?;
            
            if !parent_auth.can_delegate {
                return Err(Error::Unauthorized);
            }
            if parent_auth.agent != user {
                return Err(Error::Unauthorized);
            }
            if scope.ttl > parent_auth.scope.ttl {
                return Err(Error::Unauthorized);
            }
            parent_auth.owner
        } else {
            user.clone()
        };

        let auth = Mandate {
            owner: owner.clone(),
            soul_id: 1, // TODO: Integrate with Soul Token Registry dynamically
            agent: will_address.clone(),
            scope: scope.clone(),
            can_delegate,
            parent_agent,
        };

        env.storage().persistent().set(&DataKey::WillAuth(will_address.clone()), &auth);
        
        if let Some(will_contract) = env.storage().persistent().get::<_, Address>(&DataKey::WillContract) {
            let _: soroban_sdk::Val = env.invoke_contract(
                &will_contract,
                &Symbol::new(&env, "mint"),
                (owner.clone(), will_address.clone(), scope.ttl).into_val(&env), // adjusted args for sub-SBT
            );
        }

        env.events().publish(
            (Symbol::new(&env, "mandate_issued"), owner),
            (will_address, scope.ttl),
        );

        Ok(())
    }

    pub fn verify_authority(
        env: Env,
        agent: Address,
        action_context: ActionContext,
    ) -> bool {
        let auth_opt: Option<Mandate> = env.storage().persistent().get(&DataKey::WillAuth(agent.clone()));
        if auth_opt.is_none() {
            return false;
        }
        let auth = auth_opt.unwrap();

        if env.ledger().timestamp() > auth.scope.ttl {
            return false;
        }

        if let Some(contracts) = auth.scope.contract_allowlist {
            if !contracts.contains(&action_context.target_contract) {
                return false;
            }
        }

        if let Some(functions) = auth.scope.function_allowlist {
            if !functions.contains(&action_context.function_name) {
                return false;
            }
        }

        // Recursive check for parent
        if let Some(parent) = auth.parent_agent {
            return Self::verify_authority(env, parent, action_context);
        }

        true
    }

    pub fn export_will_authority(
        env: Env,
        _caller: Address,
        will_address: Address,
        cross_chain: CrossChainParams,
    ) -> Result<(), Error> {
        _caller.require_auth();

        let auth: Mandate = env.storage().persistent()
            .get(&DataKey::WillAuth(will_address.clone()))
            .ok_or(Error::NotInitialized)?;

        if env.ledger().timestamp() > auth.scope.ttl {
            return Err(Error::SoulBlocked);
        }

        if _caller != auth.owner && _caller != will_address {
            return Err(Error::Unauthorized);
        }

        if let Some(interop_config) = env.storage().persistent().get::<_, InteropConfig>(&DataKey::InteropConfig) {
            if interop_config.active_protocol != InteropProtocol::None {
                if let Some(fee) = env.storage().persistent().get::<_, FeeConfig>(&DataKey::FeeConfig) {
                    let treasury = env.storage().persistent().get::<_, Address>(&DataKey::Treasury).ok_or(Error::NotInitialized)?;
                    let token_client = soroban_sdk::token::Client::new(&env, &fee.token);
                    token_client.transfer(&_caller, &treasury, &fee.amount);
                }

                // Since EVM still expects "permissions" as an int in the payload, 
                // we send a default or mapped mask until the EVM adapter supports full Scope.
                let mock_permissions: u64 = 1; 

                env.invoke_contract::<()>(
                    &interop_config.adapter_address,
                    &Symbol::new(&env, "send_will_auth"),
                    (
                        _caller,
                        cross_chain.destination_chain,
                        cross_chain.destination_address,
                        cross_chain.user_destination_address,
                        auth.soul_id,
                        mock_permissions,
                        auth.scope.ttl,
                        cross_chain.ecosystem,
                    )
                        .into_val(&env),
                );
            }
        }

        Ok(())
    }

    pub fn revoke_will(env: Env, user: Address, will_address: Address) -> Result<(), Error> {
        user.require_auth();
        
        let auth: Mandate = env.storage().persistent()
            .get(&DataKey::WillAuth(will_address.clone()))
            .ok_or(Error::NotInitialized)?;
            
        // SECURITY: Ensure owner or parent can revoke
        if auth.owner != user && auth.parent_agent != Some(user.clone()) && auth.agent != user {
            return Err(Error::Unauthorized);
        }
        
        env.storage().persistent().remove(&DataKey::WillAuth(will_address.clone()));
        
        if let Some(will_contract) = env.storage().persistent().get::<_, Address>(&DataKey::WillContract) {
            let _: soroban_sdk::Val = env.invoke_contract(
                &will_contract,
                &Symbol::new(&env, "burn"),
                (env.current_contract_address(), will_address.clone()).into_val(&env),
            );
        }

        env.events().publish(
            (Symbol::new(&env, "mandate_revoked"), user),
            (will_address,),
        );
        Ok(())
    }

    pub fn set_will_contract(env: Env, admin: Address, will_contract: Address) -> Result<(), Error> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;

        env.storage().persistent().set(&DataKey::WillContract, &will_contract);
        Ok(())
    }

    pub fn set_fee_config(env: Env, admin: Address, config: FeeConfig) -> Result<(), Error> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;
        env.storage().persistent().set(&DataKey::FeeConfig, &config);
        Ok(())
    }

    pub fn set_treasury(env: Env, admin: Address, treasury: Address) -> Result<(), Error> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;
        env.storage().persistent().set(&DataKey::Treasury, &treasury);
        Ok(())
    }

    pub fn get_zenith(env: Env, soul_id: u32) -> Map<Symbol, u64> {
        Self::get_soul_reputation(env, soul_id, None)
    }

    fn assert_admin(env: &Env, admin: &Address) -> Result<(), Error> {
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).ok_or(Error::NotInitialized)?;
        if stored_admin != *admin {
            return Err(Error::NotAdmin);
        }
        Ok(())
    }

    fn extend_persistent(env: &Env, key: &DataKey) {
        env.storage().persistent().extend_ttl(key, ONE_YEAR, ONE_YEAR);
    }
}

#[cfg(test)]
mod test;
