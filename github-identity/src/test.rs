#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env, String};

struct TestEnv {
    env: Env,
    client: GithubIdentityContractClient<'static>,
    admin: Address,
    treasury: Address,
    access_control: Address,
}

fn setup() -> TestEnv {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, GithubIdentityContract);

    let client: GithubIdentityContractClient<'static> =
        unsafe { core::mem::transmute(GithubIdentityContractClient::new(&env, &contract_id)) };

    let admin = Address::generate(&env);
    let access_control = Address::generate(&env);
    let treasury = Address::generate(&env);

    client.initialize(&admin, &access_control, &treasury, &0);

    TestEnv {
        env,
        client,
        admin,
        treasury,
        access_control,
    }
}

fn stub_signature(env: &Env) -> BytesN<64> {
    BytesN::from_array(env, &[0u8; 64])
}

fn mint_for(ctx: &TestEnv, user: &Address, username: &str, contributions: u32) -> u64 {
    ctx.client.mint(
        user,
        &stub_signature(&ctx.env),
        &String::from_str(&ctx.env, username),
        &contributions,
        &Bytes::new(&ctx.env),
        &None,
        &ctx.client.get_nonce(user),
    )
}

#[test]
fn test_initialize_sets_mint_fee() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, GithubIdentityContract);
    let client = GithubIdentityContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let access_control = Address::generate(&env);
    let treasury = Address::generate(&env);
    let mint_fee = 1_000_000i128;

    client.initialize(&admin, &access_control, &treasury, &mint_fee);

    assert_eq!(client.get_mint_fee(), mint_fee);
}

#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn test_initialize_twice_fails() {
    let ctx = setup();
    ctx.client.initialize(
        &ctx.admin,
        &ctx.access_control,
        &ctx.treasury,
        &0,
    );
}

#[test]
fn test_mint_returns_token_id_one() {
    let ctx = setup();
    let user = Address::generate(&ctx.env);
    let token_id = mint_for(&ctx, &user, "devfelipenunes", 1500);
    assert_eq!(token_id, 1);
}

#[test]
fn test_mint_registers_identity() {
    let ctx = setup();
    let user = Address::generate(&ctx.env);

    assert!(!ctx.client.has_identity(&user));
    mint_for(&ctx, &user, "devfelipenunes", 1500);
    assert!(ctx.client.has_identity(&user));
}

