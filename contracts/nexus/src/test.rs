use super::*;
use soroban_sdk::{
    contract, contractimpl,
    testutils::Address as _, testutils::Ledger as _, Address, Bytes, BytesN, Env, String, Symbol,
};

// use zolvency_github::{GithubIdentityContract, GithubIdentityContractClient};

#[contract]
pub struct MockSoul;

#[contractimpl]
impl MockSoul {
    pub fn get_soul(_env: Env, soul_id: u32) -> Option<bool> {
        if soul_id == 1 { Some(true) } else { None }
    }
}

#[contract]
pub struct MockWill;

#[contractimpl]
impl MockWill {
    pub fn initialize(_env: Env, _admin: Address, _registry: Address) {}
    pub fn has_soul(_env: Env, _user: Address) -> bool { true }
    pub fn mint(env: Env, human_owner: Address, will: Address, mandate_id: u64, expiry: u64) {
        let data = (human_owner, mandate_id, expiry);
        env.storage().persistent().set(&will, &data);
    }
    pub fn burn(env: Env, _caller: Address, will: Address) {
        env.storage().persistent().remove(&will);
    }
    pub fn get_auth(env: Env, will: Address) -> (Address, u64, u64) {
        env.storage().persistent().get(&will).unwrap()
    }
}

#[contract]
pub struct MockAdapter;

#[contractimpl]
impl MockAdapter {
    pub fn export_reputation(env: Env, _user: Address, soul_id: u32, _params: CrossChainParams) {
        env.storage().persistent().set(&Symbol::new(&env, "last_reputation"), &soul_id);
    }
    pub fn send_will_auth(env: Env, _user: Address, _chain: String, _dest: String, _user_dest: Bytes, soul_id: u32, permissions: u64, expiry: u64, ecosystem: Ecosystem) {
        let data = (_user, soul_id, permissions, expiry, ecosystem);
        env.storage().persistent().set(&Symbol::new(&env, "last_will"), &data);
    }
    pub fn get_last_will_auth(env: Env) -> (Address, u32, u64, u64, Ecosystem) {
        env.storage().persistent().get(&Symbol::new(&env, "last_will")).unwrap()
    }
    pub fn send_reputation(
        env: Env,
        _caller: Address,
        _dest_chain: String,
        _dest_addr: String,
        _soul_id: u32,
        _ext_id: String,
        _tier: u32,
        _user_dest: Bytes,
        nonce: u64,
        _token_type: Symbol,
        ecosystem: Ecosystem,
    ) {
        let data = (nonce, ecosystem);
        env.storage().persistent().set(&Symbol::new(&env, "last_rep_send"), &data);
    }
    pub fn get_last_rep_send(env: Env) -> (u64, Ecosystem) {
        env.storage().persistent().get(&Symbol::new(&env, "last_rep_send")).unwrap()
    }
}

/*
#[test]
fn test_complete_will_lifecycle_interactions() {
...
    let res = will_client.try_get_auth(&will_x);
    assert!(res.is_err());
}
*/

#[test]
fn test_will_auth_expiry() {
    let env = Env::default();
    env.mock_all_auths();

    let registry_id = env.register(Nexus, ());
    let registry_client = NexusClient::new(&env, &registry_id);
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    registry_client.initialize(&admin, &signer);
    
    let will_id = env.register(MockWill, ());
    registry_client.set_will_contract(&admin, &will_id);

    let user = Address::generate(&env);
    let will = Address::generate(&env);
    let duration = 3600;


    env.ledger().set_timestamp(1000);
    let mandate_id = registry_client.issue_mandate(
        &user,
        &will,
        &Scope {
            ttl: env.ledger().timestamp() + duration,
            transfer_limit: None,
            renewal_period: None,
            scope_commitment: None,
            contract_allowlist: None,
            function_allowlist: None,
        },
        &DelegationPolicy::None,
        &None,
    );


    env.ledger().set_timestamp(2000);
    let adapter_id = env.register(MockAdapter, ());
    registry_client.set_interop_config(&admin, &InteropConfig {
        active_protocol: InteropProtocol::Axelar,
        adapter_address: adapter_id.clone(),
    });

    let cc_params = CrossChainParams {
        destination_chain: String::from_str(&env, "eth"),
        destination_address: String::from_str(&env, "0x123"),
        user_destination_address: Bytes::from_array(&env, &[0u8; 20]),
        ecosystem: Ecosystem::Evm,
    };


    registry_client.export_will_authority(&user, &will, &cc_params);


    env.ledger().set_timestamp(5000);
    
    // Verify authority fails after expiry
    let res = registry_client.verify_authority(&mandate_id, &will, &Address::generate(&env), &Symbol::new(&env, "any"), &None);
    assert!(!res);
}

