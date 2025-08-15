#![cfg(test)]
#![no_std]

extern crate alloc;
use super::*;
use alloc::vec::Vec;
use ed25519_dalek::{Keypair, Signer};
use rand::thread_rng;
use soroban_sdk::Vec as SorobanVec;
use soroban_sdk::{
    log, symbol_short,
    testutils::{
        budget::Budget, Address as _, AuthorizedFunction, AuthorizedInvocation, BytesN as _,
        Events, Ledger,
    },
    vec,
    xdr::WriteXdr,
    Bytes, BytesN, Env, IntoVal, InvokeError, TryIntoVal, Val,
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

    let xx = env.events().all();
    for e in xx.iter() {
        std::println!("event: {:?}", e);
    }
    // Read leaderboard
    std::println!("Fuckkkkkk this hsit");
    if let Some((contract, symbols, obj)) = xx.get(0) {
        std::println!("Contract: {:?}", contract);
        std::println!("First object: {:?}", obj);
        let leaderboard: SorobanVec<(Address, i128)> = obj.try_into_val(&env).unwrap();
        for val in leaderboard.iter() {
            std::println!("Value: {:?}", val);
        }
    }
    env.clone().ledger().with_mut(|li| {
        li.sequence_number = 4; // mock ledger number
        li.timestamp = 72; // mock Unix timestamp
    });
    env.budget().reset_unlimited();

    // 2. Create the Game object
    let game = Game {
        id: 1,
        league: 101,
        description: String::from_str(&env, "Test Game"),
        team_local: 10,
        team_away: 20,
        startTime: 1111,
        endTime: 2222,
        summiter: user1,
        Checker: soroban_sdk::Vec::new(&env),
    };
    // 3. Convert the Game object to BytesN

    // Encode game to Bytes (variable length)
    let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();
    let g: &[u8] = encoded.as_slice();

    let signer1 = Keypair::generate(&mut thread_rng());
    let signer2 = Keypair::generate(&mut thread_rng());
    let public_key = BytesN::<32>::from_array(&env, &signer1.public.to_bytes());

    let signaturex: BytesN<64> =
        BytesN::from_array(&env, &signer1.sign(encoded.as_slice()).to_bytes());
    client.set_game(&game, &signaturex, &public_key);
    let xx2 = env.events().all();

    for e in xx2.iter() {
        std::println!("event: {:?}", e);
    }
    if let Some((contract, symbols, obj)) = xx2.get(0) {
        std::println!("Contract: {:?}", contract);
        std::println!("First object: {:?}", obj);
        let Gr: Address = obj.try_into_val(&env).unwrap();
        std::println!("Value: {:?}", Gr);
        /*for val in leaderboard.iter() {
            std::println!("Value: {:?}", val);
        }*/
    }
    let sequence = env.ledger().sequence();
    let timestamp = env.ledger().timestamp();
    std::println!("Sequence: {:?}", sequence);
    std::println!("Timestamp: {:?}", timestamp);
}
