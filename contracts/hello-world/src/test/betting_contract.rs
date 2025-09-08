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
    use soroban_sdk::xdr::ToXdr;
    use soroban_sdk::Vec as SorobanVec;
    use soroban_sdk::{symbol_short, token};
    use soroban_sdk::{testutils::Events, vec, Env, IntoVal};
    use soroban_sdk::{
        testutils::{
            budget::Budget, Address as _, AuthorizedFunction, AuthorizedInvocation, BytesN as _,
        },
        testutils::{Address as _, Ledger, LedgerInfo},
        xdr::WriteXdr,
        Address, Bytes, BytesN, BytesN as _, InvokeError, String, TryIntoVal, Val,
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
        TokenAdminClient<'static>,
        TokenAdminClient<'static>,
    ) {
        let env = Env::default();
        env.mock_all_auths(); // Mock all authorizations for testing

        // Create mock accounts
        let admin = Address::generate(&env);
        let user = Address::generate(&env);

        // Register mock token contracts
        let (token_usd, token_usd_admin) = create_token_contract(&env, &admin);
        let (token_trust, token_trust_admin) = create_token_contract(&env, &admin);

        // Mint initial tokens to user for testing
        token_usd_admin.mint(&user, &100_000_000);
        token_trust_admin.mint(&user, &100_000_000);
        // Register the betting contract
        let contract_id = env.register(
            BettingContract,
            (&admin, &token_usd.address, &token_trust.address),
        );
        let client = BettingContractClient::new(&env, &contract_id);
        (
            env,
            client,
            admin,
            user,
            token_usd.address.clone(),
            token_trust.address.clone(),
            token_usd,
            token_trust,
            token_usd_admin,
            token_trust_admin,
        )
    }

    fn set_ledger_timestamp(env: &Env, timestamp: u32) {
        env.ledger().set(LedgerInfo {
            timestamp: timestamp as u64,
            protocol_version: 23, // Updated to match soroban-sdk 23.0.1
            sequence_number: env.ledger().sequence(),
            base_reserve: 10,
            ..Default::default()
        });
    }

    #[test]
    fn test_request_result_summiter() {
        let (env, client, admin, user, token_usd, token_trust, _, _, _, _) = create_test_env();
        //client.init(&admin, &token_usd, &token_trust);

        let game_id = 1;
        let stake_amount = 1000;

        let result = client.request_result_summiter(&user, &stake_amount);
        assert_eq!(result, true);
    }
    // Test error =Negative amount
    #[test]
    #[should_panic(expected = "Error(Contract, #223)")]
    fn test_request_result_summiter_negative_amount() {
        let (env, client, admin, user, token_usd, token_trust, _, _, _, _) = create_test_env();
        //client.init(&admin, &token_usd, &token_trust);

        client.request_result_summiter(&user, &-100); // Should panic
    }

    #[test]
    fn test_bet_public() {
        let (env, client, admin, user, token_usd, token_trust, usd_client, trust_client, _, _) =
            create_test_env();
        //client.init(&admin, &token_usd, &token_trust);

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

        let initial_usd_balance = usd_client.balance(&user);
        let initial_trust_balance = trust_client.balance(&user);

        client.bet(&user, &bet);

        // Verify token transfers
        assert_eq!(usd_client.balance(&user), initial_usd_balance - 1000);
        assert_eq!(trust_client.balance(&user), initial_trust_balance - 300); // 30% of 1000
    }

    //   Error expected = "GameHasAlreadyStarted"
    #[test]
    #[should_panic(expected = "Error(Contract, #207)")]
    fn test_bet_after_game_start() {
        let (env, client, admin, user, token_usd, token_trust, _, _, _, _) = create_test_env();
        //client.init(&admin, &token_usd, &token_trust);

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
        let (env, client, admin, user, token_usd, token_trust, usd_client, trust_client, _, _) =
            create_test_env();
        //client.init(&admin, &token_usd, &token_trust);

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
        let (env, client, admin, user, token_usd, token_trust, _, _, adm_usd, adm_trust) =
            create_test_env();
        //client.init(&admin, &token_usd, &token_trust);

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
        //add the user who wna tto participate as a summiter
        let summiter = Address::generate(&env);
        let summiter2 = Address::generate(&env);
        adm_usd.mint(&summiter, &100_000_000);
        adm_usd.mint(&summiter2, &100_000_000);
        client.request_result_summiter(&summiter, &1000);
        client.request_result_summiter(&summiter2, &1000);

        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: game_id,
            bet: BetKey::Team_local,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_id,
        };
        client.bet(&user, &bet);
        let user2 = Address::generate(&env);
        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: game_id,
            bet: BetKey::Team_away,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_id,
        };
        client.bet(&user2, &betx);
        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 2500);

        let result = ResultGame {
            id: 1,
            gameid: game_id,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };

        client.summitResult(&summiter2, &result);

        // Show events for debugging

        //assert_eq!(submitted_result, result);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #212)")]
    fn test_summit_result_unauthorized() {
        let (env, client, admin, user, token_usd, token_trust, _, _, _, _) = create_test_env();
        //client.init(&admin, &token_usd, &token_trust);

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
            distribution_executed: false,
        };

        client.summitResult(&user, &result); // Should panic
    }

    #[test]
    fn test_assess_result() {
        let (env, client, admin, user, token_usd, token_trust, _, _, adm_usd, adm_trust) =
            create_test_env();
        //client.init(&admin, &token_usd, &token_trust);

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
        //add the user who wna tto participate as a summiter
        let summiter = Address::generate(&env);
        let summiter2 = Address::generate(&env);
        adm_usd.mint(&summiter, &100_000_000);
        adm_usd.mint(&summiter2, &100_000_000);
        client.request_result_summiter(&summiter, &1000);
        client.request_result_summiter(&summiter2, &1000);

        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: game_id,
            bet: BetKey::Team_local,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_id,
        };
        client.bet(&user, &bet);
        let user2 = Address::generate(&env);
        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: game_id,
            bet: BetKey::Team_away,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_id,
        };
        client.bet(&user2, &betx);
        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 2500);

        let result = ResultGame {
            id: 1,
            gameid: game_id,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };

        client.summitResult(&summiter2, &result);

        client.assessResult(&user, &bet, &game_id, &AssessmentKey::approve);
    }
    #[test]
    fn test_adm_summitter() {
        let (
            env,
            client,
            admin,
            user,
            token_usd,
            token_trust,
            token_usd_client,
            token_trust_client,
            adm_usd,
            adm_trust,
        ) = create_test_env();
        //client.init(&admin, &token_usd, &token_trust);

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
        //add the user who wna tto participate as a summiter
        let summiter = Address::generate(&env);
        let summiter2 = Address::generate(&env);
        adm_usd.mint(&summiter, &100_000_000);
        adm_usd.mint(&summiter2, &100_000_000);

        client.request_result_summiter(&summiter, &1000);
        client.request_result_summiter(&summiter2, &1000);

        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: game_id,
            bet: BetKey::Team_local,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_id,
        };
        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);
        client.bet(&user, &bet);
        let user2 = Address::generate(&env);
        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: game_id,
            bet: BetKey::Team_away,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_id,
        };
        client.bet(&user2, &betx);
        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 2500);

        let result = ResultGame {
            id: 1,
            gameid: game_id,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };

        client.summitResult(&summiter2, &result);

        client.assessResult(&user, &bet, &game_id, &AssessmentKey::approve);

        // Execute distribution
        client.execute_distribution(&game_id);
        // Claim as winner

        client.claim(&user, &ClaimType::User, &game_id);
        client.claim(&summiter2, &ClaimType::Summiter, &game_id);

        //Repaet Process

        // Set up a game
        let game_id = 31;
        let game = Game {
            id: game_id,
            startTime: 3000,
            endTime: 4000,
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

        //let's bet to active the game
        let bet = Bet {
            id: 12,
            Setting: game_id,
            bet: BetKey::Team_local,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_id,
        };
        client.bet(&user, &bet);

        let betx = Bet {
            id: 22,
            Setting: game_id,
            bet: BetKey::Team_away,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_id,
        };
        client.bet(&user2, &betx);
        set_ledger_timestamp(&env, 4500);

        let result = ResultGame {
            id: 1,
            gameid: game_id,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };

        client.summitResult(&admin, &result);

        client.assessResult(&user, &bet, &game_id, &AssessmentKey::approve);

        // Execute distribution
        client.execute_distribution(&game_id);
        // Claim as winner

        client.claim(&user, &ClaimType::User, &game_id);
        client.claim(&admin, &ClaimType::Summiter, &game_id);
    }
    #[test]
    fn test_claim_winner_honest() {
        let (
            env,
            client,
            admin,
            user,
            token_usd,
            token_trust,
            token_usd_client,
            token_trust_client,
            adm_usd,
            adm_trust,
        ) = create_test_env();
        //client.init(&admin, &token_usd, &token_trust);

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
        //add the user who wna tto participate as a summiter
        let summiter = Address::generate(&env);
        let summiter2 = Address::generate(&env);
        adm_usd.mint(&summiter, &100_000_000);
        adm_usd.mint(&summiter2, &100_000_000);

        client.request_result_summiter(&summiter, &1000);
        client.request_result_summiter(&summiter2, &1000);

        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: game_id,
            bet: BetKey::Team_local,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_id,
        };
        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);
        client.bet(&user, &bet);
        let user2 = Address::generate(&env);
        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: game_id,
            bet: BetKey::Team_away,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_id,
        };
        client.bet(&user2, &betx);
        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 2500);

        let result = ResultGame {
            id: 1,
            gameid: game_id,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };

        client.summitResult(&summiter2, &result);

        client.assessResult(&user, &bet, &game_id, &AssessmentKey::approve);

        // Execute distribution
        client.execute_distribution(&game_id);
        // Claim as winner

        client.claim(&user, &ClaimType::User, &game_id);
        client.claim(&summiter2, &ClaimType::Summiter, &game_id);

        // Verify token transfers (winner gets bet + share of pool)
        assert!(token_usd_client.balance(&user) > initial_usd_balance);
        assert_eq!(token_trust_client.balance(&user), initial_trust_balance); // Trust tokens returned
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #220)")]
    fn test_set_error_result_supreme_court() {
        let (
            env,
            client,
            admin,
            user,
            token_usd,
            token_trust,
            token_usd_client,
            token_trust_client,
            adm_usd,
            adm_trust,
        ) = create_test_env();
        //client.init(&admin, &token_usd, &token_trust);

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
        //add the user who wna tto participate as a summiter
        let summiter = Address::generate(&env);
        let summiter2 = Address::generate(&env);
        adm_usd.mint(&summiter, &100_000_000);
        adm_usd.mint(&summiter2, &100_000_000);
        client.request_result_summiter(&summiter, &1000);
        client.request_result_summiter(&summiter2, &1000);

        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: game_id,
            bet: BetKey::Team_local,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_id,
        };
        client.bet(&user, &bet);
        let user2 = Address::generate(&env);
        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: game_id,
            bet: BetKey::Team_away,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_id,
        };
        client.bet(&user2, &betx);
        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 2500);

        let result = ResultGame {
            id: 1,
            gameid: game_id,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };

        client.summitResult(&summiter2, &result);

        client.assessResult(&user, &bet, &game_id, &AssessmentKey::reject);
        client.execute_distribution(&game_id);
    }
    #[test]
    fn test_set_result_supreme_court() {
        let (
            env,
            client,
            admin,
            user,
            token_usd,
            token_trust,
            token_usd_client,
            token_trust_client,
            adm_usd,
            adm_trust,
        ) = create_test_env();
        //client.init(&admin, &token_usd, &token_trust);

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
        //add the user who wna tto participate as a summiter
        let summiter = Address::generate(&env);
        let summiter2 = Address::generate(&env);
        adm_usd.mint(&summiter, &100_000_000);
        adm_usd.mint(&summiter2, &100_000_000);
        client.request_result_summiter(&summiter, &1000);
        client.request_result_summiter(&summiter2, &1000);

        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: game_id,
            bet: BetKey::Team_local,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_id,
        };
        client.bet(&user, &bet);
        let user2 = Address::generate(&env);
        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: game_id,
            bet: BetKey::Team_away,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_id,
        };
        client.bet(&user2, &betx);
        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 2500);

        let result = ResultGame {
            id: 1,
            gameid: game_id,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };
        let result2 = ResultGame {
            id: 1,
            gameid: game_id,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };
        client.summitResult(&summiter2, &result);

        client.assessResult(&user, &bet, &game_id, &AssessmentKey::reject);
        client.setResult_supremCourt(&user, &result2);
    }
    #[test]
    fn test_set_result_supreme_court_claim() {
        let (
            env,
            client,
            admin,
            user,
            token_usd,
            token_trust,
            token_usd_client,
            token_trust_client,
            adm_usd,
            adm_trust,
        ) = create_test_env();
        //client.init(&admin, &token_usd, &token_trust);

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
        //add the user who wna tto participate as a summiter
        let summiter = Address::generate(&env);
        let summiter2 = Address::generate(&env);
        adm_usd.mint(&summiter, &100_000_000);
        adm_usd.mint(&summiter2, &100_000_000);
        client.request_result_summiter(&summiter, &1000);
        client.request_result_summiter(&summiter2, &1000);

        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: game_id,
            bet: BetKey::Team_away,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_id,
        };
        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);
        client.bet(&user, &bet);
        let user2 = Address::generate(&env);
        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: game_id,
            bet: BetKey::Team_local,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_id,
        };
        client.bet(&user2, &betx);
        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 2500);

        let result = ResultGame {
            id: 1,
            gameid: game_id,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };
        let result2 = ResultGame {
            id: 1,
            gameid: game_id,
            result: BetKey::Team_away,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };
        client.summitResult(&summiter2, &result);

        client.assessResult(&user, &bet, &game_id, &AssessmentKey::reject);
        client.setResult_supremCourt(&user, &result2);
        // Execute distribution

        client.claim(&user, &ClaimType::User, &game_id);

        // Verify token transfers (winner gets bet + share of pool)
        assert!(token_usd_client.balance(&user) > initial_usd_balance);
        assert_eq!(token_trust_client.balance(&user), initial_trust_balance); // Trust tokens returned
    }
    #[test]
    fn test_set_private() {
        let (
            env,
            client,
            admin,
            user,
            token_usd,
            token_trust,
            token_usd_client,
            token_trust_client,
            adm_usd,
            adm_trust,
        ) = create_test_env();
        //client.init(&admin, &token_usd, &token_trust);

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
        //add the user who wna tto participate as a summiter
        let summiter = Address::generate(&env);
        let summiter2 = Address::generate(&env);
        adm_usd.mint(&summiter, &100_000_000);
        adm_usd.mint(&summiter2, &100_000_000);
        client.request_result_summiter(&summiter, &1000);
        client.request_result_summiter(&summiter2, &1000);
        let user2 = Address::generate(&env);

        let privateSetting = PrivateBet {
            id: 11,
            gameid: game_id,
            active: false,
            description: String::from_str(&env, "Private Bet 1"),
            amount_bet_min: 500,
            users_invated: vec![&env, user.clone(), user2.clone()],
        };
        client.set_private_bet(&admin, &privateSetting, &game_id);
        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: 11,
            bet: BetKey::Team_away,
            amount_bet: 1000,
            betType: BetType::Private,
            gameid: game_id,
        };
        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);
        client.bet(&user, &bet);
        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: 11,
            bet: BetKey::Team_local,
            amount_bet: 1000,
            betType: BetType::Private,
            gameid: game_id,
        };
        let initial_usd_balance2 = token_usd_client.balance(&user2);
        let initial_trust_balance2 = token_trust_client.balance(&user2);
        client.bet(&user2, &betx);
        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 2500);

        let result = ResultGame {
            id: 1,
            gameid: game_id,
            result: BetKey::Team_away,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };
        let result2 = ResultGame {
            id: 1,
            gameid: game_id,
            result: BetKey::Team_away,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };
        client.summitResult(&summiter2, &result);

        client.assessResult(&user2, &betx, &game_id, &AssessmentKey::approve);
        client.assessResult(&user, &bet, &game_id, &AssessmentKey::approve);

        //client.assessResult(&user, &bet, &game_id, &AssessmentKey::approve);
        client.execute_distribution(&game_id);
        //client.setResult_supremCourt(&user, &result2);
        // Execute distribution

        client.claim(&user, &ClaimType::User, &11);

        client.claim(&user2, &ClaimType::User, &11);

        // Verify token transfers (winner gets bet + share of pool)
        assert!(token_usd_client.balance(&user) > initial_usd_balance);
        assert_eq!(token_trust_client.balance(&user), initial_trust_balance); // Trust tokens returned
        assert!(token_usd_client.balance(&user2) < initial_usd_balance2);
        assert_eq!(token_trust_client.balance(&user2), initial_trust_balance2); // Trust tokens returned
    }
    #[test]
    fn test_set_private_x() {
        let (
            env,
            client,
            admin,
            user,
            token_usd,
            token_trust,
            token_usd_client,
            token_trust_client,
            adm_usd,
            adm_trust,
        ) = create_test_env();
        //client.init(&admin, &token_usd, &token_trust);

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
        //add the user who wna tto participate as a summiter
        let summiter = Address::generate(&env);
        let summiter2 = Address::generate(&env);
        adm_usd.mint(&summiter, &100_000_000);
        adm_usd.mint(&summiter2, &100_000_000);
        client.request_result_summiter(&summiter, &1000);
        client.request_result_summiter(&summiter2, &1000);
        let user2 = Address::generate(&env);

        let privateSetting = PrivateBet {
            id: 11,
            gameid: game_id,
            active: false,
            description: String::from_str(&env, "Private Bet 1"),
            amount_bet_min: 500,
            users_invated: vec![&env, user.clone(), user2.clone()],
        };
        client.set_private_bet(&admin, &privateSetting, &game_id);
        //let's bet to active the game
        let bet = Bet {
            id: 1,
            Setting: 11,
            bet: BetKey::Team_away,
            amount_bet: 1000,
            betType: BetType::Private,
            gameid: game_id,
        };

        let initial_usd_balance = token_usd_client.balance(&user);
        let initial_trust_balance = token_trust_client.balance(&user);
        client.bet(&user, &bet);
        adm_usd.mint(&user2, &100_000_000);
        adm_trust.mint(&user2, &100_000_000);

        let betx = Bet {
            id: 2,
            Setting: 11,
            bet: BetKey::Team_local,
            amount_bet: 1000,
            betType: BetType::Private,
            gameid: game_id,
        };
        client.bet(&user2, &betx);
        // Set ledger timestamp after game end
        set_ledger_timestamp(&env, 2500);

        let result = ResultGame {
            id: 1,
            gameid: game_id,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };
        let result2 = ResultGame {
            id: 1,
            gameid: game_id,
            result: BetKey::Team_away,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };
        client.summitResult(&summiter2, &result);

        client.assessResult(&user, &bet, &game_id, &AssessmentKey::reject);
        client.setResult_supremCourt(&user, &result2);
        // Execute distribution

        client.claim(&user, &ClaimType::User, &11);

        // Verify token transfers (winner gets bet + share of pool)
        assert!(token_usd_client.balance(&user) > initial_usd_balance);
        assert_eq!(token_trust_client.balance(&user), initial_trust_balance); // Trust tokens returned
    }
}
