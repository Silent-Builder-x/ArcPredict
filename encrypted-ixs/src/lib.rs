use arcis::*;

#[encrypted]
mod prediction_engine {
    use arcis::*;

    pub struct MarketState {
        pub yes_pool: u64,
        pub no_pool: u64,
    }

    pub struct BetPayload {
        pub amount: u64,
        pub side: u8, // 1 为 YES, 2 为 NO
    }

    pub struct SettlementOutput {
        pub is_winner: u64,
        pub payout_amount: u64,
    }

    #[instruction]
    pub fn update_market_state(
        state_ctxt: Enc<Shared, MarketState>,
        bet_ctxt: Enc<Shared, BetPayload>
    ) -> Enc<Shared, MarketState> {
        let mut state = state_ctxt.to_arcis();
        let bet = bet_ctxt.to_arcis();

        // 使用 V4 规范的 if-else Mux 逻辑进行状态路由
        if bet.side == 1u8 {
            state.yes_pool = state.yes_pool + bet.amount;
        } else {
            state.no_pool = state.no_pool + bet.amount;
        };

        state_ctxt.owner.from_arcis(state)
    }

    #[instruction]
    pub fn resolve_payout(
        state_ctxt: Enc<Shared, MarketState>,
        bet_ctxt: Enc<Shared, BetPayload>,
        winning_side_ctxt: Enc<Shared, u8>
    ) -> Enc<Shared, SettlementOutput> {
        let state = state_ctxt.to_arcis();
        let bet = bet_ctxt.to_arcis();
        let winning_side = winning_side_ctxt.to_arcis();

        let total_pool = state.yes_pool + state.no_pool;
        let is_win = if bet.side == winning_side { 1u64 } else { 0u64 };

        // 赔率逻辑：(Total / Winning_Pool) * User_Bet
        let winning_pool = if winning_side == 1u8 { state.yes_pool } else { state.no_pool };
        let safe_win_pool = if winning_pool > 0 { winning_pool } else { 1u64 };

        let payout = if is_win == 1u64 {
            (bet.amount * total_pool) / safe_win_pool
        } else {
            0u64
        };

        let res = SettlementOutput {
            is_winner: is_win,
            payout_amount: payout,
        };

        state_ctxt.owner.from_arcis(res)
    }
}