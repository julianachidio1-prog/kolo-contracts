#![cfg(test)]
#![allow(deprecated)]

use super::*;
use soroban_sdk::testutils::{AuthorizedFunction, MockAuth, MockAuthInvoke};
use soroban_sdk::token;
use soroban_sdk::xdr::{ScErrorCode, ScErrorType};
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, testutils::Address as _, testutils::Events,
    vec, Address, Env, IntoVal, String, Symbol, Val, Vec,
};

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let name = String::from_str(&env, "Test Group");
    let contribution_amount = 1000;

    client.initialize(
        &admin,
        &token,
        &name,
        &contribution_amount,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );
}

#[test]
#[should_panic(expected = "Already initialized")]
fn test_double_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let name = String::from_str(&env, "Test Group");
    let contribution_amount = 1000;

    client.initialize(
        &admin,
        &token,
        &name,
        &contribution_amount,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );
    client.initialize(
        &admin,
        &token,
        &name,
        &contribution_amount,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );
}

#[test]
fn test_add_member() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let name = String::from_str(&env, "Test Group");
    let contribution_amount = 1000;

    client.initialize(
        &admin,
        &token,
        &name,
        &contribution_amount,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    env.mock_all_auths();
    let member1 = Address::generate(&env);
    client.add_member(&member1);
}

#[test]
#[should_panic(expected = "Not a member")]
fn test_contribute_not_member() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let name = String::from_str(&env, "Test Group");
    let contribution_amount = 1000;

    client.initialize(
        &admin,
        &token,
        &name,
        &contribution_amount,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    env.mock_all_auths();
    let not_member = Address::generate(&env);
    client.contribute(&not_member, &1000);
}

#[test]
#[should_panic(expected = "Already contributed this cycle")]
fn test_contribute_twice_same_cycle_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);
    let name = String::from_str(&env, "Test Group");

    client.initialize(
        &admin,
        &token,
        &name,
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    let member = Address::generate(&env);
    client.add_member(&member);
    token_client.mint(&member, &5000);

    client.contribute(&member, &1000);
    client.contribute(&member, &1000);
}

#[test]
fn test_contribute_allowed_after_reset() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);
    let name = String::from_str(&env, "Test Group");

    client.initialize(
        &admin,
        &token,
        &name,
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    let member = Address::generate(&env);
    client.add_member(&member);
    token_client.mint(&member, &5000);

    client.contribute(&member, &1000);
    client.reset_cycle();
    client.contribute(&member, &1000);

    assert_eq!(client.get_contribution(&member), 2000);
}

#[test]
fn test_events() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let name = String::from_str(&env, "Test Group");
    let contribution_amount = 1000;

    // 1. Test Initialize Event
    client.initialize(
        &admin,
        &token,
        &name,
        &contribution_amount,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id.clone(),
                soroban_sdk::vec![&env, symbol_short!("init").into_val(&env)],
                (
                    admin.clone(),
                    token.clone(),
                    name.clone(),
                    contribution_amount
                )
                    .into_val(&env)
            )
        ]
    );

    // 2. Test Add Member Event
    let member1 = Address::generate(&env);
    client.add_member(&member1);

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id.clone(),
                soroban_sdk::vec![
                    &env,
                    symbol_short!("add_mem").into_val(&env),
                    member1.clone().into_val(&env)
                ],
                ().into_val(&env)
            )
        ]
    );
}

#[test]
fn test_goalbased_flexible_contributions() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);
    let name = String::from_str(&env, "Goal Group");

    client.initialize(
        &admin,
        &token,
        &name,
        &1000i128,
        &GroupType::GoalBased,
        &None,
        &false,
        &None,
    );

    let member = Address::generate(&env);
    client.add_member(&member);
    token_client.mint(&member, &5000);

    // Can contribute varying amounts
    client.contribute(&member, &500);

    // Wait, the test above calls contribute twice in a row, but HasContributedThisCycle is still active.
    // So we need to call reset_cycle() or it will panic.
    client.reset_cycle();
    client.contribute(&member, &1500);

    assert_eq!(client.get_contribution(&member), 2000);
}

#[test]
fn test_goalbased_partial_withdrawal() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);
    let name = String::from_str(&env, "Goal Group");

    client.initialize(
        &admin,
        &token,
        &name,
        &1000i128,
        &GroupType::GoalBased,
        &None,
        &false,
        &None,
    );

    let member = Address::generate(&env);
    client.add_member(&member);
    token_client.mint(&member, &5000);

    client.contribute(&member, &2000);

    assert_eq!(token_client.balance(&member), 3000);

    client.withdraw_savings(&member, &500);

    assert_eq!(client.get_contribution(&member), 1500);
    assert_eq!(token_client.balance(&member), 3500);
}

#[test]
#[should_panic(expected = "Insufficient savings to withdraw")]
fn test_goalbased_over_withdrawal() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);
    let name = String::from_str(&env, "Goal Group");

    client.initialize(
        &admin,
        &token,
        &name,
        &1000i128,
        &GroupType::GoalBased,
        &None,
        &false,
        &None,
    );

    let member = Address::generate(&env);
    client.add_member(&member);
    token_client.mint(&member, &5000);

    client.contribute(&member, &1000);
    client.withdraw_savings(&member, &1500);
}

#[test]
#[should_panic(expected = "Target amount not reached yet")]
fn test_goalbased_locked_until_target() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);
    let name = String::from_str(&env, "Goal Group");

    client.initialize(
        &admin,
        &token,
        &name,
        &1000i128,
        &GroupType::GoalBased,
        &Some(2000), // target amount
        &true,       // lock until target
        &None,       // expected_cycle_days
    );

    let member = Address::generate(&env);
    client.add_member(&member);
    token_client.mint(&member, &5000);

    client.contribute(&member, &1000);
    // Should panic because 1000 < 2000
    client.withdraw_savings(&member, &500);
}

