#![no_std]

mod interface;
mod storage;
mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contractevent, contractimpl, token, Address, Env, IntoVal, String, Symbol, Vec,
};

pub use interface::ZolvencyTokenTrait;
pub use types::{
    CrossChainParams, Error, IncomePeriod, MintParams, RenewalWindow,
    RevealMode, TokenMetadata, UberIncomeData, UpdateParams,
    InitializeParams, Ecosystem,
};

#[contract]
pub struct UberIncomeContract;

#[contractevent]
pub enum GigEvent {
    Minted { soul_id: u32, token_id: u64, income_band: u32 },
    Updated { soul_id: u32, token_id: u64, income_band: u32 },
}

#[contractimpl]
impl ZolvencyTokenTrait for UberIncomeContract {
    fn get_token_type(env: Env) -> Symbol {
        Symbol::new(&env, "gig")
    }

    fn get_source(env: Env) -> String {
        String::from_str(&env, "zk-tls")
    }

    fn get_metadata(env: Env) -> TokenMetadata {
        TokenMetadata {
            name: String::from_str(&env, "Zolvency Gig"),
            symbol: String::from_str(&env, "ZOLV-GIG"),
            version: String::from_str(&env, "1.0.0"),
            data_source: String::from_str(&env, "zk-tls / gig"),
        }
    }

    fn is_valid(env: Env, token_id: u64) -> bool {
        if let Ok(data) = storage::get_token_data(&env, token_id) {
            env.ledger().timestamp() < data.expires_at
        } else {
            false
        }
    }

    fn get_expiry(env: Env, token_id: u64) -> u64 {
        storage::get_token_data(&env, token_id)
            .map(|d| d.expires_at)
            .unwrap_or(0)
    }

    fn get_owner_soul(env: Env, token_id: u64) -> u32 {
        storage::get_token_data(&env, token_id).unwrap().soul_id
    }
}

#[contractimpl]
impl UberIncomeContract {
    pub fn initialize(
        env: Env,
        params: types::InitializeParams,
    ) -> Result<(), Error> {
        if storage::get_config(&env).is_ok() {
            return Err(Error::AlreadyInitialized);
        }

        let config = types::Config {
            admin: params.admin,
            registry: params.registry,
            soul_contract: params.soul_contract,
            fee_token: params.fee_token,
            access_control: params.access_control,
            treasury: params.treasury,
            mint_fee_30: params.mint_fee_30,
            mint_fee_60: params.mint_fee_60,
            mint_fee_90: params.mint_fee_90,
            max_proof_age_seconds: params.max_proof_age_seconds,
            zk_verifier: None,
        };

        storage::set_config(&env, &config);
        Ok(())
    }

    pub fn mint(
        env: Env,
        caller: Address,
        params: MintParams,
        cross_chain: Option<CrossChainParams>,
    ) -> Result<u64, Error> {
        caller.require_auth();

        let config = storage::get_config(&env)?;
        let res = env.try_invoke_contract::<Option<soroban_sdk::Val>, soroban_sdk::Error>(
            &config.soul_contract,
            &Symbol::new(&env, "get_soul"),
            soroban_sdk::vec![&env, params.soul_id.into_val(&env)],
        );

        match res {
            Ok(Ok(Some(_))) => {}
            _ => return Err(Error::Unauthorized),
        }

        if params.external_id.is_empty() || params.external_id.len() > 64 {
            return Err(Error::InvalidExternalId);
        }

        if params.currency.is_empty() || params.currency.len() > 16 {
            return Err(Error::InvalidCurrency);
        }

        if storage::has_identity(&env, params.soul_id) {
            return Err(Error::AlreadyHasIdentity);
        }

        types::validate_income_fields(
            params.income_band,
            &params.income_value,
            &params.reveal_mode,
        )?;

        let expected_nonce = storage::get_nonce(&env, params.soul_id);
        if params.nonce != expected_nonce {
            return Err(Error::InvalidNonce);
        }

        let now = env.ledger().timestamp();
        types::validate_proof_freshness(now, params.verified_at, config.max_proof_age_seconds)?;
        
        let fee = types::fee_for_window(&config, &params.window);
        if fee > 0 {
            let token_client = token::Client::new(&env, &config.fee_token);
            token_client.transfer(&caller, &config.treasury, &fee);
        }

        if let Some(verifier) = config.zk_verifier {
            let is_valid: bool = env.invoke_contract(
                &verifier,
                &Symbol::new(&env, "verify_proof"),
                Vec::from_array(&env, [params.proof_data.clone().into_val(&env)]),
            );
            if !is_valid {
                return Err(Error::Unauthorized);
            }
        }

        let token_id = storage::get_next_token_id(&env);
        storage::increment_token_counter(&env);

        let expires_at = now + params.window.to_seconds();

        let data = UberIncomeData {
            soul_id: params.soul_id,
            external_id: params.external_id.clone(),
            income_band: params.income_band,
            income_value: params.income_value,
            reveal_mode: params.reveal_mode.clone(),
            currency: params.currency.clone(),
            period: params.period.clone(),
            verified_at: params.verified_at,
            proof_hash: params.proof_hash,
            window: params.window.clone(),
            minted_at: now,
            updated_at: now,
            expires_at,
        };

        storage::set_token_data(&env, token_id, &data);
        storage::set_holder_token(&env, params.soul_id, token_id);
        storage::set_has_identity(&env, params.soul_id, true);
        storage::set_sybil_mapping(&env, &params.external_id, token_id);

        let _ = env.try_invoke_contract::<(), soroban_sdk::Error>(
            &config.registry,
            &Symbol::new(&env, "export_reputation"),
            (
                caller,
                params.soul_id,
                env.current_contract_address(),
                params.external_id,
                params.income_band,
                params.nonce,
                cross_chain,
            )
                .into_val(&env),
        );

        storage::increment_nonce(&env, params.soul_id);

        GigEvent::Minted {
            soul_id: params.soul_id,
            token_id,
            income_band: params.income_band,
        }
        .publish(&env);

        Ok(token_id)
    }

