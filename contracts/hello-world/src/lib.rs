#![no_std]
use core::{f32::consts::E, panic, result};

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, vec,
    xdr::{ScVal, ToXdr, WriteXdr},
    Address, Bytes, BytesN, Env, IntoVal, String, Symbol, Vec,
};

const LEADERBOARD: Symbol = symbol_short!("LB");
const SUMITTERS_HISTORY: Symbol = symbol_short!("H_S");
const COUNTER: Symbol = symbol_short!("COUNTER");
const x: Symbol = symbol_short!("x");

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
struct Game {
    id: i128,
    league: i128,
    description: String,
    team_local: i128,
    team_away: i128,
    startTime: u32,
    endTime: u32,
    summiter: Address,
    Checker: Vec<Address>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
struct ResultGame {
    id: i128,
    gameid: i128,
    description: String,
    result: BetKey,
    pause: bool,
}
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
struct ResultAssessment {
    id: i128,
    gameid: i128,
    CheckApprove: Vec<Address>,
    CheckReject: Vec<Address>,
    UsersApprove: Vec<Address>,
    UsersReject: Vec<Address>,
}
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
struct PrivateBet {
    id: i128,
    gameid: i128,
    active: bool,
    description: String,
    amount_bet_min: i128,
    users_invated: Vec<Address>,
}
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
struct PublicBet {
    id: i128,
    gameid: i128,
    active: bool,
    description: String,
}
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
struct Bet {
    id: i128,
    gameid: i128,
    betType: BetType,
    Setting: i128,
    bet: BetKey,
    amount_bet: i128,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Game(i128),
    Result(i128),
    ClaimWinner(Address),
    ClaimSummiter(Address),
    ResultAssessment(i128),
    GameSummiters(i128),
    History_Summiter(Address),
    SetPrivateBet(i128),
    SetPublicBet(i128),
    Bet(Address, i128),
    PrivateBetList(i128),
    lastBet(BetKey),
}
#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum BetKey {
    Team_local,
    Team_away,
    Draw,
}
#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum AssessmentKey {
    approve,
    reject,
}
#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum BetType {
    Public,
    Private,
}
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
struct Summiter {
    user: Address,
    stakeAmount: i128,
    gameId: i128,
}

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn request_result_summiter(
        env: Env,
        user: Address,
        stakeAmount: i128,
        gameId: i128,
    ) -> bool {
        user.require_auth();
        if stakeAmount <= 0 {
            panic!("Stake amount must be greater than 0");
        }
        if gameId <= 0 {
            panic!("Invalid game ID");
        }
        /*We nee to set a amount to request for the summiter rol */
        if stakeAmount == 20 {
            panic!("Stake amount must be at least 10");
        }
        // ✅ Get history map (user → score history)
        let history: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::History_Summiter(user.clone()))
            .unwrap_or(10);

        // ✅ Weighted score calculation
        let new_score = (history * 70 + stakeAmount * 30) / 100;

        // ✅ Leaderboard vector
        let mut leaderboard: Vec<(Address, i128)> = env
            .storage()
            .persistent()
            .get(&DataKey::GameSummiters(gameId))
            .unwrap_or(Vec::new(&env));

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
        env.storage()
            .persistent()
            .set(&DataKey::GameSummiters(gameId), &leaderboard);
        env.events()
            .publish((COUNTER, symbol_short!("increment")), leaderboard);

        true
    }
    pub fn select_summiter(env: Env, game_id: i128) -> Vec<(Address, i128)> {
        let (exist, startTime, endTime, summiter, checkers) =
            Self::existBet(env.clone(), game_id.clone());
        if !exist {
            panic!("Game haven't been set yet");
        }
        let leaderboard: Vec<(Address, i128)> = env
            .storage()
            .persistent()
            .get(&DataKey::GameSummiters(game_id))
            .unwrap_or(Vec::new(&env));

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

        for k in 0..5 {
            let pick = rng % top.len();
            let (addr, score) = top.get(pick).unwrap();
            let mut checks: Vec<Address> = Vec::new(&env);
            selected.push_back((addr.clone(), score));
            if k == 0 {
                env.storage().persistent().update(
                    &DataKey::Game(game_id),
                    |maybe_game: Option<Game>| {
                        let mut game = maybe_game.unwrap_or(Game {
                            id: game_id,
                            league: 1,
                            description: String::from_str(&env, ""),
                            team_local: 0,
                            team_away: 0,
                            startTime: 0,
                            endTime: 0,
                            summiter: addr.clone(),
                            Checker: Vec::new(&env),
                        });
                        game.summiter = addr.clone();
                        game
                    },
                );
            }
            if k > 0 {
                checks.push_back(addr.clone());
            }
            env.storage().persistent().update(
                &DataKey::Game(game_id),
                |maybe_game: Option<Game>| {
                    let mut game = maybe_game.unwrap_or(Game {
                        id: game_id,
                        league: 1,
                        description: String::from_str(&env, ""),
                        team_local: 0,
                        team_away: 0,
                        startTime: 0,
                        endTime: 0,
                        summiter: addr.clone(),
                        Checker: Vec::new(&env),
                    });
                    game.Checker = checks.clone();
                    game
                },
            );

            top.remove(pick); // remove picked
            if top.len() == 0 {
                break;
            }
        }

        selected
    }
    pub fn bet(env: Env, user: Address, bet: Bet) {
        user.require_auth();
        if bet.clone().amount_bet <= 0 {
            panic!("Bet amount must be greater than 0");
        }
        if bet.clone().id == 0 || bet.clone().Setting == 0 {
            panic!("Invalid bet data");
        }

        if bet.clone().betType == BetType::Private {
            let privateBet: PrivateBet = env
                .storage()
                .persistent()
                .get(&DataKey::SetPrivateBet(bet.clone().Setting))
                .unwrap_or(PrivateBet {
                    id: 0,
                    gameid: 0,
                    active: false,
                    description: String::from_slice(&env, "No private bet found"),
                    amount_bet_min: 0,
                    users_invated: Vec::new(&env),
                });

            if privateBet.clone().id == 0 {
                panic!("Private bet not found");
            }
            if !privateBet.clone().users_invated.contains(&user) {
                panic!("You are not invited to this private bet");
            }
            if bet.clone().amount_bet < privateBet.clone().amount_bet_min {
                panic!("Bet amount is less than minimum required");
            }
            let (exist, startTime, endTime, _, _) =
                Self::existBet(env.clone(), privateBet.clone().gameid);
            if !exist {
                panic!("Game haven't been set yet");
            }
            if startTime > env.ledger().timestamp() as u32 {
                panic!("Game haven't started yet");
            }
            if endTime < env.ledger().timestamp() as u32 {
                panic!("Game has already ended");
            }
            if Self::CheckUser(env.clone(), user.clone(), privateBet.clone().gameid) {
                panic!("You are not allowed to bet on this game");
            }
            let mut listedBet: Vec<(i128)> = env
                .storage()
                .persistent()
                .get(&DataKey::PrivateBetList(bet.clone().gameid))
                .unwrap_or(Vec::new(&env));
            listedBet.push_back(bet.clone().Setting);
            env.storage()
                .persistent()
                .set(&DataKey::PrivateBetList(bet.clone().gameid), &listedBet);
            env.storage()
                .persistent()
                .set(&DataKey::Bet(user.clone(), bet.clone().Setting), &bet);
        } else if (bet.clone().betType == BetType::Public) {
            let publicBet: PublicBet = env
                .storage()
                .persistent()
                .get(&DataKey::SetPublicBet(bet.clone().Setting))
                .unwrap_or(PublicBet {
                    id: 0,
                    gameid: 0,
                    active: false,
                    description: String::from_slice(&env, "No public bet found"),
                });
            if publicBet.clone().id == 0 {
                panic!("Public bet not found");
            }
            let (exist, startTime, endTime, _, _) =
                Self::existBet(env.clone(), publicBet.clone().gameid);
            if !exist {
                panic!("Game haven't been set yet");
            }
            if startTime > env.ledger().timestamp() as u32 {
                panic!("Game haven't started yet");
            }
            if endTime < env.ledger().timestamp() as u32 {
                panic!("Game has already ended");
            }
            if Self::CheckUser(env.clone(), user.clone(), publicBet.clone().gameid) {
                panic!("You are not allowed to bet on this game");
            }
            env.storage()
                .persistent()
                .set(&DataKey::Bet(user.clone(), bet.clone().Setting), &bet);
        }
    }
    //admin address
    pub fn set_game(env: Env, game: Game, signature: BytesN<64>, pub_key: BytesN<32>) {
        let (exist, startTime, endTime, summiter, checkers) =
            Self::existBet(env.clone(), game.clone().id);
        if exist {
            panic!("Game haven been set already");
        }
        let encoded = game.clone().to_xdr(&env);
        // Now wrap into Soroban Bytes
        env.crypto().ed25519_verify(&pub_key, &encoded, &signature);
        env.storage()
            .persistent()
            .set(&DataKey::Game(game.id), &game);
        let pubSetting = PublicBet {
            id: game.id,
            gameid: game.id,
            active: false,
            description: String::from_slice(&env, "Public Bet"),
        };
        env.storage()
            .persistent()
            .set(&DataKey::SetPublicBet(game.id), &pubSetting);
        Self::select_summiter(env.clone(), game.id);
    }
    fn set_private_bet(env: Env, user: Address, privateData: PrivateBet, game: Game) {
        user.require_auth();

        let (exist, startTime, endTime, summiter, checkers) =
            Self::existBet(env.clone(), game.clone().id);
        if !exist {
            panic!("Game haven't been set yet");
        }
        if (privateData.id == 0 || privateData.gameid != game.clone().id) {
            panic!("Invalid private bet data");
        }
        env.storage()
            .persistent()
            .set(&DataKey::SetPrivateBet(privateData.id), &privateData);
    }
    fn CheckUser(env: Env, user: Address, game_id: i128) -> bool {
        let mut check: bool = false;
        let receiveGame = env
            .storage()
            .persistent()
            .get(&DataKey::Game(game_id))
            .unwrap_or(Game {
                id: 0,
                league: 0,
                description: String::from_slice(&env, "No game found"),
                team_local: 0,
                team_away: 0,
                startTime: 0,
                endTime: 0,
                summiter: Address::from_string(&String::from_str(
                    &env,
                    "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
                )),
                Checker: Vec::new(&env),
            });

        if receiveGame.summiter != user && !receiveGame.Checker.contains(&user) {
            check = false;
        } else {
            check = true;
        }
        check
    }
    fn existBet(env: Env, game_id: i128) -> (bool, u32, u32, Address, Vec<Address>) {
        let mut check: bool = false;
        let receiveGame = env
            .storage()
            .persistent()
            .get(&DataKey::Game(game_id))
            .unwrap_or(Game {
                id: 0,
                league: 0,
                description: String::from_slice(&env, "No game found"),
                team_local: 0,
                team_away: 0,
                startTime: 0,
                endTime: 0,
                summiter: Address::from_string(&String::from_str(
                    &env,
                    "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
                )),
                Checker: Vec::new(&env),
            });

        if receiveGame.id == 0 {
            check = false;
        } else {
            check = true;
        }
        (
            check,
            receiveGame.startTime,
            receiveGame.endTime,
            receiveGame.summiter,
            receiveGame.Checker,
        )
    }
    pub fn summitResult(env: Env, user: Address, result: ResultGame) -> ResultGame {
        user.require_auth();
        let (exist, startTime, endTime, summiter, checkers) =
            Self::existBet(env.clone(), result.clone().gameid);
        if !exist {
            panic!("Game haven't been set yet");
        }
        if summiter != user {
            panic!("You are not allowed to summit this result");
        }

        if endTime > env.ledger().timestamp() as u32 {
            panic!("Game hasn't ended yet");
        }

        /* we need to limit the time summiter send results
        and also in case a summiter havent send the result
         within the widows 1 of the checker will be in
         charge if not then the supreme court
         */

        env.storage()
            .persistent()
            .set(&DataKey::Result(result.clone().gameid), &result);
        result
    }
    pub fn assessResult(env: Env, user: Address, bet: Bet, game_id: i128, desition: AssessmentKey) {
        user.require_auth();
        let (exist, startTime, endTime, summiter, checkers) =
            Self::existBet(env.clone(), game_id.clone());
        if !exist {
            panic!("Game haven't been set yet");
        }
        if endTime > env.ledger().timestamp() as u32 {
            panic!("Game hasn't ended yet");
        }
        if endTime + (5 * 60 * 60) < env.ledger().timestamp() as u32 {
            panic!("Game hasn't ended yet");
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

    pub fn claim(env: Env, user: Address, bet: Bet) -> i128 {
        user.require_auth();

        let betData: Bet = env
            .storage()
            .persistent()
            .get(&DataKey::Bet(user.clone(), bet.clone().Setting))
            .unwrap_or_else(|| panic!("No bet found for this user"));
        let xresult: ResultGame = env
            .storage()
            .persistent()
            .get(&DataKey::Result(bet.clone().gameid))
            .unwrap_or_else(|| panic!("No result found for this game"));
        if xresult.pause == true {
            panic!("Game result is paused, you cannot withdraw now");
        }
        let assesments: ResultAssessment = env
            .storage()
            .persistent()
            .get(&DataKey::ResultAssessment(bet.clone().gameid))
            .unwrap_or(ResultAssessment {
                id: 0,
                gameid: bet.clone().gameid,
                CheckApprove: Vec::new(&env),
                CheckReject: Vec::new(&env),
                UsersApprove: Vec::new(&env),
                UsersReject: Vec::new(&env),
            });

        if !assesments.UsersApprove.contains(&user) && !assesments.UsersReject.contains(&user) {
            panic!("You are not allowed to withdraw from this bet");
        }
        let money: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::ClaimWinner(user.clone()))
            .unwrap_or(0);
        money
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
        /*for setting in listedPrivateBet.iter() {
            Self::distibution_private(
                env.clone(),
                complain,
                result.clone().gameid,
                result.clone(),
                setting.clone(),
            );
        }*/
        env.storage()
            .persistent()
            .set(&DataKey::Result(result.clone().gameid), &result);
    }
    /*Fines:
    1. users who don't participate will get the bet amount back
    2. users who act dishonestly will lose their bet amount
    3. summiters with wrong result will lose their stake
     */
    fn distibution(env: Env, complain: i128, game_id: i128, result: ResultGame) {
        let mut winners: Vec<Address> = Vec::new(&env);
        let mut losers: Vec<Address> = Vec::new(&env);
        let mut s_honest: Vec<Address> = Vec::new(&env);
        let mut s_dishonest: Vec<Address> = Vec::new(&env);

        let mut honest: Vec<Address> = Vec::new(&env);
        let mut dishonest: Vec<Address> = Vec::new(&env);

        let mut amount_gained: i128 = 0;
        let mut amount_winner_pool: i128 = 0;

        let listedPrivateBet: Vec<(i128)> = env
            .storage()
            .persistent()
            .get(&DataKey::PrivateBetList(game_id.clone()))
            .unwrap_or(Vec::new(&env));
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
        if complain == 0 {
            dishonest = assesments.UsersApprove.clone();
            s_honest = assesments.CheckReject.clone();
            s_dishonest = assesments.CheckApprove.clone();
            honest = assesments.UsersReject.clone();
        } else {
            dishonest = assesments.UsersReject.clone();
            s_honest = assesments.CheckApprove.clone();
            s_dishonest = assesments.CheckReject.clone();
            honest = assesments.UsersApprove.clone();
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
                losers.push_back(user.clone());
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
                }
            }
        }

        let mut summiter_retribution = (amount_gained * 20) / 100;
        let protocol_retribution = (amount_gained * 10) / 100;
        amount_gained -= summiter_retribution;
        amount_gained -= protocol_retribution;
        let add = 20 * s_dishonest.len() as i128;
        summiter_retribution += add;

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
                let total = user_amount + bet.amount_bet;
                //send user_amount to user
                env.storage()
                    .persistent()
                    .set(&DataKey::ClaimWinner(user.clone()), &total);
            }
        }
        for s_user in s_dishonest.iter() {
            env.storage()
                .persistent()
                .set(&DataKey::History_Summiter(s_user), &-100);
        }
        let mut each_summiter_share = 0;
        if s_honest.len() != 0 {
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
    fn distibution_private(
        env: Env,
        complain: i128,
        game_id: i128,
        result: ResultGame,
        setting: i128,
    ) {
        let mut winners: Vec<Address> = Vec::new(&env);
        let mut losers: Vec<Address> = Vec::new(&env);

        let mut honest: Vec<Address> = Vec::new(&env);
        let mut dishonest: Vec<Address> = Vec::new(&env);

        let mut amount_gained: i128 = 0;
        let mut amount_winner_pool: i128 = 0;

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
        let mut privateBet: PrivateBet = env
            .storage()
            .persistent()
            .get(&DataKey::SetPrivateBet(setting))
            .unwrap_or(PrivateBet {
                id: 0,
                gameid: 0,
                active: false,
                description: String::from_slice(&env, "No private bet found"),
                amount_bet_min: 0,
                users_invated: Vec::new(&env),
            });
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
                amount_gained += bet.amount_bet;
                losers.push_back(user.clone());
                continue;
            }
            if complain == 0 {
                if assesments.UsersApprove.contains(&user) {
                    dishonest.push_back(user.clone());
                } else {
                    honest.push_back(user.clone());
                }
            } else {
                if assesments.UsersReject.contains(&user) {
                    dishonest.push_back(user.clone());
                } else {
                    honest.push_back(user.clone());
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
                losers.push_back(user.clone());
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
                env.storage()
                    .persistent()
                    .set(&DataKey::ClaimWinner(user.clone()), &user_amount);
            }
        }
    }
}
mod test;
