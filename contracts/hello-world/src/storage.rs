use crate::types::{
    AssessmentKey, Bet, BetKey, BetType, ClaimType, DataKey, Game, LastB, PrivateBet, PublicBet,
    ResultAssessment, ResultGame,
};
use soroban_sdk::{symbol_short, Address, Env, String, Symbol, Vec};
const ADMIN_KEY: Symbol = Symbol::short("ADMIN");
const SUPREME_KEY: Symbol = Symbol::short("SUPREME");
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
pub fn init(
    env: Env,
    admin: Address,
    token_usd: Address,
    token_trust: Address,
    supreme_court: Address,
) {
    // save the admin
    env.storage().instance().set(&ADMIN_KEY, &admin);
    // save the token addresses
    env.storage().instance().set(&TOKEN_USD_KEY, &token_usd);
    env.storage().instance().set(&TOKEN_TRUST_KEY, &token_trust);
    env.storage().instance().set(&SUPREME_KEY, &supreme_court);
}
pub fn get_supreme(env: Env) -> Address {
    env.storage()
        .instance()
        .get(&SUPREME_KEY)
        .unwrap_or_else(|| panic!("contract not initialized"))
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
pub fn get_leaderboard(env: Env) -> Vec<(Address, i128)> {
    env.storage()
        .persistent()
        .get(&DataKey::GameSummiters)
        .unwrap_or(Vec::new(&env))
}
pub fn set_leaderboard(env: Env, leaderboard: Vec<(Address, i128)>) -> bool {
    env.storage()
        .persistent()
        .set(&DataKey::GameSummiters, &leaderboard);
    true
}
pub fn set_stakeAmount_user(env: Env, user: Address, stakeAmount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::StakeUserAmount(user), &stakeAmount);
}

pub fn set_stakeAmount_user_game(env: Env, user: Address, game: i128) {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::StakeUserAmount(user.clone()))
        .unwrap_or(0);
    env.storage()
        .persistent()
        .set(&DataKey::StakeUserGameAmount(user, game), &amount);
}
pub fn get_stakeAmount_user_game(env: Env, user: Address, game: i128) -> i128 {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::StakeUserGameAmount(user, game))
        .unwrap_or(0);
    amount
}
pub fn set_Min_stakeAmount(env: Env, stakeAmount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::StakeMinAmount, &stakeAmount);
}
pub fn get_Min_stakeAmount(env: Env) -> i128 {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::StakeMinAmount)
        .unwrap_or(0);
    amount
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

pub fn get_privateSettingList(env: Env, gameId: i128) -> Vec<(i128)> {
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
pub fn get_bet(env: Env, user: Address, setting: i128) -> Bet {
    let bet: Bet = env
        .storage()
        .persistent()
        .get(&DataKey::Bet(user.clone(), setting))
        .unwrap_or_else(|| panic!("No bet found for this user"));
    bet
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
pub fn active_private_setting(env: Env, setting: i128, active: bool) {
    env.storage().persistent().update(
        &DataKey::SetPrivateBet(setting),
        |old: Option<PrivateBet>| {
            let mut res = old.unwrap_or(PrivateBet {
                id: 0,
                gameid: 0,
                active: false,
                settingAdmin: Address::from_string(&String::from_str(
                    &env,
                    "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
                )),
                description: String::from_slice(&env, "No private bet found"),
                amount_bet_min: 0,
                users_invated: Vec::new(&env),
            });
            res.active = active;
            res
        },
    );
}
pub fn active_public_setting(env: Env, setting: i128, active: bool) {
    env.storage()
        .persistent()
        .update(&DataKey::SetPublicBet(setting), |old: Option<PublicBet>| {
            let mut res = old.unwrap_or(PublicBet {
                id: 0,
                gameid: 0,
                active: false,
                description: String::from_slice(&env, "No public bet found"),
            });
            res.active = active;
            res
        });
}
pub fn get_PublicBet(env: Env, setting: i128) -> PublicBet {
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
            settingAdmin: Address::from_string(&String::from_str(
                &env,
                "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
            )),
            description: String::from_slice(&env, "No private bet found"),
            amount_bet_min: 0,
            users_invated: Vec::new(&env),
        })
}