#[test]
fn test_will_permission_masking() {
    let env = Env::default();
    env.mock_all_auths();

    let registry_id = env.register(Nexus, ());
    let registry_client = NexusClient::new(&env, &registry_id);
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    registry_client.initialize(&admin, &signer);
    
    let will_id = env.register(MockWill, ());
    registry_client.set_will_contract(&admin, &will_id);

    let user = Address::generate(&env);
    let will_read = Address::generate(&env);
    let will_write = Address::generate(&env);

    registry_client.issue_mandate(&user, &will_read, &Scope { ttl: env.ledger().timestamp() + 3600, transfer_limit: None, renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None }, &DelegationPolicy::None, &None);
    registry_client.issue_mandate(&user, &will_write, &Scope { ttl: env.ledger().timestamp() + 3600, transfer_limit: None, renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None }, &DelegationPolicy::None, &None);
}

#[test]
fn test_delegation_chain_and_revocation() {
    let env = Env::default();
    env.mock_all_auths();

    let registry_id = env.register(Nexus, ());
    let registry_client = NexusClient::new(&env, &registry_id);
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    registry_client.initialize(&admin, &signer);
    
    let will_id = env.register(MockWill, ());
    registry_client.set_will_contract(&admin, &will_id);

    let user_a = Address::generate(&env);
    let agent_b = Address::generate(&env);
    let agent_c = Address::generate(&env);


    let mandate_b_id = registry_client.issue_mandate(
        &user_a,
        &agent_b,
        &Scope { ttl: env.ledger().timestamp() + 3600, transfer_limit: None, renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None },
        &DelegationPolicy::Full,
        &None,
    );


    let mandate_c_id = registry_client.issue_mandate(
        &agent_b,
        &agent_c,
        &Scope { ttl: env.ledger().timestamp() + 1800, transfer_limit: None, renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None },
        &DelegationPolicy::None,
        &Some(mandate_b_id),
    );


    let auth_c = registry_client.verify_authority(&mandate_c_id, &agent_c, &Address::generate(&env), &Symbol::new(&env, "any"), &None);
    assert!(auth_c);


    registry_client.revoke_mandate(&user_a, &mandate_b_id);


    let auth_c_post = registry_client.verify_authority(&mandate_c_id, &agent_c, &Address::generate(&env), &Symbol::new(&env, "any"), &None);
    assert!(!auth_c_post);
}

#[test]
fn test_pro_deep_inheritance_verification() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    let nexus_id = env.register(Nexus, ());
    let nexus_client = NexusClient::new(&env, &nexus_id);
    nexus_client.initialize(&admin, &signer);

    let will_id = env.register(MockWill, ());
    nexus_client.set_will_contract(&admin, &will_id);

    let root = Address::generate(&env);
    let mut current_issuer = root.clone();
    let mut last_mandate_id = None;

    // Build a chain of exactly 8 mandates
    for _ in 0..8 {
        let agent = Address::generate(&env);
        let mid = nexus_client.issue_mandate(
            &current_issuer,
            &agent,
            &Scope { 
                ttl: env.ledger().timestamp() + 3600, 
                transfer_limit: Some(1000), 
                renewal_period: None, 
                scope_commitment: None, 
                contract_allowlist: None, 
                function_allowlist: None 
            },
            &DelegationPolicy::Full,
            &last_mandate_id,
        );
        current_issuer = agent;
        last_mandate_id = Some(mid);
    }

    let leaf_mandate = last_mandate_id.unwrap();
    let leaf_agent = current_issuer;

    // Verify authority at depth 8
    assert!(nexus_client.verify_authority(&leaf_mandate, &leaf_agent, &Address::generate(&env), &Symbol::new(&env, "any"), &Some(100)));

    // Verify another spend to ensure budget propagates correctly up to root
    assert!(nexus_client.verify_authority(&leaf_mandate, &leaf_agent, &Address::generate(&env), &Symbol::new(&env, "any"), &Some(900)));

    // Exceed budget at depth 8
    assert!(!nexus_client.verify_authority(&leaf_mandate, &leaf_agent, &Address::generate(&env), &Symbol::new(&env, "any"), &Some(1)));
}

