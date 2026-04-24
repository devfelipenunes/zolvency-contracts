#![no_std]

mod storage;
mod types;
mod interface;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, token, Address, Bytes, BytesN, Env, String, Symbol, Vec, Val, IntoVal};

pub use types::{Error, GithubData, Tier, MintParams, CrossChainParams, InteropProtocol, InteropConfig, AxelarConfig};
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

    fn get_owner_passkey(env: Env, token_id: u64) -> Option<BytesN<65>> {
        storage::get_token_data(&env, token_id)
            .map(|d| d.passkey)
            .unwrap_or(None)
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
        params: MintParams,
        _referrer: Option<Address>,
        cross_chain: Option<CrossChainParams>,
    ) -> Result<u64, Error> {
        caller.require_auth();

        if params.username.len() == 0 || params.username.len() > 64 {
            return Err(Error::EmptyUsername);
        }

        if params.external_id.len() == 0 || params.external_id.len() > 64 {
            return Err(Error::EmptyUsername);
        }

        if storage::has_identity(&env, &caller) {
            return Err(Error::AlreadyHasIdentity);
        }

        let expected_nonce = storage::get_nonce(&env, &caller);
        if params.nonce != expected_nonce {
            return Err(Error::InvalidNonce);
        }

        let config = storage::get_config(&env)?;
        if config.mint_fee > 0 {
            let _signer_address: Address = env.invoke_contract(&config.registry, &Symbol::new(&env, "get_signer"), Vec::new(&env));
        }

        #[cfg(not(test))]
        {
            match (params.passkey.clone(), params.passkey_signature.clone()) {
                (Some(pk), Some(sig)) => {
                    let mut msg_bytes = [0u8; 64];
                    let ext_id = params.external_id.clone();
                    ext_id.copy_into_slice(&mut msg_bytes[..ext_id.len() as usize]);
                    let msg_hash = env.crypto().sha256(&Bytes::from_slice(&env, &msg_bytes[..ext_id.len() as usize]));
                    env.crypto().secp256r1_verify(&pk, &msg_hash, &sig);
                },
                (None, None) => {
                    // Pula validação se ambos forem None
                },
                _ => {
                    // Retorna erro se apenas um for fornecido
                    return Err(Error::InvalidSignature);
                }
            }
        }
        
        let _ = signature;

        if config.mint_fee > 0 {
            let token_client = token::Client::new(&env, &config.fee_token);
            token_client.transfer(&caller, &config.treasury, &config.mint_fee);
        }

        storage::increment_nonce(&env, &caller);

        let token_id = storage::get_next_token_id(&env);
        storage::increment_token_counter(&env);

        let tier = Tier::from_contributions(params.contributions);
        let github_data = GithubData {
            username: params.username.clone(),
            external_id: params.external_id.clone(),
            contributions: params.contributions,
            tier: tier.clone(),
            minted_at: env.ledger().timestamp(),
            updated_at: env.ledger().timestamp(),
            expires_at: env.ledger().timestamp() + (90 * 24 * 60 * 60),
            proof_data: params.proof_data,
            passkey: params.passkey,
        };

        storage::set_token_data(&env, token_id, &github_data);
        storage::set_holder_token(&env, &caller, token_id);
        storage::set_has_identity(&env, &caller, true);
        storage::set_sybil_mapping(&env, &params.external_id, token_id);

        // 🚀 Multi-Protocol Cross-chain Push
        if let Some(cc) = cross_chain {
            if cc.destination_chain.len() > 0 && cc.destination_address.len() > 0 {
                
                if let Ok(interop_config) = storage::get_interop_config(&env) {
                    if interop_config.active_protocol != InteropProtocol::None {
                        // Enviamos dados CRUS. O adaptador decide como codificar.
                        let _: Val = env.invoke_contract(
                            &interop_config.adapter_address,
                            &Symbol::new(&env, "send"),
                            (
                                caller.clone(), 
                                cc.destination_chain, 
                                cc.destination_address,
                                params.external_id.clone(),
                                tier.to_number() as u32,
                                cc.user_destination_address
                            ).into_val(&env)
                        );
                    }
                }
            }
        }

        env.events().publish(
            (Symbol::new(&env, "identity_minted"),),
            (caller, token_id, params.username, params.contributions, tier),
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
        cross_chain: Option<CrossChainParams>,
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

        // 🚀 Multi-Protocol Cross-chain Push
        if let Some(cc) = cross_chain {
            if cc.destination_chain.len() > 0 && cc.destination_address.len() > 0 {
                
                if let Ok(interop_config) = storage::get_interop_config(&env) {
                    if interop_config.active_protocol != InteropProtocol::None {
                        // Enviamos dados CRUS. O adaptador decide como codificar.
                        let _: Val = env.invoke_contract(
                            &interop_config.adapter_address,
                            &Symbol::new(&env, "send"),
                            (
                                caller.clone(), 
                                cc.destination_chain, 
                                cc.destination_address,
                                data.external_id.clone(),
                                tier.to_number() as u32,
                                cc.user_destination_address
                            ).into_val(&env)
                        );
                    }
                }
            }
        }

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

    pub fn set_axelar_config(
        env: Env,
        admin: Address,
        gateway: Address,
        gas_service: Address,
        gas_token: Address,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;

        let config = AxelarConfig {
            gateway,
            gas_service,
            gas_token,
        };
        storage::set_axelar_config(&env, &config);
        Ok(())
    }

    pub fn set_layerzero_config(
        env: Env,
        admin: Address,
        endpoint: Address,
    ) -> Result<(), Error> {
        admin.require_auth();
        Self::assert_admin(&env, &admin)?;

        let config = types::LayerZeroConfig {
            endpoint,
        };
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

    fn assert_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let stored_admin = storage::get_admin(env)?;
        if caller != &stored_admin {
            return Err(Error::NotAdmin);
        }
        Ok(())
    }

}
