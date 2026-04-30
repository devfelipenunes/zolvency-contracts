#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Env, BytesN};

#[test]
fn test_soul_mint() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ZolvencySoulContract, ());
    let client = ZolvencySoulContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let relayer = Address::generate(&env);

    client.initialize(&admin, &relayer);

    assert_eq!(client.admin(), admin);
    assert_eq!(client.relayer(), relayer);
    assert_eq!(client.total_souls(), 0);

    let passkey = BytesN::from_array(&env, &[0u8; 65]);
    let recovery_pubkey = BytesN::from_array(&env, &[1u8; 65]);

    let id = client.mint(&relayer, &passkey, &recovery_pubkey);

    assert_eq!(id, 1);
    assert_eq!(client.total_souls(), 1);

    let soul = client.get_soul(&id).unwrap();
    assert_eq!(soul.passkey, passkey);
    assert_eq!(soul.recovery_pubkey, recovery_pubkey);
    assert_eq!(soul.id, 1);

    assert_eq!(client.get_soul_id_by_passkey(&passkey), Some(1));
}

#[test]
fn test_get_soul_by_passkey() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ZolvencySoulContract, ());
    let client = ZolvencySoulContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let relayer = Address::generate(&env);

    client.initialize(&admin, &relayer);

    let passkey = BytesN::from_array(&env, &[1u8; 65]);
    let recovery_pubkey = BytesN::from_array(&env, &[2u8; 65]);

    client.mint(&relayer, &passkey, &recovery_pubkey);

    let soul = client.get_soul_by_passkey(&passkey).unwrap();
    assert_eq!(soul.passkey, passkey);
    assert_eq!(soul.id, 1);
}

#[test]
#[should_panic] // Should panic because of invalid signature
fn test_recover_soul_invalid_signature() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ZolvencySoulContract, ());
    let client = ZolvencySoulContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let relayer = Address::generate(&env);

    client.initialize(&admin, &relayer);

    let old_passkey = BytesN::from_array(&env, &[1u8; 65]);
    let recovery_pubkey = BytesN::from_array(&env, &[2u8; 65]);
    let new_passkey = BytesN::from_array(&env, &[3u8; 65]);
    let dummy_sig = BytesN::from_array(&env, &[0u8; 64]);

    client.mint(&relayer, &old_passkey, &recovery_pubkey);

    client.recover_soul(&relayer, &old_passkey, &new_passkey, &dummy_sig);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")] // NotAuthorized
fn test_recover_soul_unauthorized_relayer() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ZolvencySoulContract, ());
    let client = ZolvencySoulContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let relayer = Address::generate(&env);
    let wrong_relayer = Address::generate(&env);

    client.initialize(&admin, &relayer);

    let old_passkey = BytesN::from_array(&env, &[1u8; 65]);
    let recovery_pubkey = BytesN::from_array(&env, &[2u8; 65]);
    let new_passkey = BytesN::from_array(&env, &[3u8; 65]);
    let dummy_sig = BytesN::from_array(&env, &[0u8; 64]);

    client.mint(&relayer, &old_passkey, &recovery_pubkey);

    client.recover_soul(&wrong_relayer, &old_passkey, &new_passkey, &dummy_sig);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")] // SoulAlreadyExists
fn test_soul_already_exists() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ZolvencySoulContract, ());
    let client = ZolvencySoulContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let relayer = Address::generate(&env);

    client.initialize(&admin, &relayer);

    let passkey = BytesN::from_array(&env, &[0u8; 65]);
    let recovery_pubkey = BytesN::from_array(&env, &[1u8; 65]);

    client.mint(&relayer, &passkey, &recovery_pubkey);
    client.mint(&relayer, &passkey, &recovery_pubkey);
}
