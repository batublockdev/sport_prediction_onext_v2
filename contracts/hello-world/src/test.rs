#![cfg(test)]

use super::*;
use soroban_sdk::{
    log, symbol_short,
    testutils::{
        budget::Budget, Address as _, AuthorizedFunction, AuthorizedInvocation, Events, Ledger,
    },
    vec, Env, IntoVal, TryIntoVal, Val, Vec,
};

extern crate std;

#[test]
fn test_leaderboard_updates_correctly() {
    let env = Env::default();

    // ✅ Convert to Address
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    let user4 = Address::generate(&env);
    let user5 = Address::generate(&env);
    let user6 = Address::generate(&env);
    let user7 = Address::generate(&env);
    let user8 = Address::generate(&env);
    let user9 = Address::generate(&env);
    let user10 = Address::generate(&env);
    let user11 = Address::generate(&env);
    let user12 = Address::generate(&env);
    env.mock_all_auths();

    // Deploy contract
    let contract_id = env.register_contract(None, Contract);
    let client = ContractClient::new(&env, &contract_id);

    // User1 stake 100
    client.request_result_summiter(&user1, &100, &1);
    client.request_result_summiter(&user11, &100, &1);
    // User2 stake 200
    client.request_result_summiter(&user2, &200, &1);

    // User1 submits again with 300 (should be top)
    client.request_result_summiter(&user3, &350, &1);
    client.request_result_summiter(&user4, &300, &1);
    client.request_result_summiter(&user5, &320, &1);
    client.request_result_summiter(&user6, &300, &1);
    client.request_result_summiter(&user7, &390, &1);
    client.request_result_summiter(&user8, &390, &1);
    client.request_result_summiter(&user9, &396, &1);
    client.request_result_summiter(&user10, &360, &1);

    let x = env.events().all();
    // Read leaderboard
    std::println!("Fuckkkkkk this hsit");
    if let Some((contract, symbols, obj)) = x.get(0) {
        std::println!("Contract: {:?}", contract);
        std::println!("First object: {:?}", obj);
        let leaderboard: Vec<(Address, i128)> = obj.try_into_val(&env).unwrap();
        for val in leaderboard.iter() {
            std::println!("Value: {:?}", val);
        }
    }
    env.clone().ledger().with_mut(|li| {
        li.sequence_number = 4; // mock ledger number
        li.timestamp = 72; // mock Unix timestamp
    });
    env.budget().reset_unlimited();

    let grasa: Vec<(Address, i128)> = client.select_summiter(&1);
    for val in grasa.iter() {
        std::println!("Value Select: {:?}", val);
    }

    let sequence = env.ledger().sequence();
    let timestamp = env.ledger().timestamp();
    std::println!("Sequence: {:?}", sequence);
    std::println!("Timestamp: {:?}", timestamp);
}
