use soroban_sdk::{Address, Env, Symbol, symbol_short, IntoVal, BytesN, token, xdr::ToXdr};
use crate::{Error, PriceTicket, EscrowEntry};
use crate::storage;
use crate::interfaces;

pub fn execute_payment(
    env: &Env,
    agent: Address,
    root_anchor: Address,
    seller: Address,
    token: Address,
    base_amount: i128,
    mandate_id: u64,
    price_ticket: Option<PriceTicket>,
    _oracle_feed_id: Option<BytesN<32>>, // Future stork integration
    relayer: Option<Address>,
    relayer_fee: Option<i128>,
    intent: Symbol, // "pay" or "charge"
) -> Result<(), Error> {
    if storage::is_paused(env) {
        return Err(Error::ContractPaused);
    }
    agent.require_auth();

    // 1. Oráculo e Impacto USD
    let _ = calculate_usd_impact(env, &token, base_amount, price_ticket)?;
    
    // 2. Cálculo de Taxas
    let (s_bps, s_min) = storage::get_service_fee_config(env);
    let (n_bps, n_min) = storage::get_nexus_fee_config(env);
    
    let service_fee = calculate_fee(base_amount, s_bps, s_min);
    let nexus_fee = calculate_fee(base_amount, n_bps, n_min);
    
    let mut applied_relayer_fee = 0;
    if let (Some(_), Some(r_fee)) = (&relayer, relayer_fee) {
        let max_r_bps = storage::get_max_relayer_fee_bps(env);
        let max_r_limit = calculate_fee(base_amount, max_r_bps, 0);
        if r_fee > max_r_limit {
            return Err(Error::MaxRelayerFeeExceeded);
        }
        applied_relayer_fee = r_fee;
    }

    let total_spend = base_amount + service_fee + nexus_fee + applied_relayer_fee;

    // 3. Verificação no Nexus
    let nexus_addr = storage::get_nexus_contract(env)?;
    let nexus_client = interfaces::NexusClient::new(env, &nexus_addr);
    let is_authorized = nexus_client.verify_authority(
        &mandate_id,
        &agent,
        &env.current_contract_address(),
        &intent,
        &Some(total_spend),
    );

    if !is_authorized {
        return Err(Error::NexusRejected);
    }

    // 4. Transferências
    let (zpay_treasury, nexus_treasury) = storage::get_treasuries(env);
    let token_client = token::Client::new(env, &token);
    
    token_client.transfer_from(&env.current_contract_address(), &root_anchor, &seller, &base_amount);
    
    if service_fee > 0 {
        token_client.transfer_from(&env.current_contract_address(), &root_anchor, &zpay_treasury, &service_fee);
    }
    if nexus_fee > 0 {
        token_client.transfer_from(&env.current_contract_address(), &root_anchor, &nexus_treasury, &nexus_fee);
    }
    if let (Some(r), Some(r_fee)) = (relayer, relayer_fee) {
        if r_fee > 0 {
            token_client.transfer_from(&env.current_contract_address(), &root_anchor, &r, &r_fee);
        }
    }

    env.events().publish(
        (Symbol::new(env, "ZPayTransaction"), mandate_id),
        (token, base_amount, service_fee, nexus_fee)
    );

    Ok(())
}

pub fn create_escrow(
    env: &Env,
    agent: Address,
    root_anchor: Address,
    seller: Address,
    token: Address,
    base_amount: i128,
    mandate_id: u64,
    price_ticket: Option<PriceTicket>,
    timeout_duration: u32,
    relayer: Option<Address>,
    relayer_fee: Option<i128>,
) -> Result<u64, Error> {
    if storage::is_paused(env) {
        return Err(Error::ContractPaused);
    }
    agent.require_auth();

    let _ = calculate_usd_impact(env, &token, base_amount, price_ticket)?;

    let (s_bps, s_min) = storage::get_service_fee_config(env);
    let (n_bps, n_min) = storage::get_nexus_fee_config(env);
    let service_fee = calculate_fee(base_amount, s_bps, s_min);
    let nexus_fee = calculate_fee(base_amount, n_bps, n_min);
    
    let mut applied_relayer_fee = 0;
    if let (Some(_), Some(r_fee)) = (&relayer, relayer_fee) {
        let max_r_bps = storage::get_max_relayer_fee_bps(env);
        let max_r_limit = calculate_fee(base_amount, max_r_bps, 0);
        if r_fee > max_r_limit {
            return Err(Error::MaxRelayerFeeExceeded);
        }
        applied_relayer_fee = r_fee;
    }

    let total_spend = base_amount + service_fee + nexus_fee + applied_relayer_fee;

    let nexus_addr = storage::get_nexus_contract(env)?;
    let nexus_client = interfaces::NexusClient::new(env, &nexus_addr);
    let is_authorized = nexus_client.verify_authority(
        &mandate_id,
        &agent,
        &env.current_contract_address(),
        &Symbol::new(env, "pay"),
        &Some(total_spend),
    );

    if !is_authorized {
        return Err(Error::NexusRejected);
    }

    let (zpay_treasury, nexus_treasury) = storage::get_treasuries(env);
    let token_client = token::Client::new(env, &token);
    
    // Fundos ficam NO CONTRATO ZPay durante o escrow
    token_client.transfer_from(&env.current_contract_address(), &root_anchor, &env.current_contract_address(), &base_amount);
    
    if service_fee > 0 {
        token_client.transfer_from(&env.current_contract_address(), &root_anchor, &zpay_treasury, &service_fee);
    }
    if nexus_fee > 0 {
        token_client.transfer_from(&env.current_contract_address(), &root_anchor, &nexus_treasury, &nexus_fee);
    }
    if let (Some(r), Some(r_fee)) = (relayer, relayer_fee) {
        if r_fee > 0 {
            token_client.transfer_from(&env.current_contract_address(), &root_anchor, &r, &r_fee);
        }
    }

    let payment_id = storage::increment_next_payment_id(env);
    let entry = EscrowEntry {
        root_anchor,
        seller,
        token,
        base_amount,
        mandate_id,
        timeout_ledger: env.ledger().sequence().checked_add(timeout_duration).unwrap_or(u32::MAX),
    };
    storage::set_escrow(env, payment_id, &entry);

    env.events().publish(
        (Symbol::new(env, "EscrowCreated"), payment_id),
        (entry.seller, entry.base_amount)
    );

    Ok(payment_id)
}

