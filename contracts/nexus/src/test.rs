use super::*;
use soroban_sdk::{
    contract, contractimpl,
    testutils::Address as _, testutils::Ledger as _, Address, Bytes, BytesN, Env, String, Symbol,
};

use zolvency_github::{GithubIdentityContract, GithubIdentityContractClient};

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
    pub fn mint(env: Env, human_owner: Address, will: Address, permissions: u64, expiry: u64) {
        let data = (human_owner, permissions, expiry);
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
    pub fn send_will_auth(env: Env, _user: Address, _chain: String, _dest: String, _user_dest: Bytes, soul_id: u32, permissions: u64, expiry: u64) {
        let data = (_user, soul_id, permissions, expiry);
        env.storage().persistent().set(&Symbol::new(&env, "last_will"), &data);
    }
    pub fn get_last_will_auth(env: Env) -> (Address, u32, u64, u64) {
        env.storage().persistent().get(&Symbol::new(&env, "last_will")).unwrap()
    }
}

#[test]
fn test_complete_will_lifecycle_interactions() {
    let env = Env::default();
    env.mock_all_auths();

    // 1. Setup Hub (Registry)
    let registry_id = env.register(Nexus, ());
    let registry_client = NexusClient::new(&env, &registry_id);
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    registry_client.initialize(&admin, &signer);

    // 2. Setup Will (Sub-SBT) Contract
    let will_contract_id = env.register(MockWill, ());
    let will_client = MockWillClient::new(&env, &will_contract_id);
    will_client.initialize(&admin, &registry_id);
    registry_client.set_will_contract(&admin, &will_contract_id);

    // 3. Setup x402 (Fees & Treasury)
    let treasury = Address::generate(&env);
    let fee_token_id = env.register_stellar_asset_contract(admin.clone());
    let fee_token_admin = soroban_sdk::token::StellarAssetClient::new(&env, &fee_token_id);
    let fee_token = soroban_sdk::token::Client::new(&env, &fee_token_id);
    
    registry_client.set_fee_config(&admin, &FeeConfig {
        token: fee_token_id.clone(),
        amount: 50, 
    });
    registry_client.set_treasury(&admin, &treasury);

    // 4. Setup Spokes (Reputation)
    let soul_mock_id = env.register(MockSoul, ());
    let github_id = env.register(GithubIdentityContract, ());
    let github_client = GithubIdentityContractClient::new(&env, &github_id);
    github_client.initialize(&admin, &registry_id, &soul_mock_id, &fee_token_id, &Address::generate(&env), &treasury, &0);
    registry_client.register_token(&admin, &github_id);

    // 5. Setup Identity (User A)
    let user_a = Address::generate(&env);
    let soul_id = 1u32;
    let params = zolvency_github::MintParams {
        username: String::from_str(&env, "will_master"),
        external_id: String::from_str(&env, "gh_master"),
        contributions: 5000u32,
        nonce: 0u64,
        proof: zolvency_github::ReclaimProof {
            claim_info: zolvency_github::ClaimInfo {
                provider: String::from_str(&env, "github"),
                parameters: String::from_str(&env, "gh_master"),
                context: String::from_str(&env, "1"),
            },
            signed_claim: BytesN::from_array(&env, &[0u8; 32]),
            signatures: soroban_sdk::vec![&env, BytesN::from_array(&env, &[0u8; 64])],
            witness_address: BytesN::from_array(&env, &[0u8; 32]),
        },
    };
    github_client.mint(&user_a, &soul_id, &params, &None);

    // 6. Interaction 1: Will Authorization (Minting Sub-SBT)
    let will_x = Address::generate(&env);
    let permissions = 0b1010;
    let duration = 3600;
    
    registry_client.authorize_will(&user_a, &will_x, &permissions, &duration);

    // Verify Will was minted to Agent X
    let (got_owner, got_perm, _got_exp) = will_client.get_auth(&will_x);
    assert_eq!(got_perm, permissions);
    assert_eq!(got_owner, user_a);

    // 7. Interaction 2: Zenith Discovery (Zenith query)
    let zenith = registry_client.get_zenith(&soul_id);
    assert!(zenith.contains_key(Symbol::new(&env, "github")));

    // 8. Interaction 3: Programmatic Export (x402 Flow)
    fee_token_admin.mint(&user_a, &1000);
    
    let adapter_id = env.register(MockAdapter, ());
    let adapter_client = MockAdapterClient::new(&env, &adapter_id);
    registry_client.set_interop_config(&admin, &InteropConfig {
        active_protocol: InteropProtocol::Axelar,
        adapter_address: adapter_id.clone(),
    });

    let cc_params = CrossChainParams {
        destination_chain: String::from_str(&env, "arbitrum"),
        destination_address: String::from_str(&env, "0xWillSpoke"),
        user_destination_address: Bytes::from_array(&env, &[0u8; 20]),
    };

    registry_client.export_will_authority(&user_a, &will_x, &cc_params);

    // Verify Fee Deduction
    assert_eq!(fee_token.balance(&user_a), 950);
    assert_eq!(fee_token.balance(&treasury), 50);

    // Verify Adapter Call
    let (_got_user, got_soul, got_p, _got_e) = adapter_client.get_last_will_auth();
    assert_eq!(got_soul, 1);
    assert_eq!(got_p, permissions);

    // 9. Interaction 4: Revocation (Burning Will)
    registry_client.revoke_will(&user_a, &will_x);

    // Verify Will was burned
    let res = will_client.try_get_auth(&will_x);
    assert!(res.is_err());
}

