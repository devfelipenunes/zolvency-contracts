use soroban_sdk::{Address, Env, Symbol, symbol_short, IntoVal};
use crate::*;
use crate::storage;

pub fn issue_mandate(env: &Env, request: IssueMandateRequest) -> Result<u64, MandateError> {
    // 1. Verificar Época
    let stored_epoch = storage::get_global_epoch(env, &request.root_anchor);
    if stored_epoch != request.current_epoch {
        return Err(MandateError::EpochMismatch);
    }

    // 2. Verificar Nonce (SEP-45)
    let nonce_key = DataKey::ConsumedNonce(request.root_anchor.clone(), stored_epoch, request.nonce.clone());
    if env.storage().persistent().has(&nonce_key) {
        return Err(MandateError::NonceAlreadyConsumed);
    }
    env.storage().persistent().set(&nonce_key, &true);

    // 3. Validação de Hierarquia (se houver pai)
    let mut depth = 0;
    let issuer = if let Some(parent_id) = request.parent_mandate_id {
        let parent = storage::get_mandate(env, parent_id).ok_or(MandateError::MandateNotFound)?;
        let parent_state = storage::get_mandate_state(env, parent_id).ok_or(MandateError::MandateNotFound)?;

        // A. Verificar se o pai está revogado
        if parent_state.is_revoked {
            return Err(MandateError::PolicyViolation);
        }

        // B. Verificar se o root_anchor é consistente
        if parent.root_anchor != request.root_anchor {
            return Err(MandateError::PolicyViolation);
        }

        // C. Validação de Escopo vs Pai
        if request.scope.expiration > parent.scope.expiration {
            return Err(MandateError::PolicyViolation);
        }
        
        if let (Some(child_limit), Some(parent_limit)) = (request.scope.transfer_limit, parent.scope.transfer_limit) {
            if child_limit > parent_limit {
                return Err(MandateError::PolicyViolation);
            }
        }

        // D. Verificar Consistência de Token
        if let (Some(child_token), Some(parent_token)) = (request.scope.token.clone(), parent.scope.token.clone()) {
            if child_token != parent_token {
                return Err(MandateError::PolicyViolation);
            }
        }

        // C. Verificar Regras de Delegação
        match parent.delegation_policy {
            DelegationPolicy::None => return Err(MandateError::PolicyViolation),
            DelegationPolicy::Restricted(ref rules) => {
                if parent.depth >= rules.max_subdepth {
                    return Err(MandateError::DepthExceeded);
                }
                
                // Verificar Fração de Orçamento (budget_fraction)
                if let Some(fraction) = rules.budget_fraction {
                    if let Some(parent_limit) = parent.scope.transfer_limit {
                        let allowed = (parent_limit * (fraction as i128)) / 100;
                        if request.scope.transfer_limit.unwrap_or(0) > allowed {
                            return Err(MandateError::PolicyViolation);
                        }
                    }
                }

                // Verificar Tags Permitidas (allowed_scope_tags)
                if let Some(ref allowed_tags) = rules.allowed_scope_tags {
                    // Verificação simplificada: se o filho tem algo que o pai não permite explicitamente
                    if request.scope.contract_allowlist.is_some() && !allowed_tags.contains(ScopeTag::ContractAllowlist) {
                         return Err(MandateError::PolicyViolation);
                    }
                    if request.scope.function_allowlist.is_some() && !allowed_tags.contains(ScopeTag::FunctionAllowlist) {
                         return Err(MandateError::PolicyViolation);
                    }
                }
            },
            DelegationPolicy::Full => {}
        }

        // D. Verificar Orçamento Cumulativo do Pai
        if let Some(child_limit) = request.scope.transfer_limit {
            if let Some(parent_limit) = parent.scope.transfer_limit {
                if parent_state.allocated_to_children + child_limit > parent_limit {
                    return Err(MandateError::PolicyViolation);
                }
                
                // Atualizar alocação do pai
                let mut new_parent_state = parent_state;
                new_parent_state.allocated_to_children += child_limit;
                storage::set_mandate_state(env, parent_id, &new_parent_state);
            }
        }

        depth = parent.depth + 1;
        parent.agent
    } else {
        request.root_anchor.clone()
    };

    if depth > 8 {
        return Err(MandateError::DepthExceeded);
    }

    // 4. Validação de Hierarquia e Escopo (Já feita acima)
    
    // 5. Checar SoulID (Sovereign Identity) do Root Anchor
    let soul_id_contract: Address = env.storage().persistent().get(&DataKey::SoulContract).ok_or(MandateError::SoulIDRequired)?;
    let has_soul: bool = env.invoke_contract(
        &soul_id_contract,
        &soroban_sdk::Symbol::new(env, "has_soul"),
        (request.root_anchor.clone(),).into_val(env),
    );
    if !has_soul {
        return Err(MandateError::SoulIDRequired);
    }

    // 6. Gravar Mandato
    let id = storage::increment_next_mandate_id(env);
    let mandate = Mandate {
        id,
        root_anchor: request.root_anchor,
        issuer,
        agent: request.agent.clone(),
        scope: request.scope,
        issued_at_epoch: stored_epoch,
        delegation_policy: request.delegation_policy,
        parent_mandate_id: request.parent_mandate_id,
        depth,
    };

    let state = MandateState {
        mandate_id: id,
        spent_budget: 0,
        current_period_start: env.ledger().timestamp(),
        allocated_to_children: 0,
        is_revoked: false,
    };

    storage::set_mandate(env, id, &mandate);
    storage::set_mandate_state(env, id, &state);
    env.storage().persistent().set(&DataKey::AgentMandate(request.agent.clone()), &id);

    env.events().publish((symbol_short!("issued"), id), request.agent);

    Ok(id)
}

