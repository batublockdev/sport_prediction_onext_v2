#![cfg(test)]

use super::*;
use soroban_sdk::{
    log, symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events, Ledger},
    vec, Env, IntoVal, TryIntoVal, Val, Vec,
};

extern crate std;

#[test]
fn test_leaderboard_updates_correctly() {
    let env = Env::default();

    // ✅ Convert to Address
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    env.mock_all_auths();

    // Deploy contract
    let contract_id = env.register_contract(None, Contract);
    let client = ContractClient::new(&env, &contract_id);

    // User1 stake 100
    client.request_result_summiter(&user1, &100, &1);

    // User2 stake 200
    client.request_result_summiter(&user2, &200, &1);

    // User1 submits again with 300 (should be top)
    client.request_result_summiter(&user1, &300, &1);
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
    env.ledger().with_mut(|li| {
        li.sequence_number = 12345; // mock ledger number
        li.timestamp = 1_726_020_000; // mock Unix timestamp
    });
    let sequence = env.ledger().sequence();
    let timestamp = env.ledger().timestamp();
    std::println!("Sequence: {:?}", sequence);
    std::println!("Timestamp: {:?}", timestamp);
}
