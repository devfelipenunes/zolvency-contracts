#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Token,
    TotalUserPrincipal,
    UserBalance(Address),
    TotalRevenue,
    DeFiAdapter,
    DelegatedAmount,
}

#[contract]
pub struct ProofVaultContract;

#[contractimpl]
impl ProofVaultContract {
    /// Initializes the ProofVault contract.
    /// @param admin The address with administrative privileges (e.g., for consuming credit).
    /// @param token The address of the underlying token.
    pub fn initialize(env: Env, admin: Address, token: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::TotalRevenue, &0i128);
        env.storage().instance().set(&DataKey::TotalUserPrincipal, &0i128);
        env.storage().instance().set(&DataKey::DelegatedAmount, &0i128);
    }

    /// Deposits tokens into the vault and updates user principal balance.
    pub fn deposit(env: Env, user: Address, amount: i128) {
        user.require_auth();
        if amount <= 0 {
            panic!("Amount must be positive");
        }

        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let client = soroban_sdk::token::Client::new(&env, &token);
        
        client.transfer(&user, &env.current_contract_address(), &amount);

        let user_balance_key = DataKey::UserBalance(user.clone());
        let user_balance: i128 = env.storage().persistent().get(&user_balance_key).unwrap_or(0);
        
        let total_principal: i128 = env.storage().instance().get(&DataKey::TotalUserPrincipal).unwrap_or(0);
        
        env.storage().persistent().set(&user_balance_key, &(user_balance.checked_add(amount).unwrap()));
        env.storage().instance().set(&DataKey::TotalUserPrincipal, &(total_principal.checked_add(amount).unwrap()));
    }

    pub fn get_total_principal(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::TotalUserPrincipal).unwrap_or(0)
    }

    pub fn get_user_balance(env: Env, user: Address) -> i128 {
        env.storage().persistent().get(&DataKey::UserBalance(user)).unwrap_or(0)
    }

    pub fn get_total_balance(env: Env) -> i128 {
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let client = soroban_sdk::token::Client::new(&env, &token);
        let liquid_balance = client.balance(&env.current_contract_address());
        let delegated_amount: i128 = env.storage().instance().get(&DataKey::DelegatedAmount).unwrap_or(0);
        liquid_balance.checked_add(delegated_amount).unwrap()
    }

    pub fn get_balance(env: Env, user: Address) -> i128 {
        Self::get_user_balance(env, user)
    }

    /// Admin-only function to consume a user's balance in exchange for off-chain or virtual credit.
    /// This is a custodial consumption requested by the protocol design.
    pub fn consume_credit(env: Env, user: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let user_balance_key = DataKey::UserBalance(user.clone());
        let user_balance: i128 = env.storage().persistent().get(&user_balance_key).unwrap_or(0);
        
        if user_balance < amount {
            panic!("Insufficient user balance");
        }

        let total_principal: i128 = env.storage().instance().get(&DataKey::TotalUserPrincipal).unwrap_or(0);
        
        env.storage().persistent().set(&user_balance_key, &(user_balance.checked_sub(amount).unwrap()));
        env.storage().instance().set(&DataKey::TotalUserPrincipal, &(total_principal.checked_sub(amount).unwrap()));

        let total_revenue: i128 = env.storage().instance().get(&DataKey::TotalRevenue).unwrap_or(0);
        env.storage().instance().set(&DataKey::TotalRevenue, &(total_revenue.checked_add(amount).unwrap()));
    }

    pub fn withdraw_revenue(env: Env, to: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let total_revenue: i128 = env.storage().instance().get(&DataKey::TotalRevenue).unwrap_or(0);
        if total_revenue < amount {
            panic!("Insufficient revenue balance");
        }
        env.storage().instance().set(&DataKey::TotalRevenue, &(total_revenue.checked_sub(amount).unwrap()));

        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let client = soroban_sdk::token::Client::new(&env, &token);
        client.transfer(&env.current_contract_address(), &to, &amount);
    }

    /// Sets the authorized DeFi adapter.
    pub fn set_defi_adapter(env: Env, admin: Address, adapter: Address) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("Unauthorized: not admin");
        }
        env.storage().instance().set(&DataKey::DeFiAdapter, &adapter);
    }

    /// Delegates liquidity to the authorized adapter.
    pub fn delegate_liquidity(env: Env, admin: Address, amount: i128) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            panic!("Unauthorized: not admin");
        }
        if amount <= 0 {
            panic!("Amount must be positive");
        }

        let adapter: Address = env.storage().instance().get(&DataKey::DeFiAdapter).expect("Adapter not set");
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let client = soroban_sdk::token::Client::new(&env, &token);
        
        client.transfer(&env.current_contract_address(), &adapter, &amount);

        let delegated_amount: i128 = env.storage().instance().get(&DataKey::DelegatedAmount).unwrap_or(0);
        env.storage().instance().set(&DataKey::DelegatedAmount, &(delegated_amount.checked_add(amount).unwrap()));
    }

    /// Reports yield earned by the adapter.
    /// This increases the DelegatedAmount without minting new shares, increasing the share price.
    pub fn harvest_yield(env: Env, adapter: Address, amount: i128) {
        adapter.require_auth();
        let stored_adapter: Address = env.storage().instance().get(&DataKey::DeFiAdapter).expect("Adapter not set");
        if adapter != stored_adapter {
            panic!("Unauthorized: not authorized adapter");
        }
        if amount <= 0 {
            panic!("Amount must be positive");
        }

        let delegated_amount: i128 = env.storage().instance().get(&DataKey::DelegatedAmount).unwrap_or(0);
        env.storage().instance().set(&DataKey::DelegatedAmount, &(delegated_amount.checked_add(amount).unwrap()));
    }

    /// Returns the total profit (surplus) generated by the vault (Total Assets - Total Principal).
    pub fn get_profit(env: Env) -> i128 {
        let total_assets = Self::get_total_balance(env.clone());
        let total_principal = Self::get_total_principal(env.clone());
        total_assets.checked_sub(total_principal).unwrap_or(0)
    }

    /// Admin-only function to withdraw vault profits.
    /// @param admin The address of the vault administrator.
    /// @param to The recipient address.
    /// @param amount The amount of profit to withdraw.
    pub fn withdraw_profit(env: Env, admin: Address, to: Address, amount: i128) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Admin not set");
        if admin != stored_admin {
            panic!("Unauthorized: not admin");
        }

        let profit = Self::get_profit(env.clone());
        if profit < amount {
            panic!("Insufficient profit balance");
        }

        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let client = soroban_sdk::token::Client::new(&env, &token);
        client.transfer(&env.current_contract_address(), &to, &amount);
    }
}

mod test;