#[test]
fn test_mint_multiple_users_get_sequential_ids() {
    let ctx = setup();
    let user_a = Address::generate(&ctx.env);
    let user_b = Address::generate(&ctx.env);
    let user_c = Address::generate(&ctx.env);

    let id_a = mint_for(&ctx, &user_a, "alice", 100);
    let id_b = mint_for(&ctx, &user_b, "bob", 200);
    let id_c = mint_for(&ctx, &user_c, "carol", 300);

    assert_eq!(id_a, 1);
    assert_eq!(id_b, 2);
    assert_eq!(id_c, 3);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_mint_empty_username_fails() {
    let ctx = setup();
    let user = Address::generate(&ctx.env);
    mint_for(&ctx, &user, "", 1500);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_mint_twice_same_user_fails() {
    let ctx = setup();
    let user = Address::generate(&ctx.env);

    mint_for(&ctx, &user, "devfelipenunes", 1500);
    mint_for(&ctx, &user, "devfelipenunes", 1500);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_mint_wrong_nonce_fails() {
    let ctx = setup();
    let user = Address::generate(&ctx.env);

    ctx.client.mint(
        &user,
        &stub_signature(&ctx.env),
        &String::from_str(&ctx.env, "devfelipenunes"),
        &1500u32,
        &Bytes::new(&ctx.env),
        &None,
        &99u64,
    );
}

#[test]
fn test_nonce_starts_at_zero() {
    let ctx = setup();
    let user = Address::generate(&ctx.env);
    assert_eq!(ctx.client.get_nonce(&user), 0);
}

#[test]
fn test_nonce_increments_after_mint() {
    let ctx = setup();
    let user = Address::generate(&ctx.env);

    assert_eq!(ctx.client.get_nonce(&user), 0);
    mint_for(&ctx, &user, "devfelipenunes", 1500);
    assert_eq!(ctx.client.get_nonce(&user), 1);
}

#[test]
fn test_get_user_token_returns_correct_id() {
    let ctx = setup();
    let user = Address::generate(&ctx.env);
    let token_id = mint_for(&ctx, &user, "devfelipenunes", 1500);
    assert_eq!(ctx.client.get_user_token(&user), token_id);
}

#[test]
fn test_get_token_data_stores_correct_values() {
    let ctx = setup();
    let user = Address::generate(&ctx.env);
    mint_for(&ctx, &user, "devfelipenunes", 1500);

    let data = ctx.client.get_token_data(&1u64);
    assert_eq!(data.username, String::from_str(&ctx.env, "devfelipenunes"));
    assert_eq!(data.contributions, 1500u32);
    assert_eq!(data.tier, Tier::Architect);
}

#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_get_token_data_missing_token_fails() {
    let ctx = setup();
    ctx.client.get_token_data(&999u64);
}

#[test]
fn test_list_tokens_of_user_with_identity() {
    let ctx = setup();
    let user = Address::generate(&ctx.env);
    mint_for(&ctx, &user, "devfelipenunes", 1500);

    let tokens = ctx.client.list_tokens_of_user(&user);
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens.get(0), Some(1u64));
}

#[test]
fn test_list_tokens_of_user_without_identity() {
    let ctx = setup();
    let user = Address::generate(&ctx.env);

    let tokens = ctx.client.list_tokens_of_user(&user);
    assert_eq!(tokens.len(), 0);
}

#[test]
fn test_update_token_changes_contributions_and_tier() {
    let ctx = setup();
    let user = Address::generate(&ctx.env);
    mint_for(&ctx, &user, "devfelipenunes", 1500);

    ctx.client.update_token(
        &user,
        &1u64,
        &String::from_str(&ctx.env, "devfelipenunes"),
        &3500u32,
        &Bytes::new(&ctx.env),
    );

    let data = ctx.client.get_token_data(&1u64);
    assert_eq!(data.contributions, 3500u32);
    assert_eq!(data.tier, Tier::Legend);
}

#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_update_token_by_non_owner_fails() {
    let ctx = setup();
    let owner = Address::generate(&ctx.env);
    let attacker = Address::generate(&ctx.env);

    mint_for(&ctx, &owner, "owner", 1500);
    mint_for(&ctx, &attacker, "attacker", 200);

    ctx.client.update_token(
        &attacker,
        &1u64,
        &String::from_str(&ctx.env, "owner"),
        &3500u32,
        &Bytes::new(&ctx.env),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_update_token_without_identity_fails() {
    let ctx = setup();
    let user = Address::generate(&ctx.env);
    ctx.client.update_token(
        &user,
        &1u64,
        &String::from_str(&ctx.env, "ghost"),
        &100u32,
        &Bytes::new(&ctx.env),
    );
}

#[test]
fn test_tier_boundaries() {
    assert_eq!(Tier::from_contributions(0), Tier::Novice);
    assert_eq!(Tier::from_contributions(199), Tier::Novice);
    assert_eq!(Tier::from_contributions(200), Tier::Pro);
    assert_eq!(Tier::from_contributions(999), Tier::Pro);
    assert_eq!(Tier::from_contributions(1000), Tier::Architect);
    assert_eq!(Tier::from_contributions(2999), Tier::Architect);
    assert_eq!(Tier::from_contributions(3000), Tier::Legend);
    assert_eq!(Tier::from_contributions(4999), Tier::Legend);
    assert_eq!(Tier::from_contributions(5000), Tier::Singularity);
    assert_eq!(Tier::from_contributions(u32::MAX), Tier::Singularity);
}

#[test]
fn test_tier_number() {
    assert_eq!(Tier::Novice.to_number(), 1);
    assert_eq!(Tier::Pro.to_number(), 2);
    assert_eq!(Tier::Architect.to_number(), 3);
    assert_eq!(Tier::Legend.to_number(), 4);
    assert_eq!(Tier::Singularity.to_number(), 5);
}

#[test]
fn test_svg_all_tiers() {
    let cases: &[(&str, u32, &str)] = &[
        (
            "novice_user",
            50,
            "<svg xmlns='http://www.w3.org/2000/svg' width='350' height='200'><rect width='100%' height='100%' fill='#b0c4de'/><text x='50%' y='100' font-size='24' fill='#181c2f' text-anchor='middle'>Novice</text></svg>",
        ),
        (
            "pro_user",
            500,
            "<svg xmlns='http://www.w3.org/2000/svg' width='350' height='200'><rect width='100%' height='100%' fill='#90ee90'/><text x='50%' y='100' font-size='24' fill='#181c2f' text-anchor='middle'>Pro</text></svg>",
        ),
        (
            "arch_user",
            1500,
            "<svg xmlns='http://www.w3.org/2000/svg' width='350' height='200'><rect width='100%' height='100%' fill='#ffd700'/><text x='50%' y='100' font-size='24' fill='#181c2f' text-anchor='middle'>Architect</text></svg>",
        ),
        (
            "legend_user",
            3500,
            "<svg xmlns='http://www.w3.org/2000/svg' width='350' height='200'><rect width='100%' height='100%' fill='#ff8c00'/><text x='50%' y='100' font-size='24' fill='#fff' text-anchor='middle'>Legend</text></svg>",
        ),
        (
            "sing_user",
            6000,
            "<svg xmlns='http://www.w3.org/2000/svg' width='350' height='200'><rect width='100%' height='100%' fill='#8a2be2'/><text x='50%' y='100' font-size='24' fill='#fff' text-anchor='middle'>Singularity</text></svg>",
        ),
    ];

    for (username, contributions, expected_svg) in cases {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, GithubIdentityContract);
        let client = GithubIdentityContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let access_control = Address::generate(&env);
        let treasury = Address::generate(&env);
        client.initialize(&admin, &access_control, &treasury, &0);

        let user = Address::generate(&env);
        let token_id = client.mint(
            &user,
            &stub_signature(&env),
            &String::from_str(&env, username),
            contributions,
            &Bytes::new(&env),
            &None,
            &0u64,
        );

        let svg = client.get_token_svg(&token_id);
        assert_eq!(svg, String::from_str(&env, expected_svg));
    }
}

#[test]
fn test_svg_architect_exact_output() {
    let ctx = setup();
    let user = Address::generate(&ctx.env);
    mint_for(&ctx, &user, "devfelipenunes", 1500);

    let svg = ctx.client.get_token_svg(&1u64);
    let expected = String::from_str(
        &ctx.env,
        "<svg xmlns='http://www.w3.org/2000/svg' width='350' height='200'>\
<rect width='100%' height='100%' fill='#ffd700'/>\
<text x='50%' y='100' font-size='24' fill='#181c2f' text-anchor='middle'>Architect</text></svg>",
    );
    assert_eq!(svg, expected);
}

#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_svg_missing_token_fails() {
    let ctx = setup();
    ctx.client.get_token_svg(&999u64);
}

