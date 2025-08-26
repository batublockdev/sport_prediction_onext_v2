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
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;

extern crate std;

fn create_token_contract<'a>(e: &Env, admin: &Address) -> (TokenClient<'a>, TokenAdminClient<'a>) {
    let sac = e.register_stellar_asset_contract_v2(admin.clone());
    (
        token::Client::new(e, &sac.address()),
        token::StellarAssetClient::new(e, &sac.address()),
    )
}
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
    let playerx = Address::generate(&env);
    let playerx2 = Address::generate(&env);
    let token_admin = Address::generate(&env);

    env.mock_all_auths();

    // Deploy contract
    let contract_id = env.register_contract(None, Contract);
    let client = ContractClient::new(&env, &contract_id);
    let (token_a, token_a_admin) = create_token_contract(&env, &token_admin);
    let (token_b, token_b_admin) = create_token_contract(&env, &token_admin);
    client.init(&token_admin, &token_a.address, &token_b.address);
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
        li.sequence_number = 0; // mock ledger number
        li.timestamp = 0; // mock Unix timestamp
    });
    env.budget().reset_unlimited();

    // 2. Create the Game object
    let game = Game {
        id: 1,
        active: false,
        league: 101,
        description: String::from_str(&env, "Test Game"),
        team_local: 10,
        team_away: 20,
        startTime: 0,
        endTime: 50,
        summiter: user1.clone(),
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

    let RD = Bet {
        id: 1,
        gameid: 1,
        betType: BetType::Public,
        Setting: 1,
        bet: BetKey::Team_away,
        amount_bet: 1500,
    };
    token_a_admin.mint(&player, &10000);
    client.bet(&player, &RD);
    let RD2 = Bet {
        id: 2,
        gameid: 1,
        betType: BetType::Public,
        Setting: 1,
        bet: BetKey::Team_local,
        amount_bet: 2000,
    };
    token_a_admin.mint(&playerx, &10000);
    client.bet(&playerx, &RD2);
    let RD3 = Bet {
        id: 3,
        gameid: 1,
        betType: BetType::Public,
        Setting: 1,
        bet: BetKey::Draw,
        amount_bet: 3000,
    };
    token_a_admin.mint(&user12, &10000);
    client.bet(&user12, &RD3);
    let RD4 = Bet {
        id: 4,
        gameid: 1,
        betType: BetType::Public,
        Setting: 1,
        bet: BetKey::Team_local,
        amount_bet: 3000,
    };
    token_a_admin.mint(&playerx2, &10000);
    //client.bet(&playerx2, &RD4);
    let results = ResultGame {
        id: 3,
        gameid: 1,
        description: String::from_str(&env, "Test Result"),
        result: BetKey::Team_away,
        pause: false,
    };
    env.clone().ledger().with_mut(|li| {
        li.sequence_number = 0; // mock ledger number
        li.timestamp = 75; // mock Unix timestamp
    });
    client.summitResult(&user9, &results);
    client.assessResult(&user3, &RD, &1, &AssessmentKey::approve);
    client.assessResult(&player, &RD, &1, &AssessmentKey::approve);
    client.assessResult(&playerx, &RD2, &1, &AssessmentKey::approve);
    client.assessResult(&user12, &RD3, &1, &AssessmentKey::approve);
    let resultsx = ResultGame {
        id: 3,
        gameid: 1,
        description: String::from_str(&env, "Test Result"),
        result: BetKey::Team_local,
        pause: false,
    };
    client.execute_distribution(&1);
    let dx = ClaimType::User;
    //client.setResult_supremCourt(&user1, &resultsx);
    let ee = client.claim(&player, &dx);
    let earnPlayer = token_a.balance(&player);
    std::println!("Earned by Player: {:?}", earnPlayer);
    let y = client.claim(&playerx, &dx);
    let earnPlayerx = token_a.balance(&playerx);
    std::println!("Earned by Playerx: {:?}", earnPlayerx);
    let z = client.claim(&user12, &dx);
    let earnUser12 = token_a.balance(&user12);
    std::println!("Earned by User12: {:?}", earnUser12);
    //let earnPlayerx2 = client.claim(&playerx2, &RD4);
    //std::println!("Earned by Playerx2: {:?}", earnPlayerx2);
    let sequence = env.ledger().sequence();
    let timestamp = env.ledger().timestamp();
    std::println!("Sequence: {:?}", sequence);
    std::println!("Timestamp: {:?}", timestamp);
}
