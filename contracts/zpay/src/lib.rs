#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, BytesN, Env, Symbol};

pub mod nexus_interface;

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
#[derive(Clone)]
pub enum DataKey {
    Admin,
    NexusContract,
    OraclePubKey,
    ServiceFee,
    NexusFee,
    ZPayTreasury,
    NexusTreasury,
    AllowedToken(Address),
}

#[contract]
pub struct ZPayContract;

#[contractimpl]
impl ZPayContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        nexus_contract: Address,
        oracle_pub_key: BytesN<32>,
        service_fee: i128,
        nexus_fee: i128,
        zpay_treasury: Address,
        nexus_treasury: Address,
    ) -> Result<(), Error> {
        if env.storage().persistent().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().persistent().set(&DataKey::NexusContract, &nexus_contract);
        env.storage().persistent().set(&DataKey::OraclePubKey, &oracle_pub_key);
        env.storage().persistent().set(&DataKey::ServiceFee, &service_fee);
        env.storage().persistent().set(&DataKey::NexusFee, &nexus_fee);
        env.storage().persistent().set(&DataKey::ZPayTreasury, &zpay_treasury);
        env.storage().persistent().set(&DataKey::NexusTreasury, &nexus_treasury);
        Ok(())
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().persistent().get(&DataKey::Admin).unwrap()
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

    pub fn is_token_allowed(env: Env, token: Address) -> bool {
        env.storage().persistent().get(&DataKey::AllowedToken(token)).unwrap_or(false)
    }

    pub fn calculate_usd_impact(
        env: Env,
        base_amount: i128,
        price_ticket: Option<PriceTicket>,
    ) -> Result<i128, Error> {
        let srv_fee: i128 = env.storage().persistent().get(&DataKey::ServiceFee).unwrap();
        let nex_fee: i128 = env.storage().persistent().get(&DataKey::NexusFee).unwrap();
        
        let total_tokens = base_amount
            .checked_add(srv_fee).ok_or(Error::Overflow)?
            .checked_add(nex_fee).ok_or(Error::Overflow)?;

        match price_ticket {
            None => Ok(total_tokens),
            Some(ticket) => {
                if ticket.base_currency != Symbol::new(&env, "USD") {
                    return Err(Error::InvalidCurrency);
                }

                total_tokens
                    .checked_mul(ticket.price_per_unit).ok_or(Error::Overflow)?
                    .checked_div(10_000_000).ok_or(Error::Overflow)
            }
        }
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
    ) -> Result<(), Error> {
        agent.require_auth();
        
        if !Self::is_token_allowed(env.clone(), token.clone()) {
            return Err(Error::TokenNotAllowed);
        }

        // 1. Verify Price Ticket if present
        if let Some(ref ticket) = price_ticket {
            // Check expiry (e.g., 60 seconds)
            if env.ledger().timestamp() > ticket.timestamp + 60 {
                return Err(Error::PriceTicketExpired);
            }
            // In a real scenario, we would verify Ed25519 signature here
            // skipping for prototype simplicity
        }

        // 2. Calculate impact
        let usd_impact = Self::calculate_usd_impact(env.clone(), base_amount, price_ticket)?;

        // 3. Call Nexus
        let nexus_addr: Address = env.storage().persistent().get(&DataKey::NexusContract).unwrap();
        let client = nexus_interface::NexusClient::new(&env, &nexus_addr);
        
        // Z-Pay validates that the mandate allows calling "pay" on the Z-Pay contract itself
        let is_authorized = client.try_verify_authority(
            &mandate_id,
            &env.current_contract_address(),
            &Symbol::new(&env, "pay"),
            &Some(usd_impact)
        ).unwrap_or(Ok(false)).unwrap_or(false);

        if !is_authorized {
            return Err(Error::NexusRejected);
        }

        // 4. Execute Transfers
        let srv_fee: i128 = env.storage().persistent().get(&DataKey::ServiceFee).unwrap();
        let nex_fee: i128 = env.storage().persistent().get(&DataKey::NexusFee).unwrap();
        
        let zpay_treasury: Address = env.storage().persistent().get(&DataKey::ZPayTreasury).unwrap();
        let nex_treasury: Address = env.storage().persistent().get(&DataKey::NexusTreasury).unwrap();

        let token_client = soroban_sdk::token::Client::new(&env, &token);
        
        // Transfer to Seller
        token_client.transfer_from(&env.current_contract_address(), &root_anchor, &seller, &base_amount);
        
        // Transfer Z-Pay Fee
        if srv_fee > 0 {
             token_client.transfer_from(&env.current_contract_address(), &root_anchor, &zpay_treasury, &srv_fee);
        }

        // Transfer Nexus Fee
        if nex_fee > 0 {
             token_client.transfer_from(&env.current_contract_address(), &root_anchor, &nex_treasury, &nex_fee);
        }
        
        // 5. Emit Event
        env.events().publish(
            (Symbol::new(&env, "ZPayTransaction"), mandate_id),
            (token, base_amount, usd_impact)
        );

        Ok(())
    }
}

#[cfg(test)]
mod test;