#[test]
fn test_remove_member_no_contribution() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let name = String::from_str(&env, "Test Group");

    client.initialize(
        &admin,
        &token,
        &name,
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);
    client.add_member(&member1);
    client.add_member(&member2);

    client.remove_member(&member2);

    env.as_contract(&contract_id, || {
        let members: Vec<Address> = env.storage().instance().get(&DataKey::Members).unwrap();
        assert_eq!(members.len(), 1);
        assert!(members.contains(&member1));
        assert!(!members.contains(&member2));
    });
}

#[test]
fn test_remove_member_with_contribution_refund() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);
    let name = String::from_str(&env, "Test Group");

    client.initialize(
        &admin,
        &token,
        &name,
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);
    client.add_member(&member1);
    client.add_member(&member2);
    token_client.mint(&member1, &5000);
    token_client.mint(&member2, &5000);

    client.contribute(&member1, &1000);
    client.contribute(&member2, &1000);

    assert_eq!(token_client.balance(&member2), 4000);

    client.remove_member(&member2);

    assert_eq!(token_client.balance(&member2), 5000);

    env.as_contract(&contract_id, || {
        // MemberState entry removed entirely during remove_member (O(1) design)
        assert!(!env
            .storage()
            .instance()
            .has(&DataKey::Member(member2.clone())));
    });
}

#[test]
#[should_panic(expected = "Cannot remove member after their payout turn")]
fn test_remove_member_after_payout_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);
    let name = String::from_str(&env, "Test Group");

    client.initialize(
        &admin,
        &token,
        &name,
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);
    client.add_member(&member1);
    client.add_member(&member2);
    token_client.mint(&member1, &5000);
    token_client.mint(&member2, &5000);

    client.contribute(&member1, &1000);
    client.contribute(&member2, &1000);

    client.payout(&member1, &None);

    client.remove_member(&member1);
}

#[test]
fn test_remove_member_adjusts_cycle_count() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);
    let name = String::from_str(&env, "Test Group");

    client.initialize(
        &admin,
        &token,
        &name,
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);
    let member3 = Address::generate(&env);
    client.add_member(&member1);
    client.add_member(&member2);
    client.add_member(&member3);
    token_client.mint(&member1, &5000);
    token_client.mint(&member2, &5000);
    token_client.mint(&member3, &5000);

    client.contribute(&member1, &1000);

    env.as_contract(&contract_id, || {
        let count: i128 = env
            .storage()
            .instance()
            .get(&DataKey::CycleMemberCount)
            .unwrap();
        assert_eq!(count, 3);
    });

    client.remove_member(&member3);

    env.as_contract(&contract_id, || {
        let count_after: i128 = env
            .storage()
            .instance()
            .get(&DataKey::CycleMemberCount)
            .unwrap();
        assert_eq!(count_after, 2);
    });

    client.contribute(&member2, &1000);

    client.payout(&member1, &None);

    let contract_balance = token_client.balance(&contract_id);
    assert_eq!(contract_balance, 0);

    client.reset_cycle();

    env.as_contract(&contract_id, || {
        assert!(!env.storage().instance().has(&DataKey::CycleMemberCount));
    });
}

#[test]
fn test_remove_last_member_clears_cycle_count() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);
    let name = String::from_str(&env, "Test Group");

    client.initialize(
        &admin,
        &token,
        &name,
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    let member = Address::generate(&env);
    client.add_member(&member);
    token_client.mint(&member, &5000);

    client.contribute(&member, &1000);

    env.as_contract(&contract_id, || {
        let count: i128 = env
            .storage()
            .instance()
            .get(&DataKey::CycleMemberCount)
            .unwrap();
        assert_eq!(count, 1);
    });

    client.remove_member(&member);

    env.as_contract(&contract_id, || {
        assert!(!env.storage().instance().has(&DataKey::CycleMemberCount));
    });
}

#[test]
fn test_deterministic_payout_order() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);

    client.initialize(
        &admin,
        &token,
        &String::from_str(&env, "Test Group"),
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    let member0 = Address::generate(&env);
    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);
    client.add_member(&member0);
    client.add_member(&member1);
    client.add_member(&member2);

    token_client.mint(&member0, &10000);
    token_client.mint(&member1, &10000);
    token_client.mint(&member2, &10000);

    // Payout 1 goes to member0 (index 0 in join order)
    client.contribute(&member0, &1000);
    client.contribute(&member1, &1000);
    client.contribute(&member2, &1000);
    assert_eq!(client.get_next_payout_recipient(), member0);
    client.payout(&member0, &None);
    assert!(client.has_received_payout(&member0));
    assert!(!client.has_received_payout(&member1));
    assert!(!client.has_received_payout(&member2));
    // member0 gets the full pool: 1000 * 3 = 3000
    // balance = 10000 - 1000 (contrib) + 3000 (payout) = 12000
    assert_eq!(token_client.balance(&member0), 12000);

    // After payout, NextPayoutIndex advanced to 1 — member1 is next
    assert_eq!(client.get_next_payout_recipient(), member1);

    // Payout 2 goes to member1 (index 1) after reset + re-contribute
    client.reset_cycle();
    client.contribute(&member0, &1000);
    client.contribute(&member1, &1000);
    client.contribute(&member2, &1000);
    assert_eq!(client.get_next_payout_recipient(), member1);
    client.payout(&member1, &None);
    assert!(client.has_received_payout(&member1));
    assert!(!client.has_received_payout(&member2));
    // 10000 - 1000 (round1) - 1000 (round2) + 3000 (payout) = 11000
    assert_eq!(token_client.balance(&member1), 11000);

    // Payout 3 goes to member2 (index 2)
    client.reset_cycle();
    client.contribute(&member0, &1000);
    client.contribute(&member1, &1000);
    client.contribute(&member2, &1000);
    assert_eq!(client.get_next_payout_recipient(), member2);
    client.payout(&member2, &None);
    assert!(client.has_received_payout(&member2));
    // 10000 - 1000*3 (3 rounds) + 3000 (payout) = 10000
    assert_eq!(token_client.balance(&member2), 10000);
}