    pub fn update_token(
        env: Env,
        admin: Address,
        token_id: u64,
        params: UpdateParams,
        nonce: u64,
        cross_chain: Option<CrossChainParams>,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;

        types::validate_income_fields(
            params.income_band,
            &params.income_value,
            &params.reveal_mode,
        )?;

        let mut data = storage::get_token_data(&env, token_id)?;
        
        let expected_nonce = storage::get_nonce(&env, data.soul_id);
        if nonce != expected_nonce {
            return Err(Error::InvalidNonce);
        }

        let now = env.ledger().timestamp();
        if now >= data.expires_at {
            return Err(Error::TokenExpired);
        }

        let config = storage::get_config(&env)?;
        types::validate_proof_freshness(now, params.verified_at, config.max_proof_age_seconds)?;
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

        storage::update_token_data(&env, token_id, &data)?;

        let _ = env.try_invoke_contract::<(), soroban_sdk::Error>(
            &config.registry,
            &Symbol::new(&env, "export_reputation"),
            (
                admin,
                data.soul_id,
                env.current_contract_address(),
                data.external_id,
                data.income_band,
                nonce,
                cross_chain,
            )
                .into_val(&env),
        );

        storage::increment_nonce(&env, data.soul_id);

        GigEvent::Updated {
            soul_id: data.soul_id,
            token_id,
            income_band: data.income_band,
        }
        .publish(&env);

        Ok(())
    }

    pub fn get_token_data(env: Env, token_id: u64) -> Result<UberIncomeData, Error> {
        storage::get_token_data(&env, token_id)
    }

    pub fn get_user_token(env: Env, soul_id: u32) -> u64 {
        storage::get_holder_token(&env, soul_id).unwrap()
    }

    pub fn has_identity(env: Env, soul_id: u32) -> bool {
        storage::has_identity(&env, soul_id)
    }

    pub fn list_tokens_of_user(env: Env, soul_id: u32) -> Vec<u64> {
        match storage::get_holder_token(&env, soul_id) {
            Ok(token_id) => Vec::from_array(&env, [token_id]),
            Err(_) => Vec::new(&env),
        }
    }

    pub fn get_nonce(env: Env, soul_id: u32) -> u64 {
        storage::get_nonce(&env, soul_id)
    }

    pub fn get_mint_fee(env: Env, window: RenewalWindow) -> i128 {
        storage::get_config(&env)
            .map(|c| types::fee_for_window(&c, &window))
            .unwrap_or(0)
    }

    fn assert_admin(env: &Env, admin: &Address) -> Result<(), Error> {
        let stored_admin: Address = storage::get_admin(env)?;
        if stored_admin != *admin {
            return Err(Error::NotAdmin);
        }
        Ok(())
    }
}