#[test]
fn test_increment_epoch_impact() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    let nexus_id = env.register(Nexus, ());
    let nexus_client = NexusClient::new(&env, &nexus_id);
    
    nexus_client.initialize(&admin, &signer);
    
    let will_id = env.register(MockWill, ());
    nexus_client.set_will_contract(&admin, &will_id);
    
    let root_anchor = Address::generate(&env);
    let agent = Address::generate(&env);
    
    let mandate_id = nexus_client.issue_mandate(
        &root_anchor,
        &agent,
        &Scope { ttl: env.ledger().timestamp() + 3600, transfer_limit: None, renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None },
        &DelegationPolicy::None,
        &None,
    );
    
    assert!(nexus_client.verify_authority(&mandate_id, &agent, &Address::generate(&env), &Symbol::new(&env, "any"), &None));
    
    // Increment epoch
    nexus_client.increment_epoch(&root_anchor);
    
    // Mandate should now be invalid
    assert!(!nexus_client.verify_authority(&mandate_id, &agent, &Address::generate(&env), &Symbol::new(&env, "any"), &None));
}

#[test]
fn test_verify_authority_allowlists() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    let nexus_id = env.register(Nexus, ());
    let nexus_client = NexusClient::new(&env, &nexus_id);
    nexus_client.initialize(&admin, &signer);
    
    let will_id = env.register(MockWill, ());
    nexus_client.set_will_contract(&admin, &will_id);

    let root_anchor = Address::generate(&env);
    let agent = Address::generate(&env);
    let allowed_contract = Address::generate(&env);
    let denied_contract = Address::generate(&env);
    let allowed_fn = Symbol::new(&env, "pay");
    let denied_fn = Symbol::new(&env, "refund");

    let allowed_id = nexus_client.issue_mandate(
        &root_anchor,
        &agent,
        &Scope {
            ttl: env.ledger().timestamp() + 3600,
            transfer_limit: None,
            renewal_period: None,
            scope_commitment: None,
            contract_allowlist: Some(soroban_sdk::vec![&env, allowed_contract.clone()]),
            function_allowlist: Some(soroban_sdk::vec![&env, allowed_fn.clone()]),
        },
        &DelegationPolicy::None,
        &None,
    );

    let denied_contract_id = nexus_client.issue_mandate(
        &root_anchor,
        &agent,
        &Scope {
            ttl: env.ledger().timestamp() + 3600,
            transfer_limit: None,
            renewal_period: None,
            scope_commitment: None,
            contract_allowlist: Some(soroban_sdk::vec![&env, allowed_contract.clone()]),
            function_allowlist: Some(soroban_sdk::vec![&env, allowed_fn.clone()]),
        },
        &DelegationPolicy::None,
        &None,
    );

    let denied_function_id = nexus_client.issue_mandate(
        &root_anchor,
        &agent,
        &Scope {
            ttl: env.ledger().timestamp() + 3600,
            transfer_limit: None,
            renewal_period: None,
            scope_commitment: None,
            contract_allowlist: Some(soroban_sdk::vec![&env, allowed_contract.clone()]),
            function_allowlist: Some(soroban_sdk::vec![&env, allowed_fn.clone()]),
        },
        &DelegationPolicy::None,
        &None,
    );

    assert!(nexus_client.verify_authority(&allowed_id, &agent, &allowed_contract, &allowed_fn, &None));
    assert!(!nexus_client.verify_authority(&denied_contract_id, &agent, &denied_contract, &allowed_fn, &None));
    assert!(!nexus_client.verify_authority(&denied_function_id, &agent, &allowed_contract, &denied_fn, &None));
}