pub fn issue_mandate_as_admin(
    env: &Env,
    root_anchor: Address,
    agent: Address,
    scope: Scope,
    delegation_policy: DelegationPolicy,
    parent_mandate_id: Option<u64>,
) -> Result<u64, MandateError> {
    // 1. Apenas o Admin pode chamar esta função
    let admin = storage::get_admin(env)?;
    admin.require_auth();

    // 2. Verificar se o root_anchor tem SoulID
    let soul_id_contract = env.storage().persistent().get(&DataKey::SoulContract).ok_or(MandateError::NotInitialized)?;
    let has_soul: bool = env.invoke_contract(
        &soul_id_contract,
        &soroban_sdk::Symbol::new(env, "has_soul"),
        (root_anchor.clone(),).into_val(env),
    );
    if !has_soul {
        return Err(MandateError::SoulIDRequired);
    }

    let id = storage::increment_next_mandate_id(env);
    let current_epoch = storage::get_global_epoch(env, &root_anchor);

    let mandate = Mandate {
        id,
        root_anchor: root_anchor.clone(),
        issuer: admin.clone(), // Admin é o emissor
        agent: agent.clone(),
        scope,
        issued_at_epoch: current_epoch,
        delegation_policy,
        parent_mandate_id,
        depth: 0,
    };

    let state = MandateState {
        mandate_id: id,
        spent_budget: 0,
        current_period_start: env.ledger().timestamp(),
        allocated_to_children: 0,
        is_revoked: false,
    };

    storage::set_mandate(env, id, &mandate);
    storage::set_mandate_state(env, id, &state);
    env.storage().persistent().set(&DataKey::AgentMandate(agent.clone()), &id);

    env.events().publish((symbol_short!("issued"), id), agent);

    Ok(id)
}

pub fn revoke_mandate(env: &Env, revoker: Address, mandate_id: u64) -> Result<(), MandateError> {
    revoker.require_auth();
    let mandate = storage::get_mandate(env, mandate_id).ok_or(MandateError::MandateNotFound)?;
    let mut state = storage::get_mandate_state(env, mandate_id).ok_or(MandateError::MandateNotFound)?;

    // Apenas o root_anchor ou o issuer imediato podem revogar
    if revoker != mandate.root_anchor && revoker != mandate.issuer {
        return Err(MandateError::NotAdmin);
    }

    state.is_revoked = true;
    storage::set_mandate_state(env, mandate_id, &state);

    env.events().publish((symbol_short!("revoked"), mandate_id), revoker);

    Ok(())
}

pub fn export_reputation(
    env: &Env,
    caller: Address,
    soul_id: u32,
    origin_contract: Address,
    external_id: soroban_sdk::String,
    tier: u32,
    nonce: u64,
    cross_chain: Option<CrossChainParams>,
) -> Result<(), MandateError> {
    caller.require_auth();

    // 1. Verificar se o contrato de origem é confiável (registrado)
    let token_exists = env.storage().persistent().has(&DataKey::TokenExists(origin_contract.clone()));
    if !token_exists {
        return Err(MandateError::PolicyViolation);
    }

    // 2. Se houver cross-chain params, despachar via adaptador
    if let Some(params) = cross_chain {
        let interop_config: InteropConfig = env.storage().persistent()
            .get(&DataKey::InteropConfig)
            .ok_or(MandateError::NotInitialized)?;

        // Chamar o adaptador (ex: Axelar)
        let _: soroban_sdk::Val = env.invoke_contract(
            &interop_config.adapter_address,
            &soroban_sdk::Symbol::new(env, "send_reputation"),
            (
                caller,
                params.destination_chain,
                params.destination_address,
                soul_id,
                external_id,
                tier,
                params.user_destination_address,
                nonce,
                soroban_sdk::Symbol::new(env, "github"), // Token type placeholder
                params.ecosystem,
            ).into_val(env),
        );
    }

    Ok(())
}

