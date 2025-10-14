#![no_std]

use soroban_sdk::{contractclient, Address, BytesN, Env, String};

use crate::types::{AssessmentKey, Bet, BetKey, ClaimType, Game, PrivateBet, ResultGame};

#[contractclient(name = "BettingClient")]
pub trait betting {
    fn __constructor(
        env: Env,
        admin: Address,
        admin_pubkey: BytesN<32>,
        token_usd: Address,
        token_trust: Address,
        supreme_court: Address,
    );
    fn request_result_summiter(env: Env, user: Address, stakeAmount: i128) -> bool;
    fn bet(env: Env, user: Address, bet: Bet) -> bool;
    fn claim_refund(env: Env, user: Address, setting: i128) -> i128;
    fn set_game(env: Env, game: Game, signature: BytesN<64>) -> bool;
    fn set_private_bet(env: Env, user: Address, privateData: PrivateBet, game_id: i128) -> bool;
    fn add_user_privateBet(env: Env, setting: i128, game: i128, newUser: Address) -> bool;
    fn summitResult(env: Env, user: Address, result: ResultGame) -> bool;
    fn assessResult(
        env: Env,
        user: Address,
        setting: i128,
        game_id: i128,
        desition: AssessmentKey,
    ) -> bool;
    fn claim(env: Env, user: Address, typeClaim: ClaimType, setting: i128) -> (i128, i128);
    fn setResult_supremCourt(env: Env, result: ResultGame) -> bool;
    fn execute_distribution(env: Env, gameId: i128) -> bool;
    fn set_stakeAmount(env: Env, user: Address, amount: i128) -> bool;
}
