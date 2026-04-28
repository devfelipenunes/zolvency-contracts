#![no_std]

mod interface;
mod messenger;
mod storage;
mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contractimpl, token, Address, Env, IntoVal, String, Symbol, Vec,
};

pub use interface::ZolvencyTokenTrait;
pub use messenger::MessengerClient;
pub use types::{
    CrossChainParams, Error, IncomePeriod, InteropConfig, InteropProtocol, MintParams, RenewalWindow,
    RevealMode, TokenMetadata, UberIncomeData, UpdateParams,
};

#[contract]
pub struct UberIncomeContract;

#[contractimpl]
impl ZolvencyTokenTrait for UberIncomeContract {
    fn get_token_type(env: Env) -> Symbol {
        Symbol::new(&env, "uber")
    }

    fn get_source(env: Env) -> String {
        String::from_str(&env, "zk-tls")
    }

    fn get_metadata(env: Env) -> TokenMetadata {
        TokenMetadata {
            name: String::from_str(&env, "Zolvency Uber Income"),
            symbol: String::from_str(&env, "ZOLV-UBER"),
            version: String::from_str(&env, "1.0.0"),
            data_source: String::from_str(&env, "zk-tls / uber"),
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

    fn get_owner_passkey(_env: Env, _token_id: u64) -> Option<soroban_sdk::BytesN<65>> {
        None
    }
}

#[contractimpl]
impl UberIncomeContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        registry: Address,
        fee_token: Address,
        access_control: Address,
        treasury: Address,
        mint_fee_30: i128,
        mint_fee_60: i128,
        mint_fee_90: i128,
        max_proof_age_seconds: u64,
        store_proof_data: bool,
    ) -> Result<(), Error> {
        if storage::get_config(&env).is_ok() {
            return Err(Error::AlreadyInitialized);
        }

        let config = types::Config {
            admin,
            registry,
            fee_token,
            access_control,
            treasury,
            mint_fee_30,
            mint_fee_60,
            mint_fee_90,
            max_proof_age_seconds,
            zk_verifier: None,
            store_proof_data,
        };

        storage::set_config(&env, &config);

        Ok(())
    }

