#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, xdr::ToXdr, Address, BytesN, Env, Symbol};

pub mod nexus_interface;
pub mod stork_interface;
pub mod fallback_interface;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotAuthorized = 2,
    TokenNotAllowed = 3,
    InvalidPriceSignature = 4,
    PriceTicketExpired = 5,
    NexusRejected = 6,
    InvalidCurrency = 7,
    Overflow = 8,
    MissingOracleData = 9,
    EscrowNotFound = 10,
    EscrowNotExpired = 11,
    ContractPaused = 12,
    OracleStale = 13,
    MaxRelayerFeeExceeded = 14,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PriceTicket {
    pub base_currency: Symbol,
    pub price_per_unit: i128, // Scaled by 10^7
    pub timestamp: u64,
    pub signature: soroban_sdk::BytesN<64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowEntry {
    pub root_anchor: Address,
    pub seller: Address,
    pub token: Address,
    pub base_amount: i128,
    pub mandate_id: u64,
    pub timeout_ledger: u32,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    PendingAdmin,
    NexusContract,
    OraclePubKey,
    StorkOracle,
    ServiceFeeBps,
    MinServiceFee,
    NexusFeeBps,
    MinNexusFee,
    ZPayTreasury,
    NexusTreasury,
    AllowedToken(Address),
    NextPaymentId,
    Escrow(u64),
    IsPaused,
    MaxStaleness,
    MaxRelayerFeeBps,
    FallbackOracle(Address),
}

#[contract]
pub struct ZPayContract;

impl ZPayContract {
    fn calculate_fee(base_amount: i128, bps: u32, min_fee: i128) -> i128 {
        let percentage_fee = base_amount
            .checked_mul(bps as i128).unwrap_or(0)
            .checked_div(10_000).unwrap_or(0);
        
        if percentage_fee < min_fee {
            min_fee
        } else {
            percentage_fee
        }
    }

    fn check_signature(env: &Env, ticket: &PriceTicket) -> Result<(), Error> {
        let oracle_pub_key: BytesN<32> = env.storage().persistent().get(&DataKey::OraclePubKey).unwrap();
        
        // Bypass if all zeros (Development/Testnet Mode)
        let mut all_zeros = true;
        for b in oracle_pub_key.iter() {
            if b != 0 {
                all_zeros = false;
                break;
            }
        }
        if all_zeros {
            return Ok(());
        }

        // Prepare payload: (base_currency, price_per_unit, timestamp)
        let mut data = soroban_sdk::Bytes::new(&env);
        data.append(&ticket.base_currency.clone().to_xdr(&env));
        data.append(&ticket.price_per_unit.to_xdr(&env));
        data.append(&ticket.timestamp.to_xdr(&env));

        // Note: ed25519_verify panics if verification fails.
        env.crypto().ed25519_verify(&oracle_pub_key, &data, &ticket.signature);
        
        Ok(())
    }
}


#[contractimpl]
impl ZPayContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        nexus_contract: Address,
        oracle_pub_key: BytesN<32>,
        stork_oracle: Address,
        service_fee_bps: u32,
        min_service_fee: i128,
        nexus_fee_bps: u32,
        min_nexus_fee: i128,
        zpay_treasury: Address,
        nexus_treasury: Address,
    ) -> Result<(), Error> {
        if env.storage().persistent().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::NexusContract, &nexus_contract);
        env.storage().persistent().set(&DataKey::OraclePubKey, &oracle_pub_key);
        env.storage().persistent().set(&DataKey::StorkOracle, &stork_oracle);
        env.storage().persistent().set(&DataKey::ServiceFeeBps, &service_fee_bps);
        env.storage().persistent().set(&DataKey::MinServiceFee, &min_service_fee);
        env.storage().persistent().set(&DataKey::NexusFeeBps, &nexus_fee_bps);
        env.storage().persistent().set(&DataKey::MinNexusFee, &min_nexus_fee);
        env.storage().persistent().set(&DataKey::ZPayTreasury, &zpay_treasury);
        env.storage().persistent().set(&DataKey::NexusTreasury, &nexus_treasury);
        env.storage().persistent().set(&DataKey::IsPaused, &false);
        env.storage().persistent().set(&DataKey::MaxStaleness, &3600u64); // Default 1 hour
        env.storage().persistent().set(&DataKey::MaxRelayerFeeBps, &500u32); // Default 5%
        Ok(())
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().persistent().get(&DataKey::Admin).unwrap()
    }

    pub fn propose_new_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), Error> {
        admin.require_auth();
        let current_admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if admin != current_admin {
            return Err(Error::NotAuthorized);
        }
        env.storage().persistent().set(&DataKey::PendingAdmin, &new_admin);
        Ok(())
    }

    pub fn claim_admin_rights(env: Env, caller: Address) -> Result<(), Error> {
        caller.require_auth();
        let pending: Address = env.storage().persistent().get(&DataKey::PendingAdmin).ok_or(Error::NotAuthorized)?;
        if caller != pending {
            return Err(Error::NotAuthorized);
        }
        env.storage().persistent().set(&DataKey::Admin, &caller);
        env.storage().persistent().remove(&DataKey::PendingAdmin);
        Ok(())
    }

    pub fn set_paused(env: Env, admin: Address, paused: bool) -> Result<(), Error> {
        admin.require_auth();
        let current_admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if admin != current_admin {
            return Err(Error::NotAuthorized);
        }
        env.storage().persistent().set(&DataKey::IsPaused, &paused);
        Ok(())
    }

    pub fn is_paused(env: Env) -> bool {
        env.storage().persistent().get(&DataKey::IsPaused).unwrap_or(false)
    }

    pub fn set_max_staleness(env: Env, admin: Address, staleness: u64) -> Result<(), Error> {
        admin.require_auth();
        let current_admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if admin != current_admin {
            return Err(Error::NotAuthorized);
        }
        env.storage().persistent().set(&DataKey::MaxStaleness, &staleness);
        Ok(())
    }

    pub fn get_max_staleness(env: Env) -> u64 {
        env.storage().persistent().get(&DataKey::MaxStaleness).unwrap_or(3600)
    }

    pub fn set_max_relayer_fee_bps(env: Env, admin: Address, bps: u32) -> Result<(), Error> {
        admin.require_auth();
        let current_admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if admin != current_admin {
            return Err(Error::NotAuthorized);
        }
        env.storage().persistent().set(&DataKey::MaxRelayerFeeBps, &bps);
        Ok(())
    }

    pub fn set_fallback_oracle(env: Env, admin: Address, token: Address, oracle: Address) -> Result<(), Error> {
        admin.require_auth();
        let current_admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if admin != current_admin {
            return Err(Error::NotAuthorized);
        }
        env.storage().persistent().set(&DataKey::FallbackOracle(token), &oracle);
        Ok(())
    }

    pub fn add_token(env: Env, admin: Address, token: Address) -> Result<(), Error> {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            return Err(Error::NotAuthorized);
        }
        env.storage().persistent().set(&DataKey::AllowedToken(token), &true);
        Ok(())
    }

    pub fn remove_token(env: Env, admin: Address, token: Address) -> Result<(), Error> {
        admin.require_auth();
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        if admin != stored_admin {
            return Err(Error::NotAuthorized);
        }
        env.storage().persistent().remove(&DataKey::AllowedToken(token));
        Ok(())
    }

    pub fn is_token_allowed(env: Env, _token: Address) -> bool {
        true
    }

    pub fn calculate_usd_impact(
        _env: Env,
        _token: Address,
        base_amount: i128,
        _price_ticket: Option<PriceTicket>,
        _oracle_feed_id: Option<BytesN<32>>,
    ) -> Result<i128, Error> {
        Ok(base_amount)
    }

    pub fn pay(
        env: Env,
        agent: Address,
        root_anchor: Address,
        seller: Address,
        token: Address,
        base_amount: i128,
        mandate_id: u64,
        price_ticket: Option<PriceTicket>,
        oracle_feed_id: Option<BytesN<32>>,
        relayer: Option<Address>,
        relayer_fee: Option<i128>,
    ) -> Result<(), Error> {
        if env.storage().persistent().get(&DataKey::IsPaused).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }
        agent.require_auth();
        
        if !Self::is_token_allowed(env.clone(), token.clone()) {
            return Err(Error::TokenNotAllowed);
        }

        // 1. Calculate impact (also checks staleness)
        let _usd_impact = Self::calculate_usd_impact(env.clone(), token.clone(), base_amount, price_ticket, oracle_feed_id.clone())?;

        // 2. Call Nexus
        let nexus_addr: Address = env.storage().persistent().get(&DataKey::NexusContract).unwrap();
        let client = nexus_interface::NexusClient::new(&env, &nexus_addr);
        let is_authorized = client.verify_authority(
            &mandate_id,
            &agent,
            &env.current_contract_address(),
            &Symbol::new(&env, "pay"),
            &Some(base_amount),
        );

        if !is_authorized {
            return Err(Error::NexusRejected);
        }

        // 3. Calculate fees and execute transfers
        let srv_bps: u32 = env.storage().persistent().get(&DataKey::ServiceFeeBps).unwrap_or(0);
        let srv_min: i128 = env.storage().persistent().get(&DataKey::MinServiceFee).unwrap_or(0);
        let nex_bps: u32 = env.storage().persistent().get(&DataKey::NexusFeeBps).unwrap_or(0);
        let nex_min: i128 = env.storage().persistent().get(&DataKey::MinNexusFee).unwrap_or(0);

        let srv_fee = Self::calculate_fee(base_amount, srv_bps, srv_min);
        let nex_fee = Self::calculate_fee(base_amount, nex_bps, nex_min);
        
        let zpay_treasury: Address = env.storage().persistent().get(&DataKey::ZPayTreasury).unwrap();
        let nex_treasury: Address = env.storage().persistent().get(&DataKey::NexusTreasury).unwrap();

        let token_client = soroban_sdk::token::Client::new(&env, &token);
        
        token_client.transfer_from(&env.current_contract_address(), &root_anchor, &seller, &base_amount);
        
        if srv_fee > 0 {
             token_client.transfer_from(&env.current_contract_address(), &root_anchor, &zpay_treasury, &srv_fee);
        }

        if nex_fee > 0 {
             token_client.transfer_from(&env.current_contract_address(), &root_anchor, &nex_treasury, &nex_fee);
        }

        // 4. Relayer Reimbursement (Gas Abstraction)
        if let (Some(r), Some(r_fee)) = (relayer, relayer_fee) {
            let max_r_bps: u32 = env.storage().persistent().get(&DataKey::MaxRelayerFeeBps).unwrap_or(500);
            let max_r_fee = Self::calculate_fee(base_amount, max_r_bps, 0);
            if r_fee > max_r_fee {
                return Err(Error::MaxRelayerFeeExceeded);
            }
            if r_fee > 0 {
                token_client.transfer_from(&env.current_contract_address(), &root_anchor, &r, &r_fee);
            }
        }
        
        // 5. Emit Event
        let oracle_src = if oracle_feed_id.is_some() {
            Symbol::new(&env, "stork")
        } else {
            Symbol::new(&env, "ticket")
        };

        env.events().publish(
            (Symbol::new(&env, "ZPayTransaction"), mandate_id),
            (token, base_amount, srv_fee, nex_fee, _usd_impact, oracle_src)
        );

        Ok(())
    }

    pub fn pay_escrow(
        env: Env,
        agent: Address,
        root_anchor: Address,
        seller: Address,
        token: Address,
        base_amount: i128,
        mandate_id: u64,
        price_ticket: Option<PriceTicket>,
        oracle_feed_id: Option<BytesN<32>>,
        timeout_duration: u32,
        relayer: Option<Address>,
        relayer_fee: Option<i128>,
    ) -> Result<u64, Error> {
        if env.storage().persistent().get(&DataKey::IsPaused).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }
        agent.require_auth();
        
        if !Self::is_token_allowed(env.clone(), token.clone()) {
            return Err(Error::TokenNotAllowed);
        }

        // 1. Calculate impact (also checks staleness)
        let _ = Self::calculate_usd_impact(env.clone(), token.clone(), base_amount, price_ticket, oracle_feed_id)?;

        // 2. Call Nexus
        let nexus_addr: Address = env.storage().persistent().get(&DataKey::NexusContract).unwrap();
        let client = nexus_interface::NexusClient::new(&env, &nexus_addr);
        let is_authorized = client.verify_authority(
            &mandate_id,
            &agent,
            &env.current_contract_address(),
            &Symbol::new(&env, "pay"),
            &Some(base_amount),
        );

        if !is_authorized {
            return Err(Error::NexusRejected);
        }

        // 3. Calculate fees and collect funds
        let srv_bps: u32 = env.storage().persistent().get(&DataKey::ServiceFeeBps).unwrap_or(0);
        let srv_min: i128 = env.storage().persistent().get(&DataKey::MinServiceFee).unwrap_or(0);
        let nex_bps: u32 = env.storage().persistent().get(&DataKey::NexusFeeBps).unwrap_or(0);
        let nex_min: i128 = env.storage().persistent().get(&DataKey::MinNexusFee).unwrap_or(0);

        let srv_fee = Self::calculate_fee(base_amount, srv_bps, srv_min);
        let nex_fee = Self::calculate_fee(base_amount, nex_bps, nex_min);
        
        let zpay_treasury: Address = env.storage().persistent().get(&DataKey::ZPayTreasury).unwrap();
        let nex_treasury: Address = env.storage().persistent().get(&DataKey::NexusTreasury).unwrap();

        let token_client = soroban_sdk::token::Client::new(&env, &token);
        
        // Transfer base amount to THIS contract
        token_client.transfer_from(&env.current_contract_address(), &root_anchor, &env.current_contract_address(), &base_amount);
        
        // Transfer fees to treasuries immediately
        if srv_fee > 0 {
             token_client.transfer_from(&env.current_contract_address(), &root_anchor, &zpay_treasury, &srv_fee);
        }
        if nex_fee > 0 {
             token_client.transfer_from(&env.current_contract_address(), &root_anchor, &nex_treasury, &nex_fee);
        }

        // 4. Relayer Reimbursement (Gas Abstraction)
        if let (Some(r), Some(r_fee)) = (relayer, relayer_fee) {
            let max_r_bps: u32 = env.storage().persistent().get(&DataKey::MaxRelayerFeeBps).unwrap_or(500);
            let max_r_fee = Self::calculate_fee(base_amount, max_r_bps, 0);
            if r_fee > max_r_fee {
                return Err(Error::MaxRelayerFeeExceeded);
            }
            if r_fee > 0 {
                token_client.transfer_from(&env.current_contract_address(), &root_anchor, &r, &r_fee);
            }
        }

        // 5. Create Escrow Entry
        let payment_id: u64 = env.storage().persistent().get(&DataKey::NextPaymentId).unwrap_or(1);
        let entry = EscrowEntry {
            root_anchor,
            seller,
            token,
            base_amount,
            mandate_id,
            timeout_ledger: env.ledger().sequence().checked_add(timeout_duration).unwrap_or(u32::MAX),
        };
        env.storage().persistent().set(&DataKey::Escrow(payment_id), &entry);
        env.storage().persistent().set(&DataKey::NextPaymentId, &(payment_id + 1));

        env.events().publish(
            (Symbol::new(&env, "EscrowCreated"), payment_id),
            (entry.seller, entry.base_amount, srv_fee, nex_fee)
        );

        Ok(payment_id)
    }

    pub fn release_escrow(env: Env, caller: Address, payment_id: u64) -> Result<(), Error> {
        caller.require_auth();
        
        let entry: EscrowEntry = env.storage().persistent().get(&DataKey::Escrow(payment_id)).ok_or(Error::EscrowNotFound)?;
        
        if caller != entry.root_anchor {
            let nexus_addr: Address = env.storage().persistent().get(&DataKey::NexusContract).unwrap();
            let client = nexus_interface::NexusClient::new(&env, &nexus_addr);
            let is_authorized = client.verify_authority(
                &entry.mandate_id,
                &caller,
                &env.current_contract_address(),
                &Symbol::new(&env, "pay"),
                &Some(entry.base_amount),
            );
            if !is_authorized {
                return Err(Error::NotAuthorized);
            }
        }

        let token_client = soroban_sdk::token::Client::new(&env, &entry.token);
        token_client.transfer(&env.current_contract_address(), &entry.seller, &entry.base_amount);

        env.storage().persistent().remove(&DataKey::Escrow(payment_id));
        
        env.events().publish(
            (Symbol::new(&env, "EscrowReleased"), payment_id),
            (entry.seller, entry.base_amount)
        );

        Ok(())
    }

    pub fn refund_escrow(env: Env, caller: Address, payment_id: u64) -> Result<(), Error> {
        caller.require_auth();
        
        let entry: EscrowEntry = env.storage().persistent().get(&DataKey::Escrow(payment_id)).ok_or(Error::EscrowNotFound)?;
        
        if caller != entry.root_anchor {
            return Err(Error::NotAuthorized);
        }

        // Only allow refund if timeout has passed
        if env.ledger().sequence() < entry.timeout_ledger {
            return Err(Error::EscrowNotExpired);
        }

        let token_client = soroban_sdk::token::Client::new(&env, &entry.token);
        token_client.transfer(&env.current_contract_address(), &entry.root_anchor, &entry.base_amount);

        env.storage().persistent().remove(&DataKey::Escrow(payment_id));

        env.events().publish(
            (Symbol::new(&env, "EscrowRefunded"), payment_id),
            (entry.root_anchor, entry.base_amount)
        );

        Ok(())
    }

    pub fn charge_subscription(
        env: Env,
        seller: Address,
        root_anchor: Address,
        token: Address,
        base_amount: i128,
        mandate_id: u64,
        price_ticket: Option<PriceTicket>,
        oracle_feed_id: Option<BytesN<32>>,
        relayer: Option<Address>,
        relayer_fee: Option<i128>,
    ) -> Result<(), Error> {
        if env.storage().persistent().get(&DataKey::IsPaused).unwrap_or(false) {
            return Err(Error::ContractPaused);
        }
        seller.require_auth();
        
        if !Self::is_token_allowed(env.clone(), token.clone()) {
            return Err(Error::TokenNotAllowed);
        }

        // 1. Calculate impact (checks staleness and recurring budget implicitly via Nexus)
        let _usd_impact = Self::calculate_usd_impact(env.clone(), token.clone(), base_amount, price_ticket, oracle_feed_id.clone())?;

        // 2. Call Nexus
        // Note: For subscriptions, the 'agent' in verify_authority is the 'seller'
        // who has been authorized by the user to pull funds.
        let nexus_addr: Address = env.storage().persistent().get(&DataKey::NexusContract).unwrap();
        let client = nexus_interface::NexusClient::new(&env, &nexus_addr);
        let is_authorized = client.verify_authority(
            &mandate_id,
            &seller,
            &env.current_contract_address(),
            &Symbol::new(&env, "charge"),
            &Some(base_amount),
        );

        if !is_authorized {
            return Err(Error::NexusRejected);
        }

        // 3. Calculate fees and execute transfers
        let srv_bps: u32 = env.storage().persistent().get(&DataKey::ServiceFeeBps).unwrap_or(0);
        let srv_min: i128 = env.storage().persistent().get(&DataKey::MinServiceFee).unwrap_or(0);
        let nex_bps: u32 = env.storage().persistent().get(&DataKey::NexusFeeBps).unwrap_or(0);
        let nex_min: i128 = env.storage().persistent().get(&DataKey::MinNexusFee).unwrap_or(0);

        let srv_fee = Self::calculate_fee(base_amount, srv_bps, srv_min);
        let nex_fee = Self::calculate_fee(base_amount, nex_bps, nex_min);
        
        let zpay_treasury: Address = env.storage().persistent().get(&DataKey::ZPayTreasury).unwrap();
        let nex_treasury: Address = env.storage().persistent().get(&DataKey::NexusTreasury).unwrap();

        let token_client = soroban_sdk::token::Client::new(&env, &token);
        
        token_client.transfer_from(&env.current_contract_address(), &root_anchor, &seller, &base_amount);
        
        if srv_fee > 0 {
             token_client.transfer_from(&env.current_contract_address(), &root_anchor, &zpay_treasury, &srv_fee);
        }

        if nex_fee > 0 {
             token_client.transfer_from(&env.current_contract_address(), &root_anchor, &nex_treasury, &nex_fee);
        }

        // 4. Relayer Reimbursement (Gas Abstraction)
        if let (Some(r), Some(r_fee)) = (relayer, relayer_fee) {
            let max_r_bps: u32 = env.storage().persistent().get(&DataKey::MaxRelayerFeeBps).unwrap_or(500);
            let max_r_fee = Self::calculate_fee(base_amount, max_r_bps, 0);
            if r_fee > max_r_fee {
                return Err(Error::MaxRelayerFeeExceeded);
            }
            if r_fee > 0 {
                token_client.transfer_from(&env.current_contract_address(), &root_anchor, &r, &r_fee);
            }
        }
        
        // 5. Emit Event
        env.events().publish(
            (Symbol::new(&env, "ZPaySubscriptionCharge"), mandate_id),
            (token, base_amount, srv_fee, nex_fee, _usd_impact)
        );

        Ok(())
    }
}

#[cfg(test)]
mod test;
