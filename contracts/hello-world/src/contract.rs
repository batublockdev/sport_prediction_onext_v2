#![no_std]
use core::{f32::consts::E, panic, result};

use crate::{
    bettingTrait::betting,
    errors::BettingError,
    storage,
    types::{
        AssessmentKey, Bet, BetKey, BetType, ClaimType, DataKey, Game, LastB, PrivateBet,
        PublicBet, ResultAssessment, ResultGame,
    },
};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, token, vec,
    xdr::{ScVal, ToXdr, WriteXdr},
    Address, Bytes, BytesN, Env, IntoVal, String, Symbol, Vec,
};

#[contract]
pub struct BettingContract;

#[contractimpl]
impl betting for BettingContract {
    fn init(env: Env, admin: Address, token_usd: Address, token_trust: Address) {
        admin.require_auth();

        // check if already initialized
        if storage::has_init(&env) {
            panic_with_error!(&env, BettingError::AlreadyInitializedError);
        }
        // Save data
        storage::init(env, admin, token_usd, token_trust);
    }
    fn request_result_summiter(env: Env, user: Address, stakeAmount: i128, gameId: i128) -> bool {
        user.require_auth();
        if stakeAmount <= 0 {
            panic_with_error!(&env, BettingError::NegativeAmountError);
        }
        if gameId <= 0 {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        /*We nee to set a amount to request for the summiter rol */
        // ✅ Get history map (user → score history)
        let history: i128 = storage::get_history(env.clone(), user.clone());

        // ✅ Weighted score calculation
        let new_score = (history * 70 + stakeAmount * 30) / 100;

        // ✅ Leaderboard vector
        let mut leaderboard: Vec<(Address, i128)> =
            storage::get_leaderboard(env.clone(), gameId.clone());

        // Remove old entry for this user
        let mut i = 0;
        while i < leaderboard.len() {
            let (addr, _) = leaderboard.get(i).unwrap();
            if addr == user {
                leaderboard.remove(i);
                break;
            }
            i += 1;
        }

        // Find position to insert in descending order
        let mut insert_index = leaderboard.len();
        for idx in 0..leaderboard.len() {
            let (_, score) = leaderboard.get(idx).unwrap();
            if new_score > score || new_score == score {
                insert_index = idx;
                break;
            }
        }

        // Insert at correct position
        leaderboard.insert(insert_index, (user, new_score));

        // Save leaderboard
        storage::set_leaderboard(env.clone(), gameId, leaderboard);

        true
    }

    fn bet(env: Env, user: Address, bet: Bet) {
        user.require_auth();
        let contract_address = env.current_contract_address();
        let usd = storage::get_usd(env.clone());
        let trust: Address = storage::get_trust(env.clone());
        if bet.clone().amount_bet <= 0 {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        if bet.clone().id == 0 || bet.clone().Setting == 0 {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }

        if bet.clone().betType == BetType::Private {
            let privateBet: PrivateBet = storage::get_PrivateBet(env.clone(), bet.clone().Setting);

            if privateBet.clone().id == 0 {
                panic_with_error!(&env, BettingError::SettingBetDoesNotExist);
            }
            if !privateBet.clone().users_invated.contains(&user) {
                panic_with_error!(&env, BettingError::PrivateBet_NotAllowToBet);
            }
            if bet.clone().amount_bet < privateBet.clone().amount_bet_min {
                panic_with_error!(&env, BettingError::PrivateBet_NotEnoughToBet);
            }
            let (exist, startTime, endTime, _, _, active) =
                storage::existBet(env.clone(), privateBet.clone().gameid);
            if !exist {
                panic_with_error!(&env, BettingError::GameDoesNotExist);
            }
            if startTime > env.ledger().timestamp() as u32 {
                panic_with_error!(&env, BettingError::GameHasAlreadyStarted);
            }
            if endTime < env.ledger().timestamp() as u32 {
                panic_with_error!(&env, BettingError::Game_HasFinished);
            }
            if storage::CheckUser(env.clone(), user.clone(), privateBet.clone().gameid) {
                panic_with_error!(&env, BettingError::Summiters_notAllowToBet);
            }

            Self::moveToken(
                &env,
                &usd,
                &user,
                &contract_address,
                &bet.clone().amount_bet,
            );
            Self::moveToken(
                &env,
                &trust,
                &user,
                &contract_address,
                &((bet.clone().amount_bet * 30) / 100),
            );

            storage::add_privateSettingList(env.clone(), bet.clone().gameid, bet.clone().Setting);
            storage::add_bet(env.clone(), user.clone(), bet.clone());
            if !privateBet.active {
                if storage::does_bet_active(env.clone(), bet.clone()) {
                    storage::active_private_setting(env.clone(), user.clone(), bet.clone().Setting);
                    if active == false {
                        Self::select_summiter(env.clone(), privateBet.gameid)
                    }
                }
            }
        } else if bet.clone().betType == BetType::Public {
            let publicBet: PublicBet = storage::get_PublicBet(env.clone(), bet.clone().Setting);
            if publicBet.clone().id == 0 {
                panic_with_error!(&env, BettingError::SettingBetDoesNotExist);
            }
            let (exist, startTime, endTime, _, _, active) =
                storage::existBet(env.clone(), publicBet.clone().gameid);
            if !exist {
                panic_with_error!(&env, BettingError::GameDoesNotExist);
            }
            if startTime > env.ledger().timestamp() as u32 {
                panic_with_error!(&env, BettingError::GameHasAlreadyStarted);
            }
            if endTime < env.ledger().timestamp() as u32 {
                panic_with_error!(&env, BettingError::Game_HasFinished);
            }
            if storage::CheckUser(env.clone(), user.clone(), publicBet.clone().gameid) {
                panic_with_error!(&env, BettingError::Summiters_notAllowToBet);
            }
            Self::moveToken(
                &env,
                &usd,
                &user,
                &contract_address,
                &bet.clone().amount_bet,
            );
            Self::moveToken(
                &env,
                &trust,
                &user,
                &contract_address,
                &((bet.clone().amount_bet * 30) / 100),
            );
            storage::add_bet(env.clone(), user.clone(), bet.clone());
            storage::add_listUsuers(env.clone(), publicBet.clone().gameid, user.clone());
            if !publicBet.active {
               if storage::does_bet_active(env.clone(), bet.clone())  {
                    storage::active_public_setting(env.clone(), bet.clone().Setting);
                        if active == false {
                            Self::select_summiter(env.clone(), publicBet.gameid)
                        }
                    }
                }
            }           
        }
    }
    fn claim_money_noactive(env: Env, user: Address, setting: i128) {
        user.require_auth();
        let contract_address = env.current_contract_address();
        let usd = storage::get_usd(env.clone());
        let trust: Address = storage::get_trust(env.clone());
        let betData: Bet = storage::get_bet(env.clone(), user.clone(), setting.clone());
        if betData.betType == BetType::Private {
            let privateBet: PrivateBet = get_PrivateBet(env.clone(), betData.clone().Setting);
            if privateBet.clone().id == 0 {
                panic_with_error!(&env, BettingError::SettingBetDoesNotExist);
            }
            if !privateBet.clone().active {
                let (exist, startTime, endTime, _, _, active) =
                storage::existBet(env.clone(), privateBet.clone().gameid);
            if startTime > env.ledger().timestamp() as u32 {
                Self::moveToken(&env, &usd, &contract_address, &user, &betData.amount_bet);
                Self::moveToken(
                    &env,
                    &trust,
                    &contract_address,
                    &user,
                    &((betData.clone().amount_bet * 30) / 100),
                );
            }else{
                panic_with_error!(&env, BettingError::GameHasNotStarted);
            }
            } else {
                panic_with_error!(&env, BettingError::GameIsActive);
            }
        } else if betData.betType == BetType::Public {
            let publicBet: PublicBet =get_PublicBet(env.clone(),setting.clone());
            if publicBet.clone().id == 0 {
                panic_with_error!(&env, BettingError::SettingBetDoesNotExist);
            }
            if !publicBet.clone().active {
                    let (exist, startTime, endTime, _, _, active) =
                    storage::existBet(env.clone(), privateBet.clone().gameid);
                if startTime > env.ledger().timestamp() as u32 {
                    Self::moveToken(&env, &usd, &contract_address, &user, &betData.amount_bet);
                    Self::moveToken(
                        &env,
                        &trust,
                        &contract_address,
                        &user,
                        &((betData.clone().amount_bet * 30) / 100),
                    );
                }else{
                    panic_with_error!(&env, BettingError::GameHasNotStarted);
                }
            } else {
                    panic_with_error!(&env, BettingError::GameIsActive);
            }
        } else {
            panic!("Invalid bet type");
        }
    }
    //admin address
    pub fn set_game(env: Env, game: Game, signature: BytesN<64>, pub_key: BytesN<32>) {
        let (exist, startTime, endTime, summiter, checkers, _) =
            Self::existBet(env.clone(), game.clone().id);
        if exist {
            panic_with_error!(&env, BettingError::GameHasAlreadySet);
        }
        let encoded = game.clone().to_xdr(&env);
        // Now wrap into Soroban Bytes
        env.crypto().ed25519_verify(&pub_key, &encoded, &signature);
        storage::set_game(env.clone(), game.clone());
        let pubSetting = PublicBet {
            id: game.id,
            gameid: game.id,
            active: false,
            description: String::from_slice(&env, "Public Bet"),
        };
        storage::set_publicSetting(env.clone(), pubSetting);
    }
    pub fn set_private_bet(env: Env, user: Address, privateData: PrivateBet, game: Game) {
        user.require_auth();

        let (exist, startTime, endTime, summiter, checkers, _) =
            Self::existBet(env.clone(), game.clone().id);
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }
        if (privateData.id == 0 || privateData.gameid != game.clone().id) {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        storage::set_privateSetting(env.clone(), privateData.clone());
    }

    pub fn summitResult(env: Env, user: Address, result: ResultGame) -> ResultGame {
        user.require_auth();
        let (exist, startTime, endTime, summiter, checkers, active) =
            Self::existBet(env.clone(), result.clone().gameid);
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }

        if endTime > env.ledger().timestamp() as u32 {
            panic_with_error!(&env, BettingError::GameHasNotFinished);
        }
        if result.clone().gameid == 0 || result.clone().id == 0 {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        if endTime + (1 * 60 * 60) < env.ledger().timestamp() as u32 {
            let receivedResult: ResultGame = storage::get_result(env.clone(), result.clone().gameid);
            if receivedResult.id == 0 {
                let newSummiter = Address::from_string(&String::from_str(
                    &env,
                    "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
                ));
                storage::update_game(
                    env.clone(),
                    result.clone().gameid,
                    newSummiter,
                    checkers.clone(),
                );
                storage::set_history(env.clone(), summiter.clone(), -100);
                //We need to set the stake amount
                let stake: i128=20;
                storage::add_fine(env.clone(), summiter.clone(),stake );
                if endTime + (2 * 60 * 60) < env.ledger().timestamp() as u32 {
                    Self::money_back(env.clone(), result.clone().gameid);
                } else {
                    if !checkers.contains(&user) {
                        panic_with_error!(&env, BettingError::NotAllowToSummitResult);
                    }
                    storage::set_ResultGame(env.clone(), result.clone());
                }
            }
        } else {
            if summiter != user {
                panic_with_error!(&env, BettingError::NotAllowToSummitResult);
            }

            storage::set_ResultGame(env.clone(), result.clone());

        }

        result
    }
    
    pub fn assessResult(env: Env, user: Address, bet: Bet, game_id: i128, desition: AssessmentKey) {
        user.require_auth();
        let (exist, startTime, endTime, summiter, checkers, _) =
            Self::existBet(env.clone(), game_id.clone());
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }
        if endTime > env.ledger().timestamp() as u32 {
            panic_with_error!(&env, BettingError::GameHasNotFinished);
        }
        if endTime + (5 * 60 * 60) < env.ledger().timestamp() as u32 {
            panic_with_error!(&env, BettingError::GameAssesmentHasFinished);
        }

        let mut results: ResultGame = env
            .storage()
            .persistent()
            .get(&DataKey::Result(game_id))
            .unwrap_or_else(|| panic!("No result found for this game"));
        let mut resultAssessment: ResultAssessment = env
            .storage()
            .persistent()
            .get(&DataKey::ResultAssessment(game_id))
            .unwrap_or(ResultAssessment {
                id: 0,
                gameid: game_id,
                CheckApprove: Vec::new(&env),
                CheckReject: Vec::new(&env),
                UsersApprove: Vec::new(&env),
                UsersReject: Vec::new(&env),
            });
        if !checkers.contains(&user) {
            let betResult: Bet = env
                .storage()
                .persistent()
                .get(&DataKey::Bet(user.clone(), bet.clone().Setting))
                .unwrap_or_else(|| panic!("You are not allowed to assess this result"));
            if resultAssessment.UsersApprove.contains(&user)
                || resultAssessment.UsersReject.contains(&user)
            {
                panic!("You have already assessed this result");
            }
            if resultAssessment.id == 0 {
                if desition == AssessmentKey::approve {
                    resultAssessment.UsersApprove.push_front(user.clone());
                } else if desition == AssessmentKey::reject {
                    resultAssessment.UsersReject.push_front(user.clone());
                    results.pause = true;
                }
                resultAssessment.id = game_id;
                env.storage().persistent().set(
                    &DataKey::ResultAssessment(resultAssessment.clone().gameid),
                    &resultAssessment,
                );
            } else {
                if desition == AssessmentKey::approve {
                    resultAssessment.UsersApprove.push_front(user.clone());
                } else if desition == AssessmentKey::reject {
                    resultAssessment.UsersReject.push_front(user.clone());
                    results.pause = true;
                }

                env.storage().persistent().update(
                    &DataKey::ResultAssessment(resultAssessment.clone().gameid),
                    |old: Option<ResultAssessment>| {
                        let mut res = old.unwrap_or(ResultAssessment {
                            id: resultAssessment.id,
                            gameid: resultAssessment.gameid,
                            CheckApprove: Vec::new(&env),
                            CheckReject: Vec::new(&env),
                            UsersApprove: Vec::new(&env),
                            UsersReject: Vec::new(&env),
                        });
                        res.UsersApprove = resultAssessment.UsersApprove.clone();
                        res.UsersReject = resultAssessment.UsersReject.clone();
                        res
                    },
                );
            }
            env.storage().persistent().update(
                &DataKey::Result(game_id),
                |old: Option<ResultGame>| {
                    let mut res = old.unwrap_or(ResultGame {
                        id: results.id,
                        gameid: results.gameid,
                        description: String::from_str(&env, ""),
                        result: BetKey::Team_local,
                        pause: false,
                    });
                    res.pause = results.pause;
                    res
                },
            );
        } else {
            if resultAssessment.CheckApprove.contains(&user)
                || resultAssessment.CheckReject.contains(&user)
            {
                panic!("You have already assessed this result");
            }
            if resultAssessment.id == 0 {
                if desition == AssessmentKey::approve {
                    resultAssessment.CheckApprove.push_front(user.clone());
                } else if desition == AssessmentKey::reject {
                    resultAssessment.CheckReject.push_front(user.clone());
                }
                resultAssessment.id = game_id;
                env.storage().persistent().set(
                    &DataKey::ResultAssessment(resultAssessment.clone().gameid),
                    &resultAssessment,
                );
            } else {
                if desition == AssessmentKey::approve {
                    resultAssessment.CheckApprove.push_front(user.clone());
                } else if desition == AssessmentKey::reject {
                    resultAssessment.CheckReject.push_front(user.clone());
                }

                env.storage().persistent().update(
                    &DataKey::ResultAssessment(resultAssessment.clone().gameid),
                    |old: Option<ResultAssessment>| {
                        let mut res = old.unwrap_or(ResultAssessment {
                            id: resultAssessment.id,
                            gameid: resultAssessment.gameid,
                            CheckApprove: Vec::new(&env),
                            CheckReject: Vec::new(&env),
                            UsersApprove: Vec::new(&env),
                            UsersReject: Vec::new(&env),
                        });
                        res.CheckApprove = resultAssessment.CheckApprove.clone();
                        res.CheckReject = resultAssessment.CheckReject.clone();
                        res
                    },
                );
            }
        }

        env.storage().persistent().set(
            &DataKey::ResultAssessment(resultAssessment.clone().gameid),
            &resultAssessment,
        );
    }