pub fn set_game(env: Env, game: Game) {
    let gameReceive = env
        .storage()
        .persistent()
        .get(&DataKey::Game(game.id))
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
    if gameReceive.id != 0 {
        panic!("Game with this ID already exists");
    }
    env.storage()
        .persistent()
        .set(&DataKey::Game(game.id), &game);
}
pub fn set_privateSetting(env: Env, privateBet: PrivateBet) {
    self::verifySettingId(env.clone(), privateBet.id);
    env.storage()
        .persistent()
        .set(&DataKey::SetPrivateBet(privateBet.id), &privateBet);
}
pub fn add_HonestyPoints(env: Env, user: Address, points: i128) {
    let honesty: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::HonestyPoints(user.clone()))
        .unwrap_or(0);
    let total_honesty = honesty + points;
    env.storage()
        .persistent()
        .set(&DataKey::HonestyPoints(user.clone()), &total_honesty);
}
pub fn get_HonestyPoints(env: Env, user: Address) -> i128 {
    let honesty: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::HonestyPoints(user.clone()))
        .unwrap_or(0);
    honesty
}

pub fn verifySettingId(env: Env, SettingId: i128) {
    let publicBet = env
        .storage()
        .persistent()
        .get(&DataKey::SetPublicBet(SettingId))
        .unwrap_or(PublicBet {
            id: 0,
            gameid: 0,
            active: false,
            description: String::from_slice(&env, "No public bet found"),
        });
    if publicBet.id != 0 {
        panic!("Public setting with this ID already exists");
    }
    let privateBet = env
        .storage()
        .persistent()
        .get(&DataKey::SetPrivateBet(SettingId))
        .unwrap_or(PrivateBet {
            id: 0,
            gameid: 0,
            active: false,
            settingAdmin: Address::from_string(&String::from_str(
                &env,
                "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
            )),
            description: String::from_slice(&env, "No private bet found"),
            amount_bet_min: 0,
            users_invated: Vec::new(&env),
        });
    if privateBet.id != 0 {
        panic!("Private setting with this ID already exists");
    }
}
pub fn set_publicSetting(env: Env, publicBet: PublicBet) {
    self::verifySettingId(env.clone(), publicBet.id);
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
pub fn set_fines_applied(env: Env, gameid: i128, fine: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::FinesApplied(gameid), &fine);
}
pub fn get_fines_applied(env: Env, gameid: i128) -> i128 {
    let fine = env
        .storage()
        .persistent()
        .get(&DataKey::FinesApplied(gameid))
        .unwrap_or(0);
    fine
}
pub fn add_Fine(env: Env, gameid: i128, finex: i128) {
    let fines: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::Fine(gameid))
        .unwrap_or(0);
    let total_fine = fines + finex;
    env.storage()
        .persistent()
        .set(&DataKey::Fine(gameid), &total_fine);
}
pub fn zero_Fine(env: Env, gameid: i128) {
    let zero: i128 = 0;
    env.storage()
        .persistent()
        .set(&DataKey::Fine(gameid), &zero);
}
pub fn get_Fine(env: Env, gameid: i128) -> i128 {
    let fine = env
        .storage()
        .persistent()
        .get(&DataKey::Fine(gameid))
        .unwrap_or(0);
    fine
}
pub fn set_ResultGame(env: Env, result: ResultGame) {
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
            distribution_executed: false,
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
                distribution_executed: false,
            });
            res.pause = pause;
            res
        });
}
pub fn distribution_ResultGame(env: Env, game_id: i128) {
    env.storage()
        .persistent()
        .update(&DataKey::Result(game_id), |old: Option<ResultGame>| {
            let mut res = old.unwrap_or(ResultGame {
                id: 0,
                gameid: 0,
                description: String::from_str(&env, ""),
                result: BetKey::Team_local,
                pause: false,
                distribution_executed: false,
            });
            res.distribution_executed = true;
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

// summitter
pub fn get_ClaimSummiter(env: Env, user: Address) -> i128 {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::ClaimSummiter(user))
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
/// protocol trust
pub fn get_ClaimProtocolTrust(env: Env) -> i128 {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::ClaimProtocolTrust)
        .unwrap_or(0);
    amount
}

pub fn add_ClaimProtocolTrust(env: Env, newAmount: i128) {
    let mut currentAmount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::ClaimProtocolTrust)
        .unwrap_or_else(|| 0);

    currentAmount += newAmount;
    env.storage()
        .persistent()
        .set(&DataKey::ClaimProtocolTrust, &currentAmount);
}
// protocol
///
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
    env.storage()
        .persistent()
        .set(&DataKey::ClaimProtocolTrust, &0);
}
pub fn add_ClaimProtocol(env: Env, newAmount: i128) {
    let mut currentAmount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::ClaimProtocol)
        .unwrap_or_else(|| 0);

    currentAmount += newAmount;
    env.storage()
        .persistent()
        .set(&DataKey::ClaimProtocol, &currentAmount);
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
pub fn add_total_bet(env: Env, game_id: i128, Amount: i128) {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::TotalBet(game_id))
        .unwrap_or(0);
    let Amountx = total_amount + Amount;
    env.storage()
        .persistent()
        .set(&DataKey::TotalBet(game_id), &Amountx);
}

