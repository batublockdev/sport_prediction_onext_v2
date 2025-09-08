use soroban_sdk::{contractevent, vec, Address, Env, String, Symbol, Vec};

#[contractevent(topics = ["BettingGame", "Seleted_Suimmiters"], data_format = "single-value")]
struct SummitersSeletedEvent {
    game_id: i128,
}

pub struct BettingEvents {}

impl BettingEvents {
    pub fn summiters_seleted(e: &Env, game_id: i128, summiters: Vec<Address>, main: Address) {
        SummitersSeletedEvent { game_id }.publish(&e);
    }

    /// Emitted when a proposal is canceled
    ///
    /// - topics - `["proposal_canceled", proposal_id: u32]`
    /// - data - ()
    pub fn proposal_canceled(e: &Env, proposal_id: u32) {
        let topics = (Symbol::new(&e, "proposal_canceled"), proposal_id);
        e.events().publish(topics, ());
    }
}
