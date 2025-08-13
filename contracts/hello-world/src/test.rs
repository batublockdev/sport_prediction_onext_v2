#![cfg(test)]

use super::*;
use ed25519_dalek::{Keypair, Signer};
use soroban_sdk::{
    log, symbol_short,
    testutils::{
        budget::Budget, Accounts, Address as _, AuthorizedFunction, AuthorizedInvocation, BytesN,
        Env, Events, IntoVal, Ledger,
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
    let player = Address::generate(&env);

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

    // 1. Create a keypair (for signing)
    let keypair = Keypair::generate(&mut rand::rngs::OsRng);
    let pub_key_bytes = BytesN::from_array(&env, &keypair.public.to_bytes());

    // 2. Build a Game instance
    let game = Game {
        id: 1,
        name: String::from_str(&env, "Test Game"),
        // ... fill in all required fields
    };

    // 3. Serialize game data the same way your signature verification expects
    let game_bytes = game.clone().into_val(&env).to_xdr(&env);

    // 4. Sign it
    let signature = keypair.sign(&game_bytes).to_bytes();
    let signature_bytes = BytesN::from_array(&env, &signature);

    // 5. Call set_game
    set_game(env.clone(), &game, signature_bytes, pub_key_bytes);

    let sequence = env.ledger().sequence();
    let timestamp = env.ledger().timestamp();
    std::println!("Sequence: {:?}", sequence);
    std::println!("Timestamp: {:?}", timestamp);
}
