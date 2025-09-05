#![no_std]
use core::{f32::consts::E, panic, result};

use crate::{
    bettingTrait::betting,
    errors::BettingError,
    events::BettingEvents,
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
            if startTime < env.ledger().timestamp() as u32 {
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
            storage::add_not_assesed_yet(
                env.clone(),
                bet.clone().Setting,
                bet.clone().amount_bet,
                bet.clone().bet,
            );
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
            if startTime < env.ledger().timestamp() as u32 {
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
            storage::add_not_assesed_yet(
                env.clone(),
                bet.clone().Setting,
                bet.clone().amount_bet,
                bet.clone().bet,
            );
            storage::add_listUsuers(env.clone(), publicBet.clone().gameid, user.clone());
            if !publicBet.active {
                if storage::does_bet_active(env.clone(), bet.clone()) {
                    storage::active_public_setting(env.clone(), bet.clone().Setting);
                    if active == false {
                        Self::select_summiter(env.clone(), publicBet.gameid)
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
            let privateBet: PrivateBet =
                storage::get_PrivateBet(env.clone(), betData.clone().Setting);
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
                } else {
                    panic_with_error!(&env, BettingError::GameHasNotStarted);
                }
            } else {
                panic_with_error!(&env, BettingError::GameIsActive);
            }
        } else if betData.betType == BetType::Public {
            let publicBet: PublicBet = storage::get_PublicBet(env.clone(), setting.clone());
            if publicBet.clone().id == 0 {
                panic_with_error!(&env, BettingError::SettingBetDoesNotExist);
            }
            if !publicBet.clone().active {
                let (exist, startTime, endTime, _, _, active) =
                    storage::existBet(env.clone(), publicBet.clone().gameid);
                if startTime < env.ledger().timestamp() as u32 {
                    Self::moveToken(&env, &usd, &contract_address, &user, &betData.amount_bet);
                    Self::moveToken(
                        &env,
                        &trust,
                        &contract_address,
                        &user,
                        &((betData.clone().amount_bet * 30) / 100),
                    );
                } else {
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
    fn set_game(env: Env, game: Game, signature: BytesN<64>, pub_key: BytesN<32>) {
        let (exist, startTime, endTime, summiter, checkers, _) =
            storage::existBet(env.clone(), game.clone().id);
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
    fn set_private_bet(env: Env, user: Address, privateData: PrivateBet, game: Game) {
        user.require_auth();

        let (exist, startTime, endTime, summiter, checkers, _) =
            storage::existBet(env.clone(), game.clone().id);
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }
        if (privateData.id == 0 || privateData.gameid != game.clone().id) {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        storage::set_privateSetting(env.clone(), privateData.clone());
    }

    fn summitResult(env: Env, user: Address, result: ResultGame) -> ResultGame {
        user.require_auth();
        let (exist, startTime, endTime, summiter, checkers, active) =
            storage::existBet(env.clone(), result.clone().gameid);
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
            let receivedResult: ResultGame =
                storage::get_ResultGame(env.clone(), result.clone().gameid);
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
                let stake: i128 = 20;
                storage::add_Fine(env.clone(), result.clone().gameid, stake);
                if endTime + (2 * 60 * 60) < env.ledger().timestamp() as u32 {
                    // 2 hours with no summition then the supreme court is in charge
                    storage::puase_ResultGame(env.clone(), result.clone().gameid, true);
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

    fn assessResult(env: Env, user: Address, bet: Bet, game_id: i128, desition: AssessmentKey) {
        user.require_auth();
        let (exist, startTime, endTime, summiter, checkers, _) =
            storage::existBet(env.clone(), game_id.clone());
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }
        if endTime > env.ledger().timestamp() as u32 {
            panic_with_error!(&env, BettingError::GameHasNotFinished);
        }
        if endTime + (5 * 60 * 60) < env.ledger().timestamp() as u32 {
            panic_with_error!(&env, BettingError::GameAssesmentHasFinished);
        }

        let mut results: ResultGame = storage::get_ResultGame(env.clone(), game_id.clone());
        if results.id == 0 {
            panic_with_error!(&env, BettingError::GameResultNotFound);
        }
        let mut resultAssessment: ResultAssessment =
            storage::get_ResultAssessment(env.clone(), game_id.clone());
        if !checkers.contains(&user) {
            let betResult: Bet = storage::get_Bet(env.clone(), user.clone(), bet.clone().Setting);
            if betResult.id == 0 {
                panic_with_error!(&env, BettingError::UserCannotVote);
            }
            if resultAssessment.UsersApprove.contains(&user)
                || resultAssessment.UsersReject.contains(&user)
            {
                panic_with_error!(&env, BettingError::UserCannotVote);
            }
            if resultAssessment.id == 0 {
                if desition == AssessmentKey::approve {
                    resultAssessment.UsersApprove.push_front(user.clone());
                } else if desition == AssessmentKey::reject {
                    resultAssessment.UsersReject.push_front(user.clone());
                    results.pause = true;
                }
                resultAssessment.id = game_id;
                storage::set_ResultAssessment(
                    env.clone(),
                    game_id.clone(),
                    resultAssessment.clone(),
                );
            } else {
                if desition == AssessmentKey::approve {
                    resultAssessment.UsersApprove.push_front(user.clone());
                } else if desition == AssessmentKey::reject {
                    resultAssessment.UsersReject.push_front(user.clone());
                    results.pause = true;
                }
                storage::set_ResultAssessment(
                    env.clone(),
                    game_id.clone(),
                    resultAssessment.clone(),
                );
            }
            storage::delete_not_assesed_yet(
                env.clone(),
                betResult.clone().Setting,
                betResult.clone().amount_bet,
                betResult.clone().bet,
            );
            if results.pause {
                storage::puase_ResultGame(env.clone(), game_id.clone(), results.clone().pause);
                storage::add_reject_total(
                    env.clone(),
                    betResult.clone().Setting,
                    betResult.clone().amount_bet,
                    betResult.clone().bet,
                );
            } else {
                storage::add_approve_total(
                    env.clone(),
                    betResult.clone().Setting,
                    betResult.clone().amount_bet,
                    betResult.clone().bet,
                );
            }
        } else {
            if resultAssessment.CheckApprove.contains(&user)
                || resultAssessment.CheckReject.contains(&user)
            {
                panic_with_error!(&env, BettingError::UserCannotVote);
            }
            if resultAssessment.id == 0 {
                if desition == AssessmentKey::approve {
                    resultAssessment.CheckApprove.push_front(user.clone());
                } else if desition == AssessmentKey::reject {
                    resultAssessment.CheckReject.push_front(user.clone());
                }
                resultAssessment.id = game_id;
                storage::set_ResultAssessment(
                    env.clone(),
                    game_id.clone(),
                    resultAssessment.clone(),
                );
            } else {
                if desition == AssessmentKey::approve {
                    resultAssessment.CheckApprove.push_front(user.clone());
                } else if desition == AssessmentKey::reject {
                    resultAssessment.CheckReject.push_front(user.clone());
                }
                storage::set_ResultAssessment(
                    env.clone(),
                    game_id.clone(),
                    resultAssessment.clone(),
                );
            }
        }
    }

    fn claim(env: Env, user: Address, typeClaim: ClaimType, setting: i128) {
        user.require_auth();
        let contract_address = env.current_contract_address();
        let adminAdr: Address = storage::get_admin(env.clone());
        let usd = storage::get_usd(env.clone());
        let trust: Address = storage::get_trust(env.clone());
        match typeClaim {
            ClaimType::Summiter => {
                let money: i128 = storage::get_ClaimSummiter(env.clone(), user.clone());
                Self::moveToken(&env, &usd, &contract_address, &user, &money);
                storage::zero_ClaimSummiter(env.clone(), user.clone());
            }
            ClaimType::Protocol => {
                adminAdr.require_auth();
                let money: i128 = storage::get_ClaimProtocol(env.clone());
                Self::moveToken(&env, &usd, &contract_address, &adminAdr, &money);
                storage::zero_ClaimProtocol(env.clone());
            }
            ClaimType::User => {
                let doneBefore =
                    storage::get_didUserWithdraw(env.clone(), user.clone(), setting.clone());
                if doneBefore {
                    panic_with_error!(&env, BettingError::UserAlreadyClaimed);
                }
                let (kindofUser, amountBet) =
                    Self::what_kind_user(env.clone(), user.clone(), setting.clone());
                let winner_pool = storage::get_winnerPool(env.clone(), setting.clone());
                let loser_pool = storage::get_loserPool(env.clone(), setting.clone());
                let amount_share = storage::get_pool_total(env.clone(), setting.clone());
                match kindofUser {
                    6 => {
                        // NO BET
                        panic_with_error!(&env, BettingError::NoBetHasBeenFound);
                    }
                    5 => {
                        // loser who didn't assess the result
                        panic_with_error!(&env, BettingError::NothingToClaim);
                    }
                    4 => {
                        // winner who didn't assess the result
                        // user gets 50% of his bet back and no trust back
                        let bet_50 = (amountBet * 50) / 100;
                        Self::moveToken(&env, &usd, &contract_address, &user, &bet_50);
                        storage::set_didUserWithdraw(env.clone(), user.clone(), setting.clone());
                    }
                    3 => {
                        // dishonest user
                        // no money no trust back
                        panic_with_error!(&env, BettingError::NothingToClaim);
                    }
                    2 => {
                        // loser honest
                        // user gets back trust tokens
                        if winner_pool == 0 {
                            let user_share = (amountBet * 100) / loser_pool;
                            let user_amount = (user_share * amount_share) / 100;
                            Self::moveToken(&env, &usd, &contract_address, &user, &user_amount);
                        } else {
                            let trust_amount = (amountBet * 30) / 100;
                            Self::moveToken(&env, &trust, &contract_address, &user, &trust_amount);
                        }
                        storage::set_didUserWithdraw(env.clone(), user.clone(), setting.clone());
                    }
                    1 => {
                        // winner honest
                        let user_share = (amountBet * 100) / winner_pool;
                        let user_amount = (user_share * amount_share) / 100;
                        let total = amountBet + user_amount;
                        Self::moveToken(&env, &usd, &contract_address, &user, &total);
                        let trust_amount = (amountBet * 30) / 100;
                        Self::moveToken(&env, &trust, &contract_address, &user, &trust_amount);
                        storage::set_didUserWithdraw(env.clone(), user.clone(), setting.clone());
                    }
                    _ => {
                        panic_with_error!(&env, BettingError::InvalidInputError);
                    }
                }
            }
            _ => {
                // default case
                panic!("Invalid claim type");
            }
        }
    }
    //set result by Supreme Court
    fn setResult_supremCourt(env: Env, user: Address, result: ResultGame) {
        user.require_auth();
        // 0 correct
        // 1 incorrect
        let mut complain = 0;
        let xresult: ResultGame = storage::get_ResultGame(env.clone(), result.clone().gameid);
        if xresult.pause == false {
            panic_with_error!(&env, BettingError::GameHasNotBeenPaused);
        }
        if xresult.id != result.id {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        if xresult.result != result.result {
            complain = 0; // The complain made by the users was correct
        } else {
            complain = 1; // The complain made by the users was incorrect
        }
        let listedPrivateBet: Vec<(i128)> =
            storage::get_privateSettingList(env.clone(), result.clone().gameid);
        Self::make_distribution(
            env.clone(),
            result.clone().gameid,
            result.clone().result,
            complain,
        );
        for setting in listedPrivateBet.iter() {
            let privateBet: PrivateBet = storage::get_PrivateBet(env.clone(), setting.clone());
            if privateBet.active == false {
                continue;
            }
            Self::make_distribution(
                env.clone(),
                setting.clone(),
                result.clone().result,
                complain,
            );
        }
        storage::set_ResultGame(env.clone(), result.clone());
    }
    /*Fines:
    1. users who don't participate will get the bet amount back
    2. users who act dishonestly will lose their bet amount
    3. summiters with wrong result will lose their stake
     */
    fn execute_distribution(env: Env, gameId: i128) {
        let complain = 2; // 2 means no complain was made

        let result: ResultGame = storage::get_ResultGame(env.clone(), gameId.clone());
        if result.pause == true {
            panic_with_error!(&env, BettingError::GameHasBeenPaused);
        }
        let listedPrivateBet: Vec<(i128)> =
            storage::get_privateSettingList(env.clone(), result.clone().gameid);
        Self::make_distribution(
            env.clone(),
            result.clone().gameid,
            result.clone().result,
            complain,
        );
        for setting in listedPrivateBet.iter() {
            let privateBet: PrivateBet = storage::get_PrivateBet(env.clone(), setting.clone());
            if privateBet.active == false {
                continue;
            }
            Self::make_distribution(
                env.clone(),
                setting.clone(),
                result.clone().result,
                complain,
            );
        }
    }
}

impl BettingContract {
    fn make_distribution(env: Env, game_id: i128, resultBet: BetKey, complain: i128) {
        let mut amount_gain_pool: i128 = 0;
        let mut losers_honest_pool: i128 = 0;
        let mut winner_pool: i128 = 0;
        let mut novote_winner: i128 = 0;
        let mut s_dishonest: Vec<Address> = Vec::new(&env);
        let mut s_honest: Vec<Address> = Vec::new(&env);
        let mut s_noVote: Vec<Address> = Vec::new(&env);
        let mut resultAssessment: ResultAssessment =
            storage::get_ResultAssessment(env.clone(), game_id.clone());

        let (exist, startTime, endTime, summiter, checkers, _) =
            storage::existBet(env.clone(), game_id.clone());
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }
        if endTime > env.ledger().timestamp() as u32 {
            panic_with_error!(&env, BettingError::GameHasNotFinished);
        }
        /// we start we those who did not assess the result
        /// 1. users who don't participate and win will get the bet 50 % amount back of their bets
        /// 2. users who don't participate and lose will lose their bet amount
        for i in 0..=2 {
            let mut bet_key: BetKey = BetKey::Team_local;
            match i {
                0 => {
                    bet_key = BetKey::Team_local;
                }
                1 => {
                    bet_key = BetKey::Draw;
                }
                2 => {
                    bet_key = BetKey::Team_away;
                }
                _ => {}
            }
            if resultBet != bet_key {
                amount_gain_pool +=
                    storage::get_not_assesed_yet(env.clone(), game_id.clone(), bet_key);
            } else {
                novote_winner = storage::get_not_assesed_yet(env.clone(), game_id.clone(), bet_key);
            }
        }
        let winner_minus_50 = (novote_winner * 50) / 100;
        amount_gain_pool += winner_minus_50;
        match complain {
            0 => {
                for checker in checkers.iter() {
                    if resultAssessment.CheckApprove.contains(&checker) {
                        s_dishonest.push_back(checker.clone());
                        storage::set_history(env.clone(), checker.clone(), -100);
                    } else if resultAssessment.CheckReject.contains(&checker) {
                        s_honest.push_back(checker.clone());
                    } else {
                        s_noVote.push_back(checker.clone());
                        storage::set_history(env.clone(), checker.clone(), -100);
                    }
                }
                amount_gain_pool +=
                    storage::get_approve_total(env.clone(), game_id.clone(), BetKey::Team_local);
                amount_gain_pool +=
                    storage::get_approve_total(env.clone(), game_id.clone(), BetKey::Draw);
                amount_gain_pool +=
                    storage::get_approve_total(env.clone(), game_id.clone(), BetKey::Team_away);
                for i in 0..=2 {
                    let mut bet_key: BetKey = BetKey::Team_local;
                    match i {
                        0 => {
                            bet_key = BetKey::Team_local;
                        }
                        1 => {
                            bet_key = BetKey::Draw;
                        }
                        2 => {
                            bet_key = BetKey::Team_away;
                        }
                        _ => {}
                    }
                    if resultBet != bet_key {
                        amount_gain_pool += storage::get_reject_total(
                            env.clone(),
                            game_id.clone(),
                            bet_key.clone(),
                        );
                        losers_honest_pool += storage::get_reject_total(
                            env.clone(),
                            game_id.clone(),
                            bet_key.clone(),
                        );
                    } else {
                        winner_pool +=
                            storage::get_reject_total(env.clone(), game_id.clone(), bet_key);
                    }
                }
            }
            1 => {
                for checker in checkers.iter() {
                    if resultAssessment.CheckApprove.contains(&checker) {
                        s_honest.push_back(checker.clone());
                    } else if resultAssessment.CheckReject.contains(&checker) {
                        s_dishonest.push_back(checker.clone());
                        storage::set_history(env.clone(), checker.clone(), -100);
                    } else {
                        s_noVote.push_back(checker.clone());
                        storage::set_history(env.clone(), checker.clone(), -100);
                    }
                }
                amount_gain_pool +=
                    storage::get_reject_total(env.clone(), game_id.clone(), BetKey::Draw);
                amount_gain_pool +=
                    storage::get_reject_total(env.clone(), game_id.clone(), BetKey::Team_away);
                amount_gain_pool +=
                    storage::get_reject_total(env.clone(), game_id.clone(), BetKey::Team_local);
                for i in 0..=2 {
                    let mut bet_key: BetKey = BetKey::Team_local;
                    match i {
                        0 => {
                            bet_key = BetKey::Team_local;
                        }
                        1 => {
                            bet_key = BetKey::Draw;
                        }
                        2 => {
                            bet_key = BetKey::Team_away;
                        }
                        _ => {}
                    }
                    if resultBet != bet_key {
                        amount_gain_pool += storage::get_approve_total(
                            env.clone(),
                            game_id.clone(),
                            bet_key.clone(),
                        );
                        losers_honest_pool += storage::get_approve_total(
                            env.clone(),
                            game_id.clone(),
                            bet_key.clone(),
                        );
                    } else {
                        winner_pool +=
                            storage::get_approve_total(env.clone(), game_id.clone(), bet_key);
                    }
                }
            }
            2 => {
                for checker in checkers.iter() {
                    if resultAssessment.CheckApprove.contains(&checker) {
                        s_honest.push_back(checker.clone());
                    } else if resultAssessment.CheckReject.contains(&checker) {
                        s_dishonest.push_back(checker.clone());
                        storage::set_history(env.clone(), checker.clone(), -100);
                    } else {
                        s_noVote.push_back(checker.clone());
                        storage::set_history(env.clone(), checker.clone(), -100);
                    }
                }
                for i in 0..=2 {
                    let mut bet_key: BetKey = BetKey::Team_local;
                    match i {
                        0 => {
                            bet_key = BetKey::Team_local;
                        }
                        1 => {
                            bet_key = BetKey::Draw;
                        }
                        2 => {
                            bet_key = BetKey::Team_away;
                        }
                        _ => {}
                    }
                    if resultBet != bet_key {
                        amount_gain_pool += storage::get_approve_total(
                            env.clone(),
                            game_id.clone(),
                            bet_key.clone(),
                        );
                        losers_honest_pool += storage::get_approve_total(
                            env.clone(),
                            game_id.clone(),
                            bet_key.clone(),
                        );
                    } else {
                        winner_pool +=
                            storage::get_approve_total(env.clone(), game_id.clone(), bet_key);
                    }
                }
            }
            _ => {
                panic_with_error!(&env, BettingError::InvalidInputError);
            }
        }

        let mut summiter_retribution = (amount_gain_pool * 20) / 100;
        let mut protocol_retribution = (amount_gain_pool * 10) / 100;
        amount_gain_pool -= summiter_retribution;
        amount_gain_pool -= protocol_retribution;
        let mut add = 20 * s_dishonest.len() as i128;
        add += 20 * s_noVote.len() as i128;
        summiter_retribution += add;
        if winner_pool == 0 {
            if losers_honest_pool == 0 {
                summiter_retribution = (amount_gain_pool * 50) / 100;
                protocol_retribution = (amount_gain_pool * 50) / 100;
                amount_gain_pool -= summiter_retribution;
                amount_gain_pool -= protocol_retribution;
            }
        }
        storage::add_ClaimProtocol(env.clone(), protocol_retribution);
        storage::save_complain(env.clone(), game_id.clone(), complain);
        storage::save_winnerPool(env.clone(), game_id.clone(), winner_pool);
        storage::save_loserPool(env.clone(), game_id.clone(), losers_honest_pool);
        storage::set_pool_total(env.clone(), game_id.clone(), amount_gain_pool);
        storage::set_pool_summiter_total(env.clone(), game_id.clone(), summiter_retribution);
    }
    fn what_kind_user(env: Env, user: Address, setting: i128) -> (i32, i128) {
        let betData: Bet = storage::get_bet(env.clone(), user.clone(), setting.clone());
        if betData.id == 0 {
            return (6, 0); // no bet
        }
        let (exist, startTime, endTime, summiter, checkers, _) =
            storage::existBet(env.clone(), betData.clone().gameid);
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }
        let xresult: ResultGame = storage::get_ResultGame(env.clone(), betData.clone().gameid);
        let resultAssessment: ResultAssessment =
            storage::get_ResultAssessment(env.clone(), betData.clone().gameid);
        let complain = storage::get_complain(env.clone(), betData.clone().gameid);
        match complain {
            0 => {
                if resultAssessment.CheckApprove.contains(&user) {
                    return (3, 0); // user dishonest
                }
                if resultAssessment.CheckReject.contains(&user) {
                    if betData.bet == xresult.result {
                        return (1, betData.clone().amount_bet); // winner and honest
                    } else {
                        return (2, betData.clone().amount_bet); // loser and honest
                    }
                }
                if !resultAssessment.CheckApprove.contains(&user)
                    && !resultAssessment.CheckReject.contains(&user)
                {
                    if betData.bet == xresult.result {
                        return (4, betData.clone().amount_bet); // winner and honest
                    } else {
                        return (5, 0); // loser and honest
                    }
                } else {
                    panic_with_error!(&env, BettingError::InvalidInputError);
                }
            }
            1 => {
                if resultAssessment.CheckApprove.contains(&user) {
                    if betData.bet == xresult.result {
                        return (1, betData.clone().amount_bet); // winner and honest
                    } else {
                        return (2, betData.clone().amount_bet); // loser and honest
                    }
                }
                if resultAssessment.CheckReject.contains(&user) {
                    return (3, 0); // user dishonest
                }
                if !resultAssessment.CheckApprove.contains(&user)
                    && !resultAssessment.CheckReject.contains(&user)
                {
                    if betData.bet == xresult.result {
                        return (4, betData.clone().amount_bet); // winner ?
                    } else {
                        return (5, 0); // loser ?
                    }
                } else {
                    panic_with_error!(&env, BettingError::InvalidInputError);
                }
            }
            2 => {
                if resultAssessment.CheckApprove.contains(&user) {
                    if betData.bet == xresult.result {
                        return (1, betData.clone().amount_bet); // winner and honest
                    } else {
                        return (2, betData.clone().amount_bet); // loser and honest
                    }
                }
                if !resultAssessment.CheckApprove.contains(&user)
                    && !resultAssessment.CheckReject.contains(&user)
                {
                    if betData.bet == xresult.result {
                        return (4, betData.clone().amount_bet); // winner ?
                    } else {
                        return (5, 0); // loser ?
                    }
                } else {
                    panic_with_error!(&env, BettingError::InvalidInputError);
                }
            }
            _ => {
                panic_with_error!(&env, BettingError::InvalidInputError);
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
        let adminAdr: Address = storage::get_admin(env.clone());

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
        let mut main_summiter: Address = adminAdr;
        for k in 0..5 {
            let pick = rng % top.len();
            let (addr, score) = top.get(pick).unwrap();
            selected.push_back((addr.clone(), score));
            if k == 0 {
                main_summiter = addr.clone();
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
            main_summiter.clone(),
            selected_Summitters.clone(),
        );
        BettingEvents::summiters_seleted(&env, game_id, selected_Summitters, main_summiter);
    }
    fn moveToken(env: &Env, token: &Address, from: &Address, to: &Address, amount: &i128) {
        let token = token::Client::new(env, token);
        token.transfer(from, &to, amount);
    }
}
