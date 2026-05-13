#![cfg(test)]
use super::*;
use soroban_sdk::{Address, testutils::Address as _, testutils::Ledger as _, Env, token::Client as TokenClient, xdr::ToXdr};

fn init_client(env: &Env) -> (ZPayContractClient, Address, Address, Address, Address) {
    let contract_id = env.register(ZPayContract, ());
    let client = ZPayContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let nexus_contract = env.register(MockNexus, ());
    let oracle_pub_key = soroban_sdk::BytesN::from_array(&env, &[0; 32]);
    let stork_oracle = env.register(MockStork, ());
    let zpay_treasury = Address::generate(&env);
    let nexus_treasury = Address::generate(&env);
    client.initialize(&admin, &nexus_contract, &oracle_pub_key, &stork_oracle, &100, &10_000_000, &50, &5_000_000, &zpay_treasury, &nexus_treasury);
    (client, admin, nexus_contract, zpay_treasury, nexus_treasury)
}

#[contract] pub struct MockNexus;
#[contractimpl] impl MockNexus {
    pub fn verify_authority(_e: Env, m: u64, a: Address, _c: Address, _f: Symbol, _t: Option<i128>) -> bool {
        if m == 123 {
            let a1: Address = _e.storage().persistent().get(&Symbol::new(&_e, "agent1")).unwrap_or(Address::generate(&_e));
            if a != a1 { return false; }
        }
        true
    }
    pub fn set_agent1(env: Env, agent: Address) { env.storage().persistent().set(&Symbol::new(&env, "agent1"), &agent); }
}
#[contract] pub struct MockStork;
#[contractimpl] impl MockStork {
    pub fn get_temporal_numeric_value_v1(env: Env, _asset_id: BytesN<32>) -> crate::interfaces::TemporalNumericValue {
        crate::interfaces::TemporalNumericValue { quantized_value: 10_000_000, timestamp: env.ledger().timestamp(), publisher_merkle_root: BytesN::from_array(&env, &[0; 32]) }
    }
}
#[contract] pub struct MockFallbackOracle;
#[contractimpl] impl MockFallbackOracle {
    pub fn get_price(_env: Env) -> i128 { 20_000_000 }
}

#[test]
fn test_business_and_security_logic() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, nexus_id, _, _) = init_client(&env);
    let token_id = env.register_stellar_asset_contract_v2(admin.clone()).address();
    client.add_token(&admin, &token_id);
    
    let fallback_id = env.register(MockFallbackOracle, ());
    client.set_fallback_oracle(&admin, &token_id, &fallback_id);

    let user = Address::generate(&env);
    let relayer = Address::generate(&env);
    let attacker = Address::generate(&env);
    soroban_sdk::token::StellarAssetClient::new(&env, &token_id).mint(&user, &2_000_000_000);
    TokenClient::new(&env, &token_id).approve(&user, &client.address, &2_000_000_000, &9999);

    // 1. Success Pay with Relayer (Using Fallback)
    client.pay(&user, &user, &admin, &token_id, &100_000_000, &999, &None, &None, &Some(relayer.clone()), &Some(5_000_000));
    assert_eq!(TokenClient::new(&env, &token_id).balance(&relayer), 5_000_000);

    // 2. Escrow and Refund (Using Fallback)
    env.ledger().with_mut(|li| li.sequence_number = 100);
    let pid = client.pay_escrow(&user, &user, &admin, &token_id, &100_000_000, &999, &None, &None, &10, &None, &None);
    env.ledger().with_mut(|li| li.sequence_number = 110);
    client.refund_escrow(&user, &pid);

    // 3. Security: Spoofing
    MockNexusClient::new(&env, &nexus_id).set_agent1(&user);
    let res_spoof = client.try_pay(&attacker, &user, &admin, &token_id, &100, &123, &None, &None, &None, &None);
    assert!(res_spoof.is_err());

    // 4. Security: Max Relayer Fee
    let res_fee = client.try_pay(&user, &user, &admin, &token_id, &100_000_000, &999, &None, &None, &Some(attacker), &Some(6_000_001));
    assert_eq!(res_fee, Err(Ok(Error::MaxRelayerFeeExceeded)));
}

#[test]
fn test_admin_rotation() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, _, _) = init_client(&env);
    let new_admin = Address::generate(&env);
    
    client.propose_new_admin(&admin, &new_admin);
    client.claim_admin_rights(&new_admin);
    assert_eq!(client.get_admin(), new_admin);
}

#[test]
fn test_pause_logic() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, _, _) = init_client(&env);
    
    client.set_paused(&admin, &true);
    assert!(client.is_paused());
    
    let user = Address::generate(&env);
    let res = client.try_pay(&user, &user, &admin, &admin, &100, &1, &None, &None, &None, &None);
    assert_eq!(res, Err(Ok(Error::ContractPaused)));
}

