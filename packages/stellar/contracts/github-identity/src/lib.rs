#![no_std]

mod interface;
mod messenger;
mod storage;
mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contractimpl, Address, Bytes, BytesN, Env, String, Symbol,
};

pub use interface::ZolvencyTokenTrait;
pub use messenger::MessengerClient;
pub use types::{
    AxelarConfig, CrossChainParams, Error, GithubData, InteropConfig, InteropProtocol, MintParams,
    Tier, TokenMetadata,
};

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

    fn get_metadata(env: Env) -> TokenMetadata {
        TokenMetadata {
            name: String::from_str(&env, "Zolvency GitHub Identity"),
            symbol: String::from_str(&env, "ZOLV-GH"),
            version: String::from_str(&env, "1.1.0"),
            data_source: String::from_str(&env, "zk-email / github-api"),
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

    fn get_owner_passkey(env: Env, token_id: u64) -> Bytes {
        storage::get_token_data(&env, token_id).unwrap().passkey
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
            zk_verifier: None,
        };

        storage::set_config(&env, &config);

        Ok(())
    }

    pub fn mint(
        env: Env,
        caller: Address,
        user: Address,
        params: MintParams,
    ) -> u64 {
        caller.require_auth();

        let token_id = storage::get_next_token_id(&env);
        storage::increment_token_counter(&env);

        let tier = Tier::from_contributions(params.contributions);
        let github_data = GithubData {
            contributions: params.contributions,
            expires_at: env.ledger().timestamp() + (90 * 24 * 60 * 60),
            external_id: params.external_id.clone(),
            minted_at: env.ledger().timestamp(),
            passkey: params.passkey.clone(),
            proof_data: params.proof_data.clone(),
            tier: tier.clone(),
            updated_at: env.ledger().timestamp(),
            username: params.username.clone(),
        };

        storage::set_token_data(&env, token_id, &github_data);
        storage::set_holder_token(&env, &user, token_id);
        storage::set_has_identity(&env, &user, true);
        storage::set_sybil_mapping(&env, &params.external_id, token_id);

        token_id
    }

    pub fn upgrade(env: Env, admin: Address, new_wasm_hash: BytesN<32>) -> Result<(), Error> {
        admin.require_auth();
        env.deployer().update_current_contract_wasm(new_wasm_hash);
        Ok(())
    }
}
