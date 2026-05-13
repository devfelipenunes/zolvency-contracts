use soroban_sdk::{Address, Env};

#[soroban_sdk::contracttype]
#[derive(Clone)]
pub enum DataKey {
    Gateway,
    GasService,
    GasToken,
    Admin,
    SoulContract,
}

const DAY_IN_LEDGERS: u32 = 17_280;
const ONE_YEAR: u32 = 365 * DAY_IN_LEDGERS;

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
    extend_instance(env);
}

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Admin)
}

pub fn set_soul_contract(env: &Env, soul_contract: &Address) {
    env.storage().instance().set(&DataKey::SoulContract, soul_contract);
}

pub fn get_soul_contract(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::SoulContract)
}

pub fn set_gateway(env: &Env, gateway: &Address) {
    env.storage().instance().set(&DataKey::Gateway, gateway);
}

pub fn get_gateway(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::Gateway)
}

pub fn set_gas_service(env: &Env, gas_service: &Address) {
    env.storage().instance().set(&DataKey::GasService, gas_service);
}

pub fn get_gas_service(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::GasService)
}

pub fn set_gas_token(env: &Env, gas_token: &Address) {
    env.storage().instance().set(&DataKey::GasToken, gas_token);
}

pub fn get_gas_token(env: &Env) -> Option<Address> {
    env.storage().instance().get(&DataKey::GasToken)
}

pub fn extend_instance(env: &Env) {
    env.storage().instance().extend_ttl(ONE_YEAR, ONE_YEAR);
}
