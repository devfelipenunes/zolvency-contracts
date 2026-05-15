use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug)]
pub struct Subscription {
    pub user: Address,
    pub service_provider: Address,
    pub token: Address,
    pub monthly_limit: i128,
    pub current_month_spent: i128,
    pub last_charge_time: u64,
    pub start_time: u64,
    pub duration_months: u32,
    pub mandate_id: u64,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Subscription(u64), // mandate_id -> Subscription
    NexusContract,
}

pub fn get_admin(env: &soroban_sdk::Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::Admin)
}

pub fn set_admin(env: &soroban_sdk::Env, admin: &Address) {
    env.storage().persistent().set(&DataKey::Admin, admin);
}

pub fn get_nexus(env: &soroban_sdk::Env) -> Option<Address> {
    env.storage().persistent().get(&DataKey::NexusContract)
}

pub fn set_nexus(env: &soroban_sdk::Env, nexus: &Address) {
    env.storage().persistent().set(&DataKey::NexusContract, nexus);
}

pub fn get_subscription(env: &soroban_sdk::Env, mandate_id: u64) -> Option<Subscription> {
    env.storage().persistent().get(&DataKey::Subscription(mandate_id))
}

pub fn set_subscription(env: &soroban_sdk::Env, mandate_id: u64, sub: &Subscription) {
    env.storage().persistent().set(&DataKey::Subscription(mandate_id), sub);
}
