#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Token,
    TotalShares,
    UserShares(Address),
    TotalRevenue,
    DeFiAdapter,
    DelegatedAmount,
}

#[contract]
pub struct ProofVaultContract;

const MINIMUM_LIQUIDITY: i128 = 1000;

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
        env.storage().instance().set(&DataKey::TotalShares, &0i128);
        env.storage().instance().set(&DataKey::DelegatedAmount, &0i128);
    }

    /// Deposits tokens into the vault and mints shares to the user.
    /// Implements protection against inflation attacks using a minimum liquidity threshold.
    pub fn deposit(env: Env, user: Address, amount: i128) {
        user.require_auth();
        if amount <= 0 {
            panic!("Amount must be positive");
        }

        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let client = soroban_sdk::token::Client::new(&env, &token);
        
        let total_balance = Self::get_total_balance(env.clone());
        let total_shares = Self::get_total_shares(env.clone());

        client.transfer(&user, &env.current_contract_address(), &amount);

        let shares_to_mint = if total_shares == 0 {
            // Inflation protection: first depositor loses MINIMUM_LIQUIDITY shares
            if amount <= MINIMUM_LIQUIDITY {
                panic!("Initial deposit too small");
            }
            amount.checked_sub(MINIMUM_LIQUIDITY).unwrap()
        } else {
            // (amount * total_shares) / total_balance
            amount.checked_mul(total_shares).unwrap()
                .checked_div(total_balance).unwrap()
        };

        if shares_to_mint <= 0 {
            panic!("Deposit result in zero shares");
        }

        let user_shares_key = DataKey::UserShares(user.clone());
        let user_shares: i128 = env.storage().persistent().get(&user_shares_key).unwrap_or(0);
        
        env.storage().persistent().set(&user_shares_key, &(user_shares.checked_add(shares_to_mint).unwrap()));
        env.storage().instance().set(&DataKey::TotalShares, &(total_shares.checked_add(shares_to_mint).unwrap()));
    }

    pub fn get_total_shares(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::TotalShares).unwrap_or(0)
    }

    pub fn get_user_shares(env: Env, user: Address) -> i128 {
        env.storage().persistent().get(&DataKey::UserShares(user)).unwrap_or(0)
    }

    pub fn get_total_balance(env: Env) -> i128 {
        let token: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let client = soroban_sdk::token::Client::new(&env, &token);
        let liquid_balance = client.balance(&env.current_contract_address());
        let delegated_amount: i128 = env.storage().instance().get(&DataKey::DelegatedAmount).unwrap_or(0);
        liquid_balance.checked_add(delegated_amount).unwrap()
    }

    pub fn get_balance(env: Env, user: Address) -> i128 {
        let total_shares = Self::get_total_shares(env.clone());
        if total_shares == 0 {
            return 0;
        }
        let user_shares = Self::get_user_shares(env.clone(), user);
        let total_balance = Self::get_total_balance(env.clone());
        
        // (user_shares * total_balance) / total_shares
        user_shares.checked_mul(total_balance).unwrap()
            .checked_div(total_shares).unwrap()
    }

    /// Admin-only function to burn a user's shares in exchange for off-chain or virtual credit.
    /// This is a custodial burn requested by the protocol design.
    pub fn consume_credit(env: Env, user: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let total_balance = Self::get_total_balance(env.clone());
        let total_shares = Self::get_total_shares(env.clone());
        
        if total_balance == 0 {
            panic!("Insufficient balance");
        }

        // Calculate shares proportional to the amount of tokens "consumed"
        let shares_to_burn = amount.checked_mul(total_shares).unwrap()
            .checked_div(total_balance).unwrap();
        
        if shares_to_burn <= 0 {
            panic!("Amount too small to consume credit");
        }

        let user_shares_key = DataKey::UserShares(user.clone());
        let user_shares: i128 = env.storage().persistent().get(&user_shares_key).unwrap_or(0);
        
        if user_shares < shares_to_burn {
            panic!("Insufficient user shares");
        }
        
        env.storage().persistent().set(&user_shares_key, &(user_shares.checked_sub(shares_to_burn).unwrap()));
        env.storage().instance().set(&DataKey::TotalShares, &(total_shares.checked_sub(shares_to_burn).unwrap()));

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
}

mod test;
