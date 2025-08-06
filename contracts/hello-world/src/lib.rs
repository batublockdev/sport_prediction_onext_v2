#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol, Vec,
};

const LEADERBOARD: Symbol = symbol_short!("LB");
const SUMITTERS_HISTORY: Symbol = symbol_short!("H_S");
const COUNTER: Symbol = symbol_short!("COUNTER");

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
struct Game {
    id: i128,
    name: String,
    description: String,
    summiters: Vec<Address>,
}

#[contracttype]
pub enum RecordGame {
    Game(i128),
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
            .get(&LEADERBOARD)
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
        env.storage().persistent().set(&LEADERBOARD, &leaderboard);
        env.events()
            .publish((COUNTER, symbol_short!("increment")), leaderboard);

        true
    }
}

mod test;
