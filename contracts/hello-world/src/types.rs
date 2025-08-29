#![no_std]
use core::{f32::consts::E, panic, result};

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, vec,
    xdr::{ScVal, ToXdr, WriteXdr},
    Address, Bytes, BytesN, Env, IntoVal, String, Symbol, Vec,
};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game {
    pub id: i128,
    pub active: bool,
    pub league: i128,
    pub description: String,
    pub team_local: i128,
    pub team_away: i128,
    pub startTime: u32,
    pub endTime: u32,
    pub summiter: Address,
    pub Checker: Vec<Address>,
}

#[contracttype]
#[derive(Clone)]
pub struct ResultGame {
    pub id: i128,
    pub gameid: i128,
    pub description: String,
    pub result: BetKey,
    pub pause: bool,
}
#[contracttype]
#[derive(Clone)]
pub struct ResultAssessment {
    pub id: i128,
    pub gameid: i128,
    pub CheckApprove: Vec<Address>,
    pub CheckReject: Vec<Address>,
    pub UsersApprove: Vec<Address>,
    pub UsersReject: Vec<Address>,
}
#[contracttype]
#[derive(Clone)]
pub struct PrivateBet {
    pub id: i128,
    pub gameid: i128,
    pub active: bool,
    pub description: String,
    pub amount_bet_min: i128,
    pub users_invated: Vec<Address>,
}
#[contracttype]
#[derive(Clone)]
pub struct LastB {
    pub id: i128,
    pub lastBet: BetKey,
}
#[contracttype]
#[derive(Clone)]
pub struct PublicBet {
    pub id: i128,
    pub gameid: i128,
    pub active: bool,
    pub description: String,
}
#[contracttype]
#[derive(Clone)]
pub struct Bet {
    pub id: i128,
    pub gameid: i128,
    pub betType: BetType,
    pub Setting: i128,
    pub bet: BetKey,
    pub amount_bet: i128,
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
