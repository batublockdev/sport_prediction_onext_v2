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
    active: bool,
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
struct LastB {
    id: i128,
    lastBet: BetKey,
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
    lastBet(i128),
    Fine(i128),
    ListBetUser(i128),
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
    fn select_summiter(env: Env, game_id: i128) {
        let (exist, startTime, endTime, summiter, checkers, _) =
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
                            active: true,
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
                        game.active = true;
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
                        active: true,
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
            let (exist, startTime, endTime, _, _, active) =
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
            if !privateBet.active {
                let lastBet: LastB = env
                    .storage()
                    .persistent()
                    .get(&DataKey::lastBet(bet.clone().Setting))
                    .unwrap_or(LastB {
                        id: 0,
                        lastBet: BetKey::Team_local,
                    });
                if lastBet.clone().id == 0 {
                    // it means this is the fisrt bet for this setting
                    env.storage().persistent().set(
                        &DataKey::lastBet(bet.clone().Setting),
                        &LastB {
                            id: bet.clone().Setting,
                            lastBet: bet.clone().bet,
                        },
                    );
                } else {
                    if lastBet.lastBet != bet.clone().bet {
                        env.storage().persistent().update(
                            &DataKey::SetPrivateBet(bet.clone().Setting),
                            |old: Option<PrivateBet>| {
                                let mut res = old.unwrap_or(PrivateBet {
                                    id: privateBet.id,
                                    gameid: privateBet.gameid,
                                    active: false,
                                    description: privateBet.description.clone(),
                                    amount_bet_min: privateBet.amount_bet_min,
                                    users_invated: privateBet.users_invated.clone(),
                                });
                                res.active = true;
                                res
                            },
                        );
                        env.storage().persistent().update(
                            &DataKey::lastBet(bet.clone().Setting),
                            |old: Option<LastB>| {
                                let mut res = old.unwrap_or(LastB {
                                    id: lastBet.id,
                                    lastBet: BetKey::Team_local,
                                });
                                res.lastBet = bet.clone().bet;
                                res
                            },
                        );
                        if active == false {
                            Self::select_summiter(env.clone(), privateBet.gameid)
                        }
                    }
                }
            }
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
            let (exist, startTime, endTime, _, _, active) =
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
            let mut listedBetAddress: Vec<(Address)> = env
                .storage()
                .persistent()
                .get(&DataKey::ListBetUser(bet.clone().gameid))
                .unwrap_or(Vec::new(&env));
            listedBetAddress.push_back(user.clone());
            env.storage()
                .persistent()
                .set(&DataKey::ListBetUser(bet.clone().gameid), &listedBetAddress);

            if !publicBet.active {
                let lastBet: LastB = env
                    .storage()
                    .persistent()
                    .get(&DataKey::lastBet(bet.clone().Setting))
                    .unwrap_or(LastB {
                        id: 0,
                        lastBet: BetKey::Team_local,
                    });
                if lastBet.clone().id == 0 {
                    // it means this is the fisrt bet for this setting
                    env.storage().persistent().set(
                        &DataKey::lastBet(bet.clone().Setting),
                        &LastB {
                            id: bet.clone().Setting,
                            lastBet: bet.clone().bet,
                        },
                    );
                } else {
                    if lastBet.lastBet != bet.clone().bet {
                        env.storage().persistent().update(
                            &DataKey::SetPublicBet(bet.clone().Setting),
                            |old: Option<PublicBet>| {
                                let mut res = old.unwrap_or(PublicBet {
                                    id: publicBet.id,
                                    gameid: publicBet.gameid,
                                    active: false,
                                    description: publicBet.description.clone(),
                                });
                                res.active = true;
                                res
                            },
                        );
                        env.storage().persistent().update(
                            &DataKey::lastBet(bet.clone().Setting),
                            |old: Option<LastB>| {
                                let mut res = old.unwrap_or(LastB {
                                    id: lastBet.id,
                                    lastBet: BetKey::Team_local,
                                });
                                res.lastBet = bet.clone().bet;
                                res
                            },
                        );
                        if active == false {
                            Self::select_summiter(env.clone(), publicBet.gameid)
                        }
                    }
                }
            }
        }
    }
    //admin address
    pub fn set_game(env: Env, game: Game, signature: BytesN<64>, pub_key: BytesN<32>) {
        let (exist, startTime, endTime, summiter, checkers, _) =
            Self::existBet(env.clone(), game.clone().id);
        if exist {
            panic!("Game have been set already");
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
    }
    fn set_private_bet(env: Env, user: Address, privateData: PrivateBet, game: Game) {
        user.require_auth();

        let (exist, startTime, endTime, summiter, checkers, _) =
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
                active: false,
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
    fn existBet(env: Env, game_id: i128) -> (bool, u32, u32, Address, Vec<Address>, bool) {
        let mut check: bool = false;
        let receiveGame = env
            .storage()
            .persistent()
            .get(&DataKey::Game(game_id))
            .unwrap_or(Game {
                id: 0,
                active: false,
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
            receiveGame.active,
        )
    }
    pub fn summitResult(env: Env, user: Address, result: ResultGame) -> ResultGame {
        user.require_auth();
        let (exist, startTime, endTime, summiter, checkers, active) =
            Self::existBet(env.clone(), result.clone().gameid);
        if !exist {
            panic!("Game haven't been set yet");
        }

        if endTime > env.ledger().timestamp() as u32 {
            panic!("Game hasn't ended yet");
        }
        if endTime + (1 * 60 * 60) < env.ledger().timestamp() as u32 {
            let receivedResult: ResultGame = env
                .storage()
                .persistent()
                .get(&DataKey::Result(result.clone().gameid))
                .unwrap_or(ResultGame {
                    id: 0,
                    gameid: result.clone().gameid,
                    description: String::from_str(&env, ""),
                    result: BetKey::Team_local,
                    pause: false,
                });
            if receivedResult.id == 0 {
                env.storage().persistent().update(
                    &DataKey::Game(result.clone().gameid),
                    |maybe_game: Option<Game>| {
                        let mut game = maybe_game.unwrap_or(Game {
                            id: 0,
                            active: true,
                            league: 1,
                            description: String::from_str(&env, ""),
                            team_local: 0,
                            team_away: 0,
                            startTime: 0,
                            endTime: 0,
                            summiter: user.clone(),
                            Checker: Vec::new(&env),
                        });
                        if game.Checker.len() == 0 {
                            panic!("No checkers available to take over as summiter");
                        } else {
                            let newSummiter = game.Checker.get(0);
                            game.summiter = newSummiter.unwrap().clone();
                            game.Checker.remove(0);
                        }

                        game
                    },
                );
                let history: i128 = env
                    .storage()
                    .persistent()
                    .get(&DataKey::History_Summiter(summiter.clone()))
                    .unwrap_or(0);
                let total_history = history - 100;
                env.storage()
                    .persistent()
                    .set(&DataKey::History_Summiter(summiter.clone()), &total_history);

                env.storage()
                    .persistent()
                    .set(&DataKey::Fine(result.clone().gameid), &20);
                if endTime + (2 * 60 * 60) < env.ledger().timestamp() as u32 {
                    Self::money_back(env.clone(), result.clone().gameid);
                    panic!("You are not allowed to summit this result anymore");
                } else {
                    if summiter != user {
                        panic!("You are not allowed to summit this result");
                    }
                    env.storage()
                        .persistent()
                        .set(&DataKey::Result(result.clone().gameid), &result);
                }
            }
        } else {
            if summiter != user {
                panic!("You are not allowed to summit this result");
            }

            env.storage()
                .persistent()
                .set(&DataKey::Result(result.clone().gameid), &result);
        }

        /* we need to limit the time summiter send results
        and also in case a summiter havent send the result
         within the widows 1 of the checker will be in
         charge if not then the supreme court
         */

        result
    }
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
    pub fn assessResult(env: Env, user: Address, bet: Bet, game_id: i128, desition: AssessmentKey) {
        user.require_auth();
        let (exist, startTime, endTime, summiter, checkers, _) =
            Self::existBet(env.clone(), game_id.clone());
        if !exist {
            panic!("Game haven't been set yet");
        }
        if endTime > env.ledger().timestamp() as u32 {
            panic!("Game hasn't ended yet");
        }
        if endTime + (5 * 60 * 60) < env.ledger().timestamp() as u32 {
            panic!("Game assessment period has ended");
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
    pub fn execute_distribution(env: Env, gameId: i128) {
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
        if complain == 0 {
            dishonest = assesments.UsersApprove.clone();
            s_honest = assesments.CheckReject.clone();
            s_dishonest = assesments.CheckApprove.clone();
            s_dishonest.push_back(game.summiter.clone());
            honest = assesments.UsersReject.clone();
        }
        if complain == 1 {
            // if the complain was incorrect
            dishonest = assesments.UsersReject.clone();
            s_honest = assesments.CheckApprove.clone();
            s_honest.push_back(game.summiter.clone());
            s_dishonest = assesments.CheckReject.clone();
            honest = assesments.UsersApprove.clone();
        }
        if complain == 2 {
            // if the complain was not made
            honest = assesments.UsersApprove.clone();
            dishonest = assesments.UsersReject.clone();
            s_dishonest = assesments.CheckReject.clone();
            s_honest = assesments.CheckApprove.clone();
            s_honest.push_back(game.summiter.clone());
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
        if complain == 0 {
            s_honest = assesments.CheckReject.clone();
            s_dishonest = assesments.CheckApprove.clone();
            s_dishonest.push_back(game.summiter.clone());
        }
        if complain == 1 {
            // if the complain was incorrect
            s_honest = assesments.CheckApprove.clone();
            s_honest.push_back(game.summiter.clone());
            s_dishonest = assesments.CheckReject.clone();
        }
        if complain == 2 {
            // if the complain was not made
            s_dishonest = assesments.CheckReject.clone();
            s_honest = assesments.CheckApprove.clone();
            s_honest.push_back(game.summiter.clone());
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
}
mod test;