#[test]
fn test_budget_fraction_enforcement() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    let nexus_id = env.register(Nexus, ());
    let nexus_client = NexusClient::new(&env, &nexus_id);
    nexus_client.initialize(&admin, &signer);
    
    let will_id = env.register(MockWill, ());
    nexus_client.set_will_contract(&admin, &will_id);
    
    let user = Address::generate(&env);
    let agent_b = Address::generate(&env);
    let agent_c = Address::generate(&env);
    
    let mandate_b_id = nexus_client.issue_mandate(
        &user,
        &agent_b,
        &Scope { ttl: env.ledger().timestamp() + 3600, transfer_limit: Some(1000), renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None },
        &DelegationPolicy::Restricted(DelegationRules {
            max_subdepth: 2,
            allowed_scope_tags: None,
            budget_fraction: Some(50), // 50%
        }),
        &None,
    );
    
    // 50% of 1000 is 500. Try to issue 600.
    let res = nexus_client.try_issue_mandate(
        &agent_b,
        &agent_c,
        &Scope { ttl: env.ledger().timestamp() + 3600, transfer_limit: Some(600), renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None },
        &DelegationPolicy::None,
        &Some(mandate_b_id),
    );
    
    assert!(res.is_err());
    
    // 500 should work
    nexus_client.issue_mandate(
        &agent_b,
        &agent_c,
        &Scope { ttl: env.ledger().timestamp() + 3600, transfer_limit: Some(500), renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None },
        &DelegationPolicy::None,
        &Some(mandate_b_id),
    );
}

#[test]
fn test_sum_child_budget_enforcement() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    let nexus_id = env.register(Nexus, ());
    let nexus_client = NexusClient::new(&env, &nexus_id);
    nexus_client.initialize(&admin, &signer);
    
    let will_id = env.register(MockWill, ());
    nexus_client.set_will_contract(&admin, &will_id);
    
    let user = Address::generate(&env);
    let agent_b = Address::generate(&env);
    let agent_c1 = Address::generate(&env);
    let agent_c2 = Address::generate(&env);
    
    let mandate_b_id = nexus_client.issue_mandate(
        &user,
        &agent_b,
        &Scope { ttl: env.ledger().timestamp() + 3600, transfer_limit: Some(1000), renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None },
        &DelegationPolicy::Full,
        &None,
    );
    
    // Issue 600 to C1
    nexus_client.issue_mandate(
        &agent_b,
        &agent_c1,
        &Scope { ttl: env.ledger().timestamp() + 3600, transfer_limit: Some(600), renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None },
        &DelegationPolicy::None,
        &Some(mandate_b_id),
    );
    
    // Try to issue another 600 to C2. Sum (1200) > Parent (1000).
    let res = nexus_client.try_issue_mandate(
        &agent_b,
        &agent_c2,
        &Scope { ttl: env.ledger().timestamp() + 3600, transfer_limit: Some(600), renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None },
        &DelegationPolicy::None,
        &Some(mandate_b_id),
    );
    
    assert!(res.is_err());
}