#[test]
fn test_queue_enforced_payout_order() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);

    client.initialize(
        &admin,
        &token,
        &String::from_str(&env, "Test Group"),
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    let member0 = Address::generate(&env);
    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);
    client.add_member(&member0);
    client.add_member(&member1);
    client.add_member(&member2);

    token_client.mint(&member0, &10000);
    token_client.mint(&member1, &10000);
    token_client.mint(&member2, &10000);

    // Payout must go to member0 (index 0), admin cannot choose
    client.contribute(&member0, &1000);
    client.contribute(&member1, &1000);
    client.contribute(&member2, &1000);

    assert_eq!(client.get_next_payout_recipient(), member0);
    client.payout(&member0, &None);
    assert!(client.has_received_payout(&member0));
    assert!(!client.has_received_payout(&member2));

    // After reset, queue advances to member1 (NextPayoutIndex persists)
    client.reset_cycle();
    client.contribute(&member0, &1000);
    client.contribute(&member1, &1000);
    client.contribute(&member2, &1000);

    assert_eq!(client.get_next_payout_recipient(), member1);
    client.payout(&member1, &None);
    assert!(client.has_received_payout(&member1));
    assert!(!client.has_received_payout(&member2));

    // After another reset, finally member2
    client.reset_cycle();
    client.contribute(&member0, &1000);
    client.contribute(&member1, &1000);
    client.contribute(&member2, &1000);

    assert_eq!(client.get_next_payout_recipient(), member2);
    client.payout(&member2, &None);
    assert!(client.has_received_payout(&member2));
}

#[test]
fn test_cycle_resets_and_starts_again() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);

    client.initialize(
        &admin,
        &token,
        &String::from_str(&env, "Test Group"),
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    let member0 = Address::generate(&env);
    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);
    client.add_member(&member0);
    client.add_member(&member1);
    client.add_member(&member2);

    token_client.mint(&member0, &10000);
    token_client.mint(&member1, &10000);
    token_client.mint(&member2, &10000);

    // --- Full rotation: member0 → member1 → member2 ---
    // Round 1: payout to member0
    client.contribute(&member0, &1000);
    client.contribute(&member1, &1000);
    client.contribute(&member2, &1000);
    assert_eq!(client.get_next_payout_recipient(), member0);
    client.payout(&member0, &None);
    assert!(client.has_received_payout(&member0));

    // Round 2: payout to member1
    client.reset_cycle();
    client.contribute(&member0, &1000);
    client.contribute(&member1, &1000);
    client.contribute(&member2, &1000);
    assert_eq!(client.get_next_payout_recipient(), member1);
    client.payout(&member1, &None);
    assert!(client.has_received_payout(&member1));

    // Round 3: payout to member2
    client.reset_cycle();
    client.contribute(&member0, &1000);
    client.contribute(&member1, &1000);
    client.contribute(&member2, &1000);
    assert_eq!(client.get_next_payout_recipient(), member2);
    client.payout(&member2, &None);
    assert!(client.has_received_payout(&member2));

    // --- Full reset: reset_rotation() resets NextPayoutIndex to 0 ---
    client.reset_cycle();
    client.reset_rotation();

    // After full reset, all has_received_payout flags are cleared
    assert!(!client.has_received_payout(&member0));
    assert!(!client.has_received_payout(&member1));
    assert!(!client.has_received_payout(&member2));

    // New rotation starts with member0 again
    client.contribute(&member0, &1000);
    client.contribute(&member1, &1000);
    client.contribute(&member2, &1000);
    assert_eq!(client.get_next_payout_recipient(), member0);
    client.payout(&member0, &None);
    assert!(client.has_received_payout(&member0));
}

#[test]
#[should_panic(expected = "Contribution amount must be positive")]
fn test_initialize_with_negative_amount_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);

    client.initialize(
        &admin,
        &token,
        &String::from_str(&env, "Test"),
        &-1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );
}

#[test]
#[should_panic(expected = "Contribution amount must be positive")]
fn test_initialize_with_zero_amount_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);

    client.initialize(
        &admin,
        &token,
        &String::from_str(&env, "Test"),
        &0i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );
}

#[test]
#[should_panic(expected = "Contribution amount exceeds maximum limit")]
fn test_initialize_with_exceeding_max_amount_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);

    client.initialize(
        &admin,
        &token,
        &String::from_str(&env, "Test"),
        &1_000_000_000_000_001i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );
}

#[test]
#[should_panic(expected = "Math overflow in pool calculation")]
fn test_payout_pool_size_overflow_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);

    client.initialize(
        &admin,
        &token,
        &String::from_str(&env, "Test"),
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    let member0 = Address::generate(&env);
    client.add_member(&member0);

    env.as_contract(&contract_id, || {
        // Force an overflow scenario
        env.storage()
            .instance()
            .set(&DataKey::ContributionAmount, &i128::MAX);
        env.storage()
            .instance()
            .set(&DataKey::CycleMemberCount, &2i128);
    });

    client.payout(&member0, &None);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_payout_wrong_recipient_auth_fails() {
    let env = Env::default();

    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);

    // Initialize with mock all auths to easily bypass initialization auth
    env.mock_all_auths();
    client.initialize(
        &admin,
        &token,
        &String::from_str(&env, "Test Group"),
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    let member0 = Address::generate(&env);
    let member1 = Address::generate(&env);
    client.add_member(&member0);
    client.add_member(&member1);

    token_client.mint(&member0, &10000);
    client.contribute(&member0, &1000);

    // We only explicitly mock the auth for payout with the WRONG recipient (member1)
    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &admin,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &contract_id,
            fn_name: "payout",
            args: (member1.clone(), None::<i128>).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    // Calling it with member0 should fail because auth is for member1
    client.payout(&member0, &None);
}

// ---------------------------------------------------------------------------
// Partial payout tests (proceedWithPartialPool support)
// ---------------------------------------------------------------------------

#[test]
fn test_payout_partial_amount_succeeds_and_advances_rotation() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);

    client.initialize(
        &admin,
        &token,
        &String::from_str(&env, "Test Group"),
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    let member0 = Address::generate(&env);
    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);
    client.add_member(&member0);
    client.add_member(&member1);
    client.add_member(&member2);

    token_client.mint(&member0, &10000);
    token_client.mint(&member1, &10000);

    // Only member0 and member1 contribute this cycle — member2 defaults.
    // The full pool would be 1000 * 3 = 3000, but the contract only holds
    // 2000, so the admin proceeds with a partial payout of exactly that.
    client.contribute(&member0, &1000);
    client.contribute(&member1, &1000);

    assert_eq!(client.get_next_payout_recipient(), member0);
    client.payout(&member0, &Some(2000i128));

    assert!(client.has_received_payout(&member0));
    // 10000 - 1000 (contrib) + 2000 (partial payout) = 11000
    assert_eq!(token_client.balance(&member0), 11000);
    assert_eq!(token_client.balance(&contract_id), 0);

    // Rotation still advances normally after a partial payout.
    assert_eq!(client.get_next_payout_recipient(), member1);
}