#[test]
fn test_set_mint_fee_by_admin() {
    let ctx = setup();
    assert_eq!(ctx.client.get_mint_fee(), 0);
    ctx.client.set_mint_fee(&ctx.admin, &5_000_000i128);
    assert_eq!(ctx.client.get_mint_fee(), 5_000_000i128);
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_set_mint_fee_by_non_admin_fails() {
    let ctx = setup();
    let not_admin = Address::generate(&ctx.env);
    ctx.client.set_mint_fee(&not_admin, &5_000_000i128);
}

#[test]
fn test_set_access_control_by_admin() {
    let ctx = setup();
    let new_ac = Address::generate(&ctx.env);
    ctx.client.set_access_control(&ctx.admin, &new_ac);
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_set_access_control_by_non_admin_fails() {
    let ctx = setup();
    let not_admin = Address::generate(&ctx.env);
    let new_ac = Address::generate(&ctx.env);
    ctx.client.set_access_control(&not_admin, &new_ac);
}

#[test]
fn test_set_treasury_by_admin() {
    let ctx = setup();
    let new_treasury = Address::generate(&ctx.env);
    ctx.client.set_treasury(&ctx.admin, &new_treasury);
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_set_treasury_by_non_admin_fails() {
    let ctx = setup();
    let not_admin = Address::generate(&ctx.env);
    let new_treasury = Address::generate(&ctx.env);
    ctx.client.set_treasury(&not_admin, &new_treasury);
}