    pub fn claim(env: Env, user: Address, typeClaim: ClaimType) {
        user.require_auth();
        let contract_address = env.current_contract_address();
        let adminAdr: Address = env
            .storage()
            .instance()
            .get(&ADMIN_KEY)
            .unwrap_or_else(|| panic!("contract not initialized"));
        let usd = env
            .storage()
            .instance()
            .get(&TOKEN_USD_KEY)
            .unwrap_or_else(|| panic!("contract not initialized"));
        match typeClaim {
            ClaimType::Summiter => {
                let money: i128 = env
                    .storage()
                    .persistent()
                    .get(&DataKey::ClaimSummiter(user.clone()))
                    .unwrap_or(0);
                Self::moveToken(&env, &usd, &contract_address, &user, &money);
                env.storage()
                    .persistent()
                    .set(&DataKey::ClaimSummiter(user.clone()), &0);
            }
            ClaimType::Protocol => {
                adminAdr.require_auth();
                let money: i128 = env
                    .storage()
                    .persistent()
                    .get(&DataKey::ClaimProtocol)
                    .unwrap_or(0);
                Self::moveToken(&env, &usd, &contract_address, &adminAdr, &money);
                env.storage().persistent().set(&DataKey::ClaimProtocol, &0);
            }
            ClaimType::User => {
                let money: i128 = env
                    .storage()
                    .persistent()
                    .get(&DataKey::ClaimWinner(user.clone()))
                    .unwrap_or(0);
                Self::moveToken(&env, &usd, &contract_address, &user, &money);
                env.storage()
                    .persistent()
                    .set(&DataKey::ClaimWinner(user.clone()), &0);
            }
            _ => {
                // default case
                panic!("Invalid claim type");
            }
        }
    }
    //set result by Supreme Court
    pub fn setResult_supremCourt(env: Env, user: Address, result: ResultGame) {
        user.require_auth();
        // 0 correct
        // 1 incorrect
        let mut complain = 0;
        let xresult: ResultGame = env
            .storage()
            .persistent()
            .get(&DataKey::Result(result.clone().gameid))
            .unwrap_or_else(|| panic!("No result found for this game"));
        if xresult.pause == false {
            panic!("Game result is not paused, you cannot set result now");
        }
        if xresult.id != result.id {
            panic!("You cannot set result for this game");
        }
        if xresult.gameid != result.gameid {
            panic!("You cannot set result for this game");
        }
        if xresult.result != result.result {
            complain = 0; // The complain made by the users was correct
        } else {
            complain = 1; // The complain made by the users was incorrect
        }
        let listedPrivateBet: Vec<(i128)> = env
            .storage()
            .persistent()
            .get(&DataKey::PrivateBetList(result.clone().gameid))
            .unwrap_or(Vec::new(&env));
        Self::distibution(env.clone(), complain, result.clone().gameid, result.clone());
        for setting in listedPrivateBet.iter() {
            let privateBet: PrivateBet = env
                .storage()
                .persistent()
                .get(&DataKey::SetPrivateBet(setting.clone()))
                .unwrap_or(PrivateBet {
                    id: 0,
                    gameid: 0,
                    active: false,
                    description: String::from_slice(&env, "No private bet found"),
                    amount_bet_min: 0,
                    users_invated: Vec::new(&env),
                });
            if privateBet.active == false {
                continue;
            }
            Self::distibution_private(
                env.clone(),
                complain,
                result.clone().gameid,
                result.clone(),
                setting.clone(),
            );
        }
        env.storage()
            .persistent()
            .set(&DataKey::Result(result.clone().gameid), &result);
    }
    /*Fines:
    1. users who don't participate will get the bet amount back
    2. users who act dishonestly will lose their bet amount
    3. summiters with wrong result will lose their stake
     */
     fn execute_distribution(env: Env, gameId: i128) {
        let complain = 2; // 2 means no complain was made
        let xresult: ResultGame = env
            .storage()
            .persistent()
            .get(&DataKey::Result(gameId.clone()))
            .unwrap_or_else(|| panic!("No result found for this game"));
        if xresult.pause == true {
            panic!("Game result was paused ");
        }
        let listedPrivateBet: Vec<(i128)> = env
            .storage()
            .persistent()
            .get(&DataKey::PrivateBetList(gameId.clone()))
            .unwrap_or(Vec::new(&env));
        Self::distibution(env.clone(), complain, gameId.clone(), xresult.clone());
        for setting in listedPrivateBet.iter() {
            let privateBet: PrivateBet = env
                .storage()
                .persistent()
                .get(&DataKey::SetPrivateBet(setting.clone()))
                .unwrap_or(PrivateBet {
                    id: 0,
                    gameid: 0,
                    active: false,
                    description: String::from_slice(&env, "No private bet found"),
                    amount_bet_min: 0,
                    users_invated: Vec::new(&env),
                });
            if privateBet.active == false {
                continue;
            }
            Self::distibution_private(
                env.clone(),
                complain,
                xresult.clone().gameid,
                xresult.clone(),
                setting.clone(),
            );
        }
    }
} 