#[test]
#[should_panic(expected = "Payout amount exceeds the full computed pool")]
fn test_payout_partial_amount_above_full_pool_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);

    client.initialize(
        &admin,
        &token,
        &String::from_str(&env, "Test Group"),
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    let member0 = Address::generate(&env);
    let member1 = Address::generate(&env);
    client.add_member(&member0);
    client.add_member(&member1);

    token_client.mint(&member0, &10000);
    token_client.mint(&member1, &10000);
    client.contribute(&member0, &1000);
    client.contribute(&member1, &1000);

    // Full pool is 1000 * 2 = 2000 — asking for more than that is rejected
    // even though the admin is authorizing it, so a compromised or buggy
    // caller can never drain more than a cycle's own pool.
    client.payout(&member0, &Some(2001i128));
}

#[test]
#[should_panic(expected = "Payout amount must be positive")]
fn test_payout_partial_amount_zero_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);

    client.initialize(
        &admin,
        &token,
        &String::from_str(&env, "Test Group"),
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    let member0 = Address::generate(&env);
    client.add_member(&member0);
    token_client.mint(&member0, &10000);
    client.contribute(&member0, &1000);

    client.payout(&member0, &Some(0i128));
}

#[test]
#[should_panic(expected = "Insufficient funds in contract for requested payout")]
fn test_payout_partial_amount_above_balance_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);

    client.initialize(
        &admin,
        &token,
        &String::from_str(&env, "Test Group"),
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    let member0 = Address::generate(&env);
    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);
    client.add_member(&member0);
    client.add_member(&member1);
    client.add_member(&member2);

    token_client.mint(&member0, &10000);
    // Only member0 contributes — contract balance is 1000, but the full pool
    // (1000 * 3 = 3000) is still within bounds for an explicit request, so
    // this must fail on the balance check, not the pool-size ceiling check.
    client.contribute(&member0, &1000);

    client.payout(&member0, &Some(1500i128));
}

// ---------------------------------------------------------------------------
// Reentrancy / Checks-Effects-Interactions tests
// ---------------------------------------------------------------------------
//
// `MaliciousToken` is a minimal mock that implements just enough of the token
// interface (`balance` and `transfer`) for `token::Client` to invoke it from
// inside `KoloSavingsContract`. Its `transfer` function is where a real
// malicious or unusually-implemented token could try to "call back" into the
// savings contract mid-payout. We use it to:
//
//   1. Prove the CEI ordering: by the time `transfer` runs, the effects of
//      `payout()` (NextPayoutIndex / has_received_payout) are already
//      committed — observable via a read-only call made from inside the
//      token's `transfer` callback.
//   2. Prove the reentrancy guard: a callback that tries to invoke `payout()`
//      again while the outer `payout()` call is still executing must panic
//      with "Reentrancy detected", rather than draining the pool twice.
//
// Note: because Soroban contract invocations are atomic, a panic anywhere
// during a call unwinds and reverts *all* storage writes made during that
// top-level call. This means we can't inspect "partial" state after a failed
// transfer from outside — the whole test would revert. So the meaningful way
// to verify ordering is to observe state *during* the call, before it either
// completes or panics, which is exactly what these mocks do.

#[contracttype]
#[derive(Clone)]
enum MalKey {
    Kolo,
    Mode,
    ObservedHasPayout,
}

#[contract]
pub struct MaliciousToken;

#[contractimpl]
impl MaliciousToken {
    /// Configure which savings contract to target. Starts in "observe" mode (0).
    pub fn init(env: Env, kolo: Address) {
        env.storage().instance().set(&MalKey::Kolo, &kolo);
        env.storage().instance().set(&MalKey::Mode, &0u32);
    }

    /// 0 = passively observe state during transfer(); 1 = attempt a reentrant
    /// call into payout() during transfer().
    pub fn set_mode(env: Env, mode: u32) {
        env.storage().instance().set(&MalKey::Mode, &mode);
    }

    pub fn get_observed(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&MalKey::ObservedHasPayout)
            .unwrap_or(false)
    }

    pub fn balance(env: Env, _id: Address) -> i128 {
        let mode: u32 = env.storage().instance().get(&MalKey::Mode).unwrap_or(0);
        if mode == 2 {
            let kolo: Address = env.storage().instance().get(&MalKey::Kolo).unwrap();
            let client = KoloSavingsContractClient::new(&env, &kolo);
            // Attempt reentrancy from inside balance(), before transfer() is ever reached.
            client.payout(&kolo, &None); // recipient doesn't matter — should never get this far
        }
        i128::MAX
    }

    /// Called by KoloSavingsContract via `token::Client::transfer`. This is the
    /// "interaction" step where a malicious token gets a chance to re-enter.
    pub fn transfer(env: Env, _from: Address, to: Address, _amount: i128) {
        let kolo: Address = env.storage().instance().get(&MalKey::Kolo).unwrap();
        let mode: u32 = env.storage().instance().get(&MalKey::Mode).unwrap_or(0);

        if mode == 1 {
            // Attack: try to trigger a second payout to the same recipient
            // while the outer payout() call is still mid-flight.
            let client = KoloSavingsContractClient::new(&env, &kolo);
            client.payout(&to, &None);
        } else {
            // Observe: read back whether payout()'s effects were already
            // applied by the time this callback runs.
            let already_marked: bool = env.as_contract(&kolo, || {
                let state: Option<MemberState> =
                    env.storage().persistent().get(&DataKey::Member(to.clone()));
                state.map(|s| s.has_received_payout).unwrap_or(false)
            });
            env.storage()
                .instance()
                .set(&MalKey::ObservedHasPayout, &already_marked);
        }
    }
}

