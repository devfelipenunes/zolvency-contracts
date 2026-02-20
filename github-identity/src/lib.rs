#![no_std]

mod storage;
mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, String, Symbol, Vec};

pub use types::{Error, GithubData, Tier};

#[contract]
pub struct GithubIdentityContract;

#[contractimpl]
impl GithubIdentityContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        access_control: Address,
        treasury: Address,
        mint_fee: i128,
    ) -> Result<(), Error> {
        if storage::get_config(&env).is_ok() {
            return Err(Error::AlreadyInitialized);
        }

        let config = types::Config {
            admin,
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
        _signature: BytesN<64>,
        username: String,
        contributions: u32,
        proof_data: Bytes,
        _referrer: Option<Address>,
        nonce: u64,
    ) -> Result<u64, Error> {
        caller.require_auth();

        if username.len() == 0 {
            return Err(Error::EmptyUsername);
        }

        if storage::has_identity(&env, &caller) {
            return Err(Error::AlreadyHasIdentity);
        }

        let expected_nonce = storage::get_nonce(&env, &caller);
        if nonce != expected_nonce {
            return Err(Error::InvalidNonce);
        }

        let _ = _signature;

        let mint_fee = storage::get_mint_fee(&env);
        if mint_fee > 0 {
            return Err(Error::InsufficientPayment);
        }

        storage::increment_nonce(&env, &caller);

        let token_id = storage::get_next_token_id(&env);
        storage::increment_token_counter(&env);

        let tier = Tier::from_contributions(contributions);
        let github_data = GithubData {
            username: username.clone(),
            contributions,
            tier: tier.clone(),
            minted_at: env.ledger().timestamp(),
            updated_at: env.ledger().timestamp(),
            proof_data,
        };

        storage::set_token_data(&env, token_id, &github_data);
        storage::set_holder_token(&env, &caller, token_id);
        storage::set_has_identity(&env, &caller, true);

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