use soroban_sdk::{Address, Env};
use crate::*;

const DAY_IN_LEDGERS: u32 = 17_280;
const ONE_YEAR: u32 = 365 * DAY_IN_LEDGERS;

pub fn extend_instance(env: &Env) {
    env.storage().instance().extend_ttl(ONE_YEAR, ONE_YEAR);
}

pub fn get_admin(env: &Env) -> Result<Address, MandateError> {
    env.storage().persistent().get(&DataKey::Admin).ok_or(MandateError::NotInitialized)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&DataKey::Admin, admin);
    extend_persistent(env, &DataKey::Admin);
}

pub fn get_mandate(env: &Env, id: u64) -> Option<Mandate> {
    env.storage().persistent().get(&DataKey::Mandate(id))
}

pub fn set_mandate(env: &Env, id: u64, mandate: &Mandate) {
    env.storage().persistent().set(&DataKey::Mandate(id), mandate);
}

pub fn get_mandate_state(env: &Env, id: u64) -> Option<MandateState> {
    env.storage().persistent().get(&DataKey::MandateState(id))
}

pub fn set_mandate_state(env: &Env, id: u64, state: &MandateState) {
    env.storage().persistent().set(&DataKey::MandateState(id), state);
}

pub fn get_next_mandate_id(env: &Env) -> u64 {
    env.storage().persistent().get(&DataKey::NextMandateId).unwrap_or(1)
}

pub fn increment_next_mandate_id(env: &Env) -> u64 {
    let id = get_next_mandate_id(env);
    env.storage().persistent().set(&DataKey::NextMandateId, &(id + 1));
    extend_persistent(env, &DataKey::NextMandateId);
    id
}

pub fn get_global_epoch(env: &Env, root_anchor: &Address) -> u64 {
    env.storage().persistent().get(&DataKey::GlobalEpoch(root_anchor.clone())).unwrap_or(0)
}

pub fn set_global_epoch(env: &Env, root_anchor: &Address, epoch: u64) {
    env.storage().persistent().set(&DataKey::GlobalEpoch(root_anchor.clone()), &epoch);
}

pub fn extend_persistent(env: &Env, key: &DataKey) {
    env.storage().persistent().extend_ttl(key, ONE_YEAR, ONE_YEAR);
}