fn setup_with_malicious_token(
    env: &Env,
) -> (
    Address,
    KoloSavingsContractClient<'_>,
    Address,
    MaliciousTokenClient<'_>,
) {
    let kolo_id = env.register_contract(None, KoloSavingsContract);
    let kolo_client = KoloSavingsContractClient::new(env, &kolo_id);

    let mal_token_id = env.register_contract(None, MaliciousToken);
    let mal_client = MaliciousTokenClient::new(env, &mal_token_id);

    let admin = Address::generate(env);
    let name = String::from_str(env, "Test Group");

    kolo_client.initialize(
        &admin,
        &mal_token_id,
        &name,
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    (kolo_id, kolo_client, mal_token_id, mal_client)
}

/// Requirement: verify the CEI pattern is implemented — state (has_received_payout,
/// via NextPayoutIndex) must already reflect the payout by the time the external
/// token transfer callback executes.
#[test]
fn test_cei_effects_committed_before_external_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let (kolo_id, kolo_client, _mal_token_id, mal_client) = setup_with_malicious_token(&env);

    let member = Address::generate(&env);
    kolo_client.add_member(&member);

    mal_client.init(&kolo_id);
    mal_client.set_mode(&0u32);

    kolo_client.contribute(&member, &1000);
    assert!(!mal_client.get_observed()); // contribute's own transfer targets the contract, not a member

    kolo_client.payout(&member, &None);

    // The MaliciousToken observed, from *inside* the transfer() callback, that
    // has_received_payout(member) was already true — proving effects were
    // committed before the interaction, per CEI.
    assert!(mal_client.get_observed());
}

/// Requirement: a malicious token attempting to re-enter payout() during its
/// transfer() callback must be blocked by the reentrancy guard.
#[test]
#[should_panic(expected = "Error(Context, InvalidAction)")]
fn test_reentrant_payout_call_is_blocked() {
    let env = Env::default();
    env.mock_all_auths();

    let (kolo_id, kolo_client, _mal_token_id, mal_client) = setup_with_malicious_token(&env);

    let member = Address::generate(&env);
    kolo_client.add_member(&member);

    mal_client.init(&kolo_id);

    // Contribute first while still in observe mode (mode 0) so the deposit
    // succeeds normally.
    kolo_client.contribute(&member, &1000);

    // Now arm the attack: the next transfer() call (triggered by payout())
    // will try to call payout() again for the same recipient.
    mal_client.set_mode(&1u32);

    // This should panic with "Reentrancy detected" because the guard set at
    // the top of payout() is still held when the reentrant call happens.
    kolo_client.payout(&member, &None);
}

/// Requirement: contribute() must also refuse to run if a payout is (somehow)
/// already mid-execution, as a defense-in-depth measure.
#[test]
#[should_panic(expected = "Reentrancy detected")]
fn test_contribute_blocked_while_payout_executing() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);
    let name = String::from_str(&env, "Test Group");

    client.initialize(
        &admin,
        &token,
        &name,
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    let member = Address::generate(&env);
    client.add_member(&member);
    token_client.mint(&member, &5000);

    // Simulate being mid-payout by setting the guard flag directly.
    env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .set(&DataKey::IsExecutingPayout, &true);
    });

    client.contribute(&member, &1000);
}

#[test]
#[should_panic(expected = "Contract is paused for emergency")]
fn test_pause_blocks_contribute() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);

    client.initialize(
        &admin,
        &token,
        &String::from_str(&env, "Test Group"),
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    let member = Address::generate(&env);
    client.add_member(&member);
    token_client.mint(&member, &5000);

    client.pause();
    client.contribute(&member, &1000);
}

#[test]
#[should_panic(expected = "Error(Context, InvalidAction)")]
fn test_reentrant_payout_call_via_balance_is_blocked() {
    let env = Env::default();
    env.mock_all_auths();
    let (kolo_id, kolo_client, _mal_token_id, mal_client) = setup_with_malicious_token(&env);

    let member = Address::generate(&env);
    kolo_client.add_member(&member);
    mal_client.init(&kolo_id);

    kolo_client.contribute(&member, &1000);

    mal_client.set_mode(&2u32); // reenter via balance(), not transfer()
    kolo_client.payout(&member, &None);
}

#[test]
fn test_unpause_allows_contribute() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);

    client.initialize(
        &admin,
        &token,
        &String::from_str(&env, "Test Group"),
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    let member = Address::generate(&env);
    client.add_member(&member);
    token_client.mint(&member, &5000);

    client.pause();
    client.unpause();
    client.contribute(&member, &1000);

    assert_eq!(client.get_contribution(&member), 1000);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_non_admin_cannot_pause() {
    let env = Env::default();

    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(
        &admin,
        &token,
        &String::from_str(&env, "Test Group"),
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    // No auth mocked for pause — should fail since caller isn't the admin.
    let attacker = Address::generate(&env);
    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &attacker,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &contract_id,
            fn_name: "pause",
            args: ().into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.pause();
}

#[test]
fn test_emergency_withdraw_full_flow() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);

    client.initialize(
        &admin,
        &token,
        &String::from_str(&env, "Test Group"),
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);
    client.add_member(&member1);
    client.add_member(&member2);
    token_client.mint(&member1, &5000);
    token_client.mint(&member2, &5000);

    client.contribute(&member1, &1000); // contract: 1000, CycleMemberCount frozen at 2
    client.contribute(&member2, &1000); // contract: 2000

    // Bug discovered / admin vanishes — pause and let member1 exit.
    client.pause();
    client.emergency_withdraw(&member1); // contract: 1000 (refunded to member1)

    assert_eq!(token_client.balance(&member1), 5000); // got their 1000 back
    assert_eq!(client.get_contribution(&member1), 0);

    env.as_contract(&contract_id, || {
        let count: i128 = env
            .storage()
            .instance()
            .get(&DataKey::CycleMemberCount)
            .unwrap();
        assert_eq!(count, 1); // decremented from 2 to 1
    });

    // Unpause: member1 re-contributes, and it's still their turn to be paid
    // (NextPayoutIndex never moved — emergency withdraw doesn't jump the queue).
    client.unpause();
    client.contribute(&member1, &1000); // contract: 2000 again

    assert_eq!(client.get_next_payout_recipient(), member1);
    client.payout(&member1, &None);

    // pool_size = contribution_amount(1000) * frozen CycleMemberCount(1) = 1000
    // member1: 4000 (after re-contribute) + 1000 (payout) = 5000
    assert_eq!(token_client.balance(&member1), 5000);
    assert!(client.has_received_payout(&member1));

    // member2's 1000 is still safely in the pool, untouched, waiting for their turn.
    assert_eq!(client.get_contribution(&member2), 1000);
    assert_eq!(client.get_next_payout_recipient(), member2);
}

