#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Token,
    UserBalance(Address),
    TotalRevenue,
}

#[contract]
pub struct BillingContract;

#[contractimpl]
impl BillingContract {
    pub fn initialize(env: Env, admin: Address, token: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::TotalRevenue, &0i128);
    }

    pub fn deposit(env: Env, user: Address, amount: i128) {
        user.require_auth();
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let client = soroban_sdk::token::Client::new(&env, &token);
        client.transfer(&user, &env.current_contract_address(), &amount);

        let key = DataKey::UserBalance(user.clone());
        let balance: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        env.storage().persistent().set(&key, &(balance + amount));
    }

    pub fn get_balance(env: Env, user: Address) -> i128 {
        env.storage().persistent().get(&DataKey::UserBalance(user)).unwrap_or(0)
    }

    pub fn consume_credit(env: Env, user: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let key = DataKey::UserBalance(user.clone());
        let balance: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        if balance < amount {
            panic!("Insufficient balance");
        }
        env.storage().persistent().set(&key, &(balance - amount));

        let total_revenue: i128 = env.storage().instance().get(&DataKey::TotalRevenue).unwrap();
        env.storage().instance().set(&DataKey::TotalRevenue, &(total_revenue + amount));
    }

    pub fn withdraw_revenue(env: Env, to: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let total_revenue: i128 = env.storage().instance().get(&DataKey::TotalRevenue).unwrap();
        if total_revenue < amount {
            panic!("Insufficient revenue balance");
        }
        env.storage().instance().set(&DataKey::TotalRevenue, &(total_revenue - amount));

        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let client = soroban_sdk::token::Client::new(&env, &token);
        client.transfer(&env.current_contract_address(), &to, &amount);
    }
}