#[test]
fn test_escrow_refund_timeout() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, _, _) = init_client(&env);
    let token_id = env.register_stellar_asset_contract_v2(admin.clone()).address();
    client.add_token(&admin, &token_id);
    let fallback_id = env.register(MockFallbackOracle, ());
    client.set_fallback_oracle(&admin, &token_id, &fallback_id);

    let user = Address::generate(&env);
    soroban_sdk::token::StellarAssetClient::new(&env, &token_id).mint(&user, &2_000_000_000);
    TokenClient::new(&env, &token_id).approve(&user, &client.address, &2_000_000_000, &9999);

    let pid = client.pay_escrow(&user, &user, &admin, &token_id, &100_000_000, &999, &None, &None, &10, &None, &None);
    
    // Should fail before timeout (ledger 0 + 10 = 10)
    env.ledger().with_mut(|li| li.sequence_number = 5);
    let res_early = client.try_refund_escrow(&user, &pid);
    assert_eq!(res_early, Err(Ok(Error::EscrowNotExpired)));
    
    // Should succeed after timeout
    env.ledger().with_mut(|li| li.sequence_number = 11);
    client.refund_escrow(&user, &pid);
}

#[test]
fn test_subscription_charge() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _nexus_id, _, _) = init_client(&env);
    let token_id = env.register_stellar_asset_contract_v2(admin.clone()).address();
    client.add_token(&admin, &token_id);
    let fallback_id = env.register(MockFallbackOracle, ());
    client.set_fallback_oracle(&admin, &token_id, &fallback_id);

    let user = Address::generate(&env);
    let seller = Address::generate(&env);
    soroban_sdk::token::StellarAssetClient::new(&env, &token_id).mint(&user, &2_000_000_000);
    TokenClient::new(&env, &token_id).approve(&user, &client.address, &2_000_000_000, &9999);

    // Mock authorized seller in Nexus
    // (MockNexus already allows charge if authorized)
    client.charge_subscription(&seller, &user, &token_id, &50_000_000, &999, &None, &None, &None, &None);
    
    assert_eq!(TokenClient::new(&env, &token_id).balance(&seller), 50_000_000);
}

#[test]
fn test_real_signature_verification() {
    use ed25519_dalek::{SigningKey, Signer};
    use rand::thread_rng;

    let env = Env::default();
    env.mock_all_auths();
    
    // 1. Generate real keypair
    let mut rng = thread_rng();
    let signing_key = SigningKey::generate(&mut rng);
    let pub_key_bytes = signing_key.verifying_key().to_bytes();
    let pub_key_n = BytesN::from_array(&env, &pub_key_bytes);

    // 2. Init client with REAL pub key
    let contract_id = env.register(ZPayContract, ());
    let client = ZPayContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let nexus_contract = env.register(MockNexus, ());
    let stork_oracle = env.register(MockStork, ());
    let zpay_treasury = Address::generate(&env);
    let nexus_treasury = Address::generate(&env);
    
    client.initialize(&admin, &nexus_contract, &pub_key_n, &stork_oracle, &0, &0, &0, &0, &zpay_treasury, &nexus_treasury);
    
    let token_id = env.register_stellar_asset_contract_v2(admin.clone()).address();
    client.add_token(&admin, &token_id);

    // 3. Prepare payload for "USD", price 100, timestamp now
    let base_currency = Symbol::new(&env, "USD");
    let price_per_unit: i128 = 100_0000000;
    let timestamp = env.ledger().timestamp();
    
    let mut data_bytes = soroban_sdk::Bytes::new(&env);
    data_bytes.append(&base_currency.clone().to_xdr(&env));
    data_bytes.append(&price_per_unit.to_xdr(&env));
    data_bytes.append(&timestamp.to_xdr(&env));
    
    // Convert Bytes to Vec<u8> for signing
    let mut payload = [0u8; 1024]; // Large enough buffer
    let len = data_bytes.len() as usize;
    data_bytes.copy_into_slice(&mut payload[..len]);
    
    // 4. Sign payload
    let signature_bytes = signing_key.sign(&payload[..len]).to_bytes();
    let signature_n = BytesN::from_array(&env, &signature_bytes);
    
    let ticket = PriceTicket {
        base_currency,
        price_per_unit,
        timestamp,
        signature: signature_n,
    };

    // 5. Execute pay and verify it DOES NOT fail signature check
    let user = Address::generate(&env);
    soroban_sdk::token::StellarAssetClient::new(&env, &token_id).mint(&user, &2_000_000_000);
    TokenClient::new(&env, &token_id).approve(&user, &client.address, &2_000_000_000, &9999);

    client.pay(&user, &user, &admin, &token_id, &100_000_000, &999, &Some(ticket), &None, &None, &None);
}

#[test]
fn test_staleness_rejection() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _, _, _) = init_client(&env);
    client.set_max_staleness(&admin, &300); // 5 minutes
    
    env.ledger().with_mut(|li| li.timestamp = 1000);

    let token_id = env.register_stellar_asset_contract_v2(admin.clone()).address();
    client.add_token(&admin, &token_id);

    let ticket = PriceTicket {
        base_currency: Symbol::new(&env, "USD"),
        price_per_unit: 100,
        timestamp: 699, // 1000 - 301
        signature: BytesN::from_array(&env, &[0; 64]),
    };

    let user = Address::generate(&env);
    let res = client.try_pay(&user, &user, &admin, &token_id, &100, &1, &Some(ticket), &None, &None, &None);
    assert_eq!(res, Err(Ok(Error::OracleStale)));
}