pub fn verify_authority(
    env: &Env,
    mandate_id: u64,
    agent: Address,
    contract: Address,
    function: Symbol,
    amount: Option<i128>,
    token: Option<Address>,
) -> bool {
    let mandate = match storage::get_mandate(env, mandate_id) {
        Some(m) => m,
        None => return false,
    };

    if mandate.agent != agent {
        return false;
    }

    if env.ledger().timestamp() > mandate.scope.expiration {
        return false;
    }

    // 1. Verificar Token
    if let (Some(scope_token), Some(provided_token)) = (mandate.scope.token.clone(), token) {
        if scope_token != provided_token {
            return false;
        }
    }

    let current_epoch = storage::get_global_epoch(env, &mandate.root_anchor);
    if mandate.issued_at_epoch != current_epoch {
        return false;
    }

    if !check_revocation_recursive(env, mandate_id) {
        return false;
    }

    let cache_key = DataKey::VerificationCacheKey(mandate_id, current_epoch);
    let is_cached: bool = env.storage().temporary().has(&cache_key);

    if !is_cached {
        if !traverse_and_verify(env, mandate_id, &contract, &function) {
            return false;
        }
        env.storage().temporary().set(&cache_key, &true);
    }

    if let Some(transfer_amount) = amount {
        if !check_and_update_budget(env, mandate_id, transfer_amount) {
            return false;
        }
    }

    true
}

/// Versão read-only (não altera storage)
pub fn check_authority(
    env: &Env,
    mandate_id: u64,
    agent: Address,
    contract: Address,
    function: Symbol,
    amount: Option<i128>,
    token: Option<Address>,
) -> bool {
    let mandate = match storage::get_mandate(env, mandate_id) {
        Some(m) => m,
        None => return false,
    };

    if mandate.agent != agent || env.ledger().timestamp() > mandate.scope.expiration {
        return false;
    }

    if let (Some(scope_token), Some(provided_token)) = (mandate.scope.token.clone(), token) {
        if scope_token != provided_token {
            return false;
        }
    }

    if mandate.issued_at_epoch != storage::get_global_epoch(env, &mandate.root_anchor) {
        return false;
    }

    if !check_revocation_recursive(env, mandate_id) {
        return false;
    }

    if !traverse_and_verify(env, mandate_id, &contract, &function) {
        return false;
    }

    if let Some(transfer_amount) = amount {
        let state = storage::get_mandate_state(env, mandate_id).unwrap();
        let mut current_spent = state.spent_budget;
        let now = env.ledger().timestamp();
        if let Some(period) = mandate.scope.renewal_period {
            if now >= state.current_period_start + period {
                current_spent = 0;
            }
        }
        if let Some(limit) = mandate.scope.transfer_limit {
            if current_spent + transfer_amount > limit {
                return false;
            }
        }
    }

    true
}

fn check_revocation_recursive(env: &Env, mandate_id: u64) -> bool {
    let state = storage::get_mandate_state(env, mandate_id).unwrap();
    if state.is_revoked {
        return false;
    }

    let mandate = storage::get_mandate(env, mandate_id).unwrap();
    if let Some(parent_id) = mandate.parent_mandate_id {
        return check_revocation_recursive(env, parent_id);
    }
    true
}

fn traverse_and_verify(env: &Env, mandate_id: u64, contract: &Address, function: &Symbol) -> bool {
    let mandate = storage::get_mandate(env, mandate_id).unwrap();

    if let Some(ref allowlist) = mandate.scope.contract_allowlist {
        if !allowlist.contains(contract) {
            return false;
        }
    }

    if let Some(ref func_list) = mandate.scope.function_allowlist {
        if !func_list.contains(function) {
            return false;
        }
    }

    if let Some(parent_id) = mandate.parent_mandate_id {
        return traverse_and_verify(env, parent_id, contract, function);
    }

    true
}

fn check_and_update_budget(env: &Env, mandate_id: u64, amount: i128) -> bool {
    let mut state = storage::get_mandate_state(env, mandate_id).unwrap();
    let mandate = storage::get_mandate(env, mandate_id).unwrap();

    if state.is_revoked {
        return false;
    }

    if let Some(limit) = mandate.scope.transfer_limit {
        let now = env.ledger().timestamp();
        if let Some(period) = mandate.scope.renewal_period {
            if now >= state.current_period_start + period {
                state.spent_budget = 0;
                state.current_period_start = now;
                env.events().publish((symbol_short!("budget"), symbol_short!("reset"), mandate_id), now);
            }
        }

        if state.spent_budget + amount > limit {
            return false;
        }

        state.spent_budget += amount;
        storage::set_mandate_state(env, mandate_id, &state);
        
        env.events().publish((symbol_short!("spend"), mandate_id), amount);
    }

    true
}
