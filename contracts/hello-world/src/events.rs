use crate::types::{
    AssessmentKey, Bet, BetKey, BetType, ClaimType, DataKey, Game, LastB, PrivateBet, PublicBet,
    ResultAssessment, ResultGame,
};
use soroban_sdk::{contractevent, vec, Address, Env, String, Symbol, Vec};

#[contractevent(topics = ["BettingGame", "Seleted_Suimmiters"], data_format = "vec")]
struct SummitersSeletedEvent {
    game_id: i128,
    summiters: Vec<Address>,
    main: Address,
}

#[contractevent(topics = ["BettingGame", "Game_Set"], data_format = "single-value")]
struct GameSetEvent {
    game_id: i128,
}

#[contractevent(topics = ["BettingGame", "Private_Setting"], data_format = "vec")]
struct PrivateSettingEvent {
    game_id: i128,
    setting: i128,
    admin: Address,
    minAmount: i128,
}
#[contractevent(topics = ["BettingGame", "Private_Setting_newUser"], data_format = "vec")]
struct NewUserAddedPrivateEvent {
    game_id: i128,
    setting: i128,
    Newuser: Address,
}
#[contractevent(topics = ["BettingGame", "Game_Result"], data_format = "vec")]
struct GameResultEvent {
    game_id: i128,
    result: BetKey,
    description: String,
}
#[contractevent(topics = ["BettingGame", "Game_ResultbySupremeCourt"], data_format = "vec")]
struct GameResultSupremeEvent {
    game_id: i128,
    result: BetKey,
}
#[contractevent(topics = ["BettingGame", "UserHonestyPoints"], data_format = "vec")]
struct UserHonestyPointsEvent {
    user: Address,
    points: i128,
}
#[contractevent(topics = ["BettingGame", "Active_Setting"], data_format = "vec")]
struct Active_SettingEvent {
    game_id: i128,
    Setting: i128,
}

#[contractevent(topics = ["BettingGame", "Game_Result_Reject"], data_format = "single-value")]
struct GameResultRejectEvent {
    game_id: i128,
}
#[contractevent(topics = ["BettingGame", "Game_allUserHaveVoted"], data_format = "single-value")]
struct GameAllUserHaveVotedEvent {
    game_id: i128,
}
#[contractevent(topics = ["BettingGame", "Game_Setting_Distributed"], data_format = "single-value")]
struct GameSettingDistributedEvent {
    setting: i128,
}
#[contractevent(topics = ["BettingGame", "Game_StakeMinAmount"], data_format = "single-value")]
struct StakeMinAmountdEvent {
    NewAmount: i128,
}

pub struct BettingEvents {}

impl BettingEvents {
    pub fn summiters_seleted(e: &Env, game_id: i128, summiters: Vec<Address>, main: Address) {
        SummitersSeletedEvent {
            game_id,
            summiters,
            main,
        }
        .publish(&e);
    }
    pub fn game_set(e: &Env, game_id: i128) {
        GameSetEvent { game_id }.publish(&e);
    }
    pub fn private_setting(e: &Env, game_id: i128, setting: i128, admin: Address, minAmount: i128) {
        PrivateSettingEvent {
            game_id,
            setting,
            admin,
            minAmount,
        }
        .publish(&e);
    }
    pub fn new_user_added_private(e: &Env, game_id: i128, setting: i128, Newuser: Address) {
        NewUserAddedPrivateEvent {
            game_id,
            setting,
            Newuser,
        }
        .publish(&e);
    }
    pub fn game_result(e: &Env, game_id: i128, result: BetKey, description: String) {
        GameResultEvent {
            game_id,
            result,
            description,
        }
        .publish(&e);
    }
    pub fn game_result_reject(e: &Env, game_id: i128) {
        GameResultRejectEvent { game_id }.publish(&e);
    }
    pub fn game_setting_distributed(e: &Env, setting: i128) {
        GameSettingDistributedEvent { setting }.publish(&e);
    }
    pub fn set_stake_amount(e: &Env, NewAmount: i128) {
        StakeMinAmountdEvent { NewAmount }.publish(&e);
    }
    pub fn game_result_supreme(e: &Env, game_id: i128, result: BetKey) {
        GameResultSupremeEvent { game_id, result }.publish(&e);
    }
    pub fn user_honesty_points(e: &Env, user: Address, points: i128) {
        UserHonestyPointsEvent { user, points }.publish(&e);
    }
    pub fn active_setting(e: &Env, game_id: i128, Setting: i128) {
        Active_SettingEvent { game_id, Setting }.publish(&e);
    }
    pub fn all_vote(e: &Env, game_id: i128) {
        GameAllUserHaveVotedEvent { game_id }.publish(&e);
    }
}
