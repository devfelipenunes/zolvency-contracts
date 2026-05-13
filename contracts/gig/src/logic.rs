use soroban_sdk::{token, Address, Env, IntoVal, Symbol, Vec};
use crate::storage;
use crate::{Config, Error, UberIncomeData, MintParams, UpdateParams, CrossChainParams, RenewalWindow, RevealMode};

pub fn mint(
    env: &Env,
    caller: Address,
    params: MintParams,
    cross_chain: Option<CrossChainParams>,
) -> Result<u64, Error> {
    caller.require_auth();

    let config = storage::get_config(env)?;

    // 1. Verificar SoulID
    let res = env.try_invoke_contract::<Option<bool>, soroban_sdk::Error>(
        &config.soul_contract,
        &Symbol::new(env, "get_soul"),
        soroban_sdk::vec![env, params.soul_id.into_val(env)],
    );

    match res {
        Ok(Ok(Some(true))) => {}
        _ => return Err(Error::Unauthorized),
    }

    if storage::has_identity(env, params.soul_id) {
        return Err(Error::AlreadyHasIdentity);
    }

    // 2. Validar Campos
    validate_income_fields(
        params.income_band,
        &params.income_value,
        &params.reveal_mode,
    )?;

    // 3. Verificar Nonce
    let expected_nonce = storage::get_nonce(env, params.soul_id);
    if params.nonce != expected_nonce {
        return Err(Error::InvalidNonce);
    }

    // 4. Validar Freshness da Prova
    let now = env.ledger().timestamp();
    validate_proof_freshness(now, params.verified_at, config.max_proof_age_seconds)?;

    // 5. Cobrar Taxa
    let fee = fee_for_window(&config, &params.window);
    if fee > 0 {
        let token_client = token::Client::new(env, &config.fee_token);
        token_client.transfer(&caller, &config.treasury, &fee);
    }

    // 6. Verificar Prova ZK (se houver verifier configurado)
    if let Some(verifier) = config.zk_verifier {
        let is_valid: bool = env.invoke_contract(
            &verifier,
            &Symbol::new(env, "verify_proof"),
            Vec::from_array(env, [params.proof_data.into_val(env)]),
        );
        if !is_valid {
            return Err(Error::Unauthorized);
        }
    }

    storage::increment_nonce(env, params.soul_id);

    let token_id = storage::get_next_token_id(env);
    storage::increment_token_counter(env);

    let expires_at = now + params.window.to_seconds();

    let data = UberIncomeData {
        soul_id: params.soul_id,
        external_id: params.external_id.clone(),
        income_band: params.income_band,
        income_value: params.income_value,
        reveal_mode: params.reveal_mode,
        currency: params.currency,
        period: params.period,
        verified_at: params.verified_at,
        proof_hash: params.proof_hash,
        window: params.window,
        minted_at: now,
        updated_at: now,
        expires_at,
    };

    storage::set_token_data(env, token_id, &data);
    storage::set_holder_token(env, params.soul_id, token_id);
    storage::set_has_identity(env, params.soul_id, true);
    storage::set_sybil_mapping(env, &params.external_id, token_id);

    // 7. Exportar Reputação
    let _ = env.try_invoke_contract::<(), soroban_sdk::Error>(
        &config.registry,
        &Symbol::new(env, "export_reputation"),
        (
            caller,
            params.soul_id,
            env.current_contract_address(),
            params.external_id,
            params.income_band,
            params.nonce,
            cross_chain,
        )
            .into_val(env),
    );

    // 8. Evento
    env.events().publish(
        (Symbol::new(env, "GigMinted"), params.soul_id),
        (token_id, params.income_band)
    );

    Ok(token_id)
}

pub fn update_token(
    env: &Env,
    admin: Address,
    token_id: u64,
    params: UpdateParams,
    nonce: u64,
    cross_chain: Option<CrossChainParams>,
) -> Result<(), Error> {
    admin.require_auth();
    let stored_admin = storage::get_admin(env)?;
    if admin != stored_admin {
        return Err(Error::NotAdmin);
    }

    validate_income_fields(
        params.income_band,
        &params.income_value,
        &params.reveal_mode,
    )?;

    let mut data = storage::get_token_data(env, token_id)?;
    
    let expected_nonce = storage::get_nonce(env, data.soul_id);
    if nonce != expected_nonce {
        return Err(Error::InvalidNonce);
    }

    let now = env.ledger().timestamp();
    if now >= data.expires_at {
        return Err(Error::TokenExpired);
    }

    let config = storage::get_config(env)?;
    validate_proof_freshness(now, params.verified_at, config.max_proof_age_seconds)?;
    
    let expires_at = now + params.window.to_seconds();

    data.income_band = params.income_band;
    data.income_value = params.income_value;
    data.reveal_mode = params.reveal_mode;
    data.currency = params.currency;
    data.period = params.period;
    data.verified_at = params.verified_at;
    data.proof_hash = params.proof_hash;
    data.window = params.window;
    data.updated_at = now;
    data.expires_at = expires_at;

    storage::update_token_data(env, token_id, &data)?;
    storage::increment_nonce(env, data.soul_id);

    // Exportar Reputação
    let _ = env.try_invoke_contract::<(), soroban_sdk::Error>(
        &config.registry,
        &Symbol::new(env, "export_reputation"),
        (
            admin,
            data.soul_id,
            env.current_contract_address(),
            data.external_id.clone(),
            data.income_band,
            nonce,
            cross_chain,
        )
            .into_val(env),
    );

    env.events().publish(
        (Symbol::new(env, "GigUpdated"), data.soul_id),
        (token_id, data.income_band)
    );

    Ok(())
}

// --- HELPERS ---

pub fn fee_for_window(config: &Config, window: &RenewalWindow) -> i128 {
    match window {
        RenewalWindow::Days30 => config.mint_fee_30,
        RenewalWindow::Days60 => config.mint_fee_60,
        RenewalWindow::Days90 => config.mint_fee_90,
    }
}

pub fn validate_income_fields(
    income_band: u32,
    income_value: &Option<i128>,
    reveal_mode: &RevealMode,
) -> Result<(), Error> {
    if income_band == 0 {
        return Err(Error::InvalidIncomeBand);
    }

    match reveal_mode {
        RevealMode::Exact => {
            if income_value.is_none() {
                return Err(Error::InvalidIncomeValue);
            }
        }
        RevealMode::Band => {
            if income_value.is_some() {
                return Err(Error::InvalidIncomeValue);
            }
        }
    }

    Ok(())
}

pub fn validate_proof_freshness(
    now: u64,
    verified_at: u64,
    max_age_seconds: u64,
) -> Result<(), Error> {
    if verified_at == 0 {
        return Err(Error::InvalidProofFreshness);
    }
    if max_age_seconds == 0 {
        return Ok(());
    }
    if now.saturating_sub(verified_at) > max_age_seconds {
        return Err(Error::InvalidProofFreshness);
    }
    Ok(())
}
