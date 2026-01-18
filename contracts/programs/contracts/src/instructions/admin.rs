use anchor_lang::prelude::*;
use crate::state::GlobalConfig;
use crate::errors::ZkPokerError;
use crate::constants::{
    GLOBAL_SEED,
    DECK_VERIFIER_PROGRAM_ID,
    DEAL_VERIFIER_PROGRAM_ID,
    REVEAL_VERIFIER_PROGRAM_ID,
    SHOWDOWN_VERIFIER_PROGRAM_ID,
    BET_VERIFIER_PROGRAM_ID
};

/// Initialize the global configuration
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = GlobalConfig::LEN,
        seeds = [GLOBAL_SEED],
        bump
    )]
    pub global_config: Account<'info, GlobalConfig>,

    /// USDC mint account
    pub usdc_mint: Account<'info, anchor_spl::token::Mint>,

    pub system_program: Program<'info, System>,
}

/// Pause the protocol
#[derive(Accounts)]
pub struct Pause<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_SEED],
        bump = global_config.bump,
        constraint = global_config.authority == authority.key() @ ZkPokerError::Unauthorized
    )]
    pub global_config: Account<'info, GlobalConfig>,
}

/// Unpause the protocol
#[derive(Accounts)]
pub struct Unpause<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_SEED],
        bump = global_config.bump,
        constraint = global_config.authority == authority.key() @ ZkPokerError::Unauthorized
    )]
    pub global_config: Account<'info, GlobalConfig>,
}

/// Initialize handler
pub fn handle_initialize(ctx: Context<Initialize>) -> Result<()> {
    let global_config = &mut ctx.accounts.global_config;
    let bump = ctx.bumps.global_config;

    global_config.init(
        ctx.accounts.authority.key(),
        ctx.accounts.usdc_mint.key(),
        DECK_VERIFIER_PROGRAM_ID,
        DEAL_VERIFIER_PROGRAM_ID,
        REVEAL_VERIFIER_PROGRAM_ID,
        SHOWDOWN_VERIFIER_PROGRAM_ID,
        BET_VERIFIER_PROGRAM_ID,
        bump,
    );

    msg!("ZkPoker initialized");
    msg!("Authority: {}", ctx.accounts.authority.key());
    msg!("USDC Mint: {}", ctx.accounts.usdc_mint.key());
    msg!("Deck Verifier: {}", DECK_VERIFIER_PROGRAM_ID);
    msg!("Deal Verifier: {}", DEAL_VERIFIER_PROGRAM_ID);
    msg!("Reveal Verifier: {}", REVEAL_VERIFIER_PROGRAM_ID);
    msg!("Showdown Verifier: {}", SHOWDOWN_VERIFIER_PROGRAM_ID);
    msg!("Bet Verifier: {}", BET_VERIFIER_PROGRAM_ID);

    Ok(())
}

/// Pause handler
pub fn handle_pause(ctx: Context<Pause>) -> Result<()> {
    let global_config = &mut ctx.accounts.global_config;
    global_config.is_paused = true;

    msg!("ZkPoker paused by {}", ctx.accounts.authority.key());

    Ok(())
}

/// Unpause handler
pub fn handle_unpause(ctx: Context<Unpause>) -> Result<()> {
    let global_config = &mut ctx.accounts.global_config;
    global_config.is_paused = false;

    msg!("ZkPoker unpaused by {}", ctx.accounts.authority.key());

    Ok(())
}
