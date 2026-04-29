#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Env, String};

#[test]
fn test_soul_mint() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ZolvencySoulContract);
    let client = ZolvencySoulContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let relayer = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(&admin, &relayer);

    let username = String::from_str(&env, "felipe");
    let passkey = Bytes::from_array(&env, &[0u8; 65]);

    client.mint(&relayer, &user, &username, &passkey);

    assert_eq!(client.balance(&user), 1);
    assert!(client.has_soul(&user));

    let soul = client.get_soul(&user).unwrap();
    assert_eq!(soul.username, username);
    assert_eq!(soul.user, user);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")] // SoulAlreadyExists
fn test_soul_already_exists() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, ZolvencySoulContract);
    let client = ZolvencySoulContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let relayer = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(&admin, &relayer);

    let username = String::from_str(&env, "felipe");
    let passkey = Bytes::from_array(&env, &[0u8; 65]);

    client.mint(&relayer, &user, &username, &passkey);
    client.mint(&relayer, &user, &username, &passkey);
}
