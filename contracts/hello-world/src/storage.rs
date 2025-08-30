use crate::types::{
    AssessmentKey, Bet, BetKey, BetType, ClaimType, DataKey, Game, LastB, PrivateBet, PublicBet,
    ResultAssessment, ResultGame,
};
use soroban_sdk::{symbol_short, Address, Env, String, Symbol, Vec};
const ADMIN_KEY: Symbol = Symbol::short("ADMIN");
const TOKEN_USD_KEY: Symbol = Symbol::short("TOKEN_USD");
const TOKEN_TRUST_KEY: Symbol = Symbol::short("TK_TRUST");
const LEADERBOARD: Symbol = symbol_short!("LB");
const SUMITTERS_HISTORY: Symbol = symbol_short!("H_S");
const COUNTER: Symbol = symbol_short!("COUNTER");
const x: Symbol = symbol_short!("x");
pub fn get_dummyusser(env: &Env) -> Address {
    Address::from_string(&String::from_str(
        env,
        "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
    ))
}

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
pub fn set_history(env: Env, user: Address, points: i128) {
    let history: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::History_Summiter(user.clone()))
        .unwrap_or(0);
    let total_history = history + points;
    env.storage()
        .persistent()
        .set(&DataKey::History_Summiter(user.clone()), &total_history);
}
pub fn get_leaderboard(env: Env, gameId: i128) -> Vec<(Address, i128)> {
    env.storage()
        .persistent()
        .get(&DataKey::GameSummiters(gameId))
        .unwrap_or(Vec::new(&env))
}
pub fn set_leaderboard(env: Env, gameId: i128, leaderboard: Vec<(Address, i128)>) -> bool {
    env.storage()
        .persistent()
        .set(&DataKey::GameSummiters(gameId), &leaderboard);
    true
}
pub fn update_game(env: Env, gameId: i128, summiter: Address, checkers: Vec<(Address)>) {
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
                summiter: summiter.clone(),
                Checker: Vec::new(&env),
            });
            game.summiter = summiter.clone();
            game.active = true;
            game.Checker = checkers.clone();

            game
        });
}
pub fn add_privateSettingList(env: Env, gameId: i128, setting: i128) {
    let mut listedBet: Vec<(i128)> = env
        .storage()
        .persistent()
        .get(&DataKey::PrivateBetList(gameId))
        .unwrap_or(Vec::new(&env));
    listedBet.push_back(setting);
    env.storage()
        .persistent()
        .set(&DataKey::PrivateBetList(gameId), &listedBet);
}
pub fn get_privateSettingList(env: Env, gameId: i128, setting: i128) -> Vec<(i128)> {
    let list: Vec<(i128)> = env
        .storage()
        .persistent()
        .get(&DataKey::PrivateBetList(gameId))
        .unwrap_or(Vec::new(&env));
    list
}

