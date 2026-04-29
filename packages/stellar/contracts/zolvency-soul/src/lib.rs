#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Bytes, Env, String, Symbol,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotAuthorized = 2,
    SoulAlreadyExists = 3,
    NotInitialized = 4,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Relayer,
    Soul(Address),
    HasSoul(Address),
    TotalSouls,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SoulData {
    pub user: Address,
    pub username: String,
    pub passkey: Bytes,
    pub minted_at: u64,
}

#[contract]
pub struct ZolvencySoulContract;

#[contractimpl]
impl ZolvencySoulContract {
    pub fn initialize(env: Env, admin: Address, relayer: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Relayer, &relayer);
        env.storage().instance().set(&DataKey::TotalSouls, &0u32);
        Ok(())
    }

    pub fn mint(
        env: Env,
        relayer: Address,
        user: Address,
        username: String,
        passkey: Bytes,
    ) -> Result<(), Error> {
        relayer.require_auth();

        // Verificar se é o relayer autorizado
        let authorized_relayer: Address = env
            .storage()
            .instance()
            .get(&DataKey::Relayer)
            .ok_or(Error::NotInitialized)?;
        
        if relayer != authorized_relayer {
            return Err(Error::NotAuthorized);
        }

        // Verificar se o usuário já tem uma Soul
        if env.storage().persistent().has(&DataKey::HasSoul(user.clone())) {
            return Err(Error::SoulAlreadyExists);
        }

        let soul_data = SoulData {
            user: user.clone(),
            username,
            passkey,
            minted_at: env.ledger().timestamp(),
        };

        // Salvar a Soul
        env.storage().persistent().set(&DataKey::Soul(user.clone()), &soul_data);
        env.storage().persistent().set(&DataKey::HasSoul(user.clone()), &true);

        // Incrementar contador
        let mut total: u32 = env.storage().instance().get(&DataKey::TotalSouls).unwrap_or(0);
        total += 1;
        env.storage().instance().set(&DataKey::TotalSouls, &total);

        // Emitir evento
        env.events().publish(
            (symbol_short!("soul"), symbol_short!("minted"), user.clone()),
            user,
        );

        Ok(())
    }

    pub fn balance(env: Env, user: Address) -> u32 {
        if env.storage().persistent().has(&DataKey::HasSoul(user)) {
            1
        } else {
            0
        }
    }

    pub fn has_soul(env: Env, user: Address) -> bool {
        env.storage().persistent().has(&DataKey::HasSoul(user))
    }

    pub fn get_soul(env: Env, user: Address) -> Option<SoulData> {
        env.storage().persistent().get(&DataKey::Soul(user))
    }

    pub fn update_relayer(env: Env, admin: Address, new_relayer: Address) -> Result<(), Error> {
        admin.require_auth();
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        
        if admin != stored_admin {
            return Err(Error::NotAuthorized);
        }

        env.storage().instance().set(&DataKey::Relayer, &new_relayer);
        Ok(())
    }
}