impl BettingContract {
    fn money_back(env: Env, gameid: i128) {
        let (exist, startTime, endTime, summiter, checkers, active) =
            Self::existBet(env.clone(), gameid.clone());
        if !exist {
            panic!("Game haven't been set yet");
        }

        if endTime > env.ledger().timestamp() as u32 {
            panic!("Game hasn't ended yet");
        }
        if endTime + (2 * 60 * 60) < env.ledger().timestamp() as u32 {
            let receivedResult: ResultGame = env
                .storage()
                .persistent()
                .get(&DataKey::Result(gameid.clone()))
                .unwrap_or(ResultGame {
                    id: 0,
                    gameid: 0,
                    description: String::from_str(&env, ""),
                    result: BetKey::Team_local,
                    pause: false,
                });
            if receivedResult.id != 0 {
                panic!("Result has been set, you cannot withdraw now");
            }
        } else {
            panic!("You are not allowed to withdraw now");
        }
        let listedBetAddress: Vec<(Address)> = env
            .storage()
            .persistent()
            .get(&DataKey::ListBetUser(gameid.clone()))
            .unwrap_or(Vec::new(&env));
        for user in listedBetAddress.iter() {
            let betData: Bet = env
                .storage()
                .persistent()
                .get(&DataKey::Bet(user.clone(), gameid))
                .unwrap_or_else(|| panic!("No bet found for this user"));
            let money: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::ClaimWinner(user.clone()))
                .unwrap_or(0);
            let total_money = money + betData.amount_bet;
            env.storage()
                .persistent()
                .set(&DataKey::ClaimWinner(user.clone()), &total_money);
        }
        let listedPrivateBet: Vec<(i128)> = env
            .storage()
            .persistent()
            .get(&DataKey::PrivateBetList(gameid.clone()))
            .unwrap_or(Vec::new(&env));
        for setting in listedPrivateBet.iter() {
            let privateBet: PrivateBet = env
                .storage()
                .persistent()
                .get(&DataKey::SetPrivateBet(setting.clone()))
                .unwrap_or(PrivateBet {
                    id: 0,
                    gameid: 0,
                    active: false,
                    description: String::from_slice(&env, "No private bet found"),
                    amount_bet_min: 0,
                    users_invated: Vec::new(&env),
                });
            if privateBet.active == false {
                continue;
            } else {
                for usersx in privateBet.users_invated.iter() {
                    let betDatax: Bet = env
                        .storage()
                        .persistent()
                        .get(&DataKey::Bet(usersx.clone(), setting.clone()))
                        .unwrap_or(Bet {
                            id: 0,
                            gameid: 0,
                            betType: BetType::Private,
                            Setting: 0,
                            bet: BetKey::Team_local,
                            amount_bet: 0,
                        });
                    if betDatax.id == 0 {
                        continue;
                    }
                    let moneyx: i128 = env
                        .storage()
                        .persistent()
                        .get(&DataKey::ClaimWinner(usersx.clone()))
                        .unwrap_or(0);
                    let total_moneyx = moneyx + betDatax.amount_bet;
                    env.storage()
                        .persistent()
                        .set(&DataKey::ClaimWinner(usersx.clone()), &total_moneyx);
                }
            }
        }
    }
    fn select_summiter(env: Env, game_id: i128) {
        let (exist, startTime, endTime, summiter, checkers, _) =
            storage::existBet(env.clone(), game_id.clone());
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }
        let leaderboard: Vec<(Address, i128)> =
            storage::get_leaderboard(env.clone(), game_id.clone());

        // Limit to top 5
        let mut top = Vec::new(&env);
        for i in 0..10 {
            if let Some((addr, score)) = leaderboard.get(i) {
                if score == 0 {
                    break;
                }
                top.push_back((addr.clone(), score));
            }
        }

        let sequence = env.ledger().sequence();
        let timestamp = env.ledger().timestamp() as u32;
        let mut rng = (sequence + timestamp) % (top.len() as u32);
        let mut selected = Vec::new(&env);
        let mut selected_Summitters: Vec<Address> = Vec::new(&env);
        for k in 0..5 {
            let pick = rng % top.len();
            let (addr, score) = top.get(pick).unwrap();
            let mut checks: Vec<Address> = Vec::new(&env);
            selected.push_back((addr.clone(), score));
            if k == 0 {
                selected_Summitters.push_back(addr.clone());
            }
            if k > 0 {
                selected_Summitters.push_back(addr.clone());
            }

            top.remove(pick); // remove picked
            if top.len() == 0 {
                break;
            }
        }
        storage::update_game(
            env.clone(),
            game_id,
            selected_Summitters.get(0).unwrap().clone(),
            selected_Summitters,
        );
    }
    fn moveToken(env: &Env, token: &Address, from: &Address, to: &Address, amount: &i128) {
        let token = token::Client::new(env, token);
        token.transfer(from, &to, amount);
    }
    fn distibution_private(
        env: Env,
        complain: i128,
        game_id: i128,
        result: ResultGame,
        setting: i128,
    ) {
        let mut winners: Vec<Address> = Vec::new(&env);
        let mut losers: Vec<Address> = Vec::new(&env);
        let mut s_honest: Vec<Address> = Vec::new(&env);
        let mut s_dishonest: Vec<Address> = Vec::new(&env);
        let mut honest: Vec<Address> = Vec::new(&env);
        let mut dishonest: Vec<Address> = Vec::new(&env);

        let mut amount_gained: i128 = 0;
        let mut amount_winner_pool: i128 = 0;
        let game: Game = env
            .storage()
            .persistent()
            .get(&DataKey::Game(game_id.clone()))
            .unwrap_or_else(|| panic!("No game found for this id"));
        let privateBet: PrivateBet = env
            .storage()
            .persistent()
            .get(&DataKey::SetPrivateBet(setting.clone()))
            .unwrap_or(PrivateBet {
                id: 0,
                gameid: 0,
                active: false,
                description: String::from_slice(&env, "No private bet found"),
                amount_bet_min: 0,
                users_invated: Vec::new(&env),
            });
        let assesments: ResultAssessment = env
            .storage()
            .persistent()
            .get(&DataKey::ResultAssessment(game_id.clone()))
            .unwrap_or(ResultAssessment {
                id: 0,
                gameid: game_id.clone(),
                CheckApprove: Vec::new(&env),
                CheckReject: Vec::new(&env),
                UsersApprove: Vec::new(&env),
                UsersReject: Vec::new(&env),
            });
        let dummyUser = Address::from_string(&String::from_str(
            &env,
            "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
        ));
        if complain == 0 {
            s_honest = assesments.CheckReject.clone();
            s_dishonest = assesments.CheckApprove.clone();
            if game.summiter != dummyUser {
                s_dishonest.push_back(game.summiter.clone());
            }
        }
        if complain == 1 {
            // if the complain was incorrect
            s_honest = assesments.CheckApprove.clone();
            if game.summiter != dummyUser {
                s_honest.push_back(game.summiter.clone());
            }
            s_dishonest = assesments.CheckReject.clone();
        }
        if complain == 2 {
            // if the complain was not made
            s_dishonest = assesments.CheckReject.clone();
            s_honest = assesments.CheckApprove.clone();
            if game.summiter != dummyUser {
                s_honest.push_back(game.summiter.clone());
            }
        }
        for user in privateBet.users_invated.iter() {
            let bet: Bet = env
                .storage()
                .persistent()
                .get(&DataKey::Bet(user.clone(), setting))
                .unwrap_or(Bet {
                    id: 0,
                    gameid: 0,
                    betType: BetType::Public,
                    Setting: 0,
                    bet: BetKey::Team_local,
                    amount_bet: 0,
                });
            if bet.id == 0 {
                continue;
            }
            if !assesments.UsersApprove.contains(&user) && !assesments.UsersReject.contains(&user) {
                if bet.bet == result.result {
                    let amount_bet_withFine = (bet.amount_bet * 50) / 100;
                    amount_winner_pool += amount_bet_withFine;
                    let x_amount = env
                        .storage()
                        .persistent()
                        .get(&DataKey::ClaimWinner(user.clone()))
                        .unwrap_or(0);
                    let total = x_amount + amount_bet_withFine;
                    env.storage()
                        .persistent()
                        .set(&DataKey::ClaimWinner(user.clone()), &total);
                    //users lose some trust
                    continue;
                } else {
                    amount_gained += bet.amount_bet;
                    //user loses the trust coins too
                    continue;
                }
            }
            if complain == 0 {
                if assesments.UsersApprove.contains(&user) {
                    dishonest.push_back(user.clone());
                } else {
                    honest.push_back(user.clone());
                }
            }
            if complain == 1 {
                if assesments.UsersReject.contains(&user) {
                    dishonest.push_back(user.clone());
                } else {
                    honest.push_back(user.clone());
                }
            }
            if complain == 2 {
                if assesments.UsersApprove.contains(&user) {
                    honest.push_back(user.clone());
                } else {
                    dishonest.push_back(user.clone());
                }
            }
        }

        for user in dishonest.iter() {
            let bet: Bet = env
                .storage()
                .persistent()
                .get(&DataKey::Bet(user.clone(), setting))
                .unwrap_or(Bet {
                    id: 0,
                    gameid: 0,
                    betType: BetType::Public,
                    Setting: 0,
                    bet: BetKey::Team_local,
                    amount_bet: 0,
                });

            if bet.id == 0 {
                continue;
            } else {
                amount_gained += bet.amount_bet;
                // user lose trust
            }
        }

        for user in honest.iter() {
            let bet: Bet = env
                .storage()
                .persistent()
                .get(&DataKey::Bet(user.clone(), setting))
                .unwrap_or(Bet {
                    id: 0,
                    gameid: 0,
                    betType: BetType::Public,
                    Setting: 0,
                    bet: BetKey::Team_local,
                    amount_bet: 0,
                });

            if bet.id == 0 {
                continue;
            } else {
                if bet.bet == result.result {
                    winners.push_back(user.clone());
                    amount_winner_pool += bet.amount_bet;
                } else {
                    losers.push_back(user.clone());
                    amount_gained += bet.amount_bet;
                }
            }
        }
        let mut summiter_retribution = (amount_gained * 20) / 100;
        let protocol_retribution = (amount_gained * 10) / 100;
        amount_gained -= summiter_retribution;
        amount_gained -= protocol_retribution;
        let add = 20 * s_dishonest.len() as i128;
        summiter_retribution += add;
        if winners.len() == 0 {
            for loser in losers.iter() {
                let bet: Bet = env
                    .storage()
                    .persistent()
                    .get(&DataKey::Bet(loser.clone(), setting))
                    .unwrap_or(Bet {
                        id: 0,
                        gameid: 0,
                        betType: BetType::Public,
                        Setting: 0,
                        bet: BetKey::Team_local,
                        amount_bet: 0,
                    });
                if bet.id == 0 {
                    continue;
                } else {
                    //send user amount to user
                    let user_amount = (30 * bet.clone().amount_bet) / 100;
                    let amount_pay = bet.clone().amount_bet - user_amount;
                    let x_amount = env
                        .storage()
                        .persistent()
                        .get(&DataKey::ClaimWinner(loser.clone()))
                        .unwrap_or(0);
                    let total = x_amount + amount_pay;
                    env.storage()
                        .persistent()
                        .set(&DataKey::ClaimWinner(loser.clone()), &total);
                }
            }
        } else {
            for user in winners.iter() {
                let bet: Bet = env
                    .storage()
                    .persistent()
                    .get(&DataKey::Bet(user.clone(), setting))
                    .unwrap_or(Bet {
                        id: 0,
                        gameid: 0,
                        betType: BetType::Public,
                        Setting: 0,
                        bet: BetKey::Team_local,
                        amount_bet: 0,
                    });

                if bet.id == 0 {
                    continue;
                } else {
                    let user_share = (bet.amount_bet * 100) / amount_winner_pool;
                    let user_amount = (user_share * amount_gained) / 100;
                    //send user_amount to user
                    let x_amount = env
                        .storage()
                        .persistent()
                        .get(&DataKey::ClaimWinner(user.clone()))
                        .unwrap_or(0);
                    let total = x_amount + user_amount + bet.amount_bet;
                    env.storage()
                        .persistent()
                        .set(&DataKey::ClaimWinner(user.clone()), &total);
                }
            }
        }

        let mut each_summiter_share = 0;
        if s_honest.len() > 0 {
            each_summiter_share = summiter_retribution / s_honest.len() as i128;
        }
        let total_summiter = each_summiter_share + 20;
        for s_user in s_honest.iter() {
            env.storage()
                .persistent()
                .set(&DataKey::History_Summiter(s_user.clone()), &100);
            let add = 20 * s_dishonest.len() as i128;
            env.storage()
                .persistent()
                .set(&DataKey::ClaimSummiter(s_user.clone()), &total_summiter);
        }
    }
    fn distibution(env: Env, complain: i128, game_id: i128, result: ResultGame) {
        let mut winners: Vec<Address> = Vec::new(&env);
        let mut losers: Vec<Address> = Vec::new(&env);
        let mut s_honest: Vec<Address> = Vec::new(&env);
        let mut s_dishonest: Vec<Address> = Vec::new(&env);
        let mut summiter_retribution: i128 = 0;
        let mut protocol_retribution: i128 = 0;
        let mut honest: Vec<Address> = Vec::new(&env);
        let mut dishonest: Vec<Address> = Vec::new(&env);

        let mut amount_gained: i128 = 0;
        let mut amount_loser_honest: i128 = 0;

        let mut amount_winner_pool: i128 = 0;
        let game: Game = env
            .storage()
            .persistent()
            .get(&DataKey::Game(game_id.clone()))
            .unwrap_or_else(|| panic!("No game found for this id"));
        let assesments: ResultAssessment = env
            .storage()
            .persistent()
            .get(&DataKey::ResultAssessment(game_id.clone()))
            .unwrap_or(ResultAssessment {
                id: 0,
                gameid: game_id.clone(),
                CheckApprove: Vec::new(&env),
                CheckReject: Vec::new(&env),
                UsersApprove: Vec::new(&env),
                UsersReject: Vec::new(&env),
            });
        let listedBetAddress: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::ListBetUser(game_id.clone()))
            .unwrap_or(Vec::new(&env));
        if listedBetAddress.len() == 0 {
            panic!("No bets found for this game");
        } else {
            // if only one user bet, he is the winner
            for user in listedBetAddress.iter() {
                if !assesments.UsersApprove.contains(&user)
                    && !assesments.UsersReject.contains(&user)
                {
                    let bet: Bet = env
                        .storage()
                        .persistent()
                        .get(&DataKey::Bet(user.clone(), game_id.clone()))
                        .unwrap_or(Bet {
                            id: 0,
                            gameid: 0,
                            betType: BetType::Public,
                            Setting: 0,
                            bet: BetKey::Team_local,
                            amount_bet: 0,
                        });

                    if bet.bet == result.result {
                        let amount_bet_withFine = (bet.amount_bet * 50) / 100;
                        let x_amount = env
                            .storage()
                            .persistent()
                            .get(&DataKey::ClaimWinner(user.clone()))
                            .unwrap_or(0);
                        let total = x_amount + amount_bet_withFine;
                        env.storage()
                            .persistent()
                            .set(&DataKey::ClaimWinner(user.clone()), &total);
                        amount_gained += amount_bet_withFine;
                        //users lose some trust
                        continue;
                    } else {
                        amount_gained += bet.amount_bet;
                        //user loses the trust coins too
                        continue;
                    }
                }
            }
        }
        let dummyUser = Address::from_string(&String::from_str(
            &env,
            "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
        ));
        if complain == 0 {
            dishonest = assesments.UsersApprove.clone();
            s_honest = assesments.CheckReject.clone();
            s_dishonest = assesments.CheckApprove.clone();
            if game.summiter != dummyUser {
                s_dishonest.push_back(game.summiter.clone());
            }
            honest = assesments.UsersReject.clone();
        }
        if complain == 1 {
            // if the complain was incorrect
            dishonest = assesments.UsersReject.clone();
            s_honest = assesments.CheckApprove.clone();
            if game.summiter != dummyUser {
                s_honest.push_back(game.summiter.clone());
            }
            s_dishonest = assesments.CheckReject.clone();
            honest = assesments.UsersApprove.clone();
        }
        if complain == 2 {
            // if the complain was not made
            honest = assesments.UsersApprove.clone();
            dishonest = assesments.UsersReject.clone();
            s_dishonest = assesments.CheckReject.clone();
            s_honest = assesments.CheckApprove.clone();
            if game.summiter != dummyUser {
                s_honest.push_back(game.summiter.clone());
            }
        }

        for user in dishonest.iter() {
            let bet: Bet = env
                .storage()
                .persistent()
                .get(&DataKey::Bet(user.clone(), game_id.clone()))
                .unwrap_or(Bet {
                    id: 0,
                    gameid: 0,
                    betType: BetType::Public,
                    Setting: 0,
                    bet: BetKey::Team_local,
                    amount_bet: 0,
                });

            if bet.id == 0 {
                continue;
            } else {
                amount_gained += bet.amount_bet;
            }
        }

        for user in honest.iter() {
            let bet: Bet = env
                .storage()
                .persistent()
                .get(&DataKey::Bet(user.clone(), game_id.clone()))
                .unwrap_or(Bet {
                    id: 0,
                    gameid: 0,
                    betType: BetType::Public,
                    Setting: 0,
                    bet: BetKey::Team_local,
                    amount_bet: 0,
                });

            if bet.id == 0 {
                continue;
            } else {
                if bet.bet == result.result {
                    winners.push_back(user.clone());
                    amount_winner_pool += bet.amount_bet;
                } else {
                    losers.push_back(user.clone());
                    amount_gained += bet.amount_bet;
                    amount_loser_honest += bet.amount_bet;
                }
            }
        }
        let fine: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Fine(game_id.clone()))
            .unwrap_or(0);
        amount_gained += fine;
        summiter_retribution = (amount_gained * 20) / 100;
        protocol_retribution = (amount_gained * 10) / 100;
        amount_gained -= summiter_retribution;
        amount_gained -= protocol_retribution;
        let add = 20 * s_dishonest.len() as i128;
        summiter_retribution += add;
        if winners.len() == 0 {
            if losers.len() == 0 {
                summiter_retribution = (amount_gained * 50) / 100;
                protocol_retribution = (amount_gained * 50) / 100;
                amount_gained -= summiter_retribution;
                amount_gained -= protocol_retribution;
            }
            for loser in losers.iter() {
                let bet: Bet = env
                    .storage()
                    .persistent()
                    .get(&DataKey::Bet(loser.clone(), game_id.clone()))
                    .unwrap_or(Bet {
                        id: 0,
                        gameid: 0,
                        betType: BetType::Public,
                        Setting: 0,
                        bet: BetKey::Team_local,
                        amount_bet: 0,
                    });
                if bet.id == 0 {
                    continue;
                } else {
                    //send user amount to user
                    let user_share = (bet.amount_bet * 100) / amount_loser_honest;

                    let user_amount = (user_share * amount_gained) / 100;

                    let x_amount = env
                        .storage()
                        .persistent()
                        .get(&DataKey::ClaimWinner(loser.clone()))
                        .unwrap_or(0);
                    let total = x_amount + user_amount;
                    env.storage()
                        .persistent()
                        .set(&DataKey::ClaimWinner(loser.clone()), &total);
                }
            }
        } else {
            for user in winners.iter() {
                let bet: Bet = env
                    .storage()
                    .persistent()
                    .get(&DataKey::Bet(user.clone(), game_id.clone()))
                    .unwrap_or(Bet {
                        id: 0,
                        gameid: 0,
                        betType: BetType::Public,
                        Setting: 0,
                        bet: BetKey::Team_local,
                        amount_bet: 0,
                    });

                if bet.id == 0 {
                    continue;
                } else {
                    let user_share = (bet.amount_bet * 100) / amount_winner_pool;

                    let user_amount = (user_share * amount_gained) / 100;
                    //send user_amount to user
                    let x_amount = env
                        .storage()
                        .persistent()
                        .get(&DataKey::ClaimWinner(user.clone()))
                        .unwrap_or(0);
                    let total = x_amount + user_amount + bet.amount_bet;
                    env.storage()
                        .persistent()
                        .set(&DataKey::ClaimWinner(user.clone()), &total);
                }
            }
        }
        for s_user in s_dishonest.iter() {
            let history: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::History_Summiter(s_user.clone()))
                .unwrap_or(0);
            let total_history = history - 100;
            env.storage()
                .persistent()
                .set(&DataKey::History_Summiter(s_user), &total_history);
        }
        let mut each_summiter_share = 0;
        if s_honest.len() > 0 {
            each_summiter_share = summiter_retribution / s_honest.len() as i128;
        }
        let total_summiter = each_summiter_share + 20;
        for s_user in s_honest.iter() {
            let history: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::History_Summiter(s_user.clone()))
                .unwrap_or(0);
            let total_history = history + 100;
            env.storage()
                .persistent()
                .set(&DataKey::History_Summiter(s_user.clone()), &total_history);
            let add = 20 * s_dishonest.len() as i128;
            let x_amount: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::ClaimSummiter(s_user.clone()))
                .unwrap_or(0);
            let total = x_amount + total_summiter;
            env.storage()
                .persistent()
                .set(&DataKey::ClaimSummiter(s_user.clone()), &total);
        }

        let x_amountx: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::ClaimProtocol)
            .unwrap_or(0);
        let total = x_amountx + protocol_retribution;
        env.storage()
            .persistent()
            .set(&DataKey::ClaimProtocol, &total);
    }
}
