use soroban_sdk::{vec, Address, Env, String, Symbol, Vec};

pub struct BettingEvents {}

impl BettingEvents {
    pub fn summiters_seleted(e: &Env, game_id: i128, summiters: Vec<Address>, main: Address) {
        let topics = (Symbol::new(&e, "summiters_seleted"), game_id);
        e.events().publish(topics, (summiters, main));
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