    pub fn set_zk_verifier(env: Env, admin: Address, verifier: Option<Address>) -> Result<(), Error> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;
        let mut config = storage::get_config(&env)?;
        config.zk_verifier = verifier;
        storage::set_config(&env, &config);
        Ok(())
    }

    pub fn set_fees(
        env: Env,
        admin: Address,
        fee_30: i128,
        fee_60: i128,
        fee_90: i128,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;
        let mut config = storage::get_config(&env)?;
        config.mint_fee_30 = fee_30;
        config.mint_fee_60 = fee_60;
        config.mint_fee_90 = fee_90;
        storage::set_config(&env, &config);
        Ok(())
    }

    pub fn set_max_proof_age(
        env: Env,
        admin: Address,
        max_proof_age_seconds: u64,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;
        let mut config = storage::get_config(&env)?;
        config.max_proof_age_seconds = max_proof_age_seconds;
        storage::set_config(&env, &config);
        Ok(())
    }

    pub fn set_access_control(
        env: Env,
        admin: Address,
        access_control: Address,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;
        let mut config = storage::get_config(&env)?;
        config.access_control = access_control;
        storage::set_config(&env, &config);
        Ok(())
    }

    pub fn set_treasury(env: Env, admin: Address, treasury: Address) -> Result<(), Error> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;
        let mut config = storage::get_config(&env)?;
        config.treasury = treasury;
        storage::set_config(&env, &config);
        Ok(())
    }

    pub fn set_axelar_config(
        env: Env,
        admin: Address,
        gateway: Address,
        gas_service: Address,
        gas_token: Address,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;
        let config = types::AxelarConfig {
            gateway,
            gas_service,
            gas_token,
        };
        storage::set_axelar_config(&env, &config);
        Ok(())
    }

    pub fn set_layerzero_config(env: Env, admin: Address, endpoint: Address) -> Result<(), Error> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;
        let config = types::LayerZeroConfig { endpoint };
        storage::set_layerzero_config(&env, &config);
        Ok(())
    }

    pub fn set_active_protocol(
        env: Env,
        admin: Address,
        protocol: InteropProtocol,
        adapter: Address,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;
        let config = InteropConfig {
            active_protocol: protocol,
            adapter_address: adapter,
        };
        storage::set_interop_config(&env, &config);
        Ok(())
    }

    pub fn mint(
        env: Env,
        admin: Address,
        params: MintParams,
        cross_chain: Option<CrossChainParams>,
    ) -> Result<u64, Error> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;

        if params.external_id.is_empty() || params.external_id.len() > 64 {
            return Err(Error::InvalidExternalId);
        }

        if params.currency.is_empty() || params.currency.len() > 16 {
            return Err(Error::InvalidCurrency);
        }

        if storage::has_identity(&env, &params.recipient) {
            return Err(Error::AlreadyHasIdentity);
        }

        types::validate_income_fields(
            params.income_band,
            &params.income_value,
            &params.reveal_mode,
        )?;

        let expected_nonce = storage::get_nonce(&env, &params.recipient);
        if params.nonce != expected_nonce {
            return Err(Error::InvalidNonce);
        }

        let config = storage::get_config(&env)?;
        let now = env.ledger().timestamp();
        types::validate_proof_freshness(now, params.verified_at, config.max_proof_age_seconds)?;
        let fee = types::fee_for_window(&config, &params.window);
        if fee > 0 {
            let token_client = token::Client::new(&env, &config.fee_token);
            token_client.transfer(&admin, &config.treasury, &fee);
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

        storage::increment_nonce(&env, &params.recipient);

        let token_id = storage::get_next_token_id(&env);
        storage::increment_token_counter(&env);

        let expires_at = now + params.window.to_seconds();
        let proof_data = types::normalize_proof_data(&env, config.store_proof_data, params.proof_data);

        let data = UberIncomeData {
            recipient: params.recipient.clone(),
            external_id: params.external_id.clone(),
            income_band: params.income_band,
            income_value: params.income_value,
            reveal_mode: params.reveal_mode.clone(),
            currency: params.currency.clone(),
            period: params.period.clone(),
            verified_at: params.verified_at,
            proof_hash: params.proof_hash,
            proof_data,
            window: params.window.clone(),
            minted_at: now,
            updated_at: now,
            expires_at,
        };

        storage::set_token_data(&env, token_id, &data);
        storage::set_holder_token(&env, &params.recipient, token_id);
        storage::set_has_identity(&env, &params.recipient, true);
        storage::set_sybil_mapping(&env, &params.external_id, token_id);

        if let Some(cc) = cross_chain {
            if !cc.destination_chain.is_empty() && !cc.destination_address.is_empty() {
                if let Ok(interop_config) = storage::get_interop_config(&env) {
                    if interop_config.active_protocol != InteropProtocol::None {
                        let messenger = MessengerClient::new(&env, &interop_config.adapter_address);
                        messenger.send(
                            &admin,
                            &cc.destination_chain,
                            &cc.destination_address,
                            &params.external_id,
                            &params.income_band,
                            &cc.user_destination_address,
                            &params.nonce,
                        );
                    }
                }
            }
        }

        env.events().publish(
            (Symbol::new(&env, "uber_income_minted"),),
            (params.recipient, token_id, params.income_band),
        );

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
        
        let expected_nonce = storage::get_nonce(&env, &data.recipient);
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
        let proof_data = types::normalize_proof_data(&env, config.store_proof_data, params.proof_data);

        data.income_band = params.income_band;
        data.income_value = params.income_value;
        data.reveal_mode = params.reveal_mode;
        data.currency = params.currency;
        data.period = params.period;
        data.verified_at = params.verified_at;
        data.proof_hash = params.proof_hash;
        data.proof_data = proof_data;
        data.window = params.window;
        data.updated_at = now;
        data.expires_at = expires_at;

        storage::update_token_data(&env, token_id, &data)?;
        storage::increment_nonce(&env, &data.recipient);

        if let Some(cc) = cross_chain {
            if !cc.destination_chain.is_empty() && !cc.destination_address.is_empty() {
                if let Ok(interop_config) = storage::get_interop_config(&env) {
                    if interop_config.active_protocol != InteropProtocol::None {
                        let messenger = MessengerClient::new(&env, &interop_config.adapter_address);
                        messenger.send(
                            &admin,
                            &cc.destination_chain,
                            &cc.destination_address,
                            &data.external_id,
                            &data.income_band,
                            &cc.user_destination_address,
                            &nonce,
                        );
                    }
                }
            }
        }

        env.events().publish(
            (Symbol::new(&env, "uber_income_updated"),),
            (data.recipient, token_id, data.income_band),
        );

        Ok(())
    }

    pub fn get_token_data(env: Env, token_id: u64) -> Result<UberIncomeData, Error> {
        storage::get_token_data(&env, token_id)
    }

    pub fn get_user_token(env: Env, user: Address) -> Result<u64, Error> {
        storage::get_holder_token(&env, &user)
    }

    pub fn has_identity(env: Env, user: Address) -> bool {
        storage::has_identity(&env, &user)
    }

    pub fn list_tokens_of_user(env: Env, user: Address) -> Vec<u64> {
        match storage::get_holder_token(&env, &user) {
            Ok(token_id) => Vec::from_array(&env, [token_id]),
            Err(_) => Vec::new(&env),
        }
    }

    pub fn get_nonce(env: Env, user: Address) -> u64 {
        storage::get_nonce(&env, &user)
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
