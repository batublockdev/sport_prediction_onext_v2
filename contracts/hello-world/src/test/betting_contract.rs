#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage;
    use crate::types::{
        AssessmentKey, Bet, BetKey, BetType, ClaimType, Game, PrivateBet, PublicBet, ResultGame,
    };
    use crate::{BettingContract, BettingContractClient};
    use alloc::vec::Vec;
    use ed25519_dalek::{Keypair, Signer};
    use rand::thread_rng;
    use soroban_sdk::token;
    use soroban_sdk::xdr::ToXdr;
    use soroban_sdk::Vec as SorobanVec;
    use soroban_sdk::{
        testutils::{
            budget::Budget, Address as _, AuthorizedFunction, AuthorizedInvocation, BytesN as _,
            Events,
        },
        testutils::{Address as _, Ledger, LedgerInfo},
        xdr::WriteXdr,
        Address, Bytes, BytesN, BytesN as _, Env, IntoVal, InvokeError, String, TryIntoVal, Val,
    };
    use token::Client as TokenClient;
    use token::StellarAssetClient as TokenAdminClient;
    extern crate alloc;

    fn create_token_contract<'a>(
        e: &Env,
        admin: &Address,
    ) -> (TokenClient<'a>, TokenAdminClient<'a>) {
        let sac = e.register_stellar_asset_contract_v2(admin.clone());
        (
            token::Client::new(e, &sac.address()),
            token::StellarAssetClient::new(e, &sac.address()),
        )
    }
    fn create_test_env() -> (
        Env,
        BettingContractClient<'static>,
        Address,
        Address,
        Address,
        Address,
        TokenClient<'static>,
        TokenClient<'static>,
    ) {
        let env = Env::default();
        env.mock_all_auths(); // Mock all authorizations for testing

        // Register the betting contract
        let contract_id = env.register_contract(None, BettingContract);
        let client = BettingContractClient::new(&env, &contract_id);

        // Create mock accounts
        let admin = Address::generate(&env);
        let user = Address::generate(&env);

        // Register mock token contracts
        let (token_usd, token_usd_admin) = create_token_contract(&env, &admin);
        let (token_trust, token_trust_admin) = create_token_contract(&env, &admin);

        // Mint initial tokens to user for testing
        token_usd_admin.mint(&user, &100_000_000);
        token_trust_admin.mint(&user, &100_000_000);
        let addUsd = token_usd.address.clone();
        let addTrust = token_trust.address.clone();
        (
            env,
            client,
            admin,
            user,
            token_usd.address.clone(),
            token_trust.address.clone(),
            token_usd,
            token_trust,
        )
    }

    fn set_ledger_timestamp(env: &Env, timestamp: u32) {
        env.ledger().set(LedgerInfo {
            timestamp: timestamp as u64,
            protocol_version: 22, // Updated to match soroban-sdk 22.0.8
            sequence_number: env.ledger().sequence(),
            base_reserve: 10,
            ..Default::default()
        });
    }

    #[test]
    fn test_init_success() {
        let (env, client, admin, _, token_usd, token_trust, _, _) = create_test_env();

        client.init(&admin, &token_usd, &token_trust);
    }

    #[test]
    #[should_panic(expected = "AlreadyInitializedError")]
    fn test_init_already_initialized() {
        let (env, client, admin, _, token_usd, token_trust, _, _) = create_test_env();

        client.init(&admin, &token_usd, &token_trust);
        //client.init(&admin, &token_usd, &token_trust); // Should panic
    }

    #[test]
    fn test_request_result_summiter() {
        let (env, client, admin, user, token_usd, token_trust, _, _) = create_test_env();
        client.init(&admin, &token_usd, &token_trust);

        let game_id = 1;
        let stake_amount = 1000;

        let result = client.request_result_summiter(&user, &stake_amount, &game_id);
        assert_eq!(result, true);
    }

    #[test]
    #[should_panic(expected = "NegativeAmountError")]
    fn test_request_result_summiter_negative_amount() {
        let (env, client, admin, user, token_usd, token_trust, _, _) = create_test_env();
        client.init(&admin, &token_usd, &token_trust);

        client.request_result_summiter(&user, &-100, &1); // Should panic
    }

    #[test]
    fn test_bet_public() {
        let (env, client, admin, user, token_usd, token_trust, usd_client, trust_client) =
            create_test_env();
        client.init(&admin, &token_usd, &token_trust);

        // Set up a game
        let game_id = 1;

        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,
            summiter: Address::generate(&env),
            Checker: soroban_sdk::Vec::new(&env),
            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();

        let signer1 = Keypair::generate(&mut thread_rng());
        let public_key = BytesN::<32>::from_array(&env, &signer1.public.to_bytes());

        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &signer1.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex, &public_key);

        // Set ledger timestamp
        set_ledger_timestamp(&env, 1500);

        // Place a public bet
        let bet = Bet {
            id: 1,
            Setting: game_id,
            bet: BetKey::Team_local,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_id,
        };

        let initial_usd_balance = usd_client.balance(&user);
        let initial_trust_balance = trust_client.balance(&user);

        client.bet(&user, &bet);

        // Verify token transfers
        assert_eq!(usd_client.balance(&user), initial_usd_balance - 1000);
        assert_eq!(trust_client.balance(&user), initial_trust_balance - 300); // 30% of 1000
    }

    #[test]
    #[should_panic(expected = "GameHasAlreadyStarted")]
    fn test_bet_before_game_start() {
        let (env, client, admin, user, token_usd, token_trust, _, _) = create_test_env();
        client.init(&admin, &token_usd, &token_trust);

        let game_id = 1;
        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,
            summiter: Address::generate(&env),
            Checker: soroban_sdk::Vec::new(&env),
            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();

        let signer1 = Keypair::generate(&mut thread_rng());
        let public_key = BytesN::<32>::from_array(&env, &signer1.public.to_bytes());

        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &signer1.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex, &public_key);

        set_ledger_timestamp(&env, 1500);

        let bet = Bet {
            id: 1,
            Setting: game_id,
            bet: BetKey::Team_local,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_id,
        };

        client.bet(&user, &bet); // Should panic
    }

    #[test]
    fn test_claim_money_noactive_public() {
        let (env, client, admin, user, token_usd, token_trust, usd_client, trust_client) =
            create_test_env();
        client.init(&admin, &token_usd, &token_trust);

        // Set up a game
        let game_id = 1;
        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,
            summiter: Address::generate(&env),
            Checker: soroban_sdk::Vec::new(&env),
            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();

        let signer1 = Keypair::generate(&mut thread_rng());
        let public_key = BytesN::<32>::from_array(&env, &signer1.public.to_bytes());

        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &signer1.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex, &public_key);

        // Place a public bet
        let bet = Bet {
            id: 1,
            Setting: game_id,
            bet: BetKey::Team_local,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_id,
        };
        client.bet(&user, &bet);
        set_ledger_timestamp(&env, 1500);

        // Claim before game starts
        let initial_usd_balance = usd_client.balance(&user);
        let initial_trust_balance = trust_client.balance(&user);

        client.claim_money_noactive(&user, &game_id);

        // Verify token refunds
        assert_eq!(usd_client.balance(&user), initial_usd_balance + 1000);
        assert_eq!(trust_client.balance(&user), initial_trust_balance + 300);
    }

    #[test]
    fn test_summit_result() {
        let (env, client, admin, user, token_usd, token_trust, _, _) = create_test_env();
        client.init(&admin, &token_usd, &token_trust);

        // Set up a game
        let game_id = 1;
        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,
            summiter: Address::generate(&env),
            Checker: soroban_sdk::Vec::new(&env),
            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();

        let signer1 = Keypair::generate(&mut thread_rng());
        let public_key = BytesN::<32>::from_array(&env, &signer1.public.to_bytes());

        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &signer1.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex, &public_key);

        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 2500);

        let result = ResultGame {
            id: 1,
            gameid: game_id,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_slice(&env, "Final Score 2-1"),
        };

        let submitted_result = client.summitResult(&user, &result);
        assert_eq!(submitted_result, result);
    }

    #[test]
    #[should_panic(expected = "NotAllowToSummitResult")]
    fn test_summit_result_unauthorized() {
        let (env, client, admin, user, token_usd, token_trust, _, _) = create_test_env();
        client.init(&admin, &token_usd, &token_trust);

        let game_id = 1;
        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,
            summiter: Address::generate(&env),
            Checker: soroban_sdk::Vec::new(&env),
            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();

        let signer1 = Keypair::generate(&mut thread_rng());
        let public_key = BytesN::<32>::from_array(&env, &signer1.public.to_bytes());

        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &signer1.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex, &public_key);

        set_ledger_timestamp(&env, 2500);

        let result = ResultGame {
            id: 1,
            gameid: game_id,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_slice(&env, "Final Score 2-1"),
        };

        client.summitResult(&user, &result); // Should panic
    }

    #[test]
    fn test_assess_result() {
        let (env, client, admin, user, token_usd, token_trust, _, _) = create_test_env();
        client.init(&admin, &token_usd, &token_trust);

        // Set up a game
        let game_id = 1;
        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,
            summiter: Address::generate(&env),
            Checker: soroban_sdk::Vec::new(&env),
            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();

        let signer1 = Keypair::generate(&mut thread_rng());
        let public_key = BytesN::<32>::from_array(&env, &signer1.public.to_bytes());

        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &signer1.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex, &public_key);

        // Assess result
        let bet = Bet {
            id: 1,
            Setting: game_id,
            bet: BetKey::Team_local,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_id,
        };
        set_ledger_timestamp(&env, 1500);

        client.bet(&user, &bet);
        // Submit result
        set_ledger_timestamp(&env, 2500);
        let result = ResultGame {
            id: 1,
            gameid: game_id,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_slice(&env, "Final Score 2-1"),
        };
        client.summitResult(&game.summiter, &result);

        client.assessResult(&user, &bet, &game_id, &AssessmentKey::approve);
    }

    #[test]
    fn test_claim_winner_honest() {
        let (env, client, admin, user, token_usd, token_trust, usd_client, trust_client) =
            create_test_env();
        client.init(&admin, &token_usd, &token_trust);

        // Set up a game
        let game_id = 1;
        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,
            summiter: Address::generate(&env),
            Checker: soroban_sdk::Vec::new(&env),
            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();

        let signer1 = Keypair::generate(&mut thread_rng());
        let public_key = BytesN::<32>::from_array(&env, &signer1.public.to_bytes());

        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &signer1.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex, &public_key);

        // Place a bet
        let bet = Bet {
            id: 1,
            Setting: game_id,
            bet: BetKey::Team_local,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_id,
        };
        set_ledger_timestamp(&env, 1500);
        client.bet(&user, &bet);

        // Submit and assess result
        set_ledger_timestamp(&env, 2500);
        let result = ResultGame {
            id: 1,
            gameid: game_id,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_slice(&env, "Final Score 2-1"),
        };
        client.summitResult(&game.summiter, &result);
        client.assessResult(&user, &bet, &game_id, &AssessmentKey::approve);

        // Execute distribution
        client.execute_distribution(&game_id);

        // Claim as winner
        let initial_usd_balance = usd_client.balance(&user);
        let initial_trust_balance = trust_client.balance(&user);

        client.claim(&user, &ClaimType::User, &game_id);

        // Verify token transfers (winner gets bet + share of pool)
        assert!(usd_client.balance(&user) > initial_usd_balance);
        assert_eq!(trust_client.balance(&user), initial_trust_balance + 300); // Trust tokens returned
    }

    #[test]
    fn test_set_result_supreme_court() {
        let (env, client, admin, user, token_usd, token_trust, _, _) = create_test_env();
        client.init(&admin, &token_usd, &token_trust);

        // Set up a game
        let game_id = 1;
        let game = Game {
            id: game_id,
            startTime: 1000,
            endTime: 2000,
            summiter: Address::generate(&env),
            Checker: soroban_sdk::Vec::new(&env),
            active: false,
            league: 1,
            description: String::from_slice(&env, "Team A vs Team B"),
            team_local: 33,
            team_away: 44,
        };
        // Encode game to Bytes (variable length)
        let encoded: Vec<u8> = game.clone().to_xdr(&env).iter().collect();

        let signer1 = Keypair::generate(&mut thread_rng());
        let public_key = BytesN::<32>::from_array(&env, &signer1.public.to_bytes());

        let signaturex: BytesN<64> =
            BytesN::from_array(&env, &signer1.sign(encoded.as_slice()).to_bytes());
        client.set_game(&game, &signaturex, &public_key);

        // Submit initial result
        set_ledger_timestamp(&env, 2500);
        let initial_result = ResultGame {
            id: 1,
            gameid: game_id,
            result: BetKey::Team_local,
            pause: true,
            description: String::from_slice(&env, "Final Score 2-1"),
        };
        client.summitResult(&game.summiter, &initial_result);

        // Supreme court sets new result
        let new_result = ResultGame {
            id: 1,
            gameid: game_id,
            result: BetKey::Team_away,
            pause: false,
            description: String::from_slice(&env, "Final Score 1-2"),
        };
        client.setResult_supremCourt(&user, &new_result);

        let stored_result = storage::get_ResultGame(env.clone(), game_id);
        assert_eq!(stored_result, new_result);
    }
}