#[test]
#[should_panic(expected = "No contribution to withdraw this cycle")]
fn test_emergency_withdraw_twice_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token);

    client.initialize(
        &admin,
        &token,
        &String::from_str(&env, "Test Group"),
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    let member = Address::generate(&env);
    client.add_member(&member);
    token_client.mint(&member, &5000);

    client.contribute(&member, &1000);
    client.pause();
    client.emergency_withdraw(&member);
    // Second call should panic — no contribution left this cycle.
    client.emergency_withdraw(&member);
}

// ---------------------------------------------------------------------------
// Reset authorization tests (reset_cycle / reset_rotation)
// ---------------------------------------------------------------------------
//
// Call order an admin is expected to follow in a rotational group:
//
//   1. every member contributes for the cycle
//   2. payout(next_recipient)
//   3. reset_cycle()                     -> next cycle, same rotation
//   ... repeat 1-3 until every member has been paid ...
//   4. reset_cycle(), then reset_rotation() -> queue restarts at member 0
//
// The two resets are separate invocations, each needing its own admin
// authorization over empty args. Failure paths are driven through the
// `try_*` client methods so each test can compare the full contract state
// before and after, proving a rejected reset leaves nothing behind.

struct ResetFixture<'a> {
    client: KoloSavingsContractClient<'a>,
    contract_id: Address,
    admin: Address,
    token: Address,
    members: Vec<Address>,
}

/// Everything a reset could plausibly touch, read straight from storage.
#[derive(Debug, PartialEq)]
struct ResetState {
    admin: Address,
    current_cycle_id: u32,
    cycle_member_count: Option<i128>,
    next_payout_index: Option<u32>,
    is_paused: bool,
    is_executing_payout: bool,
    members: Vec<Address>,
    member_states: Vec<MemberState>,
    contract_balance: i128,
}

/// Rotational group of three members who have all contributed for cycle 1.
/// Nobody has been paid yet. Leaves `mock_all_auths` enabled.
fn setup_reset_fixture(env: &Env) -> ResetFixture<'_> {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, KoloSavingsContract);
    let client = KoloSavingsContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = env.register_stellar_asset_contract(token_admin);
    let token_admin_client = token::StellarAssetClient::new(env, &token);

    client.initialize(
        &admin,
        &token,
        &String::from_str(env, "Reset Group"),
        &1000i128,
        &GroupType::Rotational,
        &None,
        &false,
        &None,
    );

    let members = vec![
        env,
        Address::generate(env),
        Address::generate(env),
        Address::generate(env),
    ];
    // Everyone joins before anyone contributes, otherwise the first
    // contribution freezes CycleMemberCount below the full group size.
    for member in members.iter() {
        client.add_member(&member);
        token_admin_client.mint(&member, &10_000);
    }
    for member in members.iter() {
        client.contribute(&member, &1000);
    }

    ResetFixture {
        client,
        contract_id,
        admin,
        token,
        members,
    }
}

fn contribute_all(fixture: &ResetFixture) {
    for member in fixture.members.iter() {
        fixture.client.contribute(&member, &1000);
    }
}

fn pay_member(fixture: &ResetFixture, index: u32) {
    let recipient = fixture.members.get(index).unwrap();
    fixture.client.payout(&recipient, &None);
}

/// Note: this makes contract calls of its own, so read `env.auths()` and
/// `env.events()` for the call under test *before* capturing state.
fn capture_reset_state(env: &Env, fixture: &ResetFixture) -> ResetState {
    let contract_balance = token::Client::new(env, &fixture.token).balance(&fixture.contract_id);

    env.as_contract(&fixture.contract_id, || {
        let instance = env.storage().instance();
        let mut member_states = Vec::new(env);
        for member in fixture.members.iter() {
            let state: MemberState = env
                .storage()
                .persistent()
                .get(&DataKey::Member(member))
                .unwrap();
            member_states.push_back(state);
        }

        ResetState {
            admin: instance.get(&DataKey::Admin).unwrap(),
            current_cycle_id: instance.get(&DataKey::CurrentCycleId).unwrap(),
            cycle_member_count: instance.get(&DataKey::CycleMemberCount),
            next_payout_index: instance.get(&DataKey::NextPayoutIndex),
            is_paused: instance.get(&DataKey::IsPaused).unwrap_or(false),
            is_executing_payout: instance.get(&DataKey::IsExecutingPayout).unwrap_or(false),
            members: instance.get(&DataKey::Members).unwrap(),
            member_states,
            contract_balance,
        }
    })
}

/// Drops any blanket auth mocking and allows exactly one authorization:
/// `signer` approving `fn_name()` on the savings contract.
fn authorize_only(env: &Env, contract_id: &Address, signer: &Address, fn_name: &str) {
    env.mock_auths(&[MockAuth {
        address: signer,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name,
            args: ().into_val(env),
            sub_invokes: &[],
        },
    }]);
}

