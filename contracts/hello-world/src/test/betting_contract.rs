#[cfg(test)]
mod tests {

    use std::env;
    use std::string::ToString;

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
        Address, Bytes, BytesN, BytesN as _, InvokeError, String, Symbol, Symbol as _, TryIntoVal,
        Val,
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
    fn events_handler(
        env: Env,
        all_events: std::vec::Vec<
            soroban_sdk::Vec<(
                soroban_sdk::Address,
                soroban_sdk::Vec<soroban_sdk::Val>,
                soroban_sdk::Val,
            )>,
        >,
    ) {
        for event in all_events.iter() {
            for e in event.iter() {
                let (contract_id, topics, value) = e;

                for topic in topics.iter() {
                    let sym: Result<soroban_sdk::Symbol, _> = topic.try_into_val(&env);
                    match sym {
                        Ok(symbol) => {
                            if symbol != soroban_sdk::Symbol::new(&env, "BettingGame")
                                && symbol != soroban_sdk::Symbol::new(&env, "transfer")
                            {
                                std::println!("Topic: {:?}", symbol);

                                // Convert symbol to string for easier matching
                                let symbol_str: std::string::String = symbol.to_string();

                                if symbol_str == "Game_Set" {
                                    let game_id: i128 = value.try_into_val(&env).unwrap();
                                    std::println!("Event GameSet - Game ID: {}", game_id,);
                                } else if symbol_str == "Private_Setting" {
                                    let raw: SorobanVec<Val> = value.try_into_val(&env).unwrap();
                                    let game_id: i128 =
                                        raw.get(1).unwrap().try_into_val(&env).unwrap();
                                    let setting_id: i128 =
                                        raw.get(3).unwrap().try_into_val(&env).unwrap();
                                    let user: Address =
                                      raw.get(0).unwrap().try_into_val(&env).unwrap();
                                    let amount_bet: i128 =
                                     raw.get(2).unwrap().try_into_val(&env).unwrap();
                                    std::println!(
                                "Event PrivateSetting- Game ID: {}, Admin User: {:?}, Setting: {}, Amount Bet: {}",
                                game_id, user,setting_id ,amount_bet 
                            );
                                } else if symbol_str == "Game_Result" {
                                    let raw: SorobanVec<Val> = value.try_into_val(&env).unwrap();
                                    let game_id: i128 =
                                        raw.get(0).unwrap().try_into_val(&env).unwrap();
                                    let result: BetKey =
                                        raw.get(1).unwrap().try_into_val(&env).unwrap();
                                    std::println!(
                                        "Event summit_result - Game ID: {}, Result: {:?}",
                                        game_id,
                                        result
                                    );
                                } else if symbol_str == "Seleted_Suimmiters" {
                                    let raw: SorobanVec<Val> = value.try_into_val(&env).unwrap();

                                    let game_id: i128 =
                                        raw.get(0).unwrap().try_into_val(&env).unwrap();
                                    let main: Address =
                                        raw.get(1).unwrap().try_into_val(&env).unwrap();
                                    let summiters: SorobanVec<Address> =
                                        raw.get(2).unwrap().try_into_val(&env).unwrap();

                                    std::println!("Game ID: {}", game_id);
                                    std::println!("Main: {:?}", main);
                                    for s in summiters.iter() {
                                        std::println!("Summiter: {:?}", s);
                                    }
                                }
                                else if symbol_str == "Game_Result_Reject" {
                                    let game_id: i128 = value.try_into_val(&env).unwrap();
                                    std::println!("Event GameSet Reject - Game ID: {}", game_id,);
                                }
                                else if symbol_str == "Game_ResultbySupremeCourt" {
                                    let raw: SorobanVec<Val> = value.try_into_val(&env).unwrap();
                                    let game_id: i128 =
                                        raw.get(0).unwrap().try_into_val(&env).unwrap();
                                    let result: BetKey =
                                        raw.get(1).unwrap().try_into_val(&env).unwrap();
                                    std::println!(
                                        "Event summit_result - Game ID: {}, Result: {:?}",
                                        game_id,
                                        result
                                    );
                                }
                                else if symbol_str == "UserHonestyPoints" {
                                    let raw: SorobanVec<Val> = value.try_into_val(&env).unwrap();
                                    let user: Address =
                                        raw.get(1).unwrap().try_into_val(&env).unwrap();
                                    let points: i128 =
                                        raw.get(0).unwrap().try_into_val(&env).unwrap();
                                    std::println!(
                                        "Event Users Points - User: {:?}, Poins: {:?}",
                                        user,
                                        points
                                    );
                                }
                            }
                        }
                        Err(_) => std::println!(" "),
                    }
                }
            }
        }
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
        let supreme = Address::generate(&env);

        // Register mock token contracts
        let (token_usd, token_usd_admin) = create_token_contract(&env, &admin);
        let (token_trust, token_trust_admin) = create_token_contract(&env, &admin);

        // Mint initial tokens to user for testing
        token_usd_admin.mint(&user, &100_000_000);
        token_trust_admin.mint(&user, &100_000_000);
        // Register the betting contract
        let contract_id = env.register(
            BettingContract,
            (&admin, &token_usd.address, &token_trust.address, &supreme),
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

        client.claim_refund(&user, &game_id);

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

        client.assessResult(&user, &game_id, &game_id, &AssessmentKey::approve);
    }
    #[test]
    fn test_cancel_sameSummiter() {
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
            result: BetKey::Cancel,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };

        client.summitResult(&summiter2, &result);

        client.assessResult(&user, &game_id, &game_id, &AssessmentKey::approve);

        // Execute distribution
        client.execute_distribution(&game_id);
        // Claim as winner

        //client.claim(&user, &ClaimType::User, &game_id);
        //client.claim(&summiter2, &ClaimType::Summiter, &game_id);

        //Repaet Process

        // Set up a game
        let game_idx = 31;
        let gamex = Game {
            id: game_idx,
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
        let encodedx: Vec<u8> = gamex.clone().to_xdr(&env).iter().collect();

        let signer1 = Keypair::generate(&mut thread_rng());
        let public_keyx = BytesN::<32>::from_array(&env, &signer1.public.to_bytes());

        let signaturex2: BytesN<64> =
            BytesN::from_array(&env, &signer1.sign(encodedx.as_slice()).to_bytes());
        client.set_game(&gamex, &signaturex2, &public_keyx);

        //let's bet to active the game
        let betxz = Bet {
            id: 122,
            Setting: game_idx,
            bet: BetKey::Team_local,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_idx,
        };
        client.bet(&user, &betxz);

        let betx2 = Bet {
            id: 222,
            Setting: game_idx,
            bet: BetKey::Team_away,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_idx,
        };
        client.bet(&user2, &betx2);
        set_ledger_timestamp(&env, 4002);

        let result = ResultGame {
            id: 1,
            gameid: game_idx,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };
        std::println!("Admin address: {:?}", admin);
        std::println!("User address: {:?}", user);
        std::println!("User2 address: {:?}", user2);
        std::println!("Summiter2 address: {:?}", summiter2);
        std::println!("Summiter address : {:?}", summiter);

        client.summitResult(&summiter, &result);

        client.assessResult(&user, &game_idx, &game_idx, &AssessmentKey::approve);

        // Execute distribution
        client.execute_distribution(&game_idx);
        // Claim as winner

        client.claim(&user, &ClaimType::User, &game_idx);
        client.claim(&admin, &ClaimType::Summiter, &game_idx);
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

        client.assessResult(&user, &game_id, &game_id, &AssessmentKey::approve);

        // Execute distribution
        client.execute_distribution(&game_id);
        // Claim as winner

        client.claim(&user, &ClaimType::User, &game_id);
        client.claim(&summiter2, &ClaimType::Summiter, &game_id);

        //Repaet Process

        // Set up a game
        let game_idx = 31;
        let gamex = Game {
            id: game_idx,
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
        let encodedx: Vec<u8> = gamex.clone().to_xdr(&env).iter().collect();

        let signer1 = Keypair::generate(&mut thread_rng());
        let public_keyx = BytesN::<32>::from_array(&env, &signer1.public.to_bytes());

        let signaturex2: BytesN<64> =
            BytesN::from_array(&env, &signer1.sign(encodedx.as_slice()).to_bytes());
        client.set_game(&gamex, &signaturex2, &public_keyx);

        //let's bet to active the game
        let betxz = Bet {
            id: 122,
            Setting: game_idx,
            bet: BetKey::Team_local,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_idx,
        };
        client.bet(&user, &betxz);

        let betx2 = Bet {
            id: 222,
            Setting: game_idx,
            bet: BetKey::Team_away,
            amount_bet: 1000,
            betType: BetType::Public,
            gameid: game_idx,
        };
        client.bet(&user2, &betx2);
        set_ledger_timestamp(&env, 4002);

        let result = ResultGame {
            id: 1,
            gameid: game_idx,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };
        std::println!("Admin address: {:?}", admin);
        std::println!("User address: {:?}", user);
        std::println!("User2 address: {:?}", user2);
        std::println!("Summiter2 address: {:?}", summiter2);
        std::println!("Summiter address : {:?}", summiter);

        client.summitResult(&admin, &result);

        client.assessResult(&user, &game_idx, &game_idx, &AssessmentKey::approve);

        // Execute distribution
        client.execute_distribution(&game_idx);
        // Claim as winner

        client.claim(&user, &ClaimType::User, &game_idx);
        client.claim(&admin, &ClaimType::Summiter, &game_idx);
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

        client.assessResult(&user, &game_id, &game_id, &AssessmentKey::approve);

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
    fn test_refund_nosummition() {
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
        // Set ledger timestamp after game end plus 3 hours
        set_ledger_timestamp(&env, 12810);

        client.claim_refund(&user, &game_id);
        client.claim_refund(&user2, &game_id);

        std::println!("User balance final {:?}", token_usd_client.balance(&user));
        std::println!("User2 balance final {:?}", token_usd_client.balance(&user2));
        std::println!(
            "summiter balance final {:?}",
            token_usd_client.balance(&summiter)
        );
        std::println!(
            "summiter2 balance final {:?}",
            token_usd_client.balance(&summiter2)
        );
        std::println!("admin balance final {:?}", token_usd_client.balance(&admin));
        // Verify token transfers (winner gets bet + share of pool)
        assert!(token_usd_client.balance(&user) > initial_usd_balance); // Trust tokens returned
        assert_eq!(token_trust_client.balance(&user), initial_trust_balance); // Trust tokens returned
    }
    #[test]
    fn test_cancel() {
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
            result: BetKey::Cancel,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };

        client.summitResult(&summiter2, &result);

        client.assessResult(&user, &game_id, &game_id, &AssessmentKey::approve);

        // Execute distribution
        client.execute_distribution(&game_id);
        // Claim as winner

        client.claim_refund(&user, &game_id);
        client.claim_refund(&user2, &game_id);

        std::println!("User balance final {:?}", token_usd_client.balance(&user));
        std::println!("User2 balance final {:?}", token_usd_client.balance(&user2));
        std::println!(
            "summiter balance final {:?}",
            token_usd_client.balance(&summiter)
        );
        std::println!(
            "summiter2 balance final {:?}",
            token_usd_client.balance(&summiter2)
        );
        std::println!("admin balance final {:?}", token_usd_client.balance(&admin));
        // Verify token transfers (winner gets bet + share of pool)
        assert_eq!(token_usd_client.balance(&user), initial_usd_balance); // Trust tokens returned
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
        struct SummitersSeletedEvent {
            game_id: i128,
            main: Address,
            summiters: Vec<Address>,
        }

        let result = ResultGame {
            id: 1,
            gameid: game_id,
            result: BetKey::Team_local,
            pause: false,
            description: String::from_str(&env, "Final Score 2-1"),
            distribution_executed: false,
        };
        let events = env.events().all();
        let (_, _, value) = events.get(0).unwrap();
        let raw: SorobanVec<Val> = value.try_into_val(&env).unwrap();

        let game_id: i128 = raw.get(0).unwrap().try_into_val(&env).unwrap();
        let main: Address = raw.get(1).unwrap().try_into_val(&env).unwrap();
        let summiters: SorobanVec<Address> = raw.get(2).unwrap().try_into_val(&env).unwrap();

        std::println!("Game ID: {}", game_id);
        std::println!("Main: {:?}", main);
        for s in summiters.iter() {
            std::println!("Summiter: {:?}", s);
        }

        client.summitResult(&summiter2, &result);

        client.assessResult(&user, &game_id, &game_id, &AssessmentKey::reject);
        client.execute_distribution(&game_id);
        std::println!("User balance final {:?}", token_usd_client.balance(&user));
        std::println!("User2 balance final {:?}", token_usd_client.balance(&user2));
        std::println!(
            "summiter balance final {:?}",
            token_usd_client.balance(&summiter)
        );
        std::println!(
            "summiter2 balance final {:?}",
            token_usd_client.balance(&summiter2)
        );
        std::println!("admin balance final {:?}", token_usd_client.balance(&admin));
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

        client.assessResult(&user2, &game_id, &game_id, &AssessmentKey::approve);
        client.assessResult(&user, &game_id, &game_id, &AssessmentKey::reject);

        client.setResult_supremCourt(&result2);

        client.claim(&user2, &ClaimType::User, &game_id);
        client.claim(&summiter2, &ClaimType::Summiter, &game_id);
        client.claim(&admin, &ClaimType::Protocol, &game_id);
        std::println!("User balance final {:?}", token_usd_client.balance(&user));
        std::println!("User2 balance final {:?}", token_usd_client.balance(&user2));
        std::println!(
            "summiter balance final {:?}",
            token_usd_client.balance(&summiter)
        );
        std::println!(
            "summiter2 balance final {:?}",
            token_usd_client.balance(&summiter2)
        );
        std::println!("admin balance final {:?}", token_usd_client.balance(&admin));
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

        client.assessResult(&user, &game_id, &game_id, &AssessmentKey::reject);
        client.assessResult(&user2, &game_id, &game_id, &AssessmentKey::reject);
        client.assessResult(&summiter, &0, &game_id, &AssessmentKey::approve);
        client.setResult_supremCourt(&result2);
        // Execute distribution

        client.claim(&user, &ClaimType::User, &game_id);
        client.claim(&summiter, &ClaimType::Summiter, &game_id);
        client.claim(&user2, &ClaimType::User, &game_id);
        client.claim(&admin, &ClaimType::Protocol, &game_id);
        std::println!("User balance final {:?}", token_usd_client.balance(&user));
        std::println!("User2 balance final {:?}", token_usd_client.balance(&user2));
        std::println!(
            "summiter balance final {:?}",
            token_usd_client.balance(&summiter)
        );
        std::println!(
            "summiter2 balance final {:?}",
            token_usd_client.balance(&summiter2)
        );
        std::println!("admin balance final {:?}", token_usd_client.balance(&admin));
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
            settingAdmin: user2.clone(),
            description: String::from_str(&env, "Private Bet 1"),
            amount_bet_min: 500,
            users_invated: vec![&env, user.clone(), user2.clone()],
        };
        client.set_private_bet(&user2, &privateSetting, &game_id);
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

        client.summitResult(&summiter2, &result);

        client.assessResult(&user2, &11, &game_id, &AssessmentKey::approve);
        client.assessResult(&user, &11, &game_id, &AssessmentKey::approve);
        client.assessResult(&summiter, &0, &game_id, &AssessmentKey::approve);

        //client.assessResult(&user, &bet, &game_id, &AssessmentKey::approve);
        client.execute_distribution(&game_id);
        //client.setResult_supremCourt(&user, &result2);
        // Execute distribution

        client.claim(&user, &ClaimType::User, &11);
        client.claim(&user2, &ClaimType::User, &11);
        client.claim(&summiter, &ClaimType::Summiter, &game_id);
        client.claim(&summiter2, &ClaimType::Summiter, &game_id);
        client.claim(&admin, &ClaimType::Protocol, &game_id);

        // Verify token transfers (winner gets bet + share of pool)
        std::println!("User balance final {:?}", token_usd_client.balance(&user));
        std::println!("User2 balance final {:?}", token_usd_client.balance(&user2));
        std::println!(
            "summiter balance final {:?}",
            token_usd_client.balance(&summiter)
        );
        std::println!(
            "summiter2 balance final {:?}",
            token_usd_client.balance(&summiter2)
        );
        std::println!("admin balance final {:?}", token_usd_client.balance(&admin));

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

        let mut all_events = Vec::new();

        // Set up a game
        let game_id = 51;
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
        all_events.push(env.events().all());

        //add the user who wna tto participate as a summiter
        let summiter = Address::generate(&env);
        let summiter2 = Address::generate(&env);
        std::println!("summiter address: {:?}", summiter);
        std::println!("summiter2 address: {:?}", summiter2);
        adm_usd.mint(&summiter, &1000);
        adm_usd.mint(&summiter2, &1000);
        std::println!(
            "summiter balance initial {:?}",
            token_usd_client.balance(&summiter)
        );
        std::println!(
            "summiter2 balance initial {:?}",
            token_usd_client.balance(&summiter2)
        );
        client.request_result_summiter(&summiter, &1000);
        client.request_result_summiter(&summiter2, &1000);
        let user2 = Address::generate(&env);

        let privateSetting = PrivateBet {
            id: 11,
            gameid: game_id,
            active: false,
            settingAdmin: user2.clone(),
            description: String::from_str(&env, "Private Bet 1"),
            amount_bet_min: 500,
            users_invated: vec![&env, user.clone(), user2.clone()],
        };
        client.set_private_bet(&user2, &privateSetting, &game_id);
        all_events.push(env.events().all());

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
        std::println!("User1 honest balance initial {:?}", initial_usd_balance);
        client.bet(&user, &bet);
        all_events.push(env.events().all());

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
        let initial_usd_balance = token_usd_client.balance(&user2);
        std::println!("User2 novote balance initial {:?}", initial_usd_balance);

        client.bet(&user2, &betx);
        all_events.push(env.events().all());

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
        all_events.push(env.events().all());

        client.assessResult(&user, &11, &game_id, &AssessmentKey::reject);
        all_events.push(env.events().all());
        client.assessResult(&summiter, &0, &game_id, &AssessmentKey::reject);

        client.setResult_supremCourt(&result2);
        all_events.push(env.events().all());

        // Execute distribution

        client.claim(&user, &ClaimType::User, &11);
        all_events.push(env.events().all());

        //client.claim(&user2, &ClaimType::User, &11);
        client.claim(&summiter, &ClaimType::Summiter, &game_id);
        client.claim(&summiter2, &ClaimType::Summiter, &game_id);
        client.claim(&admin, &ClaimType::Protocol, &game_id);
        all_events.push(env.events().all());

        // Verify token transfers (winner gets bet + share of pool)
        events_handler(env.clone(), all_events);
        std::println!("User balance final {:?}", token_usd_client.balance(&user));
        std::println!("User2 balance final {:?}", token_usd_client.balance(&user2));
        std::println!(
            "summiter balance initial {:?}",
            token_usd_client.balance(&summiter)
        );
        std::println!(
            "summiter2 balance initial {:?}",
            token_usd_client.balance(&summiter2)
        );
        std::println!(
            "admin balance initial {:?}",
            token_usd_client.balance(&admin)
        );
        assert!(token_usd_client.balance(&user) > initial_usd_balance);
        assert_eq!(token_trust_client.balance(&user), initial_trust_balance); // Trust tokens returned
    }
}
