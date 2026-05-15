#![cfg(test)]

use super::*;
use soroban_sdk::testutils::{Address as _};
use soroban_sdk::{token, Address, Env, vec, Symbol, BytesN};
use nexus::Nexus;
use zolvency_soul::{ZolvencySoulContract, ZolvencySoulContractClient};

#[test]
fn test_full_sovereign_flow() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let agent = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let relayer = Address::generate(&env);
    
    // 1. Deploy Token
    let token_addr = env.register_stellar_asset_contract_v2(token_admin).address();
    let token_client = token::StellarAssetClient::new(&env, &token_addr);
    token_client.mint(&user, &1000);
    let token_standard_client = token::Client::new(&env, &token_addr);

    // 2. Deploy Soul Contract
    let soul_addr = env.register(ZolvencySoulContract, ());
    let soul_client = ZolvencySoulContractClient::new(&env, &soul_addr);
    soul_client.initialize(&admin, &relayer);
    
    // User needs a Soul
    let dummy_pk = BytesN::from_array(&env, &[0u8; 65]);
    soul_client.mint(&relayer, &user, &dummy_pk, &dummy_pk);

    // 3. Deploy Nexus
    let nexus_addr = env.register(Nexus, ());
    let client = nexus_client::NexusClient::new(&env, &nexus_addr);
    client.initialize(&admin, &admin);
    client.set_soul_contract(&admin, &soul_addr);

    // 4. Deploy DirectSovereign
    let ds_addr = env.register(DirectSovereign, ());
    let ds_client = DirectSovereignClient::new(&env, &ds_addr);
    ds_client.initialize(&admin, &nexus_addr);

    // 5. Setup Mandate in Nexus
    let scope = nexus::Scope {
        expiration: env.ledger().timestamp() + 3600,
        transfer_limit: Some(100),
        token: Some(token_addr.clone()),
        renewal_period: Some(30 * 24 * 60 * 60),
        metadata_uri: None,
        scope_commitment: None,
        contract_allowlist: Some(vec![&env, ds_addr.clone()]),
        function_allowlist: Some(vec![&env, Symbol::new(&env, "charge")]),
    };

    let mandate_id = client.issue_mandate_as_admin(
        &user,
        &agent,
        &scope,
        &nexus::DelegationPolicy::None,
        &None,
    );

    // 6. Subscribe
    ds_client.subscribe(&user, &agent, &token_addr, &mandate_id, &100, &12);

    // 7. User grants allowance
    token_standard_client.approve(&user, &ds_addr, &1000, &10000);

    // 8. Agent charges
    ds_client.charge(&mandate_id, &20);

    assert_eq!(token_standard_client.balance(&agent), 20);
    assert_eq!(token_standard_client.balance(&user), 980);

    // 9. Agent charges again
    ds_client.charge(&mandate_id, &30);
    assert_eq!(token_standard_client.balance(&agent), 50);
    assert_eq!(token_standard_client.balance(&user), 950);
}