pub fn add_bet(env: Env, user: Address, bet: Bet) {
    env.storage()
        .persistent()
        .set(&DataKey::Bet(user.clone(), bet.clone().Setting), &bet);
}
pub fn does_bet_active(env: Env, bet: Bet) -> bool {
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
    } else {
        return false;
    }
}
pub fn active_private_setting(env: Env, user: Address, setting: i128) {
    env.storage().persistent().update(
        &DataKey::SetPrivateBet(setting),
        |old: Option<PrivateBet>| {
            let mut res = old.unwrap_or(PrivateBet {
                id: 0,
                gameid: 0,
                active: false,
                description: String::from_slice(&env, "No private bet found"),
                amount_bet_min: 0,
                users_invated: Vec::new(&env),
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
    publicBet
}
pub fn get_PrivateBet(env: Env, setting: i128) -> PrivateBet {
    env.storage()
        .persistent()
        .get(&DataKey::SetPrivateBet(setting))
        .unwrap_or(PrivateBet {
            id: 0,
            gameid: 0,
            active: false,
            description: String::from_slice(&env, "No private bet found"),
            amount_bet_min: 0,
            users_invated: Vec::new(&env),
        })
}
pub fn add_listUsuers(env: Env, gameid: i128, user: Address) {
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

pub fn set_game(env: Env, game: Game) {
    env.storage()
        .persistent()
        .set(&DataKey::Game(game.id), &game);
}
pub fn set_privateSetting(env: Env, privateBet: PrivateBet) {
    env.storage()
        .persistent()
        .set(&DataKey::SetPrivateBet(privateBet.id), &privateBet);
}
pub fn set_publicSetting(env: Env, publicBet: PublicBet) {
    env.storage()
        .persistent()
        .set(&DataKey::SetPublicBet(publicBet.id), &publicBet);
}

pub fn get_game(env: Env, game_id: i128) -> Game {
    let game = env
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
    game
}
pub fn add_Fine(env: Env, gameid: i128, fine: i128) {
    let fines: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::Fine(gameid))
        .unwrap_or(0);
    let total_fine = fines + fine;
    env.storage()
        .persistent()
        .set(&DataKey::Fine(gameid), &total_fine);
}
pub fn get_Fine(env: Env, gameid: i128) -> i128 {
    let fine = env
        .storage()
        .persistent()
        .get(&DataKey::Fine(gameid))
        .unwrap_or(0);
    fine
}
pub fn set_ResultGame(env: Env, user: Address, result: ResultGame) {
    env.storage()
        .persistent()
        .set(&DataKey::Result(result.clone().gameid), &result);
}
pub fn get_ResultGame(env: Env, game_id: i128) -> ResultGame {
    let result = env
        .storage()
        .persistent()
        .get(&DataKey::Result(game_id))
        .unwrap_or(ResultGame {
            id: 0,
            gameid: 0,
            description: String::from_slice(&env, "No result found"),
            result: BetKey::Team_local,
            pause: false,
        });
    result
}
pub fn puase_ResultGame(env: Env, game_id: i128, pause: bool) {
    env.storage()
        .persistent()
        .update(&DataKey::Result(game_id), |old: Option<ResultGame>| {
            let mut res = old.unwrap_or(ResultGame {
                id: 0,
                gameid: 0,
                description: String::from_str(&env, ""),
                result: BetKey::Team_local,
                pause: false,
            });
            res.pause = pause;
            res
        });
}
pub fn get_ListBetUser(env: Env, gameid: i128) -> Vec<(Address)> {
    env.storage()
        .persistent()
        .get(&DataKey::ListBetUser(gameid))
        .unwrap_or(Vec::new(&env))
}
pub fn get_Bet(env: Env, user: Address, setting: i128) -> Bet {
    env.storage()
        .persistent()
        .get(&DataKey::Bet(user.clone(), setting))
        .unwrap_or(Bet {
            id: 0,
            gameid: 0,
            betType: BetType::Public,
            Setting: 0,
            bet: BetKey::Team_local,
            amount_bet: 0,
        })
}
pub fn get_ClaimWinner(env: Env, user: Address) -> i128 {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::ClaimWinner(user.clone()))
        .unwrap_or(0);
    amount
}
pub fn zero_ClaimWinner(env: Env, user: Address) {
    env.storage()
        .persistent()
        .set(&DataKey::ClaimWinner(user.clone()), &0);
}
pub fn add_ClaimWinner(env: Env, user: Address, newAmount: i128) {
    let money: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::ClaimWinner(user.clone()))
        .unwrap_or(0);
    let total_money = money + newAmount;
    env.storage()
        .persistent()
        .set(&DataKey::ClaimWinner(user.clone()), &total_money);
}
// summitter
pub fn get_ClaimSummiter(env: Env, user: Address) -> i128 {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::ClaimSummiter(user.clone()))
        .unwrap_or(0);
    amount
}
pub fn zero_ClaimSummiter(env: Env, user: Address) {
    env.storage()
        .persistent()
        .set(&DataKey::ClaimSummiter(user.clone()), &0);
}
pub fn add_ClaimSummiter(env: Env, user: Address, newAmount: i128) {
    let money: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::ClaimSummiter(user.clone()))
        .unwrap_or(0);
    let total_money = money + newAmount;
    env.storage()
        .persistent()
        .set(&DataKey::ClaimSummiter(user.clone()), &total_money);
}
// protocol
pub fn get_ClaimProtocol(env: Env) -> i128 {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::ClaimProtocol)
        .unwrap_or(0);
    amount
}
pub fn zero_ClaimProtocol(env: Env) {
    env.storage().persistent().set(&DataKey::ClaimProtocol, &0);
}
pub fn add_ClaimProtocol(env: Env, newAmount: i128) {
    let money: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::ClaimProtocol)
        .unwrap_or(0);
    let total_money = money + newAmount;
    env.storage()
        .persistent()
        .set(&DataKey::ClaimProtocol, &total_money);
}
pub fn get_ResultAssessment(env: Env, game_id: i128) -> ResultAssessment {
    env.storage()
        .persistent()
        .get(&DataKey::ResultAssessment(game_id))
        .unwrap_or(ResultAssessment {
            id: 0,
            gameid: game_id,
            CheckApprove: Vec::new(&env),
            CheckReject: Vec::new(&env),
            UsersApprove: Vec::new(&env),
            UsersReject: Vec::new(&env),
        })
}
pub fn set_ResultAssessment(env: Env, gameid: i128, data: ResultAssessment) {
    env.storage()
        .persistent()
        .set(&DataKey::ResultAssessment(gameid), &data);
}
pub fn CheckUser(env: Env, user: Address, game_id: i128) -> bool {
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
pub fn existBet(env: Env, game_id: i128) -> (bool, u32, u32, Address, Vec<Address>, bool) {
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
