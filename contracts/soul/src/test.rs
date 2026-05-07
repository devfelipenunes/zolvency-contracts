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

    let owner = Address::generate(&env);
    let id = client.mint(&relayer, &owner, &passkey, &recovery_pubkey);

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

    let owner = Address::generate(&env);
    client.mint(&relayer, &owner, &passkey, &recovery_pubkey);

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

    let owner = Address::generate(&env);
    client.mint(&relayer, &owner, &old_passkey, &recovery_pubkey);

    client.recover_soul(&relayer, &old_passkey, &new_passkey, &dummy_sig);
}

#[test]
fn test_recover_soul_success() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ZolvencySoulContract, ());
    let client = ZolvencySoulContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let relayer = Address::generate(&env);

    client.initialize(&admin, &relayer);

    // Test vector generated via python cryptography (Prehashed, normalized to low-S)
    let recovery_pubkey = BytesN::from_array(&env, &[
        0x04, 0x8b, 0xfc, 0x7c, 0x35, 0x67, 0x03, 0x5c, 0x1d, 0x77, 0x01, 0x56, 0x50, 0x50, 0x50, 0x3f,
        0xed, 0xd9, 0xd3, 0xd5, 0xda, 0x52, 0x32, 0x41, 0x56, 0x70, 0x5f, 0x8c, 0x19, 0xb8, 0x75, 0xde,
        0xd7, 0xb7, 0x6e, 0x58, 0xa1, 0x12, 0xd9, 0x1f, 0x04, 0x86, 0xf0, 0xc1, 0xfd, 0x82, 0xee, 0xfe,
        0x11, 0x98, 0x17, 0xf7, 0xda, 0x03, 0xa5, 0xe9, 0xb5, 0x3f, 0x3f, 0xc0, 0x89, 0xb3, 0xb2, 0x02,
        0x8c
    ]);

    let old_passkey = BytesN::from_array(&env, &[0u8; 65]);
    let new_passkey = BytesN::from_array(&env, &[1u8; 65]);

    let signature = BytesN::from_array(&env, &[
        0xb4, 0x7f, 0x75, 0x5d, 0xc9, 0x49, 0x27, 0x98, 0xa3, 0x2d, 0x55, 0x65, 0xbf, 0x6f, 0xf9, 0x75,
        0x2f, 0x97, 0xb1, 0x37, 0xd5, 0x12, 0xdf, 0x95, 0xaf, 0x26, 0x83, 0x2f, 0x62, 0x58, 0x29, 0xf3,
        0x5e, 0x9e, 0x89, 0xf5, 0x91, 0x48, 0xcd, 0xdd, 0x68, 0xa1, 0xc9, 0x03, 0x9c, 0xb9, 0xd2, 0xd4,
        0x04, 0x83, 0x17, 0x9b, 0x33, 0xb9, 0x7b, 0x61, 0x82, 0x8a, 0x54, 0x1a, 0x3e, 0x7a, 0x51, 0xc6
    ]);

    let owner = Address::generate(&env);
    let id = client.mint(&relayer, &owner, &old_passkey, &recovery_pubkey);
    assert_eq!(id, 1);

    // Recover soul (updates passkey)
    client.recover_soul(&relayer, &old_passkey, &new_passkey, &signature);

    // Verify update
    let soul = client.get_soul(&id).unwrap();
    assert_eq!(soul.passkey, new_passkey);

    // Verify mapping update
    assert_eq!(client.get_soul_id_by_passkey(&old_passkey), None);
    assert_eq!(client.get_soul_id_by_passkey(&new_passkey), Some(1));
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

    let owner = Address::generate(&env);
    client.mint(&relayer, &owner, &old_passkey, &recovery_pubkey);

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

    let owner = Address::generate(&env);
    client.mint(&relayer, &owner, &passkey, &recovery_pubkey);
    client.mint(&relayer, &owner, &passkey, &recovery_pubkey);
}

#[test]
fn test_initialize_already_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(ZolvencySoulContract, ());
    let client = ZolvencySoulContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &admin);
    let res = client.try_initialize(&admin, &admin);
    assert_eq!(res, Err(Ok(Error::AlreadyInitialized)));
}

#[test]
fn test_unauthorized_mint() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(ZolvencySoulContract, ());
    let client = ZolvencySoulContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let relayer = Address::generate(&env);
    let attacker = Address::generate(&env);
    client.initialize(&admin, &relayer);

    let owner = Address::generate(&env);
    let res = client.try_mint(&attacker, &owner, &BytesN::from_array(&env, &[0u8; 65]), &BytesN::from_array(&env, &[0u8; 65]));
    assert_eq!(res, Err(Ok(Error::NotAuthorized)));
}
