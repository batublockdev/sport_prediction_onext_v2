#![no_std]
use core::{f32::consts::E, panic, result};

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, vec,
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
    ClaimProtocol,
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
#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum ClaimType {
    Summiter,
    Protocol,
    User,
}
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
struct Summiter {
    user: Address,
    stakeAmount: i128,
    gameId: i128,
}
const ADMIN_KEY: Symbol = Symbol::short("ADMIN");
const TOKEN_USD_KEY: Symbol = Symbol::short("TOKEN_USD");
const TOKEN_TRUST_KEY: Symbol = Symbol::short("TK_TRUST");

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn has_init(env: &Env) -> bool {
        env.storage().instance().has(&ADMIN_KEY)
    }
    pub fn init(env: Env, admin: Address, token_usd: Address, token_trust: Address) {
        // save the admin
        env.storage().instance().set(&ADMIN_KEY, &admin);
        // save the token addresses
        env.storage().instance().set(&TOKEN_USD_KEY, &token_usd);
        env.storage().instance().set(&TOKEN_TRUST_KEY, &token_trust);
    }
    pub fn get_usd(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&TOKEN_USD_KEY)
            .unwrap_or_else(|| panic!("contract not initialized"))
    }
    pub fn get_trust(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&TOKEN_TRUST_KEY)
            .unwrap_or_else(|| panic!("contract not initialized"))
    }
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&ADMIN_KEY)
            .unwrap_or_else(|| panic!("contract not initialized"))
    }
    pub fn get_history(env: Env, user: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::History_Summiter(user.clone()))
            .unwrap_or(0)
    }
    pub fn get_leaderboard(env: Env, gameId: i128) -> Vec<(Address, i128)> {
        env.storage()
            .persistent()
            .get(&DataKey::GameSummiters(gameId))
            .unwrap_or(Vec::new(&env))
    }
    pub fn set_leaderboard(env: Env, leaderboard: Vec<(Address, i128)>) -> bool {
        env.storage()
            .persistent()
            .set(&DataKey::GameSummiters(gameId), &leaderboard);
        true
    }
    pub fn update_game(env: Env, gameId: i128, summiter: Address, Active: bool) {
        env.storage()
            .persistent()
            .update(&DataKey::Game(gameId), |maybe_game: Option<Game>| {
                let mut game = maybe_game.unwrap_or(Game {
                    id: gameId,
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
                game.Checker = checks.clone();

                game
            });
    }
    pub fn add_privateSettingList(env: Env, gameId: i128, setting: i128) {
        {
            let mut listedBet: Vec<(i128)> = env
                .storage()
                .persistent()
                .get(&DataKey::PrivateBetList(bet.clone().gameid))
                .unwrap_or(Vec::new(&env));
            listedBet.push_back(setting);
            env.storage()
                .persistent()
                .set(&DataKey::PrivateBetList(bet.clone().gameid), &listedBet);
        }

        pub fn add_bet(env: Env, user: Address, bet: Bet) {
            env.storage()
                .persistent()
                .set(&DataKey::Bet(user.clone(), bet.clone().Setting), &bet);
        }
        pub fn does_bet_active(env: Env, setting: i128) -> bool {
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
                return false;
            }
            if lastBet.lastBet != bet.clone().bet {
                return true;
            }
        }
        pub fn active_private_setting(env: Env, user: Address, setting: i128) {
            env.storage().persistent().update(
                &DataKey::SetPrivateBet(setting),
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
        }
        pub fn get_PublicBet(env: Env, user: Address, setting: i128) -> PublicBet {
            let publicBet: PublicBet = env
                .storage()
                .persistent()
                .get(&DataKey::SetPublicBet(setting))
                .unwrap_or(PublicBet {
                    id: 0,
                    gameid: 0,
                    active: false,
                    description: String::from_slice(&env, "No public bet found"),
                });
        }
        pub fn get_PrivateBet(env: Env, user: Address, setting: i128) -> PrivateBet {
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
            pub fn add_listUsuers(env: Env, gameid: i128, user: Address) -> Vec<(Address)> {
                let mut listedBetAddress: Vec<(Address)> = env
                    .storage()
                    .persistent()
                    .get(&DataKey::ListBetUser(gameid))
                    .unwrap_or(Vec::new(&env));

                listedBetAddress.push_back(user.clone());
                env.storage()
                    .persistent()
                    .set(&DataKey::ListBetUser(gameid), &listedBetAddress);
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
    pub fn set_private_bet(env: Env, user: Address, privateData: PrivateBet, game: Game) {
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
        if result.clone().gameid == 0 || result.clone().id == 0 {
            panic!("Invalid result data");
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
                            let newSummiter = Address::from_string(&String::from_str(
                                &env,
                                "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
                            ));
                            game.summiter = newSummiter.clone();
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
                    if !checkers.contains(&user) {
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
    fn moveToken(env: &Env, token: &Address, from: &Address, to: &Address, amount: &i128) {
        let token = token::Client::new(env, token);
        token.transfer(from, &to, amount);
    }
}
mod test;
