#![no_std]

use soroban_sdk::{contract, contractimpl, contracterror, contracttype, symbol_short, Address, Env, IntoVal, Symbol, Vec};

// --- TYPES ---

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
    SoulContract,
    Mandate(u64),
    MandateState(u64),
    GlobalEpoch(Address),
    VerificationCacheKey(u64, u64),
    ConsumedNonce(Address, u64, soroban_sdk::BytesN<32>),
    NextMandateId,
    AgentMandate(Address),
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
    pub expiration: u64,
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
    pub epoch: u64,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MandateError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotAdmin = 3,
    InvalidIssuer = 4,
    MandateNotFound = 5,
    Expired = 6,
    ScopeViolation = 7,
    PolicyViolation = 8,
    BudgetExceeded = 9,
    DepthExceeded = 10,
    InvalidSignature = 11,
    InvalidChainId = 12,
    SoulIDRequired = 13,
    EpochMismatch = 14,
    NonceAlreadyConsumed = 15,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InteropProtocol {
    Axelar,
    LayerZero,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Ecosystem {
    Evm,
    Cosmos,
    Sui,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteropConfig {
    pub active_protocol: InteropProtocol,
    pub adapter_address: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChainParams {
    pub destination_chain: soroban_sdk::String,
    pub destination_address: soroban_sdk::String,
    pub user_destination_address: soroban_sdk::Bytes,
    pub ecosystem: Ecosystem,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct IssueMandateRequest {
    pub root_anchor: Address,
    pub agent: Address,
    pub scope: Scope,
    pub delegation_policy: DelegationPolicy,
    pub parent_mandate_id: Option<u64>,
    pub current_epoch: u64,
    pub nonce: soroban_sdk::BytesN<32>,
    pub sep45_signature: soroban_sdk::BytesN<64>,
}

// --- MODULES ---

mod storage;
mod logic;

#[cfg(test)]
mod test;

#[contract]
pub struct Nexus;

#[contractimpl]
impl Nexus {
    /// Inicializa o Nexus com o administrador e o assinante global.
    pub fn initialize(env: Env, admin: Address, signer: Address) -> Result<(), MandateError> {
        if env.storage().persistent().has(&DataKey::Admin) {
            return Err(MandateError::AlreadyInitialized);
        }
        storage::set_admin(&env, &admin);
        env.storage().persistent().set(&DataKey::Signer, &signer);
        env.storage().persistent().set(&DataKey::TokenCount, &0u32);
        
        Ok(())
    }

    /// Emite um novo mandato (Agentic Delegation).
    pub fn issue_mandate(env: Env, request: IssueMandateRequest) -> Result<u64, MandateError> {
        logic::issue_mandate(&env, request)
    }

    pub fn revoke_mandate(env: Env, revoker: Address, mandate_id: u64) -> Result<(), MandateError> {
        logic::revoke_mandate(&env, revoker, mandate_id)
    }

    /// O coração do protocolo: Verifica se um agente tem autoridade para realizar uma ação.
    pub fn verify_authority(
        env: Env,
        mandate_id: u64,
        agent: Address,
        contract: Address,
        function: Symbol,
        amount: Option<i128>,
    ) -> bool {
        logic::verify_authority(&env, mandate_id, agent, contract, function, amount)
    }

    /// Atualiza o Epoch Global de um humano, invalidando todos os seus agentes instantaneamente.
    pub fn set_global_epoch(env: Env, root_anchor: Address, epoch: u64) -> Result<(), MandateError> {
        root_anchor.require_auth();
        storage::set_global_epoch(&env, &root_anchor, epoch);
        
        env.events().publish((symbol_short!("epoch"), root_anchor), epoch);
        Ok(())
    }

    pub fn increment_epoch(env: Env, root_anchor: Address) -> Result<(), MandateError> {
        let current = storage::get_global_epoch(&env, &root_anchor);
        Self::set_global_epoch(env, root_anchor, current + 1)
    }

    pub fn register_token(env: Env, admin: Address, token_contract: Address) -> Result<(), MandateError> {
        admin.require_auth();
        let stored_admin = storage::get_admin(&env)?;
        if admin != stored_admin {
            return Err(MandateError::NotAdmin);
        }
        env.storage().persistent().set(&DataKey::TokenExists(token_contract), &true);
        Ok(())
    }

    pub fn set_interop_config(env: Env, admin: Address, config: InteropConfig) -> Result<(), MandateError> {
        admin.require_auth();
        let stored_admin = storage::get_admin(&env)?;
        if admin != stored_admin {
            return Err(MandateError::NotAdmin);
        }
        env.storage().persistent().set(&DataKey::InteropConfig, &config);
        Ok(())
    }

    pub fn export_will_authority(
        env: Env,
        root_anchor: Address,
        agent: Address,
        params: CrossChainParams,
    ) -> Result<(), MandateError> {
        root_anchor.require_auth();
        
        let config: InteropConfig = env.storage().persistent().get(&DataKey::InteropConfig).ok_or(MandateError::NotInitialized)?;
        let mandate_id: u64 = env.storage().persistent().get(&DataKey::AgentMandate(agent.clone())).ok_or(MandateError::MandateNotFound)?;
        let mandate = storage::get_mandate(&env, mandate_id).ok_or(MandateError::MandateNotFound)?;

        if mandate.root_anchor != root_anchor {
            return Err(MandateError::PolicyViolation);
        }

        // Chamar o adaptador Axelar (ou outro configurado)
        let _: soroban_sdk::Val = env.invoke_contract(
            &config.adapter_address,
            &soroban_sdk::Symbol::new(&env, "send_will_auth"),
            (
                root_anchor,
                params.destination_chain,
                params.destination_address,
                params.user_destination_address,
                0u32, // soul_id placeholder
                0u64, // permissions placeholder
                mandate.scope.expiration,
                params.ecosystem,
            ).into_val(&env),
        );

        Ok(())
    }

    pub fn set_soul_contract(env: Env, admin: Address, soul_contract: Address) -> Result<(), MandateError> {
        admin.require_auth();
        let stored_admin = storage::get_admin(&env)?;
        if admin != stored_admin {
            return Err(MandateError::NotAdmin);
        }
        env.storage().persistent().set(&DataKey::SoulContract, &soul_contract);
        Ok(())
    }

    pub fn get_mandate(env: Env, id: u64) -> Option<Mandate> {
        storage::get_mandate(&env, id)
    }

    pub fn get_mandate_state(env: Env, id: u64) -> Option<MandateState> {
        storage::get_mandate_state(&env, id)
    }

    pub fn get_global_epoch_value(env: Env, root_anchor: Address) -> u64 {
        storage::get_global_epoch(&env, &root_anchor)
    }

    pub fn export_reputation(
        env: Env,
        caller: Address,
        soul_id: u32,
        origin_contract: Address,
        external_id: soroban_sdk::String,
        tier: u32,
        nonce: u64,
        cross_chain: Option<CrossChainParams>,
    ) -> Result<(), MandateError> {
        logic::export_reputation(&env, caller, soul_id, origin_contract, external_id, tier, nonce, cross_chain)
    }
}
