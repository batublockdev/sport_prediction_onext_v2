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
    description: String,
    amount_bet_min: i128,
    users_invated: Vec<Address>,
}
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
struct PublicBet {
    id: i128,
    gameid: i128,
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
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Game(i128),
    Result(i128),
    ResultAssessment(i128),
    GameSummiters(i128),
    History(Address),
    SetPrivateBet(i128),
    SetPublicBet(i128),
    Bet(Address, i128),
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

        // ✅ Get history map (user → score history)
        let history: i128 = env.storage().persistent().get(History(&user)).unwrap_or(10);

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
            .get(&DataKey::Game(game_id))
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
            selected.push_back((addr.clone(), score));
            if k == 0 {
                env.storage()
                    .persistent()
                    .update(&DataKey::Game(game_id), |game: &mut Game| {
                        game.summiter = addr.clone();
                    });
            }
            if k > 0 {
                env.storage()
                    .persistent()
                    .update(&DataKey::Game(game_id), |game: &mut Game| {
                        if !game.Checker.contains(&addr) {
                            game.Checker.push(addr.clone());
                        }
                    });
            }
            top.remove(pick); // remove picked
            if top.len() == 0 {
                break;
            }
        }

        selected
    }
    pub fn bet(env: Env, user: Address, bet: Bet, amount_bet: i128) -> bool {
        user.require_auth();
        if amount_bet <= 0 {
            panic!("Bet amount must be greater than 0");
        }
        if bet.clone().id == 0 || bet.clone().Setting == 0 {
            panic!("Invalid bet data");
        }
        if (bet.clone().betType == BetType::Private) {
            let privateBet: PrivateBet = env
                .storage()
                .persistent()
                .get(&DataKey::SetPrivateBet(bet.clone().Setting))
                .unwrap_or(PrivateBet {
                    id: 0,
                    gameid: 0,
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
            if amount_bet < privateBet.clone().amount_bet_min {
                panic!("Bet amount is less than minimum required");
            }
            let (exist, startTime, endTime) =
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
            env.storage()
                .persistent()
                .set(&DataKey::Bet(user.clone(), bet.clone().id), &bet);
        } else if (bet.clone().betType == BetType::Public) {
            let publicBet: PublicBet = env
                .storage()
                .persistent()
                .get(&DataKey::SetPublicBet(bet.clone().Setting))
                .unwrap_or(PublicBet {
                    id: 0,
                    gameid: 0,
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
            env.storage()
                .persistent()
                .set(&DataKey::Bet(user.clone(), bet.clone().id), &bet);
        }
    }
    //admin address
    fn set_game(env: Env, game: &Game, signature: BytesN<64>, pub_key: BytesN<32>) {
        if Self::existBet(env.clone(), game.id) {
            panic!("Game already exists");
        }
        let encoded = game.clone().to_xdr(&env);
        env.crypto().ed25519_verify(&pub_key, &encoded, &signature);
        env.storage()
            .persistent()
            .set(&DataKey::Game(game.id), game);
        let pubSetting = PublicBet {
            id: game.id,
            gameid: game.id,
            description: String::from_slice(&env, "Public Bet"),
        };
        env.storage()
            .persistent()
            .set(&DataKey::SetPublicBet(game.id), &pubSetting);
        select_summiter(env.clone(), game.id);
    }
    fn set_private_bet(env: Env, user: Address, privateData: PrivateBet, game: Game) {
        user.require_auth();
        if !(Self::existBet(env.clone(), game.clone().id)) {
            panic!("Game haven't been set yet");
        }
        if (privateData.id == 0 || privateData.gameid != game.clone().id) {
            panic!("Invalid private bet data");
        }
        env.storage()
            .persistent()
            .set(&DataKey::SetPrivateBet(privateData.id), &privateData);
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
                summiter: Address::default(),
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
    fn summitResult(env: Env, user: Address, result: ResultGame) -> ResultGame {
        user.require_auth();
        let (exist, startTime, endTime, summiter, checkers) =
            Self::existBet(env.clone(), result.clone().gameid);
        if !exist {
            panic!("Game haven't been set yet");
        }
        if user != summiter {
            panic!("You are not the summiter for this game");
        }
        if endTime > env.ledger().timestamp() as u32 {
            panic!("Game hasn't ended yet");
        }

        env.storage()
            .persistent()
            .set(&DataKey::Result(game_id), &result);
        result
    }
    fn assessResult(
        env: Env,
        user: Address,
        bet: Bet,
        game_id: i128,
        desition: AssessmentKey,
    ) -> ResultAssessment {
        user.require_auth();
        let (exist, startTime, endTime, summiter, checkers) =
            Self::existBet(env.clone(), game_id.clone());
        if !exist {
            panic!("Game haven't been set yet");
        }
        if endTime > env.ledger().timestamp() as u32 {
            panic!("Game hasn't ended yet");
        }
        let resultAssessment: ResultAssessment = env
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
            env.storage()
                .persistent()
                .get(&DataKey::Bet(user.clone(), bet.clone().id))
                .unwrap_or_else(|| panic!("You are not allowed to assess this result"));
            if resultAssessment.UsersApprove.contains(&user)
                || resultAssessment.UsersReject.contains(&user)
            {
                panic!("You have already assessed this result");
            }
            if resultAssessment.id == 0 {
                if desition == AssessmentKey::approve {
                    resultAssessment.UsersApprove.push(user.clone());
                } else if desition == AssessmentKey::reject {
                    resultAssessment.UsersReject.push(user.clone());
                }
                resultAssessment.id = game_id;
                env.storage().persistent().set(
                    &DataKey::ResultAssessment(resultAssessment.clone().gameid),
                    &resultAssessment,
                );
            } else {
                if desition == AssessmentKey::approve {
                    resultAssessment.UsersApprove.push(user.clone());
                } else if desition == AssessmentKey::reject {
                    resultAssessment.UsersReject.push(user.clone());
                }
                env.storage().persistent().update(
                    &DataKey::ResultAssessment(resultAssessment.clone().gameid),
                    |res: &mut ResultAssessment| {
                        res.UsersApprove = resultAssessment.UsersApprove.clone();
                        res.UsersReject = resultAssessment.UsersReject.clone();
                    },
                );
            }
        } else {
            if resultAssessment.CheckApprove.contains(&user)
                || resultAssessment.CheckReject.contains(&user)
            {
                panic!("You have already assessed this result");
            }
            if resultAssessment.id == 0 {
                if desition == AssessmentKey::approve {
                    resultAssessment.CheckApprove.push(user.clone());
                } else if desition == AssessmentKey::reject {
                    resultAssessment.CheckReject.push(user.clone());
                }
                resultAssessment.id = game_id;
                env.storage().persistent().set(
                    &DataKey::ResultAssessment(resultAssessment.clone().gameid),
                    &resultAssessment,
                );
            } else {
                if desition == AssessmentKey::approve {
                    resultAssessment.CheckApprove.push(user.clone());
                } else if desition == AssessmentKey::reject {
                    resultAssessment.CheckReject.push(user.clone());
                }
                env.storage().persistent().update(
                    &DataKey::ResultAssessment(resultAssessment.clone().gameid),
                    |res: &mut ResultAssessment| {
                        res.CheckApprove = resultAssessment.CheckApprove.clone();
                        res.CheckReject = resultAssessment.CheckReject.clone();
                    },
                );
            }
        }

        env.storage().persistent().set(
            &DataKey::ResultAssessment(resultAssessment.clone().gameid),
            &resultAssessment,
        );
        resultAssessment
    }

    fn withdraw(env: Env, user: Address, bet: Bet) -> bool {
        user.require_auth();
        let result: ResultGame = env
            .storage()
            .persistent()
            .get(&DataKey::Result(bet.clone().gameid))
            .unwrap_or_else(|| panic!("No result found for this game"));
        if result.pause {
            panic!("Game result is paused, you cannot withdraw now");
        }
        let betData: Bet = env
            .storage()
            .persistent()
            .get(&DataKey::Bet(user.clone(), bet.clone().id))
            .unwrap_or_else(|| panic!("No bet found for this user"));

        if betData.bet != result.result {
            false
        } else {
            true
        }
    }
}
mod test;