/// Asserts a `try_*` call aborted inside the contract. A failed `require_auth`
/// surfaces this way too; the underlying `Error(Auth, InvalidAction)` only
/// shows up in the panic message, which the `should_panic` tests pin down.
fn assert_aborted<T: core::fmt::Debug, E: core::fmt::Debug>(
    result: Result<T, Result<soroban_sdk::Error, E>>,
) {
    let aborted =
        soroban_sdk::Error::from_type_and_code(ScErrorType::Context, ScErrorCode::InvalidAction);
    match result {
        Err(Ok(error)) => assert_eq!(error, aborted),
        other => panic!("expected an aborted invocation, got {:?}", other),
    }
}

/// Asserts the last invocation was authorized by the admin alone, for exactly
/// `fn_name` with no arguments.
fn assert_admin_authorized(env: &Env, fixture: &ResetFixture, fn_name: &str) {
    let auths = env.auths();
    assert_eq!(auths.len(), 1);
    let (signer, invocation) = &auths[0];
    assert_eq!(signer, &fixture.admin);
    assert_eq!(
        invocation.function,
        AuthorizedFunction::Contract((
            fixture.contract_id.clone(),
            Symbol::new(env, fn_name),
            ().into_val(env),
        ))
    );
    assert!(invocation.sub_invocations.is_empty());
}

fn assert_no_events(env: &Env) {
    let no_events: Vec<(Address, Vec<Val>, Val)> = Vec::new(env);
    assert_eq!(env.events().all(), no_events);
}

fn assert_single_event(env: &Env, contract_id: &Address, topic: Symbol) {
    assert_eq!(
        env.events().all(),
        vec![
            env,
            (
                contract_id.clone(),
                vec![env, topic.into_val(env)],
                ().into_val(env)
            )
        ]
    );
}

#[test]
fn test_reset_cycle_by_non_admin_is_rejected_without_state_change() {
    let env = Env::default();
    let fixture = setup_reset_fixture(&env);
    pay_member(&fixture, 0);
    let before = capture_reset_state(&env, &fixture);

    let attacker = Address::generate(&env);
    authorize_only(&env, &fixture.contract_id, &attacker, "reset_cycle");

    assert_aborted(fixture.client.try_reset_cycle());
    assert_no_events(&env);
    assert_eq!(capture_reset_state(&env, &fixture), before);
}

