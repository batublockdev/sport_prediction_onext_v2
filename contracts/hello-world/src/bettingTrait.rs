#![no_std]

use soroban_sdk::{contractclient, Address, BytesN, Env, String};

use crate::types::{AssessmentKey, Bet, BetKey, ClaimType, Game, PrivateBet, ResultGame};

#[contractclient(name = "BettingClient")]
pub trait betting {
    fn __constructor(env: Env, admin: Address, token_usd: Address, token_trust: Address);
    fn request_result_summiter(env: Env, user: Address, stakeAmount: i128, gameId: i128) -> bool;
    fn bet(env: Env, user: Address, bet: Bet);
    fn claim_money_noactive(env: Env, user: Address, setting: i128);
    fn set_game(env: Env, game: Game, signature: BytesN<64>, pub_key: BytesN<32>);
    fn set_private_bet(env: Env, user: Address, privateData: PrivateBet, game_id: i128);
    fn summitResult(env: Env, user: Address, result: ResultGame) -> ResultGame;
    fn assessResult(env: Env, user: Address, bet: Bet, game_id: i128, desition: AssessmentKey);
    fn claim(env: Env, user: Address, typeClaim: ClaimType, setting: i128);
    fn setResult_supremCourt(env: Env, user: Address, result: ResultGame);
    fn execute_distribution(env: Env, gameId: i128);
}
