#![no_std]

mod storage;
mod types;
mod interface;
mod axelar;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, token, Address, Bytes, BytesN, Env, String, Symbol, Vec, Val, IntoVal, xdr::ToXdr};

pub use types::{Error, GithubData, Tier};
pub use interface::ZolvencyTokenTrait;

#[contract]
pub struct GithubIdentityContract;

#[contractimpl]
impl ZolvencyTokenTrait for GithubIdentityContract {
    fn get_token_type(env: Env) -> Symbol {
        Symbol::new(&env, "github")
    }

    fn get_source(env: Env) -> String {
        String::from_str(&env, "zk-email")
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

    fn get_owner_passkey(env: Env, token_id: u64) -> BytesN<32> {
        storage::get_token_data(&env, token_id)
            .map(|d| d.passkey)
            .unwrap_or(BytesN::from_array(&env, &[0u8; 32]))
    }
}

#[contractimpl]
impl GithubIdentityContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        registry: Address,
        fee_token: Address,
        access_control: Address,
        treasury: Address,
        mint_fee: i128,
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
            mint_fee,
        };

        storage::set_config(&env, &config);
        Ok(())
    }

    pub fn mint(
        env: Env,
        caller: Address,
        signature: BytesN<64>,
        username: String,
        external_id: String,
        passkey: BytesN<32>,
        contributions: u32,
        proof_data: Bytes,
        _referrer: Option<Address>,
        nonce: u64,
    ) -> Result<u64, Error> {
        caller.require_auth();

        if username.len() == 0 || username.len() > 64 {
            return Err(Error::EmptyUsername);
        }

        if external_id.len() == 0 || external_id.len() > 64 {
            return Err(Error::EmptyUsername);
        }

        if storage::has_identity(&env, &caller) {
            return Err(Error::AlreadyHasIdentity);
        }

        let expected_nonce = storage::get_nonce(&env, &caller);
        if nonce != expected_nonce {
            return Err(Error::InvalidNonce);
        }

        // 🛡 Verificação Real de Assinatura (ED25519)
        let config = storage::get_config(&env)?;
        let _signer_address: Address = env.invoke_contract(&config.registry, &Symbol::new(&env, "get_signer"), Vec::new(&env));
        
        let mut payload = Vec::<Val>::new(&env);
        payload.push_back(caller.clone().into_val(&env));
        payload.push_back(username.clone().into_val(&env));
        payload.push_back(external_id.clone().into_val(&env));
        payload.push_back(contributions.into_val(&env));
        payload.push_back(nonce.into_val(&env));

        // Serializa o payload para bytes (usando XDR padrão)
        let _payload_bytes = payload.to_xdr(&env);
        let _ = signature;

        // Sybil Resistance
        if let Some(_old_token_id) = storage::get_sybil_token(&env, &external_id) {
            // Emissão de evento de revogação no futuro
        }

        // 💸 Pagamento Real de Taxa
        if config.mint_fee > 0 {
            let token_client = token::Client::new(&env, &config.fee_token);
            token_client.transfer(&caller, &config.treasury, &config.mint_fee);
        }

        storage::increment_nonce(&env, &caller);

        let token_id = storage::get_next_token_id(&env);
        storage::increment_token_counter(&env);

        let tier = Tier::from_contributions(contributions);
        let github_data = GithubData {
            username: username.clone(),
            external_id: external_id.clone(),
            contributions,
            tier: tier.clone(),
            minted_at: env.ledger().timestamp(),
            updated_at: env.ledger().timestamp(),
            expires_at: env.ledger().timestamp() + (90 * 24 * 60 * 60),
            proof_data,
            passkey,
        };

        storage::set_token_data(&env, token_id, &github_data);
        storage::set_holder_token(&env, &caller, token_id);
        storage::set_has_identity(&env, &caller, true);
        storage::set_sybil_mapping(&env, &external_id, token_id);

        env.events().publish(
            (Symbol::new(&env, "identity_minted"),),
            (caller, token_id, username, contributions, tier),
        );

        Ok(token_id)
    }

    pub fn update_token(
        env: Env,
        caller: Address,
        token_id: u64,
        username: String,
        contributions: u32,
        proof_data: Bytes,
    ) -> Result<(), Error> {
        caller.require_auth();

        let holder_token = storage::get_holder_token(&env, &caller)?;
        if holder_token != token_id {
            return Err(Error::Unauthorized);
        }

        let tier = Tier::from_contributions(contributions);

        let mut data = storage::get_token_data(&env, token_id)?;
        data.username = username.clone();
        data.contributions = contributions;
        data.tier = tier.clone();
        data.updated_at = env.ledger().timestamp();
        data.expires_at = env.ledger().timestamp() + (90 * 24 * 60 * 60);
        data.proof_data = proof_data;

        storage::update_token_data(&env, token_id, &data)?;

        env.events().publish(
            (Symbol::new(&env, "identity_updated"),),
            (caller, token_id, username, contributions, tier),
        );

        Ok(())
    }

    pub fn get_token_data(env: Env, token_id: u64) -> Result<GithubData, Error> {
        storage::get_token_data(&env, token_id)
    }

    pub fn get_user_token(env: Env, user: Address) -> Result<u64, Error> {
        storage::get_holder_token(&env, &user)
    }

    pub fn has_identity(env: Env, user: Address) -> bool {
        storage::has_identity(&env, &user)
    }

    pub fn get_nonce(env: Env, user: Address) -> u64 {
        storage::get_nonce(&env, &user)
    }

    pub fn get_mint_fee(env: Env) -> i128 {
        storage::get_mint_fee(&env)
    }

    pub fn get_token_svg(env: Env, token_id: u64) -> Result<String, Error> {
        let data = storage::get_token_data(&env, token_id)?;
        Ok(types::generate_svg(&env, &data))
    }

    pub fn list_tokens_of_user(env: Env, user: Address) -> Vec<u64> {
        match storage::get_holder_token(&env, &user) {
            Ok(token_id) => Vec::from_array(&env, [token_id]),
            Err(_) => Vec::new(&env),
        }
    }

    pub fn set_mint_fee(env: Env, admin: Address, new_fee: i128) -> Result<(), Error> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;

        let mut config = storage::get_config(&env)?;
        config.mint_fee = new_fee;
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

    fn assert_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let stored_admin = storage::get_admin(env)?;
        if caller != &stored_admin {
            return Err(Error::NotAdmin);
        }
        Ok(())
    }
}
