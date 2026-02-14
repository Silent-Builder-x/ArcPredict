use arcis::*;

#[encrypted]
mod prediction_engine {
    use arcis::*;

    pub struct MarketState {
        pub yes_pool: u64,
        pub no_pool: u64,
    }

    pub struct BetInput {
        pub amount: u64,
        pub side: u64, // 1 = YES, 2 = NO
    }

    // State update output
    pub struct StateUpdateResult {
        pub new_yes_pool: u64,
        pub new_no_pool: u64,
    }

    #[instruction]
    pub fn update_market_state(
        current_state: Enc<Shared, MarketState>,
        user_bet: Enc<Shared, BetInput>
    ) -> Enc<Shared, StateUpdateResult> {
        let state = current_state.to_arcis();
        let bet = user_bet.to_arcis();

        // Core logic: Use Mux to update the corresponding pool based on the user's choice
        // Side 1 (YES): YesPool + Amount
        // Side 2 (NO):  NoPool + Amount
        
        let is_yes = bet.side == 1;
        
        let added_yes = if is_yes { bet.amount } else { 0 };
        let added_no  = if is_yes { 0 } else { bet.amount };

        let new_yes = state.yes_pool + added_yes;
        let new_no  = state.no_pool + added_no;

        let result = StateUpdateResult {
            new_yes_pool: new_yes,
            new_no_pool: new_no,
        };

        // Return to the Program (Owner) to update the on-chain state
        current_state.owner.from_arcis(result)
    }
}