#[test]
fn test_will_auth_expiry() {
    let env = Env::default();
    env.mock_all_auths();

    let registry_id = env.register(Nexus, ());
    let registry_client = NexusClient::new(&env, &registry_id);
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    registry_client.initialize(&admin, &signer);

    let user = Address::generate(&env);
    let will = Address::generate(&env);
    let permissions = 1;
    let duration = 3600; // 1 hour

    // Start at timestamp 1000
    env.ledger().set_timestamp(1000);
    registry_client.authorize_will(&user, &will, &permissions, &duration);

    // Should work at T=2000 (within duration)
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
    };

    // This should succeed
    registry_client.export_will_authority(&user, &will, &cc_params);

    // Expire at T=5000 (1000 + 3600 = 4600 is expiry)
    env.ledger().set_timestamp(5000);
    
    let res = registry_client.try_export_will_authority(&user, &will, &cc_params);
    assert!(res.is_err());
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

    let user = Address::generate(&env);
    let will_read = Address::generate(&env);
    let will_write = Address::generate(&env);

    registry_client.authorize_will(&user, &will_read, &0b0001, &3600);
    registry_client.authorize_will(&user, &will_write, &0b0011, &3600);
}

#[test]
fn test_security_soul_lock_impact() {
    let env = Env::default();
    env.mock_all_auths();

    let registry_id = env.register(Nexus, ());
    let registry_client = NexusClient::new(&env, &registry_id);
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    registry_client.initialize(&admin, &signer);

    let user = Address::generate(&env);
    let will = Address::generate(&env);
    let soul_id = 1;

    registry_client.authorize_will(&user, &will, &1, &3600);

    // Lock the soul for 1 year
    let unlock_at = env.ledger().timestamp() + 31_536_000;
    registry_client.lock_soul_reputation(&admin, &soul_id, &unlock_at);

    assert!(registry_client.is_soul_locked(&soul_id));

    // Try to export reputation while locked -> Should Fail
    let github_id = env.register(GithubIdentityContract, ());
    registry_client.register_token(&admin, &github_id);

    let cc_params = Some(CrossChainParams {
        destination_chain: String::from_str(&env, "base"),
        destination_address: String::from_str(&env, "0xBase"),
        user_destination_address: Bytes::from_array(&env, &[0u8; 20]),
    });

    let res = registry_client.try_export_reputation(
        &user, 
        &soul_id, 
        &github_id, 
        &String::from_str(&env, "ext_id"), 
        &1, 
        &0, 
        &cc_params
    );

    assert!(res.is_err()); // Error::SoulBlocked
}

#[test]
fn test_unauthorized_spoke_prevention() {
    let env = Env::default();
    env.mock_all_auths();

    let registry_id = env.register(Nexus, ());
    let registry_client = NexusClient::new(&env, &registry_id);
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    registry_client.initialize(&admin, &signer);

    let user = Address::generate(&env);
    let fake_token = Address::generate(&env); // Not registered

    let res = registry_client.try_export_reputation(
        &user, 
        &1, 
        &fake_token, 
        &String::from_str(&env, "ext_id"), 
        &1, 
        &0, 
        &None
    );

    assert!(res.is_err()); // Error::NotAdmin
}

#[test]
fn test_multi_chain_export() {
    let env = Env::default();
    env.mock_all_auths();

    let registry_id = env.register(Nexus, ());
    let registry_client = NexusClient::new(&env, &registry_id);
    let admin = Address::generate(&env);
    let signer = Address::generate(&env);
    registry_client.initialize(&admin, &signer);

    let adapter_id = env.register(MockAdapter, ());
    let adapter_client = MockAdapterClient::new(&env, &adapter_id);
    registry_client.set_interop_config(&admin, &InteropConfig {
        active_protocol: InteropProtocol::Axelar,
        adapter_address: adapter_id.clone(),
    });

    let user = Address::generate(&env);
    let will = Address::generate(&env);
    registry_client.authorize_will(&user, &will, &1, &3600);

    // Export to Chain A
    let cc_a = CrossChainParams {
        destination_chain: String::from_str(&env, "arbitrum"),
        destination_address: String::from_str(&env, "0xWillSpoke"),
        user_destination_address: Bytes::from_array(&env, &[0u8; 20]),
    };
    registry_client.export_will_authority(&user, &will, &cc_a);
    let (_, got_soul, _, _) = adapter_client.get_last_will_auth();
    assert_eq!(got_soul, 1);

    // Export to Chain B
    let cc_b = CrossChainParams {
        destination_chain: String::from_str(&env, "polygon"),
        destination_address: String::from_str(&env, "0xPoly"),
        user_destination_address: Bytes::from_array(&env, &[0u8; 20]),
    };
    registry_client.export_will_authority(&user, &will, &cc_b);
    let (_, got_soul_b, _, _) = adapter_client.get_last_will_auth();
    assert_eq!(got_soul_b, 1);
}
