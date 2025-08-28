#![no_std]
use core::{f32::consts::E, panic, result};

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, vec,
    xdr::{ScVal, ToXdr, WriteXdr},
    Address, Bytes, BytesN, Env, IntoVal, String, Symbol, Vec,
};

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
#[derive(Clone)]
struct ResultGame {
    id: i128,
    gameid: i128,
    description: String,
    result: BetKey,
    pause: bool,
}
#[contracttype]
#[derive(Clone)]
struct ResultAssessment {
    id: i128,
    gameid: i128,
    CheckApprove: Vec<Address>,
    CheckReject: Vec<Address>,
    UsersApprove: Vec<Address>,
    UsersReject: Vec<Address>,
}
#[contracttype]
#[derive(Clone)]
struct PrivateBet {
    id: i128,
    gameid: i128,
    active: bool,
    description: String,
    amount_bet_min: i128,
    users_invated: Vec<Address>,
}
#[contracttype]
#[derive(Clone)]
struct LastB {
    id: i128,
    lastBet: BetKey,
}
#[contracttype]
#[derive(Clone)]
struct PublicBet {
    id: i128,
    gameid: i128,
    active: bool,
    description: String,
}
#[contracttype]
#[derive(Clone)]
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
#[derive(Clone)]
#[contracttype]
pub enum AssessmentKey {
    approve,
    reject,
}
#[derive(Clone)]
#[contracttype]
pub enum BetType {
    Public,
    Private,
}
#[derive(Clone)]
#[contracttype]
pub enum ClaimType {
    Summiter,
    Protocol,
    User,
}
#[contracttype]
#[derive(Clone)]
struct Summiter {
    user: Address,
    stakeAmount: i128,
    gameId: i128,
}
const ADMIN_KEY: Symbol = Symbol::short("ADMIN");
const TOKEN_USD_KEY: Symbol = Symbol::short("TOKEN_USD");
const TOKEN_TRUST_KEY: Symbol = Symbol::short("TK_TRUST");
const LEADERBOARD: Symbol = symbol_short!("LB");
const SUMITTERS_HISTORY: Symbol = symbol_short!("H_S");
const COUNTER: Symbol = symbol_short!("COUNTER");
const x: Symbol = symbol_short!("x");
const DUMMYUSSER: Address = Address::from_string(&String::from_str(
    &env,
    "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
));
