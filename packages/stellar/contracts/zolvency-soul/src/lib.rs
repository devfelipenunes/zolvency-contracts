#![no_std]

mod types;
mod storage;

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contractimpl, symbol_short, Address, Env, BytesN, Bytes,
};
use crate::types::{Error, SoulData, DataKey};
use crate::storage::{
    get_admin, get_relayer, get_total_souls, increment_total_souls, 
    set_soul, get_soul_by_id, get_soul_id_by_passkey, extend_instance,
    remove_passkey_mapping
};

#[contract]
pub struct ZolvencySoulContract;

#[contractimpl]
impl ZolvencySoulContract {
    pub fn initialize(env: Env, admin: Address, relayer: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Relayer, &relayer);
        env.storage().instance().set(&DataKey::TotalSouls, &0u32);
        extend_instance(&env);
        Ok(())
    }

    pub fn admin(env: Env) -> Result<Address, Error> {
        extend_instance(&env);
        get_admin(&env)
    }

    pub fn relayer(env: Env) -> Result<Address, Error> {
        extend_instance(&env);
        get_relayer(&env)
    }

    pub fn total_souls(env: Env) -> Result<u32, Error> {
        extend_instance(&env);
        Ok(get_total_souls(&env))
    }

    pub fn mint(
        env: Env,
        relayer: Address,
        passkey: BytesN<65>,
        recovery_pubkey: BytesN<65>,
    ) -> Result<u32, Error> {
        relayer.require_auth();
        extend_instance(&env);

        let stored_relayer = get_relayer(&env)?;
        if relayer != stored_relayer {
            return Err(Error::NotAuthorized);
        }

        if get_soul_id_by_passkey(&env, &passkey).is_some() {
            return Err(Error::SoulAlreadyExists);
        }

        let id = increment_total_souls(&env);
        let soul_data = SoulData {
            id,
            passkey,
            recovery_pubkey,
            minted_at: env.ledger().timestamp(),
        };

        set_soul(&env, &soul_data);

        env.events().publish(
            (symbol_short!("soul"), symbol_short!("minted"), id),
            id,
        );

        Ok(id)
    }

    pub fn get_soul(env: Env, id: u32) -> Option<SoulData> {
        extend_instance(&env);
        get_soul_by_id(&env, id)
    }

    pub fn get_soul_id_by_passkey(env: Env, passkey: BytesN<65>) -> Option<u32> {
        extend_instance(&env);
        get_soul_id_by_passkey(&env, &passkey)
    }

    pub fn get_soul_by_passkey(env: Env, passkey: BytesN<65>) -> Option<SoulData> {
        extend_instance(&env);
        let id = crate::storage::get_soul_id_by_passkey(&env, &passkey)?;
        crate::storage::get_soul_by_id(&env, id)
    }

    pub fn recover_soul(
        env: Env,
        relayer: Address,
        old_passkey: BytesN<65>,
        new_passkey: BytesN<65>,
        recovery_signature: BytesN<64>,
    ) -> Result<(), Error> {
        relayer.require_auth();
        extend_instance(&env);
        
        let stored_relayer = get_relayer(&env)?;
        if relayer != stored_relayer {
            return Err(Error::NotAuthorized);
        }

        let soul_id = crate::storage::get_soul_id_by_passkey(&env, &old_passkey).ok_or(Error::SoulNotFound)?;
        let mut soul_data = crate::storage::get_soul_by_id(&env, soul_id).unwrap();

        // Verify recovery signature: sign(hash(old_passkey + new_passkey))
        let mut msg = Bytes::new(&env);
        msg.append(&old_passkey.clone().into());
        msg.append(&new_passkey.clone().into());
        let msg_hash = env.crypto().sha256(&msg);

        // This is the core sovereign recovery check
        env.crypto().secp256r1_verify(
            &soul_data.recovery_pubkey,
            &msg_hash,
            &recovery_signature
        );

        // Update mappings
        remove_passkey_mapping(&env, &old_passkey);
        soul_data.passkey = new_passkey.clone();
        set_soul(&env, &soul_data);

        env.events().publish(
            (symbol_short!("soul"), symbol_short!("recovered"), soul_id),
            new_passkey,
        );

        Ok(())
    }

    pub fn update_relayer(env: Env, admin: Address, new_relayer: Address) -> Result<(), Error> {
        extend_instance(&env);
        admin.require_auth();
        
        let stored_admin = get_admin(&env)?;
        if admin != stored_admin {
            return Err(Error::NotAuthorized);
        }

        env.storage().instance().set(&DataKey::Relayer, &new_relayer);
        Ok(())
    }
}
