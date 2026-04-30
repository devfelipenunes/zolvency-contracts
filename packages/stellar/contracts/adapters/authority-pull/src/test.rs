#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Events as _;
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String, Symbol, FromVal};

#[test]
fn test_authority_pull_send_event() {
    let env = Env::default();
    env.mock_all_auths();

    let adapter_id = env.register(AuthorityPullAdapter, ());
    let client = AuthorityPullAdapterClient::new(&env, &adapter_id);

    let caller = Address::generate(&env);
    let dest_chain = String::from_str(&env, "ethereum");
    let dest_addr = String::from_str(&env, "0x123");
    let ext_id = String::from_str(&env, "id_1");
    let tier = 1u32;
    let user_evm = Bytes::from_array(&env, &[0xAA; 20]);
    let nonce = 1u64;

    client.send(&caller, &dest_chain, &dest_addr, &ext_id, &tier, &user_evm, &nonce);

    // Verificar evento de reputação
    let events = env.events().all();
    let has_event = events.iter().any(|e| {
        e.1.get(0).map(|v| Symbol::from_val(&env, &v) == Symbol::new(&env, "reputation_export")).unwrap_or(false)
    });

    assert!(has_event);
}
