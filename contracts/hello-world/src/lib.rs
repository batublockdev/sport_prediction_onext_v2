#![no_std]
use core::{f32::consts::E, panic};

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
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
struct ResultGame {
    id: i128,
    gameid: i128,
    description: String,
    team_local_score: i128,
    team_away_score: i128,
}
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
struct Privatebet {
    id: i128,
    gameid: i128,
    description: String,
    amount_bet_min: i128,
    users_invated: Vec<Address>,
}
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
struct Bet {
    id: i128,
    gameid: i128,
    bet_type: BetKey,
}

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Game(i128),
    PrivateBet(i128),
}
#[derive(Clone)]
#[contracttype]
pub enum BetKey {
    Team_local,
    Team_away,
    Draw,
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
        let history: i128 = env
            .storage()
            .persistent()
            .get(&SUMITTERS_HISTORY)
            .unwrap_or(10);

        // ✅ Weighted score calculation
        let new_score = (history * 70 + stakeAmount * 30) / 100;

        // ✅ Leaderboard vector
        let mut leaderboard: Vec<(Address, i128)> = env
            .storage()
            .persistent()
            .get(&DataKey::Game(gameId))
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
            .set(&DataKey::Game(gameId), &leaderboard);
        env.events()
            .publish((COUNTER, symbol_short!("increment")), leaderboard);

        true
    }
    pub fn select_summiter(env: Env, game_id: i128) -> Vec<(Address, i128)> {
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

        for _ in 0..5 {
            let pick = rng % top.len();
            let (addr, score) = top.get(pick).unwrap();
            selected.push_back((addr.clone(), score));
            top.remove(pick); // remove picked
            if top.len() == 0 {
                break;
            }
        }

        selected
    }
    pub fn bet(env: Env, user: Address, bet: Bet, amount_bet: i128) -> bool {
        user.require_auth();
        /*
        gotta take the game info to stop users from bettin when the game has started
         */
        let counter: i128 = env.storage().persistent().get(&COUNTER).unwrap_or(0);
        let new_bet = privatebet {
            id: counter + 1,
            gameid: gameId,
            description: String::from_slice(&env, "private bet"),
            amount_bet_min: amount_bet_min,
            users_invated: users_invated,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Game(new_bet.id), &new_bet);
        env.storage().persistent().set(&COUNTER, &(counter + 1));
        true
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
    }
    fn set_private_bet(
        env: Env,
        user: Address,
        privateData: Privatebet,
        game: Game,
        signature: BytesN<64>,
        pub_key: BytesN<32>,
    ) {
        user.require_auth();
        if !(Self::existBet(env.clone(), game.clone().id)) {
            panic!("Game haven't been set yet");
        }
        Self::set_game(env.clone(), &game, signature, pub_key);
        if (privateData.id == 0 || privateData.gameid != game.clone().id) {
            panic!("Invalid private bet data");
        }
        env.storage()
            .persistent()
            .set(&DataKey::PrivateBet(privateData.id), &privateData);
    }
    fn existBet(env: Env, game_id: i128) -> bool {
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
            });

        if receiveGame.id == 0 {
            check = false;
        } else {
            check = true;
        }
        check
    }
}
mod test;