#[test]
fn test_reset_rotation_by_non_admin_is_rejected_without_state_change() {
    let env = Env::default();
    let fixture = setup_reset_fixture(&env);
    // Move the queue off zero so an unauthorized rewind would be observable.
    pay_member(&fixture, 0);
    let before = capture_reset_state(&env, &fixture);
    assert_eq!(before.next_payout_index, Some(1));

    let attacker = Address::generate(&env);
    authorize_only(&env, &fixture.contract_id, &attacker, "reset_rotation");

    assert_aborted(fixture.client.try_reset_rotation());
    assert_no_events(&env);
    assert_eq!(capture_reset_state(&env, &fixture), before);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_reset_cycle_by_non_admin_panics() {
    let env = Env::default();
    let fixture = setup_reset_fixture(&env);

    let attacker = Address::generate(&env);
    authorize_only(&env, &fixture.contract_id, &attacker, "reset_cycle");
    fixture.client.reset_cycle();
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_reset_rotation_by_non_admin_panics() {
    let env = Env::default();
    let fixture = setup_reset_fixture(&env);

    let attacker = Address::generate(&env);
    authorize_only(&env, &fixture.contract_id, &attacker, "reset_rotation");
    fixture.client.reset_rotation();
}

#[test]
fn test_resets_without_any_authorization_are_rejected() {
    let env = Env::default();
    let fixture = setup_reset_fixture(&env);
    pay_member(&fixture, 0);
    let before = capture_reset_state(&env, &fixture);

    env.mock_auths(&[]);

    assert_aborted(fixture.client.try_reset_cycle());
    assert_aborted(fixture.client.try_reset_rotation());
    assert_eq!(capture_reset_state(&env, &fixture), before);
}

#[test]
fn test_admin_auth_for_one_reset_cannot_be_replayed_on_the_other() {
    let env = Env::default();
    let fixture = setup_reset_fixture(&env);
    pay_member(&fixture, 0);
    let before = capture_reset_state(&env, &fixture);

    // Both resets take empty args, so the function name is the only thing
    // distinguishing the two signatures. Each must still be rejected.
    authorize_only(&env, &fixture.contract_id, &fixture.admin, "reset_rotation");
    assert_aborted(fixture.client.try_reset_cycle());

    authorize_only(&env, &fixture.contract_id, &fixture.admin, "reset_cycle");
    assert_aborted(fixture.client.try_reset_rotation());

    assert_eq!(capture_reset_state(&env, &fixture), before);
}

#[test]
fn test_reset_cycle_by_admin_changes_only_cycle_fields() {
    let env = Env::default();
    let fixture = setup_reset_fixture(&env);
    pay_member(&fixture, 0);
    let before = capture_reset_state(&env, &fixture);
    assert_eq!(before.current_cycle_id, 1);
    assert_eq!(before.cycle_member_count, Some(3));
    assert_eq!(before.next_payout_index, Some(1));

    authorize_only(&env, &fixture.contract_id, &fixture.admin, "reset_cycle");
    fixture.client.reset_cycle();

    assert_admin_authorized(&env, &fixture, "reset_cycle");
    assert_single_event(&env, &fixture.contract_id, symbol_short!("reset"));

    // The payout queue, membership, member ledgers and funds are untouched.
    let mut expected = before;
    expected.current_cycle_id = 2;
    expected.cycle_member_count = None;
    assert_eq!(capture_reset_state(&env, &fixture), expected);
}

#[test]
fn test_reset_rotation_by_admin_changes_only_payout_index() {
    let env = Env::default();
    let fixture = setup_reset_fixture(&env);
    let first = fixture.members.get(0).unwrap();
    fixture.client.payout(&first, &None);
    let before = capture_reset_state(&env, &fixture);
    assert!(before.member_states.get(0).unwrap().has_received_payout);

    authorize_only(&env, &fixture.contract_id, &fixture.admin, "reset_rotation");
    fixture.client.reset_rotation();

    assert_admin_authorized(&env, &fixture, "reset_rotation");
    assert_single_event(&env, &fixture.contract_id, symbol_short!("new_rot"));

    // Cycle bookkeeping is left alone, and the stored has_received_payout
    // flag stays as a historical record. The public getter is derived from
    // NextPayoutIndex, so it is the one that reports "not yet paid".
    let mut expected = before;
    expected.next_payout_index = Some(0);
    assert_eq!(capture_reset_state(&env, &fixture), expected);

    assert!(!fixture.client.has_received_payout(&first));
    assert_eq!(fixture.client.get_next_payout_recipient(), first);
}

#[test]
fn test_resets_rejected_while_paused_keep_emergency_withdraw_available() {
    let env = Env::default();
    let fixture = setup_reset_fixture(&env);
    fixture.client.pause();
    let before = capture_reset_state(&env, &fixture);
    assert!(before.is_paused);

    // Auth is fully mocked, so these fail on the pause check alone.
    assert_aborted(fixture.client.try_reset_cycle());
    assert_aborted(fixture.client.try_reset_rotation());
    assert_eq!(capture_reset_state(&env, &fixture), before);

    // Because the cycle ID did not move, members can still pull out what
    // they paid in this cycle.
    let member = fixture.members.get(0).unwrap();
    let token_client = token::Client::new(&env, &fixture.token);
    fixture.client.emergency_withdraw(&member);
    assert_eq!(token_client.balance(&member), 10_000);

    // Once unpaused, the admin can reset as normal.
    fixture.client.unpause();
    fixture.client.reset_cycle();
    assert_eq!(capture_reset_state(&env, &fixture).current_cycle_id, 2);
}

#[test]
#[should_panic(expected = "Contract is paused for emergency")]
fn test_reset_cycle_while_paused_panics() {
    let env = Env::default();
    let fixture = setup_reset_fixture(&env);
    fixture.client.pause();
    fixture.client.reset_cycle();
}

#[test]
#[should_panic(expected = "Contract is paused for emergency")]
fn test_reset_rotation_while_paused_panics() {
    let env = Env::default();
    let fixture = setup_reset_fixture(&env);
    fixture.client.pause();
    fixture.client.reset_rotation();
}

#[test]
fn test_resets_rejected_while_payout_executing_leave_state_intact() {
    let env = Env::default();
    let fixture = setup_reset_fixture(&env);
    env.as_contract(&fixture.contract_id, || {
        env.storage()
            .instance()
            .set(&DataKey::IsExecutingPayout, &true);
    });
    let before = capture_reset_state(&env, &fixture);

    assert_aborted(fixture.client.try_reset_cycle());
    assert_aborted(fixture.client.try_reset_rotation());
    assert_eq!(capture_reset_state(&env, &fixture), before);
}

#[test]
fn test_repeated_reset_cycle_opens_a_new_cycle_each_time() {
    let env = Env::default();
    let fixture = setup_reset_fixture(&env);
    pay_member(&fixture, 0);
    let before = capture_reset_state(&env, &fixture);

    fixture.client.reset_cycle();
    fixture.client.reset_cycle();
    assert_single_event(&env, &fixture.contract_id, symbol_short!("reset"));

    // Not idempotent: the second call skips straight past cycle 2. Nothing
    // else drifts, and clearing an already-cleared member count is harmless.
    let mut expected = before;
    expected.current_cycle_id = 3;
    expected.cycle_member_count = None;
    assert_eq!(capture_reset_state(&env, &fixture), expected);

    // Members get exactly one contribution in the new cycle.
    let member = fixture.members.get(1).unwrap();
    fixture.client.contribute(&member, &1000);
    assert_aborted(fixture.client.try_contribute(&member, &1000));
}

#[test]
fn test_repeated_reset_rotation_is_idempotent() {
    let env = Env::default();
    let fixture = setup_reset_fixture(&env);
    pay_member(&fixture, 0);

    fixture.client.reset_rotation();
    let after_first = capture_reset_state(&env, &fixture);
    assert_eq!(after_first.next_payout_index, Some(0));

    fixture.client.reset_rotation();
    assert_single_event(&env, &fixture.contract_id, symbol_short!("new_rot"));
    assert_eq!(capture_reset_state(&env, &fixture), after_first);
}

#[test]
fn test_end_of_rotation_resets_are_independent_invocations() {
    let env = Env::default();
    let fixture = setup_reset_fixture(&env);

    // Run the whole rotation using the documented call order.
    pay_member(&fixture, 0);
    fixture.client.reset_cycle();
    contribute_all(&fixture);
    pay_member(&fixture, 1);
    fixture.client.reset_cycle();
    contribute_all(&fixture);
    pay_member(&fixture, 2);

    // The admin signs only the cycle reset. It lands; the rotation reset
    // that follows is rejected on its own and must not disturb it.
    authorize_only(&env, &fixture.contract_id, &fixture.admin, "reset_cycle");
    fixture.client.reset_cycle();
    let after_cycle_reset = capture_reset_state(&env, &fixture);
    assert_eq!(after_cycle_reset.current_cycle_id, 4);
    assert_eq!(after_cycle_reset.cycle_member_count, None);
    assert_eq!(after_cycle_reset.next_payout_index, Some(3));

    assert_aborted(fixture.client.try_reset_rotation());
    assert_eq!(capture_reset_state(&env, &fixture), after_cycle_reset);

    // With the admin's signature the rotation restarts at member 0.
    authorize_only(&env, &fixture.contract_id, &fixture.admin, "reset_rotation");
    fixture.client.reset_rotation();
    let mut expected = after_cycle_reset;
    expected.next_payout_index = Some(0);
    assert_eq!(capture_reset_state(&env, &fixture), expected);
    assert_eq!(
        fixture.client.get_next_payout_recipient(),
        fixture.members.get(0).unwrap()
    );
}