pub fn release_escrow(env: &Env, caller: Address, payment_id: u64) -> Result<(), Error> {
    caller.require_auth();
    let entry = storage::get_escrow(env, payment_id).ok_or(Error::EscrowNotFound)?;
    
    if caller != entry.root_anchor {
        let nexus_addr = storage::get_nexus_contract(env)?;
        let client = interfaces::NexusClient::new(env, &nexus_addr);
        let is_authorized = client.verify_authority(
            &entry.mandate_id,
            &caller,
            &env.current_contract_address(),
            &Symbol::new(env, "pay"),
            &Some(entry.base_amount),
        );
        if !is_authorized {
            return Err(Error::NotAuthorized);
        }
    }

    let token_client = token::Client::new(env, &entry.token);
    token_client.transfer(&env.current_contract_address(), &entry.seller, &entry.base_amount);

    storage::remove_escrow(env, payment_id);

    env.events().publish((Symbol::new(env, "EscrowReleased"), payment_id), entry.seller);
    Ok(())
}

pub fn refund_escrow(env: &Env, caller: Address, payment_id: u64) -> Result<(), Error> {
    caller.require_auth();
    let entry = storage::get_escrow(env, payment_id).ok_or(Error::EscrowNotFound)?;
    
    if caller != entry.root_anchor {
        return Err(Error::NotAuthorized);
    }

    if env.ledger().sequence() < entry.timeout_ledger {
        return Err(Error::EscrowNotExpired);
    }

    let token_client = token::Client::new(env, &entry.token);
    token_client.transfer(&env.current_contract_address(), &entry.root_anchor, &entry.base_amount);

    storage::remove_escrow(env, payment_id);

    env.events().publish((Symbol::new(env, "EscrowRefunded"), payment_id), entry.root_anchor);
    Ok(())
}

// --- ENGINE (Calculations & Oracle) ---

pub fn calculate_fee(base_amount: i128, bps: u32, min_fee: i128) -> i128 {
    let percentage_fee = base_amount
        .checked_mul(bps as i128).unwrap_or(0)
        .checked_div(10_000).unwrap_or(0);
    
    if percentage_fee < min_fee {
        min_fee
    } else {
        percentage_fee
    }
}

pub fn check_oracle_ticket(env: &Env, ticket: &PriceTicket) -> Result<(), Error> {
    let oracle_pub_key = storage::get_oracle_pubkey(env)?;
    
    let mut all_zeros = true;
    for b in oracle_pub_key.iter() {
        if b != 0 {
            all_zeros = false;
            break;
        }
    }

    if !all_zeros {
        let mut data = soroban_sdk::Bytes::new(env);
        data.append(&ticket.base_currency.clone().to_xdr(env));
        data.append(&ticket.price_per_unit.to_xdr(env));
        data.append(&ticket.timestamp.to_xdr(env));
        env.crypto().ed25519_verify(&oracle_pub_key, &data, &ticket.signature);
    }

    let max_staleness = storage::get_max_staleness(env);
    let current_time = env.ledger().timestamp();
    
    if ticket.timestamp > current_time {
        return Err(Error::PriceTicketExpired);
    }
    
    if current_time - ticket.timestamp > max_staleness {
        return Err(Error::OracleStale);
    }

    Ok(())
}

pub fn calculate_usd_impact(
    env: &Env,
    _token: &Address,
    base_amount: i128,
    price_ticket: Option<PriceTicket>,
) -> Result<i128, Error> {
    if let Some(ticket) = price_ticket {
        check_oracle_ticket(env, &ticket)?;
        Ok(base_amount)
    } else {
        Ok(base_amount)
    }
}
