use soroban_sdk::{Address, Env, BytesN, Bytes, symbol_short};
use crate::{Error, SoulData};
use crate::storage;

pub fn mint(
    env: &Env,
    relayer: Address,
    owner: Address,
    passkey: BytesN<65>,
    recovery_pubkey: BytesN<65>,
) -> Result<u32, Error> {
    relayer.require_auth();
    storage::extend_instance(env);

    let stored_relayer = storage::get_relayer(env)?;
    if relayer != stored_relayer {
        return Err(Error::NotAuthorized);
    }

    if storage::get_soul_id_by_address(env, &owner).is_some() {
        return Err(Error::SoulAlreadyExists);
    }

    let id = storage::increment_total_souls(env);
    let soul_data = SoulData {
        id,
        owner,
        passkey,
        recovery_pubkey,
        minted_at: env.ledger().timestamp(),
    };

    storage::set_soul(env, &soul_data);

    env.events().publish((symbol_short!("minted"),), id);

    Ok(id)
}

pub fn recover_soul(
    env: &Env,
    relayer: Address,
    old_passkey: BytesN<65>,
    new_passkey: BytesN<65>,
    recovery_signature: BytesN<64>,
) -> Result<(), Error> {
    relayer.require_auth();
    storage::extend_instance(env);
    
    let stored_relayer = storage::get_relayer(env)?;
    if relayer != stored_relayer {
        return Err(Error::NotAuthorized);
    }

    if old_passkey == new_passkey {
        return Err(Error::SoulAlreadyExists);
    }

    if storage::get_soul_id_by_passkey(env, &new_passkey).is_some() {
        return Err(Error::SoulAlreadyExists);
    }

    let soul_id = storage::get_soul_id_by_passkey(env, &old_passkey).ok_or(Error::SoulNotFound)?;
    let mut soul_data = storage::get_soul_by_id(env, soul_id).unwrap();

    let mut msg = Bytes::new(env);
    msg.append(&old_passkey.clone().into());
    msg.append(&new_passkey.clone().into());
    let msg_hash = env.crypto().sha256(&msg);

    env.crypto().secp256r1_verify(
        &soul_data.recovery_pubkey,
        &msg_hash,
        &recovery_signature
    );

    storage::remove_passkey_mapping(env, &old_passkey);
    soul_data.passkey = new_passkey.clone();
    storage::set_soul(env, &soul_data);

    env.events().publish((symbol_short!("recovery"), soul_id), new_passkey);

    Ok(())
}

pub fn update_relayer(env: &Env, admin: Address, new_relayer: Address) -> Result<(), Error> {
    storage::extend_instance(env);
    admin.require_auth();
    
    let stored_admin = storage::get_admin(env)?;
    if admin != stored_admin {
        return Err(Error::NotAuthorized);
    }

    storage::set_relayer(env, &new_relayer);
    
    env.events().publish((symbol_short!("relayer"),), new_relayer.clone());
    
    Ok(())
}
