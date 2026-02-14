use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

const COMP_DEF_OFFSET_UPDATE: u32 = comp_def_offset("update_market_state");

declare_id!("ARyfGGycpReujchifKTXsGZxD1KDxDsg8bLjnnJEeRWQ");

#[arcium_program]
pub mod arcpredict {
    use super::*;

    pub fn init_config(ctx: Context<InitConfig>) -> Result<()> {
        init_comp_def(ctx.accounts, None, None)?;
        Ok(())
    }

    /// [新增] 初始化预测市场
    pub fn create_market(ctx: Context<CreateMarket>, topic: String) -> Result<()> {
        let market = &mut ctx.accounts.market;
        market.authority = ctx.accounts.authority.key();
        // 初始化加密池为 0 (假设前端已生成了代表0的密文，或者由首次计算初始化)
        // 这里为了简化，我们假设初始状态是特定的空密文
        market.encrypted_yes_pool = [0u8; 32]; 
        market.encrypted_no_pool = [0u8; 32];
        market.topic = topic;
        market.is_resolved = false;
        Ok(())
    }

    /// [升级] 下注 (Place Bet)
    /// 读取当前的加密池状态，发送给 MXE 进行累加更新
    pub fn place_bet(
        ctx: Context<PlaceBet>,
        computation_offset: u64,
        encrypted_amount: [u8; 32], // 用户加密的金额
        encrypted_side: [u8; 32],   // 用户加密的选择 (1 or 2)
        pubkey: [u8; 32],
        nonce: u128,
    ) -> Result<()> {
        let market = &ctx.accounts.market;
        require!(!market.is_resolved, MarketError::MarketResolved);

        ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;
        
        // 构建参数：Current State + User Bet
        let args = ArgBuilder::new()
            .x25519_pubkey(pubkey)
            .plaintext_u128(nonce)
            // MarketState struct fields
            .encrypted_u64(market.encrypted_yes_pool)
            .encrypted_u64(market.encrypted_no_pool)
            // BetInput struct fields
            .encrypted_u64(encrypted_amount)
            .encrypted_u64(encrypted_side)
            .build();

        queue_computation(
            ctx.accounts,
            computation_offset,
            args,
            // 修正 1: 使用正确的结构体名称 UpdateMarketStateCallback
            vec![UpdateMarketStateCallback::callback_ix(
                computation_offset,
                &ctx.accounts.mxe_account,
                &[]
            )?],
            1,
            0,
        )?;
        Ok(())
    }

    #[arcium_callback(encrypted_ix = "update_market_state")]
    pub fn update_market_state_callback(
        // 修正 2: 使用正确的结构体名称
        ctx: Context<UpdateMarketStateCallback>,
        output: SignedComputationOutputs<UpdateMarketStateOutput>,
    ) -> Result<()> {
        let o = match output.verify_output(&ctx.accounts.cluster_account, &ctx.accounts.computation_account) {
            Ok(UpdateMarketStateOutput { field_0 }) => field_0,
            Err(_) => return Err(ErrorCode::AbortedComputation.into()),
        };

        // 更新链上状态为新的密文
        let market = &mut ctx.accounts.market;
        
        // Arcis 返回: { new_yes_pool, new_no_pool }
        let new_yes_bytes: [u8; 32] = o.ciphertexts[0]; // 保持密文状态 (32 bytes)
        let new_no_bytes: [u8; 32] = o.ciphertexts[1];

        market.encrypted_yes_pool = new_yes_bytes;
        market.encrypted_no_pool = new_no_bytes;

        msg!("Market State Updated. Pools remain encrypted.");
        
        emit!(PoolUpdateEvent {
            market: market.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }
}

// --- Accounts ---

#[derive(Accounts)]
pub struct CreateMarket<'info> {
    #[account(
        init, 
        payer = authority, 
        space = 8 + 32 + 32 + 32 + 64 + 1, 
        seeds = [b"market", authority.key().as_ref()],
        bump
    )]
    pub market: Account<'info, Market>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct Market {
    pub authority: Pubkey,
    pub encrypted_yes_pool: [u8; 32], // 存储密文
    pub encrypted_no_pool: [u8; 32],  // 存储密文
    pub topic: String,
    pub is_resolved: bool,
}

#[queue_computation_accounts("update_market_state", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64)]
pub struct PlaceBet<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub market: Account<'info, Market>, // 需要读取并稍后更新
    
    #[account(init_if_needed, space = 9, payer = payer, seeds = [&SIGN_PDA_SEED], bump, address = derive_sign_pda!())]
    pub sign_pda_account: Account<'info, ArciumSignerAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut, address = derive_mempool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: Mempool
    pub mempool_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_execpool_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: Execpool
    pub executing_pool: UncheckedAccount<'info>,
    #[account(mut, address = derive_comp_pda!(computation_offset, mxe_account, ErrorCode::ClusterNotSet))]
    /// CHECK: Comp
    pub computation_account: UncheckedAccount<'info>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_UPDATE))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(mut, address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(mut, address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS)]
    pub pool_account: Account<'info, FeePool>,
    #[account(mut, address = ARCIUM_CLOCK_ACCOUNT_ADDRESS)]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
}

// 修正 3: 结构体名称必须是 UpdateMarketStateCallback
#[callback_accounts("update_market_state")]
#[derive(Accounts)]
pub struct UpdateMarketStateCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(address = derive_comp_def_pda!(COMP_DEF_OFFSET_UPDATE))]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    /// CHECK: Comp
    pub computation_account: UncheckedAccount<'info>,
    #[account(mut)] // 写入新的密文状态
    pub market: Account<'info, Market>,
    #[account(address = derive_cluster_pda!(mxe_account, ErrorCode::ClusterNotSet))]
    pub cluster_account: Account<'info, Cluster>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: Sysvar
    pub instructions_sysvar: AccountInfo<'info>,
}

#[init_computation_definition_accounts("update_market_state", payer)]
#[derive(Accounts)]
pub struct InitConfig<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut, address = derive_mxe_pda!())]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: Def
    pub comp_def_account: UncheckedAccount<'info>,
    #[account(mut, address = derive_mxe_lut_pda!(mxe_account.lut_offset_slot))]
    /// CHECK: LUT
    pub address_lookup_table: UncheckedAccount<'info>,
    #[account(address = LUT_PROGRAM_ID)]
    /// CHECK: LUT Prog
    pub lut_program: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct PoolUpdateEvent {
    pub market: Pubkey,
    pub timestamp: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Aborted")] AbortedComputation,
    #[msg("No Cluster")] ClusterNotSet,
}

#[error_code]
pub enum MarketError {
    #[msg("Market is closed")] MarketResolved,
}