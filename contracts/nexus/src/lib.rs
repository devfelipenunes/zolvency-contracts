#![no_std]

use soroban_sdk::{contract, contracterror, contractevent, contractimpl, contracttype, symbol_short, Address, Env, IntoVal, Map, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    SoulLocks(u32),
    SoulBlacklist(u32),
    Admin,
    PendingAdmin,
    Signer,
    TokenCount,
    TokenId(u32),
    TokenExists(Address),
    AxelarConfig,
    InteropConfig,
    FeeConfig,
    Treasury,
    WillContract,
    Mandate(u64),
    MandateState(u64),
    MandateChildren(u64), // Map mandate_id to Vec<u64> of children ids
    GlobalEpoch(Address),
    VerificationCacheKey(u64, u64), // mandate_id, epoch
    ConsumedNonce(Address, u64, soroban_sdk::BytesN<32>), // root_anchor, epoch, nonce
    NextMandateId,
    AgentMandate(Address), // Mapping agent address to its primary mandate_id for quick lookup
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
    pub transfer_limit: Option<i128>,
    pub renewal_period: Option<u64>,
    pub scope_commitment: Option<soroban_sdk::BytesN<32>>,
    pub contract_allowlist: Option<Vec<Address>>,
    pub function_allowlist: Option<soroban_sdk::Vec<Symbol>>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScopeTag {
    TransferLimit,
    ContractAllowlist,
    FunctionAllowlist,
    ScopeCommitment,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegationRules {
    pub max_subdepth: u32,
    pub allowed_scope_tags: Option<Vec<ScopeTag>>,
    pub budget_fraction: Option<u32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DelegationPolicy {
    None,
    Full,
    Restricted(DelegationRules),
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Mandate {
    pub id: u64,
    pub root_anchor: Address,
    pub issuer: Address,
    pub agent: Address,
    pub scope: Scope,
    pub issued_at_epoch: u64,
    pub delegation_policy: DelegationPolicy,
    pub parent_mandate_id: Option<u64>,
    pub depth: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct MandateState {
    pub mandate_id: u64,
    pub spent_budget: i128,
    pub current_period_start: u64,
    pub allocated_to_children: i128,
    pub is_revoked: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct VerificationCache {
    pub mandate_id: u64,
    pub epoch_at_cache: u64,
    pub is_valid: bool,
    pub cached_at_ledger: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct MandateRequest {
    pub root_anchor: Address,
    pub agent: Address,
    pub scope: Scope,
    pub delegation_policy: DelegationPolicy,
    pub epoch: u64,
    pub nonce: soroban_sdk::BytesN<32>,
    pub sep45_signature: soroban_sdk::BytesN<64>,
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
pub enum MandateError {
    AlreadyInitialized = 1,
    NotAdmin = 2,
    NotPendingAdmin = 3,
    NotInitialized = 4,
    SoulBlocked = 5,
    Unauthorized = 6,
    MandateNotFound = 7,
    MandateRevoked = 8,
    MandateExpired = 9,
    EpochMismatch = 10,
    BudgetExceeded = 11,
    ContractNotAllowed = 12,
    FunctionNotAllowed = 13,
    DepthExceeded = 14,
    DelegationNotAllowed = 15,
    ScopeViolation = 16,
    BudgetFractionViolated = 17,
    NonceAlreadyConsumed = 18,
    InvalidSep45Signature = 19,
    MandateAlreadyExists = 20,
    SoulIDRequired = 21,
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
const MAX_DELEGATION_DEPTH: u32 = 8;
const CACHE_TTL_LEDGERS: u32 = 100;

#[contract]
pub struct Nexus;

// Manual event publishing to avoid macro panics

#[contractimpl]
impl Nexus {
    pub fn initialize(env: Env, admin: Address, signer: Address) -> Result<(), MandateError> {
        if env.storage().persistent().has(&DataKey::Admin) {
            return Err(MandateError::AlreadyInitialized);
        }
        
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::Signer, &signer);
        env.storage().persistent().set(&DataKey::TokenCount, &0u32);
        
        Self::extend_persistent(&env, &DataKey::Admin);
        Self::extend_persistent(&env, &DataKey::Signer);
        Self::extend_persistent(&env, &DataKey::TokenCount);
        
        Ok(())
    }

    pub fn register_token(env: Env, admin: Address, token_contract: Address) -> Result<(), MandateError> {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).ok_or(MandateError::NotInitialized)?;
        if admin != stored_admin {
            return Err(MandateError::NotAdmin);
        }


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
            soroban_sdk::vec![env, soul_id.into_val(env)],
        );

        if let Ok(Ok(true)) = has_res {
            let id_res = env.try_invoke_contract::<u64, soroban_sdk::Error>(
                token_address,
                &Symbol::new(env, "get_user_token"),
                soroban_sdk::vec![env, soul_id.into_val(env)],
            );

            let type_res = env.try_invoke_contract::<Symbol, soroban_sdk::Error>(
                token_address,
                &Symbol::new(env, "get_token_type"),
                soroban_sdk::Vec::new(env),
            );

            if let (Ok(Ok(token_id)), Ok(Ok(token_type))) = (id_res, type_res) {
                reputation.set(token_type, token_id);
            }
        }
    }

    pub fn get_signer(env: Env) -> Result<Address, MandateError> {
        let signer = env.storage().persistent().get(&DataKey::Signer).ok_or(MandateError::NotInitialized)?;
        Self::extend_persistent(&env, &DataKey::Signer);
        Ok(signer)
    }

    pub fn update_signer(env: Env, admin: Address, new_signer: Address) -> Result<(), MandateError> {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).ok_or(MandateError::NotInitialized)?;
        if admin != stored_admin {
            return Err(MandateError::NotAdmin);
        }
        env.storage().persistent().set(&DataKey::Signer, &new_signer);
        Self::extend_persistent(&env, &DataKey::Signer);
        Ok(())
    }

    pub fn transfer_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), MandateError> {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).ok_or(MandateError::NotInitialized)?;
        if admin != stored_admin {
            return Err(MandateError::NotAdmin);
        }
        env.storage().persistent().set(&DataKey::PendingAdmin, &new_admin);
        Self::extend_persistent(&env, &DataKey::PendingAdmin);
        Ok(())
    }

    pub fn accept_admin(env: Env, new_admin: Address) -> Result<(), MandateError> {
        new_admin.require_auth();
        let pending_admin: Address = env.storage().persistent().get(&DataKey::PendingAdmin).ok_or(MandateError::NotPendingAdmin)?;
        if new_admin != pending_admin {
            return Err(MandateError::NotPendingAdmin);
        }
        env.storage().persistent().set(&DataKey::Admin, &new_admin);
        env.storage().persistent().remove(&DataKey::PendingAdmin);
        Self::extend_persistent(&env, &DataKey::Admin);
        Ok(())
    }

    pub fn lock_soul_reputation(env: Env, admin: Address, soul_id: u32, unlock_timestamp: u64) -> Result<(), MandateError> {
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

    pub fn apply_soul_slashing(env: Env, admin: Address, soul_id: u32) -> Result<(), MandateError> {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).ok_or(MandateError::NotInitialized)?;
        if admin != stored_admin {
            return Err(MandateError::NotAdmin);
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
            soroban_sdk::Vec::new(&env),
        )
    }

    pub fn get_epoch(env: Env, root_anchor: Address) -> u64 {
        let key = DataKey::GlobalEpoch(root_anchor.clone());
        let epoch = env.storage().persistent().get(&key).unwrap_or(0);
        if epoch > 0 {
            Self::extend_persistent(&env, &key);
        }
        epoch
    }

    pub fn increment_epoch(env: Env, root_anchor: Address) -> Result<u64, MandateError> {
        root_anchor.require_auth();
        
        let current_epoch: u64 = Self::get_epoch(env.clone(), root_anchor.clone());
        let new_epoch = current_epoch + 1;
        
        let key = DataKey::GlobalEpoch(root_anchor.clone());
        env.storage().persistent().set(&key, &new_epoch);
        Self::extend_persistent(&env, &key);
        
        env.events().publish((symbol_short!("epoch"), root_anchor), new_epoch);
        
        Ok(new_epoch)
    }

    pub fn set_interop_config(env: Env, admin: Address, config: InteropConfig) -> Result<(), MandateError> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;
        env.storage().persistent().set(&DataKey::InteropConfig, &config);
        Ok(())
    }

    pub fn set_axelar_config(env: Env, admin: Address, config: AxelarConfig) -> Result<(), MandateError> {
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
    ) -> Result<(), MandateError> {
        token_address.require_auth();


        if Self::is_soul_blacklisted(env.clone(), soul_id) || Self::is_soul_locked(env.clone(), soul_id) {
            return Err(MandateError::SoulBlocked);
        }


        if !env.storage().persistent().has(&DataKey::TokenExists(token_address.clone())) {
            return Err(MandateError::NotAdmin); 
        }


        if let Some(fee) = env.storage().persistent().get::<_, FeeConfig>(&DataKey::FeeConfig) {
            let treasury = env.storage().persistent().get::<_, Address>(&DataKey::Treasury).ok_or(MandateError::NotInitialized)?;
            let token_client = soroban_sdk::token::Client::new(&env, &fee.token);
            token_client.transfer(&_caller, &treasury, &fee.amount);
        }


        if let Some(cc) = cross_chain {
            if let Some(interop_config) = env.storage().persistent().get::<_, InteropConfig>(&DataKey::InteropConfig) {
                if interop_config.active_protocol != InteropProtocol::None {

                    env.events().publish((symbol_short!("export"), soul_id), (token_address.clone(), external_id.clone(), tier, nonce));


                    let token_type: Symbol = match env.try_invoke_contract::<Symbol, soroban_sdk::Error>(
                        &token_address,
                        &Symbol::new(&env, "get_token_type"),
                        soroban_sdk::Vec::new(&env),
                    ) {
                        Ok(Ok(s)) => s,
                        _ => Symbol::new(&env, "unknown"),
                    };

                    env.invoke_contract::<()>(
                        &interop_config.adapter_address,
                        &Symbol::new(&env, "send_reputation"),
                        soroban_sdk::vec![
                            &env,
                            _caller.into_val(&env),
                            cc.destination_chain.into_val(&env),
                            cc.destination_address.into_val(&env),
                            soul_id.into_val(&env),
                            external_id.into_val(&env),
                            tier.into_val(&env),
                            cc.user_destination_address.into_val(&env),
                            nonce.into_val(&env),
                            token_type.into_val(&env),
                            cc.ecosystem.into_val(&env),
                        ],
                    );
                }
            }
        }

        Ok(())
    }

    pub fn issue_mandate(
        env: Env,
        issuer: Address,
        agent: Address,
        scope: Scope,
        delegation_policy: DelegationPolicy,
        parent_mandate_id: Option<u64>,
    ) -> Result<u64, MandateError> {
        issuer.require_auth();
        
        Self::perform_issue_mandate(
            &env,
            issuer,
            agent,
            scope,
            delegation_policy,
            parent_mandate_id,
        )
    }

    pub fn issue_mandate_remote(
        env: Env,
        request: MandateRequest,
    ) -> Result<u64, MandateError> {
        // 1. Verify Root Anchor current epoch matches request
        let current_epoch = Self::get_epoch(env.clone(), request.root_anchor.clone());
        if request.epoch != current_epoch {
            return Err(MandateError::EpochMismatch);
        }

        // 2. Check nonce uniqueness for this anchor and epoch
        let nonce_key = DataKey::ConsumedNonce(request.root_anchor.clone(), request.epoch, request.nonce.clone());
        if env.storage().persistent().has(&nonce_key) {
            return Err(MandateError::NonceAlreadyConsumed);
        }

        // 3. Verify SEP-45 Signature (Placeholder - requires ed25519 verification)
        // In a real implementation, we would verify request.sep45_signature 
        // against request.root_anchor.
        
        // Mark nonce as consumed
        env.storage().persistent().set(&nonce_key, &true);
        Self::extend_persistent(&env, &nonce_key);

        Self::perform_issue_mandate(
            &env,
            request.root_anchor,
            request.agent,
            request.scope,
            request.delegation_policy,
            None, 
        )
    }

    fn perform_issue_mandate(
        env: &Env,
        issuer: Address,
        agent: Address,
        scope: Scope,
        delegation_policy: DelegationPolicy,
        parent_mandate_id: Option<u64>,
    ) -> Result<u64, MandateError> {
        let mut depth = 0;
        let mut root_anchor = issuer.clone();

        if let Some(pid) = parent_mandate_id {
            let parent: Mandate = env
                .storage()
                .persistent()
                .get(&DataKey::Mandate(pid))
                .ok_or(MandateError::MandateNotFound)?;

            if parent.agent != issuer {
                return Err(MandateError::Unauthorized);
            }

            // Check if parent allows delegation
            match parent.delegation_policy {
                DelegationPolicy::None => return Err(MandateError::DelegationNotAllowed),
                DelegationPolicy::Restricted(ref rules) => {
                    if parent.depth >= rules.max_subdepth {
                        return Err(MandateError::DepthExceeded);
                    }
                    
                    // Check allowed_scope_tags
                    if let Some(ref allowed_tags) = rules.allowed_scope_tags {
                        if scope.transfer_limit.is_some() && !allowed_tags.contains(ScopeTag::TransferLimit) {
                            return Err(MandateError::ScopeViolation);
                        }
                        if scope.contract_allowlist.is_some() && !allowed_tags.contains(ScopeTag::ContractAllowlist) {
                            return Err(MandateError::ScopeViolation);
                        }
                        if scope.function_allowlist.is_some() && !allowed_tags.contains(ScopeTag::FunctionAllowlist) {
                            return Err(MandateError::ScopeViolation);
                        }
                        if scope.scope_commitment.is_some() && !allowed_tags.contains(ScopeTag::ScopeCommitment) {
                            return Err(MandateError::ScopeViolation);
                        }
                    }

                    if let Some(frac) = rules.budget_fraction {
                        if let (Some(p_limit), Some(c_limit)) = (parent.scope.transfer_limit, scope.transfer_limit) {
                            let max_child_limit = (p_limit * (frac as i128)) / 100;
                            if c_limit > max_child_limit {
                                return Err(MandateError::BudgetFractionViolated);
                            }
                        }
                    }
                }
                DelegationPolicy::Full => {}
            }

            // Verify scope monotonicity (child <= parent)
            if scope.ttl > parent.scope.ttl { return Err(MandateError::ScopeViolation); }
            
            if let (Some(p_limit), Some(c_limit)) = (parent.scope.transfer_limit, scope.transfer_limit) {
                if c_limit > p_limit { return Err(MandateError::ScopeViolation); }
                
                // Enforce that sum of all active child transfer_limit values does not exceed parent.transfer_limit
                let mut parent_state: MandateState = env.storage().persistent().get(&DataKey::MandateState(pid)).ok_or(MandateError::MandateNotFound)?;
                
                if parent_state.allocated_to_children + c_limit > p_limit {
                    return Err(MandateError::BudgetExceeded);
                }

                parent_state.allocated_to_children += c_limit;
                env.storage().persistent().set(&DataKey::MandateState(pid), &parent_state);
            } else if parent.scope.transfer_limit.is_some() && scope.transfer_limit.is_none() {
                return Err(MandateError::ScopeViolation); // Cannot remove limit
            }

            // Verify allowlists are subsets
            if let (Some(ref p_contracts), Some(ref c_contracts)) = (&parent.scope.contract_allowlist, &scope.contract_allowlist) {
                for contract in c_contracts.iter() {
                    if !p_contracts.contains(contract) { return Err(MandateError::ScopeViolation); }
                }
            } else if parent.scope.contract_allowlist.is_some() && scope.contract_allowlist.is_none() {
                return Err(MandateError::ScopeViolation);
            }

            depth = parent.depth + 1;
            root_anchor = parent.root_anchor;

            if depth > MAX_DELEGATION_DEPTH {
                return Err(MandateError::DepthExceeded);
            }
        }

        // --- SOUL ID MANDATORY CHECK ---
        // Verify that the root_anchor (Human) has a valid SoulID passport
        let soul_id_contract: Address = env
            .storage()
            .persistent()
            .get(&DataKey::WillContract)
            .ok_or(MandateError::SoulIDRequired)?;

        let has_soul: bool = env.invoke_contract(
            &soul_id_contract,
            &Symbol::new(&env, "has_soul"),
            soroban_sdk::vec![&env, root_anchor.clone().into_val(env)],
        );

        if !has_soul {
            return Err(MandateError::SoulIDRequired);
        }
        // --------------------------------

        let mandate_id: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::NextMandateId)
            .unwrap_or(1);

        let mandate = Mandate {
            id: mandate_id,
            root_anchor: root_anchor.clone(),
            issuer: issuer.clone(),
            agent: agent.clone(),
            scope,
            issued_at_epoch: Self::get_epoch(env.clone(), root_anchor),
            delegation_policy,
            parent_mandate_id,
            depth,
        };

        let state = MandateState {
            mandate_id,
            spent_budget: 0,
            current_period_start: env.ledger().timestamp(),
            allocated_to_children: 0,
            is_revoked: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Mandate(mandate_id), &mandate);
        env.storage()
            .persistent()
            .set(&DataKey::MandateState(mandate_id), &state);

        if let Some(pid) = parent_mandate_id {
            let mut children: Vec<u64> = env
                .storage()
                .persistent()
                .get(&DataKey::MandateChildren(pid))
                .unwrap_or(Vec::new(&env));
            children.push_back(mandate_id);
            env.storage()
                .persistent()
                .set(&DataKey::MandateChildren(pid), &children);
            Self::extend_persistent(&env, &DataKey::MandateChildren(pid));
        }

        env.storage()
            .persistent()
            .set(&DataKey::NextMandateId, &(mandate_id + 1));

        env.storage()
            .persistent()
            .set(&DataKey::AgentMandate(agent.clone()), &mandate_id);

        Self::extend_persistent(&env, &DataKey::Mandate(mandate_id));
        Self::extend_persistent(&env, &DataKey::MandateState(mandate_id));
        Self::extend_persistent(&env, &DataKey::NextMandateId);
        Self::extend_persistent(&env, &DataKey::AgentMandate(agent.clone()));

        // Mint Will SBT if will_contract is set
        if let Some(will_contract) = env
            .storage()
            .persistent()
            .get::<_, Address>(&DataKey::WillContract)
        {
            env.invoke_contract::<()>(
                &will_contract,
                &Symbol::new(&env, "mint"),
                soroban_sdk::vec![&env, mandate.issuer.clone().into_val(env), agent.into_val(env), mandate_id.into_val(env), mandate.scope.ttl.into_val(env)],
            );
        }

        env.events().publish((symbol_short!("issued"), mandate_id), mandate.agent.clone());

        Ok(mandate_id)
    }

    pub fn verify_authority(
        env: Env,
        mandate_id: u64,
        agent: Address,
        contract: Address,
        function: Symbol,
        transfer_amount: Option<i128>,
    ) -> Result<bool, MandateError> {
        let mandate: Mandate = env
            .storage()
            .persistent()
            .get(&DataKey::Mandate(mandate_id))
            .ok_or(MandateError::MandateNotFound)?;

        if mandate.agent != agent {
            return Ok(false);
        }

        let current_epoch = Self::get_epoch(env.clone(), mandate.root_anchor.clone());

        // 2. Check VerificationCache
        let cache_key = DataKey::VerificationCacheKey(mandate_id, current_epoch);
        if let Some(cache) = env.storage().persistent().get::<_, VerificationCache>(&cache_key) {
            if cache.epoch_at_cache == current_epoch 
                && cache.is_valid 
                && (env.ledger().sequence() < cache.cached_at_ledger + CACHE_TTL_LEDGERS) {
                
                // Even on cache hit, we MUST check if revoked
                let mut state: MandateState = env.storage().persistent().get(&DataKey::MandateState(mandate_id)).ok_or(MandateError::MandateNotFound)?;
                if state.is_revoked {
                    return Ok(false);
                }

                // Check/Reset Recurring Budget
                if let Some(period) = mandate.scope.renewal_period {
                    let now = env.ledger().timestamp();
                    if now >= state.current_period_start.checked_add(period).unwrap_or(u64::MAX) {
                        let periods_passed = (now - state.current_period_start) / period;
                        state.current_period_start += periods_passed * period;
                        state.spent_budget = 0;
                        env.storage().persistent().set(&DataKey::MandateState(mandate_id), &state);
                    }
                }
                
                // Even on cache hit, we MUST check budget if transfer_amount is Some
                if let Some(amount) = transfer_amount {
                    if let Some(limit) = mandate.scope.transfer_limit {
                        if state.spent_budget + amount > limit {
                            return Ok(false);
                        }
                    }
                    state.spent_budget += amount;
                    env.storage().persistent().set(&DataKey::MandateState(mandate_id), &state);
                    
                    env.events().publish((symbol_short!("spend"), mandate_id), amount);
                }
                return Ok(true);
            }
        }

        // 3. Traverse the chain
        let mut current_id = mandate_id;
        let mut is_valid = true;
        let now = env.ledger().timestamp();

        for _ in 0..=MAX_DELEGATION_DEPTH {
            let m: Mandate = env.storage().persistent().get(&DataKey::Mandate(current_id)).ok_or(MandateError::MandateNotFound)?;
            let mut s: MandateState = env.storage().persistent().get(&DataKey::MandateState(current_id)).ok_or(MandateError::MandateNotFound)?;

            // Basic checks for every node in the chain
            if m.issued_at_epoch != current_epoch || s.is_revoked || now > m.scope.ttl {
                is_valid = false;
                break;
            }

            // Leaf-specific checks (only for the starting mandate_id)
            if current_id == mandate_id {
                if let Some(ref allowlist) = m.scope.contract_allowlist {
                    if !allowlist.contains(&contract) {
                        is_valid = false;
                        break;
                    }
                }
                if let Some(ref allowlist) = m.scope.function_allowlist {
                    if !allowlist.contains(&function) {
                        is_valid = false;
                        break;
                    }
                }

                // Check/Reset Recurring Budget
                if let Some(period) = m.scope.renewal_period {
                    if now >= s.current_period_start.checked_add(period).unwrap_or(u64::MAX) {
                        let periods_passed = (now - s.current_period_start) / period;
                        s.current_period_start += periods_passed * period;
                        s.spent_budget = 0;
                        // We save this later in step 5 if valid, 
                        // but if we are in the traversal, we should update the local state variable 's' 
                        // so the limit check below uses the reset budget.
                    }
                }

                if let Some(limit) = m.scope.transfer_limit {
                    let amount = transfer_amount.unwrap_or(0);
                    if s.spent_budget + amount > limit {
                        is_valid = false;
                        break;
                    }
                }
            }

            if let Some(pid) = m.parent_mandate_id {
                current_id = pid;
            } else {
                // Reached Root Anchor
                break;
            }
        }

        // 4. Update Cache
        let new_cache = VerificationCache {
            mandate_id,
            epoch_at_cache: current_epoch,
            is_valid,
            cached_at_ledger: env.ledger().sequence(),
        };
        env.storage().persistent().set(&cache_key, &new_cache);
        Self::extend_persistent(&env, &cache_key);

        // 5. Update budget if valid (including potential reset)
        if is_valid {
            let mut state: MandateState = env.storage().persistent().get(&DataKey::MandateState(mandate_id)).ok_or(MandateError::MandateNotFound)?;
            
            // Re-apply reset logic to state before saving
            if let Some(period) = mandate.scope.renewal_period {
                if now >= state.current_period_start.checked_add(period).unwrap_or(u64::MAX) {
                    let periods_passed = (now - state.current_period_start) / period;
                    state.current_period_start += periods_passed * period;
                    state.spent_budget = 0;
                }
            }

            if let Some(amount) = transfer_amount {
                state.spent_budget += amount;
                env.events().publish((symbol_short!("spend"), mandate_id), amount);
            }
            env.storage().persistent().set(&DataKey::MandateState(mandate_id), &state);
        }

        Ok(is_valid)
    }

    pub fn export_will_authority(
        env: Env,
        caller: Address,
        agent: Address,
        cc: CrossChainParams,
    ) -> Result<(), MandateError> {
        caller.require_auth();

        // 1. Fee deduction
        if let Some(fee) = env.storage().persistent().get::<_, FeeConfig>(&DataKey::FeeConfig) {
            let treasury = env.storage().persistent().get::<_, Address>(&DataKey::Treasury).ok_or(MandateError::NotInitialized)?;
            let token_client = soroban_sdk::token::Client::new(&env, &fee.token);
            token_client.transfer(&caller, &treasury, &fee.amount);
        }

        // 2. Lookup mandate and check expiry
        let mandate_id = env.storage().persistent().get::<_, u64>(&DataKey::AgentMandate(agent.clone())).ok_or(MandateError::MandateNotFound)?;
        let mandate: Mandate = env.storage().persistent().get(&DataKey::Mandate(mandate_id)).ok_or(MandateError::MandateNotFound)?;

        if env.ledger().timestamp() > mandate.scope.ttl {
            return Err(MandateError::MandateExpired);
        }

        // 3. Export via interop adapter
        if let Some(interop_config) = env.storage().persistent().get::<_, InteropConfig>(&DataKey::InteropConfig) {
             if interop_config.active_protocol != InteropProtocol::None {
                 env.invoke_contract::<()>(
                    &interop_config.adapter_address,
                    &Symbol::new(&env, "send_will_auth"),
                    soroban_sdk::vec![
                        &env,
                        caller.into_val(&env),
                        cc.destination_chain.into_val(&env),
                        cc.destination_address.into_val(&env),
                        cc.user_destination_address.into_val(&env),
                        1u32.into_val(&env), // soul_id placeholder
                        0b111u64.into_val(&env), // permissions placeholder
                        mandate.scope.ttl.into_val(&env),
                        cc.ecosystem.into_val(&env),
                    ],
                );
             }
        }

        Ok(())
    }

    pub fn revoke_mandate(
        env: Env,
        caller: Address,
        mandate_id: u64,
    ) -> Result<(), MandateError> {
        caller.require_auth();

        let mandate: Mandate = env
            .storage()
            .persistent()
            .get(&DataKey::Mandate(mandate_id))
            .ok_or(MandateError::MandateNotFound)?;

        // Caller MUST be either the root_anchor or the direct issuer
        if caller != mandate.root_anchor && caller != mandate.issuer {
            return Err(MandateError::Unauthorized);
        }

        Self::perform_recursive_revocation(&env, mandate_id)?;

        Ok(())
    }

    fn perform_recursive_revocation(env: &Env, mandate_id: u64) -> Result<(), MandateError> {
        if let Some(mut state) = env.storage().persistent().get::<_, MandateState>(&DataKey::MandateState(mandate_id)) {
            if !state.is_revoked {
                state.is_revoked = true;
                env.storage().persistent().set(&DataKey::MandateState(mandate_id), &state);

                // Invalidate all VerificationCache entries for this mandate
                // (Across all epochs - we just remove the current one for simplicity, 
                // or we could iterate but we don't know all epochs).
                // Actually, verify_authority checks state.is_revoked, so cache invalidation 
                // is an optimization but state check is the truth.
                // Let's just emit event.
                
                env.events().publish((symbol_short!("revoked"), mandate_id), ());

                // Burn associated SBT if exists
                if let Some(will_contract) = env.storage().persistent().get::<_, Address>(&DataKey::WillContract) {
                    let mandate: Mandate = env.storage().persistent().get(&DataKey::Mandate(mandate_id)).unwrap();
                    env.invoke_contract::<()>(
                        &will_contract,
                        &Symbol::new(env, "burn"),
                        soroban_sdk::vec![env, mandate.issuer.into_val(env), mandate.agent.into_val(env)],
                    );
                }

                // Recurse to children
                let children: Vec<u64> = env.storage().persistent().get(&DataKey::MandateChildren(mandate_id)).unwrap_or(Vec::new(env));
                for child_id in children.iter() {
                    Self::perform_recursive_revocation(env, child_id)?;
                }
            }
        }
        Ok(())
    }

    pub fn set_will_contract(env: Env, admin: Address, will_contract: Address) -> Result<(), MandateError> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;

        env.storage().persistent().set(&DataKey::WillContract, &will_contract);
        Ok(())
    }

    pub fn set_fee_config(env: Env, admin: Address, config: FeeConfig) -> Result<(), MandateError> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;
        env.storage().persistent().set(&DataKey::FeeConfig, &config);
        Ok(())
    }

    pub fn set_treasury(env: Env, admin: Address, treasury: Address) -> Result<(), MandateError> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;
        env.storage().persistent().set(&DataKey::Treasury, &treasury);
        Ok(())
    }

    pub fn get_zenith(env: Env, soul_id: u32) -> Map<Symbol, u64> {
        Self::get_soul_reputation(env, soul_id, None)
    }

    fn assert_admin(env: &Env, admin: &Address) -> Result<(), MandateError> {
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).ok_or(MandateError::NotInitialized)?;
        if stored_admin != *admin {
            return Err(MandateError::NotAdmin);
        }
        Ok(())
    }

    fn extend_persistent(env: &Env, key: &DataKey) {
        env.storage().persistent().extend_ttl(key, ONE_YEAR, ONE_YEAR);
    }
}

#[cfg(test)]
mod test;