pub fn get_total_bet(env: Env, game_id: i128) -> i128 {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::TotalBet(game_id))
        .unwrap_or(0);
    total_amount
}
pub fn add_not_assesed_yet(env: Env, game_id: i128, Amount: i128, bet: BetKey) {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::NotAssesedYet(game_id, bet.clone()))
        .unwrap_or(0);
    let Amountx = total_amount + Amount;
    env.storage()
        .persistent()
        .set(&DataKey::NotAssesedYet(game_id, bet.clone()), &Amountx);
}
pub fn delete_not_assesed_yet(env: Env, game_id: i128, Amount: i128, bet: BetKey) {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::NotAssesedYet(game_id, bet.clone()))
        .unwrap_or(0);
    let Amountx = total_amount - Amount;
    env.storage()
        .persistent()
        .set(&DataKey::NotAssesedYet(game_id, bet.clone()), &Amountx);
}
pub fn get_not_assesed_yet(env: Env, game_id: i128, bet: BetKey) -> i128 {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::NotAssesedYet(game_id, bet.clone()))
        .unwrap_or(0);
    total_amount
}
pub fn add_approve_total(env: Env, game_id: i128, Amount: i128, bet: BetKey) {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::Approved(game_id, bet.clone()))
        .unwrap_or(0);
    let Amountx = total_amount + Amount;
    env.storage()
        .persistent()
        .set(&DataKey::Approved(game_id, bet.clone()), &Amountx);
}
pub fn get_approve_total(env: Env, game_id: i128, bet: BetKey) -> i128 {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::Approved(game_id, bet.clone()))
        .unwrap_or(0);
    total_amount
}

pub fn add_reject_total(env: Env, game_id: i128, Amount: i128, bet: BetKey) {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::Rejected(game_id, bet.clone()))
        .unwrap_or(0);
    let Amountx = total_amount + Amount;
    env.storage()
        .persistent()
        .set(&DataKey::Rejected(game_id, bet.clone()), &Amountx);
}
pub fn get_reject_total(env: Env, game_id: i128, bet: BetKey) -> i128 {
    let total_amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::Rejected(game_id, bet.clone()))
        .unwrap_or(0);
    total_amount
}
pub fn set_pool_total(env: Env, game_id: i128, amount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::pool(game_id), &amount);
}
pub fn get_pool_total(env: Env, game_id: i128) -> i128 {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::pool(game_id))
        .unwrap_or(0);
    amount
}
pub fn set_pool_summiter_total(env: Env, game_id: i128, amount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::poolSummiter(game_id), &amount);
}
pub fn save_complain(env: Env, game_id: i128, complain: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::Complain(game_id), &complain);
}
pub fn get_complain(env: Env, game_id: i128) -> i128 {
    let complain: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::Complain(game_id))
        .unwrap_or(0);
    complain
}
pub fn save_winnerPool(env: Env, game_id: i128, amount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::winnerPool(game_id), &amount);
}
pub fn get_winnerPool(env: Env, game_id: i128) -> i128 {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::winnerPool(game_id))
        .unwrap_or(0);
    amount
}
pub fn save_loserPool(env: Env, game_id: i128, amount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::loserPool(game_id), &amount);
}
pub fn get_loserPool(env: Env, game_id: i128) -> i128 {
    let amount: i128 = env
        .storage()
        .persistent()
        .get(&DataKey::loserPool(game_id))
        .unwrap_or(0);
    amount
}
pub fn set_didUserWithdraw(env: Env, user: Address, game_id: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::UserWithdraw(game_id, user), &true);
}
pub fn get_didUserWithdraw(env: Env, user: Address, game_id: i128) -> bool {
    let didWithdraw: bool = env
        .storage()
        .persistent()
        .get(&DataKey::UserWithdraw(game_id, user))
        .unwrap_or(false);
    didWithdraw
}
