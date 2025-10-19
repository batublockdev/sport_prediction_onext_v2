#![no_std]

use crate::{
    bettingTrait::betting,
    errors::BettingError,
    events::BettingEvents,
    storage,
    types::{
        AssessmentKey, Bet, BetKey, BetType, ClaimType, DataKey, Game, LastB, PrivateBet,
        PublicBet, ResultAssessment, ResultGame,
    },
    Constants::{
        FIFTY_PERCENT, FIFTY_POINTS, HUNDRED_POINTS, LESS_HUNDRED_POINTS, MINUS_TWENTY_POINTS,
        ONE_HOUR_SECONDS, SCORE_HISTORY_WEIGHT, TEN_PERCENT, TRUST_TOKEN_PERCENTAGE,
        TWENTY_PERCENT, TWENTY_POINTS, VOTE_HISTORY_WEIGHT,
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
    fn __constructor(
        env: Env,
        admin: Address,
        admin_pubkey: BytesN<32>,
        token_usd: Address,
        token_trust: Address,
        supreme_court: Address,
    ) {
        admin.require_auth();

        // check if already initialized
        if storage::has_init(&env) {
            panic_with_error!(&env, BettingError::AlreadyInitializedError);
        }
        // Save data
        storage::init(
            env,
            admin,
            admin_pubkey,
            token_usd,
            token_trust,
            supreme_court,
        );
    }

    /*
       @dev this funtion request to be a result summiter
       @param env Environment
       @param user Address The address of the user
       @param stakeAmount i128 The amount to stake
    */
    fn request_result_summiter(env: Env, user: Address, stakeAmount: i128) -> bool {
        //add the fun that allows user to be summiter using his honesty points
        user.require_auth();
        let min_stake = storage::get_Min_stakeAmount(env.clone());
        if stakeAmount <= min_stake {
            panic_with_error!(&env, BettingError::NotEnoughStake);
        }
        let usd = storage::get_usd(env.clone());
        let contract_address = env.current_contract_address();
        Self::moveToken(&env, &usd, &user, &contract_address, &stakeAmount);
        Self::adduser_board(&env, user.clone(), stakeAmount.clone());
        true
    }
    /*
       @dev this funtion bet on a game
       @param env Environment
       @param user Address The address of the user
       @param bet Bet The bet data
    */
    fn bet(env: Env, user: Address, bet: Bet) -> bool {
        user.require_auth();
        let contract_address = env.current_contract_address();
        let usd = storage::get_usd(env.clone());
        let trust: Address = storage::get_trust(env.clone());
        if bet.clone().amount_bet <= 0 {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        if bet.clone().id == 0 || bet.clone().Setting == 0 || bet.clone().gameid == 0 {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        let (exist, startTime, endTime, _, _, active) =
            storage::existBet(env.clone(), bet.clone().gameid);
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }
        if startTime < env.ledger().timestamp() as u32 {
            panic_with_error!(&env, BettingError::GameHasAlreadyStarted);
        }
        if storage::CheckUser(env.clone(), user.clone(), bet.clone().gameid) {
            panic_with_error!(&env, BettingError::Summiters_notAllowToBet);
        }

        if bet.clone().betType == BetType::Private {
            let privateBet: PrivateBet = storage::get_PrivateBet(env.clone(), bet.clone().Setting);

            if privateBet.clone().id == 0 {
                panic_with_error!(&env, BettingError::SettingBetDoesNotExist);
            }
            if !privateBet.clone().users_invated.contains(&user) {
                panic_with_error!(&env, BettingError::PrivateBet_NotAllowToBet);
            }
            if bet.clone().amount_bet != privateBet.clone().amount_bet_min {
                panic_with_error!(&env, BettingError::PrivateBet_NotEnoughToBet);
            }
            storage::add_bet(env.clone(), user.clone(), bet.clone());
            storage::add_not_assesed_yet(
                env.clone(),
                bet.clone().Setting,
                bet.clone().amount_bet,
                bet.clone().bet,
            );
            if !privateBet.active {
                if storage::does_bet_active(env.clone(), bet.clone()) {
                    storage::active_private_setting(env.clone(), bet.clone().Setting, true);
                    BettingEvents::active_setting(&env, privateBet.gameid, privateBet.id);
                    //if the game hasn't been actived we select the summiter
                    if active == false {
                        Self::select_summiter(env.clone(), privateBet.gameid)
                    }
                }
            }
        } else {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        storage::add_UsersAmount(env.clone(), bet.clone().Setting);
        storage::add_total_bet(env.clone(), bet.clone().gameid, bet.clone().amount_bet);
        storage::add_HonestyPoints(env.clone(), user.clone(), MINUS_TWENTY_POINTS);
        let points = storage::get_HonestyPoints(env.clone(), user.clone());
        BettingEvents::user_honesty_points(&env, user.clone(), points);
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
            &((bet.clone().amount_bet * TRUST_TOKEN_PERCENTAGE) / 100),
        );
        true
    }
    /*
       @dev This function claim the refund in two conditions
       1. the bet setting has not been activated ( all users have bet on the same )
        2. the game has finished and no result has been summited after 3 hours of the end time
       @param env Environment
       @param user Address The address of the user
       @param setting i128 The id of the setting
    */
    fn claim_refund(env: Env, user: Address, setting: i128) -> i128 {
        user.require_auth();
        let contract_address = env.current_contract_address();
        let usd = storage::get_usd(env.clone());
        let trust: Address = storage::get_trust(env.clone());
        let mut totalBet = 0;
        let betData: Bet = storage::get_bet(env.clone(), user.clone(), setting.clone());
        if betData.id == 0 {
            panic_with_error!(&env, BettingError::NoBetHasBeenFound);
        }
        let mut amountUsd = betData.clone().amount_bet;
        let receivedResult: ResultGame =
            storage::get_ResultGame(env.clone(), betData.clone().gameid);
        let finesApplied = storage::get_fines_applied(env.clone(), betData.clone().gameid);
        let (_, startTime, endTime, summiter, checkers, _) =
            storage::existBet(env.clone(), betData.clone().gameid);

        let doneBefore = storage::get_didUserWithdraw(env.clone(), user.clone(), setting.clone());
        if doneBefore {
            panic_with_error!(&env, BettingError::UserAlreadyClaimed);
        }
        if startTime < env.ledger().timestamp() as u32 {
            if betData.betType == BetType::Private {
                let privateBet: PrivateBet =
                    storage::get_PrivateBet(env.clone(), betData.clone().Setting);
                if privateBet.clone().id == 0 {
                    panic_with_error!(&env, BettingError::SettingBetDoesNotExist);
                }
                if privateBet.clone().active {
                    if endTime + (3 * ONE_HOUR_SECONDS) < env.ledger().timestamp() as u32 {
                        if receivedResult.id != 0 {
                            panic_with_error!(&env, BettingError::GameIsActive);
                        }
                    } else {
                        panic_with_error!(&env, BettingError::GameIsActive);
                    }
                }
            } else {
                panic_with_error!(&env, BettingError::InvalidInputError);
            }
            if endTime + (3 * ONE_HOUR_SECONDS) < env.ledger().timestamp() as u32 {
                if receivedResult.id == 0 {
                    //check if the fines have been applied if not, we do it
                    if finesApplied != 2 {
                        let mut summiters: Vec<Address> = checkers.clone();
                        summiters.push_front(summiter.clone());
                        for user in summiters.iter() {
                            storage::set_history(env.clone(), user.clone(), LESS_HUNDRED_POINTS);
                            let stake: i128 = storage::get_stakeAmount_user_game(
                                env.clone(),
                                user.clone(),
                                betData.clone().gameid,
                            );
                            storage::add_Fine(env.clone(), betData.clone().gameid, stake);
                        }
                        storage::set_fines_applied(env.clone(), betData.clone().gameid, 2);
                    }
                    let totalFine = storage::get_Fine(env.clone(), betData.clone().gameid);
                    totalBet += storage::get_total_bet(env.clone(), betData.clone().gameid);
                    let user_share = (betData.clone().amount_bet * 100) / totalBet;
                    let user_amount = (user_share * totalFine) / 100;
                    amountUsd = user_amount + betData.clone().amount_bet
                }
            }
            Self::moveToken(&env, &usd, &contract_address, &user, &amountUsd);
            Self::moveToken(
                &env,
                &trust,
                &contract_address,
                &user,
                &((betData.clone().amount_bet * TRUST_TOKEN_PERCENTAGE) / 100),
            );
            storage::add_HonestyPoints(env.clone(), user.clone(), TWENTY_POINTS);
            let points = storage::get_HonestyPoints(env.clone(), user.clone());
            BettingEvents::user_honesty_points(&env, user.clone(), points);
            storage::set_didUserWithdraw(env.clone(), user.clone(), setting.clone());
        } else {
            panic_with_error!(&env, BettingError::NothingToClaim);
        }
        amountUsd
    }
    /*
       @dev This function set a game to be bet on with the admin premission
       @param env Environment
       @param game Game The game data
       @param signature BytesN<64> The signature of the game data
       @param pub_key BytesN<32> The public key of the signer
    */
    fn set_game(env: Env, game: Game, signature: BytesN<64>) -> bool {
        let (exist, startTime, endTime, summiter, checkers, _) =
            storage::existBet(env.clone(), game.clone().id);
        if exist {
            panic_with_error!(&env, BettingError::GameHasAlreadySet);
        }
        if game.clone().id == 0
            || game.clone().startTime == 0
            || game.clone().endTime == 0
            || game.clone().startTime >= game.clone().endTime
            || game.active
        {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        let encoded = game.clone().to_xdr(&env);
        // Now wrap into Soroban Bytes
        let admin_pubkey = storage::get_admin_pubkey(env.clone());

        env.crypto()
            .ed25519_verify(&admin_pubkey, &encoded, &signature);
        storage::set_game(env.clone(), game.clone());
        BettingEvents::game_set(&env, game.id);
        true
    }
    /*
       @dev This function set a private bet setting for a game
       @param env Environment
       @param user Address The address of the user
       @param privateData PrivateBet The private bet data
       @param game_id i128 The id of the game
    */
    fn set_private_bet(env: Env, user: Address, privateData: PrivateBet, game_id: i128) -> bool {
        user.require_auth();

        let (exist, startTime, endTime, summiter, checkers, _) =
            storage::existBet(env.clone(), game_id.clone());
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }
        if (privateData.id == 0
            || privateData.gameid != game_id.clone()
            || privateData.amount_bet_min <= 0
            || privateData.users_invated.len() == 0
            || privateData.active
            || privateData.settingAdmin != user.clone())
        {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        storage::set_privateSetting(env.clone(), privateData.clone());
        storage::add_privateSettingList(env.clone(), game_id.clone(), privateData.id);
        BettingEvents::private_setting(
            &env,
            game_id,
            privateData.id,
            user,
            privateData.amount_bet_min,
        );
        true
    }
    /*
    @dev This function add a user to a private bet setting
    @param env Environment
    @param setting i128 The id of the setting
    @param game i128 The id of the game
    @param newUser Address The address of the new user to be added
     */
    fn add_user_privateBet(env: Env, setting: i128, game: i128, newUser: Address) -> bool {
        // add fun that allows users to be added in private rooms because of his honesty points
        let mut privateBet: PrivateBet = storage::get_PrivateBet(env.clone(), setting.clone());
        privateBet.settingAdmin.require_auth();
        let (exist, startTime, endTime, summiter, checkers, _) =
            storage::existBet(env.clone(), game.clone());
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }
        if startTime < env.ledger().timestamp() as u32 {
            panic_with_error!(&env, BettingError::GameHasAlreadyStarted);
        }
        if privateBet.clone().id == 0 || privateBet.clone().gameid != game.clone() {
            panic_with_error!(&env, BettingError::SettingBetDoesNotExist);
        }
        if privateBet.users_invated.contains(&newUser) {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        privateBet.users_invated.push_front(newUser.clone());
        storage::set_privateSetting(env.clone(), privateBet.clone());
        BettingEvents::new_user_added_private(&env, game, setting, newUser);
        true
    }
    /*
       @dev This function summit the result of the game
       @param env Environment
       @param user Address The address of the user
       @param result ResultGame The result of the game
    */
    fn summitResult(env: Env, user: Address, result: ResultGame) -> bool {
        user.require_auth();
        if result.clone().id == 0 || result.clone().gameid == 0 {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        let (exist, startTime, endTime, summiter, checkers, active) =
            storage::existBet(env.clone(), result.clone().gameid);
        let receivedResult: ResultGame =
            storage::get_ResultGame(env.clone(), result.clone().gameid);
        let finesApplied = storage::get_fines_applied(env.clone(), result.clone().gameid);
        let admin = storage::get_admin(env.clone());
        if receivedResult.id != 0 {
            panic_with_error!(&env, BettingError::GameResultAlreadySet);
        }
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }

        if endTime > env.ledger().timestamp() as u32 {
            panic_with_error!(&env, BettingError::GameHasNotFinished);
        }
        if result.clone().gameid == 0 || result.clone().id == 0 {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        if result.clone().distribution_executed || result.clone().pause {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        if endTime + (1 * ONE_HOUR_SECONDS) < env.ledger().timestamp() as u32 {
            //we check if the first summiter is the admin if not we apply fine to the summiter and replace it with the admin
            // then checkers have the next hour to summit the result or they will be fined
            if summiter != admin {
                if finesApplied == 0 {
                    storage::set_history(env.clone(), summiter.clone(), -100);
                    let stake: i128 = storage::get_stakeAmount_user_game(
                        env.clone(),
                        summiter.clone(),
                        result.clone().gameid,
                    );
                    storage::add_Fine(env.clone(), result.clone().gameid, stake);
                    storage::update_game(
                        env.clone(),
                        result.clone().gameid,
                        admin.clone(),
                        checkers.clone(),
                    );
                    storage::set_fines_applied(env.clone(), result.clone().gameid, 1);
                }
            }
            if endTime + (2 * ONE_HOUR_SECONDS) < env.ledger().timestamp() as u32 {
                // 2 hours with no summition then the admin can set the result to be assessed by the users
                // and checkers will be fined
                if finesApplied == 1 {
                    for checker in checkers.iter() {
                        storage::set_history(env.clone(), checker.clone(), -100);
                        let stake: i128 = storage::get_stakeAmount_user_game(
                            env.clone(),
                            checker.clone(),
                            result.clone().gameid,
                        );
                        storage::add_Fine(env.clone(), result.clone().gameid, stake);
                    }
                    storage::update_game(
                        env.clone(),
                        result.clone().gameid,
                        admin.clone(),
                        Vec::new(&env),
                    );
                    storage::set_fines_applied(env.clone(), result.clone().gameid, 2);
                }
                if endTime + (3 * ONE_HOUR_SECONDS) > env.ledger().timestamp() as u32 {
                    if summiter != user {
                        panic_with_error!(&env, BettingError::NotAllowToSummitResult);
                    }

                    storage::set_ResultGame(env.clone(), result.clone());
                    BettingEvents::game_result(
                        &env,
                        result.gameid,
                        result.result,
                        result.description,
                    );
                }
            } else {
                if !checkers.contains(&user) {
                    panic_with_error!(&env, BettingError::NotAllowToSummitResult);
                }
                storage::set_ResultGame(env.clone(), result.clone());
                BettingEvents::game_result(&env, result.gameid, result.result, result.description);
            }
        } else {
            if summiter != user {
                panic_with_error!(&env, BettingError::NotAllowToSummitResult);
            }

            storage::set_ResultGame(env.clone(), result.clone());
            BettingEvents::game_result(&env, result.gameid, result.result, result.description);
        }
        true
    }
    /*
       @dev This function assess the result of the game for users and checkers
       @param env Environment
       @param user Address The address of the user
       @param bet Bet The bet of the user
       @param game_id i128 The id of the game
       @param desition AssessmentKey The desition of the user (approve or reject)
    */
    fn assessResult(
        env: Env,
        user: Address,
        setting: i128,
        game_id: i128,
        desition: AssessmentKey,
    ) -> bool {
        user.require_auth();
        let (exist, startTime, endTime, summiter, checkers, _) =
            storage::existBet(env.clone(), game_id.clone());
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }
        if endTime > env.ledger().timestamp() as u32 {
            panic_with_error!(&env, BettingError::GameHasNotFinished);
        }
        if endTime + (5 * ONE_HOUR_SECONDS) < env.ledger().timestamp() as u32 {
            panic_with_error!(&env, BettingError::GameAssesmentHasFinished);
        }

        let mut results: ResultGame = storage::get_ResultGame(env.clone(), game_id.clone());
        if results.id == 0 {
            panic_with_error!(&env, BettingError::GameResultNotFound);
        }
        let mut resultAssessment: ResultAssessment =
            storage::get_ResultAssessment(env.clone(), game_id.clone());

        if !checkers.contains(&user) {
            let betResult: Bet = storage::get_Bet(env.clone(), user.clone(), setting.clone());
            if betResult.id == 0 {
                panic_with_error!(&env, BettingError::BetNotFound);
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
            } else {
                if desition == AssessmentKey::approve {
                    resultAssessment.UsersApprove.push_front(user.clone());
                } else if desition == AssessmentKey::reject {
                    resultAssessment.UsersReject.push_front(user.clone());
                    results.pause = true;
                }
            }
            storage::add_UsersAmountVoted(env.clone(), setting.clone());
            if storage::UsersAmount(env.clone(), setting.clone()) {
                BettingEvents::all_vote(&env, setting.clone());
            }
            storage::set_ResultAssessment(env.clone(), game_id.clone(), resultAssessment.clone());
            storage::delete_not_assesed_yet(
                env.clone(),
                betResult.clone().Setting,
                betResult.clone().amount_bet,
                betResult.clone().bet,
            );
            if results.pause {
                BettingEvents::game_result_reject(&env, game_id.clone());
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
            } else {
                if desition == AssessmentKey::approve {
                    resultAssessment.CheckApprove.push_front(user.clone());
                } else if desition == AssessmentKey::reject {
                    resultAssessment.CheckReject.push_front(user.clone());
                }
            }
            storage::set_ResultAssessment(env.clone(), game_id.clone(), resultAssessment.clone());
        }
        true
    }
    /*
       @dev This function claim the money won or the money staked for the summiter and the protocol
       @param env Environment
       @param user Address The address of the user
       @param typeClaim ClaimType The type of claim (summiter, protocol, user)
       @param setting i128 The id of the setting (only for user claim)
    */
    fn claim(env: Env, user: Address, typeClaim: ClaimType, setting: i128) -> (i128, i128) {
        user.require_auth();
        let contract_address = env.current_contract_address();
        let adminAdr: Address = storage::get_admin(env.clone());
        let usd = storage::get_usd(env.clone());
        let trust: Address = storage::get_trust(env.clone());
        let mut amountWithdrew: (i128, i128) = (0, 0);
        match typeClaim {
            ClaimType::Summiter => {
                let money: i128 = storage::get_ClaimSummiter(env.clone(), user.clone());
                Self::moveToken(&env, &usd, &contract_address, &user, &money);
                amountWithdrew = (money, 0);

                storage::zero_ClaimSummiter(env.clone(), user.clone());
            }
            ClaimType::Protocol => {
                adminAdr.require_auth();
                let money: i128 = storage::get_ClaimProtocol(env.clone());
                let trustAmount: i128 = storage::get_ClaimProtocolTrust(env.clone());
                Self::moveToken(&env, &usd, &contract_address, &adminAdr, &money);
                Self::moveToken(&env, &trust, &contract_address, &adminAdr, &trustAmount);
                amountWithdrew = (money, trustAmount);
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
                        amountWithdrew = (bet_50, 0);
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
                            let trust_amount = (amountBet * TRUST_TOKEN_PERCENTAGE) / 100;
                            Self::moveToken(&env, &trust, &contract_address, &user, &trust_amount);
                            amountWithdrew = (user_amount, trust_amount);
                        } else {
                            let trust_amount = (amountBet * TRUST_TOKEN_PERCENTAGE) / 100;
                            Self::moveToken(&env, &trust, &contract_address, &user, &trust_amount);
                            amountWithdrew = (0, trust_amount);
                        }
                        storage::add_HonestyPoints(env.clone(), user.clone(), FIFTY_POINTS);
                        let points = storage::get_HonestyPoints(env.clone(), user.clone());
                        BettingEvents::user_honesty_points(&env, user.clone(), points);
                        storage::set_didUserWithdraw(env.clone(), user.clone(), setting.clone());
                    }
                    1 => {
                        // winner honest
                        let user_share = (amountBet * 100) / winner_pool;
                        let user_amount = (user_share * amount_share) / 100;
                        let total = amountBet + user_amount;
                        Self::moveToken(&env, &usd, &contract_address, &user, &total);
                        let trust_amount = (amountBet * TRUST_TOKEN_PERCENTAGE) / 100;
                        Self::moveToken(&env, &trust, &contract_address, &user, &trust_amount);
                        storage::add_HonestyPoints(env.clone(), user.clone(), FIFTY_POINTS);
                        let points = storage::get_HonestyPoints(env.clone(), user.clone());
                        BettingEvents::user_honesty_points(&env, user.clone(), points);
                        storage::set_didUserWithdraw(env.clone(), user.clone(), setting.clone());
                        amountWithdrew = (total, trust_amount);
                    }
                    _ => {
                        panic_with_error!(&env, BettingError::InvalidInputError);
                    }
                }
            }
            _ => {
                // default case
                panic_with_error!(&env, BettingError::InvalidInputError);
            }
        }
        amountWithdrew
    }
    /*
       @dev This function set the result of the game when a complain has been made and the time to summit the result has passed
       @param env Environment
       @param result ResultGame The result of the game
       This supreme address will be a multi sign address with the trusted respect, and reliable sources will summited the final desition
    */
    fn setResult_supremCourt(env: Env, result: ResultGame) -> bool {
        let supreme = storage::get_supreme(env.clone());
        supreme.require_auth();
        // 0 correct
        // 1 incorrect
        let mut complain = 0;
        let xresult: ResultGame = storage::get_ResultGame(env.clone(), result.clone().gameid);
        let (exist, startTime, endTime, summiter, checkers, _) =
            storage::existBet(env.clone(), xresult.clone().gameid);
        if endTime + (5 * ONE_HOUR_SECONDS) > env.ledger().timestamp() as u32 {
            panic_with_error!(&env, BettingError::GameAssesmentHasFinished);
        }
        if xresult.clone().distribution_executed {
            panic_with_error!(&env, BettingError::GameHasAlreadyBeenExecuted);
        }
        if xresult.pause == false {
            panic_with_error!(&env, BettingError::GameHasNotBeenPaused);
        }
        if xresult.id != result.id || xresult.gameid != result.gameid {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        if result.clone().distribution_executed
            || result.clone().pause
            || xresult.clone().distribution_executed
        {
            panic_with_error!(&env, BettingError::InvalidInputError);
        }
        if xresult.result != result.result {
            complain = 0; // The complain made by the users was correct
        } else {
            complain = 1; // The complain made by the users was incorrect
        }
        let listedPrivateBet: Vec<(i128)> =
            storage::get_privateSettingList(env.clone(), result.clone().gameid);

        if result.result == BetKey::Cancel {
            storage::active_public_setting(env.clone(), result.clone().gameid, false);

            let mut summittersList: Vec<Address> = checkers.clone();
            summittersList.push_front(summiter.clone());
            for summiter in summittersList.iter() {
                let stake = storage::get_stakeAmount_user_game(
                    env.clone(),
                    summiter.clone(),
                    result.clone().gameid,
                );
                Self::adduser_board(&env, summiter.clone(), stake);
            }
        }
        if listedPrivateBet.len() != 0 {
            for setting in listedPrivateBet.iter() {
                let privateBet: PrivateBet = storage::get_PrivateBet(env.clone(), setting.clone());
                if privateBet.active == false {
                    continue;
                }

                if result.result == BetKey::Cancel {
                    storage::active_private_setting(env.clone(), setting.clone(), false);
                } else {
                    Self::make_distribution(
                        env.clone(),
                        privateBet.clone().gameid,
                        setting.clone(),
                        result.clone().result,
                        complain,
                    );
                }
            }
        }

        storage::set_ResultGame(env.clone(), result.clone());
        BettingEvents::game_result_supreme(&env, result.gameid, result.result);
        true
    }
    /*
       @dev This function execute the distribution of the pools according to the rules, fines and betting
       @param env Environment
       @param game_id i128 The id of the game
    */
    fn execute_distribution(env: Env, gameId: i128, setting: i128) -> bool {
        let complain = 2; // 2 means no complain was made
        let result: ResultGame = storage::get_ResultGame(env.clone(), gameId.clone());
        let (exist, startTime, endTime, summiter, checkers, _) =
            storage::existBet(env.clone(), result.clone().gameid);
        if endTime + (5 * ONE_HOUR_SECONDS) > env.ledger().timestamp() as u32 {
            if !storage::UsersAmount(env.clone(), setting.clone()) {
                panic_with_error!(&env, BettingError::GameAssesmentHasFinished);
            }
        }
        let listedPrivateBet: Vec<(i128)> =
            storage::get_privateSettingList(env.clone(), result.clone().gameid);
        let privateBet: PrivateBet = storage::get_PrivateBet(env.clone(), setting.clone());
        if privateBet.active == false {
            panic_with_error!(&env, BettingError::SettingNotActive);
        }
        if result.id == 0 {
            panic_with_error!(&env, BettingError::GameNoResult);
        }
        if result.pause == true {
            panic_with_error!(&env, BettingError::GameHasBeenPaused);
        }
        if result.clone().distribution_executed {
            panic_with_error!(&env, BettingError::GameHasAlreadyBeenExecuted);
        }
        if result.result == BetKey::Cancel {
            storage::active_public_setting(env.clone(), gameId, false);
            let mut summittersList: Vec<Address> = checkers.clone();
            summittersList.push_front(summiter.clone());
            for summiter in summittersList.iter() {
                let stake = storage::get_stakeAmount_user_game(
                    env.clone(),
                    summiter.clone(),
                    result.clone().gameid,
                );
                Self::adduser_board(&env, summiter.clone(), stake);
            }
        }

        if result.result == BetKey::Cancel {
            storage::active_private_setting(env.clone(), setting.clone(), false);
        } else {
            Self::make_distribution(
                env.clone(),
                privateBet.clone().gameid,
                setting.clone(),
                result.clone().result,
                complain,
            );
        }

        true
    }
    /*
       @dev This function set the min amount to stake
       @param env Environment
       @param game_id i128 The id of the game
    */
    fn set_stakeAmount(env: Env, user: Address, amount: i128) -> bool {
        user.require_auth();
        let adminAdr: Address = storage::get_admin(env.clone());
        if adminAdr != user {
            panic_with_error!(&env, BettingError::NotAdmin);
        }
        if amount <= 0 {
            panic_with_error!(&env, BettingError::NegativeAmountError);
        }
        storage::set_Min_stakeAmount(env.clone(), amount);
        BettingEvents::set_stake_amount(&env, amount);
        true
    }
}

impl BettingContract {
    /*
       @dev This  funtion make the distribution of the pools according to the rules, fines and betting
       @param env Environment
       @param game_id i128 The id of the game
       @param setting i128 The id of the setting
       @param resultBet BetKey The result of the game
       @param complain i128 The complain made by the users
       Complain 0 = The complain made by the users was correct
       Complain 1 = The complain made by the users was incorrect
       Complain 2 = No complain was made
    */
    fn make_distribution(
        env: Env,
        game_id: i128,
        setting: i128,
        resultBet: BetKey,
        complain: i128,
    ) {
        let mut result: ResultGame = storage::get_ResultGame(env.clone(), game_id.clone());
        let mut amount_gain_pool: i128 = 0;
        let mut trust_taken_pool: i128 = 0;
        let mut losers_honest_pool: i128 = 0;
        let mut winner_pool: i128 = 0;
        let mut novote_winner: i128 = 0;
        let mut s_dishonest: Vec<Address> = Vec::new(&env);
        let mut s_honest: Vec<Address> = Vec::new(&env);
        let mut s_noVote: Vec<Address> = Vec::new(&env);
        let mut add = 0;
        let admin = storage::get_admin(env.clone());

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
            let amountBet =
                storage::get_not_assesed_yet(env.clone(), setting.clone(), bet_key.clone());
            if resultBet != bet_key {
                amount_gain_pool += amountBet;
                trust_taken_pool += amountBet;
            } else {
                novote_winner += amountBet;
                trust_taken_pool += amountBet;
            }
        }
        let winner_minus_50 = (novote_winner * 50) / 100;
        amount_gain_pool += winner_minus_50;
        match complain {
            0 => {
                // summiter
                s_dishonest.push_back(summiter.clone());
                add += storage::get_stakeAmount_user_game(
                    env.clone(),
                    summiter.clone(),
                    game_id.clone(),
                );
                //Checkers
                for checker in checkers.iter() {
                    if resultAssessment.CheckApprove.contains(&checker) {
                        s_dishonest.push_back(checker.clone());
                        add += storage::get_stakeAmount_user_game(
                            env.clone(),
                            checker.clone(),
                            game_id.clone(),
                        );
                        storage::set_history(env.clone(), checker.clone(), LESS_HUNDRED_POINTS);
                    } else if resultAssessment.CheckReject.contains(&checker) {
                        s_honest.push_back(checker.clone());
                        storage::set_history(env.clone(), checker.clone(), HUNDRED_POINTS);
                    } else {
                        s_noVote.push_back(checker.clone());
                        add += storage::get_stakeAmount_user_game(
                            env.clone(),
                            checker.clone(),
                            game_id.clone(),
                        );
                        storage::set_history(env.clone(), checker.clone(), LESS_HUNDRED_POINTS);
                    }
                }
                let dishonestTeamLocal =
                    storage::get_approve_total(env.clone(), setting.clone(), BetKey::Team_local);
                let dishonestDraw =
                    storage::get_approve_total(env.clone(), setting.clone(), BetKey::Draw);
                let dishonestTeamAway =
                    storage::get_approve_total(env.clone(), setting.clone(), BetKey::Team_away);
                amount_gain_pool += dishonestTeamAway;
                amount_gain_pool += dishonestTeamLocal;
                amount_gain_pool += dishonestDraw;
                trust_taken_pool += dishonestTeamAway;
                trust_taken_pool += dishonestTeamLocal;
                trust_taken_pool += dishonestDraw;

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
                    let amountBet =
                        storage::get_reject_total(env.clone(), setting.clone(), bet_key.clone());
                    if resultBet != bet_key {
                        amount_gain_pool += amountBet;
                        losers_honest_pool += amountBet;
                    } else {
                        winner_pool += amountBet;
                    }
                }
            }
            1 => {
                // summiter
                s_honest.push_back(summiter.clone());
                //Checkers
                for checker in checkers.iter() {
                    if resultAssessment.CheckApprove.contains(&checker) {
                        s_honest.push_back(checker.clone());
                        storage::set_history(env.clone(), checker.clone(), HUNDRED_POINTS);
                    } else if resultAssessment.CheckReject.contains(&checker) {
                        s_dishonest.push_back(checker.clone());
                        add += storage::get_stakeAmount_user_game(
                            env.clone(),
                            checker.clone(),
                            game_id.clone(),
                        );
                        storage::set_history(env.clone(), checker.clone(), LESS_HUNDRED_POINTS);
                    } else {
                        s_noVote.push_back(checker.clone());
                        add += storage::get_stakeAmount_user_game(
                            env.clone(),
                            checker.clone(),
                            game_id.clone(),
                        );
                        storage::set_history(env.clone(), checker.clone(), LESS_HUNDRED_POINTS);
                    }
                }

                let dishonestTeamLocal =
                    storage::get_reject_total(env.clone(), setting.clone(), BetKey::Team_local);
                let dishonestDraw =
                    storage::get_reject_total(env.clone(), setting.clone(), BetKey::Draw);
                let dishonestTeamAway =
                    storage::get_reject_total(env.clone(), setting.clone(), BetKey::Team_away);
                amount_gain_pool += dishonestTeamAway;
                amount_gain_pool += dishonestTeamLocal;
                amount_gain_pool += dishonestDraw;
                trust_taken_pool += dishonestTeamAway;
                trust_taken_pool += dishonestTeamLocal;
                trust_taken_pool += dishonestDraw;
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
                    let amountBet =
                        storage::get_approve_total(env.clone(), setting.clone(), bet_key.clone());
                    if resultBet != bet_key {
                        amount_gain_pool += amountBet;
                        losers_honest_pool += amountBet;
                    } else {
                        winner_pool += amountBet;
                    }
                }
            }
            2 => {
                // summiter
                s_honest.push_back(summiter.clone());
                //Checkers
                for checker in checkers.iter() {
                    if resultAssessment.CheckApprove.contains(&checker) {
                        s_honest.push_back(checker.clone());
                        storage::set_history(env.clone(), checker.clone(), HUNDRED_POINTS);
                    } else if resultAssessment.CheckReject.contains(&checker) {
                        s_dishonest.push_back(checker.clone());
                        add += storage::get_stakeAmount_user_game(
                            env.clone(),
                            checker.clone(),
                            game_id.clone(),
                        );
                        storage::set_history(env.clone(), checker.clone(), LESS_HUNDRED_POINTS);
                    } else {
                        s_noVote.push_back(checker.clone());
                        add += storage::get_stakeAmount_user_game(
                            env.clone(),
                            checker.clone(),
                            game_id.clone(),
                        );
                        storage::set_history(env.clone(), checker.clone(), HUNDRED_POINTS);
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
                    let amountBet =
                        storage::get_approve_total(env.clone(), setting.clone(), bet_key.clone());
                    if resultBet != bet_key {
                        amount_gain_pool += amountBet;
                        losers_honest_pool += amountBet;
                    } else {
                        winner_pool += amountBet;
                    }
                }
            }
            _ => {
                panic_with_error!(&env, BettingError::InvalidInputError);
            }
        }
        amount_gain_pool += add;
        amount_gain_pool += storage::get_Fine(env.clone(), game_id.clone());
        storage::zero_Fine(env.clone(), game_id.clone());
        let mut summiter_retribution = 0;
        let mut protocol_retribution = (amount_gain_pool * TEN_PERCENT) / 100;
        if winner_pool == 0 && losers_honest_pool == 0 {
            if s_honest.len() != 0 {
                if summiter == admin {
                    protocol_retribution = amount_gain_pool;
                } else {
                    summiter_retribution = (amount_gain_pool * FIFTY_PERCENT) / 100;
                    for honest in s_honest.iter() {
                        let mut amount = summiter_retribution / s_honest.len() as i128;
                        amount += storage::get_stakeAmount_user_game(
                            env.clone(),
                            honest.clone(),
                            game_id.clone(),
                        );
                        storage::add_ClaimSummiter(env.clone(), honest.clone(), amount);
                    }
                    protocol_retribution = (amount_gain_pool * FIFTY_PERCENT) / 100;
                }
            }
        } else if s_honest.len() != 0 {
            if summiter != admin {
                summiter_retribution = (amount_gain_pool * TWENTY_PERCENT) / 100;
                for honest in s_honest.iter() {
                    let mut amount = summiter_retribution / s_honest.len() as i128;
                    amount += storage::get_stakeAmount_user_game(
                        env.clone(),
                        honest.clone(),
                        game_id.clone(),
                    );
                    storage::add_ClaimSummiter(env.clone(), honest.clone(), amount);
                }
            }
        }
        amount_gain_pool -= protocol_retribution;
        amount_gain_pool -= summiter_retribution;
        let trust_amount = (trust_taken_pool * TRUST_TOKEN_PERCENTAGE) / 100;
        storage::add_ClaimProtocol(env.clone(), protocol_retribution);
        storage::add_ClaimProtocolTrust(env.clone(), trust_amount);
        storage::save_complain(env.clone(), game_id.clone(), complain);
        storage::save_winnerPool(env.clone(), setting.clone(), winner_pool);
        storage::save_loserPool(env.clone(), setting.clone(), losers_honest_pool);
        storage::set_pool_total(env.clone(), setting.clone(), amount_gain_pool);
        storage::distribution_ResultGame(env.clone(), game_id.clone());
        result.distribution_executed = true;
        storage::set_ResultGame(env.clone(), result.clone());
        BettingEvents::game_setting_distributed(&env, setting);
    }

    /*
       @dev Function to determine the kind of user based on their bet and assessment
       @param env The contract environment
       @param user The address of the user
       @param setting The setting ID of the bet
       @return A tuple containing the kind of user (as an integer) and the amount bet (if applicable)
       1: winner and honest
       2: loser and honest
       3: user dishonest
       4: winner who didn't assess the result
       5: loser who didn't assess the result
       6: no bet
    */
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
                if resultAssessment.UsersApprove.contains(&user) {
                    return (3, 0); // user dishonest
                }
                if resultAssessment.UsersReject.contains(&user) {
                    if betData.bet == xresult.result {
                        return (1, betData.clone().amount_bet); // winner and honest
                    } else {
                        return (2, betData.clone().amount_bet); // loser and honest
                    }
                }
                if !resultAssessment.UsersApprove.contains(&user)
                    && !resultAssessment.UsersReject.contains(&user)
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
                if resultAssessment.UsersApprove.contains(&user) {
                    if betData.bet == xresult.result {
                        return (1, betData.clone().amount_bet); // winner and honest
                    } else {
                        return (2, betData.clone().amount_bet); // loser and honest
                    }
                }
                if resultAssessment.UsersReject.contains(&user) {
                    return (3, 0); // user dishonest
                }
                if !resultAssessment.UsersApprove.contains(&user)
                    && !resultAssessment.UsersReject.contains(&user)
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
                if resultAssessment.UsersApprove.contains(&user) {
                    if betData.bet == xresult.result {
                        return (1, betData.clone().amount_bet); // winner and honest
                    } else {
                        return (2, betData.clone().amount_bet); // loser and honest
                    }
                }
                if !resultAssessment.UsersApprove.contains(&user)
                    && !resultAssessment.UsersReject.contains(&user)
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
    /*
       @dev Function to select summiter and checkers for a game
       @param env The contract environment
       @param game_id The ID of the game
    */
    fn select_summiter(env: Env, game_id: i128) {
        let (exist, startTime, endTime, summiter, checkers, _) =
            storage::existBet(env.clone(), game_id.clone());
        if !exist {
            panic_with_error!(&env, BettingError::GameDoesNotExist);
        }
        let mut leaderboard: Vec<(Address, i128)> = storage::get_leaderboard(env.clone());
        let adminAdr: Address = storage::get_admin(env.clone());
        let mut main_summiter: Address = adminAdr.clone();
        let mut selected_Summitters: Vec<Address> = Vec::new(&env);

        // Limit to top 5
        if leaderboard.len() != 0 {
            let mut top = Vec::new(&env);
            if leaderboard.len() < 10 {
                top = leaderboard.clone();
            } else {
                for i in 0..9 {
                    if let Some((addr, score)) = leaderboard.get(i) {
                        if score == 0 {
                            break;
                        }
                        top.push_back((addr.clone(), score));
                    }
                }
            }

            let sequence = env.ledger().sequence();
            let timestamp = env.ledger().timestamp() as u32;
            let mut rng = (sequence + timestamp) % (top.len() as u32);
            for k in 0..4 {
                let pick = rng % top.len();
                let (addr, score) = top.get(pick).unwrap();
                if k == 0 {
                    main_summiter = addr.clone();
                    storage::set_stakeAmount_user_game(env.clone(), addr.clone(), game_id.clone());
                } else {
                    selected_Summitters.push_back(addr.clone());
                    storage::set_stakeAmount_user_game(env.clone(), addr.clone(), game_id.clone());
                }

                top.remove(pick); // remove picked
                if let Some(pos) = leaderboard.iter().position(|(a, _)| a == addr) {
                    leaderboard.remove(pos.try_into().unwrap());
                }
                if top.len() == 0 {
                    break;
                }
            }
            storage::set_leaderboard(env.clone(), leaderboard);
        }
        storage::update_game(
            env.clone(),
            game_id,
            main_summiter.clone(),
            selected_Summitters.clone(),
        );
        BettingEvents::summiters_seleted(&env, game_id, selected_Summitters, main_summiter);
    }
    /*
    @dev Function to move tokens from one address to another
    @param env The contract environment
    @param token The address of the token contract
    @param from The address to move tokens from
    @param to The address to move tokens to
    @param amount The amount of tokens to move
     */
    fn moveToken(env: &Env, token: &Address, from: &Address, to: &Address, amount: &i128) {
        let token = token::Client::new(env, token);
        token.transfer(from, to, amount);
    }
    /*
        @dev This function add a user to the leaderboard or update his score if he is already in
        @param env Environment
        @param user Address The address of the user
        @param stakeAmount i128 The amount staked by the user
    */
    fn adduser_board(env: &Env, user: Address, stakeAmount: i128) {
        /*We nee to set a amount to request for the summiter rol */
        // ✅ Get history map (user → score history)
        let history: i128 = storage::get_history(env.clone(), user.clone());

        // ✅ Weighted score calculation
        let new_score = (history * VOTE_HISTORY_WEIGHT + stakeAmount * SCORE_HISTORY_WEIGHT) / 100;

        // ✅ Leaderboard vector
        let mut leaderboard: Vec<(Address, i128)> = storage::get_leaderboard(env.clone());

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
        leaderboard.insert(insert_index, (user.clone(), new_score));

        // Save leaderboard
        storage::set_leaderboard(env.clone(), leaderboard);
        storage::set_stakeAmount_user(env.clone(), user.clone(), stakeAmount);
    }
}