#[test]
fn test_remote_mandate_issuance() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    let nexus_id = env.register(Nexus, ());
    let nexus_client = NexusClient::new(&env, &nexus_id);
    nexus_client.initialize(&admin, &signer);
    
    let will_id = env.register(MockWill, ());
    nexus_client.set_will_contract(&admin, &will_id);
    
    let root_anchor = Address::generate(&env);
    let agent = Address::generate(&env);
    
    let request = MandateRequest {
        root_anchor: root_anchor.clone(),
        agent: agent.clone(),
        scope: Scope {
            ttl: env.ledger().timestamp() + 3600,
            transfer_limit: Some(1000),
            renewal_period: None,
            scope_commitment: None,
            contract_allowlist: None,
            function_allowlist: None,
        },
        delegation_policy: DelegationPolicy::None,
        epoch: 0,
        nonce: BytesN::from_array(&env, &[1u8; 32]),
        sep45_signature: BytesN::from_array(&env, &[0u8; 64]),
    };
    
    let mandate_id = nexus_client.issue_mandate_remote(&request);
    
    assert_eq!(mandate_id, 1);
    
    // Try to replay the same request (same nonce)
    let res = nexus_client.try_issue_mandate_remote(&request);
    assert!(res.is_err());
    
    // Try with different nonce but wrong epoch
    let mut request_wrong_epoch = request.clone();
    request_wrong_epoch.nonce = BytesN::from_array(&env, &[2u8; 32]);
    request_wrong_epoch.epoch = 1;
    
    let res_epoch = nexus_client.try_issue_mandate_remote(&request_wrong_epoch);
    assert!(res_epoch.is_err());
    }

    #[test]
    fn test_scope_tag_restrictions() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    let nexus_id = env.register(Nexus, ());
    let nexus_client = NexusClient::new(&env, &nexus_id);
    nexus_client.initialize(&admin, &signer);
    
    let will_id = env.register(MockWill, ());
    nexus_client.set_will_contract(&admin, &will_id);

    let user = Address::generate(&env);
    let agent_b = Address::generate(&env);
    let agent_c = Address::generate(&env);

    // Parent restricts children to only TransferLimit
    let mandate_b_id = nexus_client.issue_mandate(
        &user,
        &agent_b,
        &Scope { ttl: env.ledger().timestamp() + 3600, transfer_limit: Some(1000), renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None },
        &DelegationPolicy::Restricted(DelegationRules {
            max_subdepth: 2,
            allowed_scope_tags: Some(soroban_sdk::vec![&env, ScopeTag::TransferLimit]),
            budget_fraction: None,
        }),
        &None,
    );

    // Try to issue a mandate with contract_allowlist (violates allowed_scope_tags)
    let res = nexus_client.try_issue_mandate(
        &agent_b,
        &agent_c,
        &Scope { 
            ttl: env.ledger().timestamp() + 3600, 
            transfer_limit: Some(500), 
            renewal_period: None,
            scope_commitment: None, 
            contract_allowlist: Some(soroban_sdk::vec![&env, Address::generate(&env)]), 
            function_allowlist: None 
        },
        &DelegationPolicy::None,
        &Some(mandate_b_id),
    );

    assert!(res.is_err());
    }

    #[test]
    fn test_max_delegation_depth() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    let nexus_id = env.register(Nexus, ());
    let nexus_client = NexusClient::new(&env, &nexus_id);
    nexus_client.initialize(&admin, &signer);
    
    let will_id = env.register(MockWill, ());
    nexus_client.set_will_contract(&admin, &will_id);

    let mut current_issuer = Address::generate(&env);
    let mut last_mandate_id: Option<u64> = None;

    // Create a chain of 8 mandates (depth 0 to 7)
    for _ in 0..8 {
        let agent = Address::generate(&env);
        let mid = nexus_client.issue_mandate(
            &current_issuer,
            &agent,
            &Scope { ttl: env.ledger().timestamp() + 3600, transfer_limit: None, renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None },
            &DelegationPolicy::Full,
            &last_mandate_id,
        );
        current_issuer = agent;
        last_mandate_id = Some(mid);
    }

    // Now depth is 8 (Root Anchor is 0, so the 8th mandate is at depth 7).
    // Wait, let's check internal logic: Root anchor is 0, first mandate is depth 0?
    // In lib.rs: depth = parent.depth + 1; Root Anchor = depth 0.
    // So:
    // M1: issuer=Root, parent=None, depth=0
    // M2: issuer=M1.agent, parent=M1, depth=1
    // ...
    // M8: depth=7.
    // M9: depth=8.

    // Try to issue the 9th mandate (depth 8) - this should fail if MAX_DELEGATION_DEPTH is 8?
    // No, MAX_DELEGATION_DEPTH is 8. So depth 8 is allowed, depth 9 is not.

    let agent_9 = Address::generate(&env);
    let mid_9 = nexus_client.issue_mandate(
        &current_issuer,
        &agent_9,
        &Scope { ttl: env.ledger().timestamp() + 3600, transfer_limit: None, renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None },
        &DelegationPolicy::Full,
        &last_mandate_id,
    );

    // Try to issue the 10th mandate (depth 9)
    let res = nexus_client.try_issue_mandate(
        &agent_9,
        &Address::generate(&env),
        &Scope { ttl: env.ledger().timestamp() + 3600, transfer_limit: None, renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None },
        &DelegationPolicy::Full,
        &Some(mid_9),
    );

    assert!(res.is_err());
    }

    #[test]
    fn test_verification_cache_hit() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    let nexus_id = env.register(Nexus, ());
    let nexus_client = NexusClient::new(&env, &nexus_id);
    nexus_client.initialize(&admin, &signer);
    
    let will_id = env.register(MockWill, ());
    nexus_client.set_will_contract(&admin, &will_id);

    let user = Address::generate(&env);
    let agent = Address::generate(&env);

    let mandate_id = nexus_client.issue_mandate(
        &user,
        &agent,
        &Scope { ttl: env.ledger().timestamp() + 3600, transfer_limit: Some(1000), renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None },
        &DelegationPolicy::None,
        &None,
    );

    // First verification (Cache miss)
    assert!(nexus_client.verify_authority(&mandate_id, &agent, &Address::generate(&env), &Symbol::new(&env, "any"), &Some(100)));

    // Second verification (Cache hit)
    assert!(nexus_client.verify_authority(&mandate_id, &agent, &Address::generate(&env), &Symbol::new(&env, "any"), &Some(100)));

    // Verify budget was updated twice
    let state = nexus_client.verify_authority(&mandate_id, &agent, &Address::generate(&env), &Symbol::new(&env, "any"), &Some(801));
    assert!(!state); // Should fail because 100+100+801 > 1000
    }

    #[test]
    fn test_verification_cache_invalidation_on_epoch() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    let nexus_id = env.register(Nexus, ());
    let nexus_client = NexusClient::new(&env, &nexus_id);
    nexus_client.initialize(&admin, &signer);
    
    let will_id = env.register(MockWill, ());
    nexus_client.set_will_contract(&admin, &will_id);

    let user = Address::generate(&env);
    let agent = Address::generate(&env);

    let mandate_id = nexus_client.issue_mandate(
        &user,
        &agent,
        &Scope { ttl: env.ledger().timestamp() + 3600, transfer_limit: None, renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None },
        &DelegationPolicy::None,
        &None,
    );

    // Verify to populate cache
    assert!(nexus_client.verify_authority(&mandate_id, &agent, &Address::generate(&env), &Symbol::new(&env, "any"), &None));

    // Increment epoch
    nexus_client.increment_epoch(&user);

    // Verify again - should be invalid even if cache exists for old epoch
    assert!(!nexus_client.verify_authority(&mandate_id, &agent, &Address::generate(&env), &Symbol::new(&env, "any"), &None));
    }

    #[test]
    fn test_cascading_revocation_deep() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    let nexus_id = env.register(Nexus, ());
    let nexus_client = NexusClient::new(&env, &nexus_id);
    nexus_client.initialize(&admin, &signer);
    
    let will_id = env.register(MockWill, ());
    nexus_client.set_will_contract(&admin, &will_id);

    let root = Address::generate(&env);
    let agent_1 = Address::generate(&env);
    let agent_2 = Address::generate(&env);
    let agent_3 = Address::generate(&env);

    let m1 = nexus_client.issue_mandate(&root, &agent_1, &Scope { ttl: env.ledger().timestamp() + 3600, transfer_limit: None, renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None }, &DelegationPolicy::Full, &None);
    let m2 = nexus_client.issue_mandate(&agent_1, &agent_2, &Scope { ttl: env.ledger().timestamp() + 3600, transfer_limit: None, renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None }, &DelegationPolicy::Full, &Some(m1));
    let m3 = nexus_client.issue_mandate(&agent_2, &agent_3, &Scope { ttl: env.ledger().timestamp() + 3600, transfer_limit: None, renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None }, &DelegationPolicy::None, &Some(m2));

    assert!(nexus_client.verify_authority(&m3, &agent_3, &Address::generate(&env), &Symbol::new(&env, "any"), &None));

    // Revoke m1
    nexus_client.revoke_mandate(&root, &m1);

    // m3 should be invalid
    assert!(!nexus_client.verify_authority(&m3, &agent_3, &Address::generate(&env), &Symbol::new(&env, "any"), &None));
    }

    #[test]
    fn test_child_scope_violations() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    let nexus_id = env.register(Nexus, ());
    let nexus_client = NexusClient::new(&env, &nexus_id);
    nexus_client.initialize(&admin, &signer);
    
    let will_id = env.register(MockWill, ());
    nexus_client.set_will_contract(&admin, &will_id);

    let user = Address::generate(&env);
    let agent_b = Address::generate(&env);
    let agent_c = Address::generate(&env);

    let mandate_b_id = nexus_client.issue_mandate(
        &user,
        &agent_b,
        &Scope { ttl: 1000, transfer_limit: Some(1000), renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None },
        &DelegationPolicy::Full,
        &None,
    );

    // Violation 1: Child TTL > Parent TTL
    let res_ttl = nexus_client.try_issue_mandate(
        &agent_b,
        &agent_c,
        &Scope { ttl: 1001, transfer_limit: Some(500), renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None },
        &DelegationPolicy::None,
        &Some(mandate_b_id),
    );
    assert!(res_ttl.is_err());

    // Violation 2: Child transfer_limit > Parent transfer_limit
    let res_limit = nexus_client.try_issue_mandate(
        &agent_b,
        &agent_c,
        &Scope { ttl: 1000, transfer_limit: Some(1001), renewal_period: None, scope_commitment: None, contract_allowlist: None, function_allowlist: None },
        &DelegationPolicy::None,
        &Some(mandate_b_id),
    );
    assert!(res_limit.is_err());
    }

    #[test]
    fn test_recurring_budget_reset() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let signer = Address::generate(&env);
        let nexus_id = env.register(Nexus, ());
        let nexus_client = NexusClient::new(&env, &nexus_id);
        nexus_client.initialize(&admin, &signer);
    
    let will_id = env.register(MockWill, ());
    nexus_client.set_will_contract(&admin, &will_id);

        let user = Address::generate(&env);
        let agent = Address::generate(&env);
        let contract = Address::generate(&env);

        // Period: 1 hour (3600s)
        let mandate_id = nexus_client.issue_mandate(
            &user,
            &agent,
            &Scope { 
                ttl: env.ledger().timestamp() + 86400, 
                transfer_limit: Some(100), 
                renewal_period: Some(3600), 
                scope_commitment: None, 
                contract_allowlist: None, 
                function_allowlist: None 
            },
            &DelegationPolicy::None,
            &None,
        );

        // 1. Spend 100 (Full budget)
        assert!(nexus_client.verify_authority(&mandate_id, &agent, &contract, &Symbol::new(&env, "any"), &Some(100)));
        
        // 2. Spend 1 (Fails)
        assert!(!nexus_client.verify_authority(&mandate_id, &agent, &contract, &Symbol::new(&env, "any"), &Some(1)));

        // 3. Move time forward by 1 hour
        env.ledger().with_mut(|li| li.timestamp += 3601);

        // 4. Spend 1 (Succeeds - Budget reset)
        assert!(nexus_client.verify_authority(&mandate_id, &agent, &contract, &Symbol::new(&env, "any"), &Some(1)));
        
        // 5. Spend another 100 (Fails - new period budget exceeded)
        assert!(!nexus_client.verify_authority(&mandate_id, &agent, &contract, &Symbol::new(&env, "any"), &Some(100)));

        // 6. Move time forward by another hour
        env.ledger().with_mut(|li| li.timestamp += 3601);

        // 7. Spend 100 (Succeeds - Second reset)
        assert!(nexus_client.verify_authority(&mandate_id, &agent, &contract, &Symbol::new(&env, "any"), &Some(100)));
    }
