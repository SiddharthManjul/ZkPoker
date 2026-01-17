use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::{GlobalConfig, Table, TableStatus};
use crate::errors::ZkPokerError;
use crate::constants::{GLOBAL_SEED, TABLE_SEED, VAULT_SEED, DEFAULT_ACTION_TIMEOUT, MIN_ACTION_TIMEOUT, MAX_ACTION_TIMEOUT};

/// Create a new table
#[derive(Accounts)]
pub struct CreateTable<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_SEED],
        bump = global_config.bump,
        constraint = !global_config.is_paused @ ZkPokerError::GamePaused
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(
        init,
        payer = creator,
        space = Table::LEN,
        seeds = [TABLE_SEED, &global_config.table_count.to_le_bytes()],
        bump
    )]
    pub table: Account<'info, Table>,

    /// Table vault for holding USDC
    #[account(
        init,
        payer = creator,
        token::mint = usdc_mint,
        token::authority = table,
        seeds = [VAULT_SEED, table.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, TokenAccount>,

    /// USDC mint
    #[account(
        constraint = usdc_mint.key() == global_config.usdc_mint @ ZkPokerError::InvalidMint
    )]
    pub usdc_mint: Account<'info, anchor_spl::token::Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Join a table
#[derive(Accounts)]
pub struct JoinTable<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    #[account(
        seeds = [GLOBAL_SEED],
        bump = global_config.bump,
        constraint = !global_config.is_paused @ ZkPokerError::GamePaused
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(
        mut,
        seeds = [TABLE_SEED, &table.table_id.to_le_bytes()],
        bump = table.bump
    )]
    pub table: Account<'info, Table>,

    /// Player's USDC token account
    #[account(
        mut,
        constraint = player_token_account.owner == player.key(),
        constraint = player_token_account.mint == global_config.usdc_mint
    )]
    pub player_token_account: Account<'info, TokenAccount>,

    /// Table vault
    #[account(
        mut,
        seeds = [VAULT_SEED, table.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

/// Leave a table
#[derive(Accounts)]
pub struct LeaveTable<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    #[account(
        mut,
        seeds = [TABLE_SEED, &table.table_id.to_le_bytes()],
        bump = table.bump
    )]
    pub table: Account<'info, Table>,

    /// Player's USDC token account
    #[account(
        mut,
        constraint = player_token_account.owner == player.key()
    )]
    pub player_token_account: Account<'info, TokenAccount>,

    /// Table vault
    #[account(
        mut,
        seeds = [VAULT_SEED, table.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

/// Add chips to stack
#[derive(Accounts)]
pub struct AddChips<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    #[account(
        seeds = [GLOBAL_SEED],
        bump = global_config.bump,
        constraint = !global_config.is_paused @ ZkPokerError::GamePaused
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(
        mut,
        seeds = [TABLE_SEED, &table.table_id.to_le_bytes()],
        bump = table.bump
    )]
    pub table: Account<'info, Table>,

    /// Player's USDC token account
    #[account(
        mut,
        constraint = player_token_account.owner == player.key(),
        constraint = player_token_account.mint == global_config.usdc_mint
    )]
    pub player_token_account: Account<'info, TokenAccount>,

    /// Table vault
    #[account(
        mut,
        seeds = [VAULT_SEED, table.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

/// Create table handler
pub fn handle_create_table(
    ctx: Context<CreateTable>,
    small_blind: u64,
    big_blind: u64,
    min_buy_in: u64,
    max_buy_in: u64,
    action_timeout: Option<i64>,
) -> Result<()> {
    // Validate configuration
    require!(small_blind > 0, ZkPokerError::InvalidTableConfig);
    require!(big_blind >= small_blind, ZkPokerError::InvalidTableConfig);
    require!(min_buy_in >= big_blind * 10, ZkPokerError::InvalidTableConfig); // At least 10 BB
    require!(max_buy_in >= min_buy_in, ZkPokerError::InvalidTableConfig);

    let timeout = action_timeout.unwrap_or(DEFAULT_ACTION_TIMEOUT);
    require!(timeout >= MIN_ACTION_TIMEOUT && timeout <= MAX_ACTION_TIMEOUT, ZkPokerError::InvalidTimeoutConfig);

    let global_config = &mut ctx.accounts.global_config;
    let table = &mut ctx.accounts.table;

    let table_id = global_config.next_table_id();
    let clock = Clock::get()?;
    let bump = ctx.bumps.table;

    table.init(
        table_id,
        small_blind,
        big_blind,
        min_buy_in,
        max_buy_in,
        timeout,
        clock.unix_timestamp,
        bump,
    );

    msg!("Table {} created", table_id);
    msg!("Blinds: {}/{}", small_blind, big_blind);
    msg!("Buy-in: {}-{}", min_buy_in, max_buy_in);

    Ok(())
}

/// Join table handler
pub fn handle_join_table(ctx: Context<JoinTable>, buy_in_amount: u64) -> Result<()> {
    let table = &mut ctx.accounts.table;
    let player = ctx.accounts.player.key();

    // Validate buy-in amount
    require!(
        buy_in_amount >= table.min_buy_in && buy_in_amount <= table.max_buy_in,
        ZkPokerError::InvalidBuyIn
    );

    // Check player not already at table
    require!(
        table.player_one != Some(player) && table.player_two != Some(player),
        ZkPokerError::PlayerAlreadyAtTable
    );

    // Check table has empty seat
    require!(table.has_empty_seat(), ZkPokerError::TableFull);

    // Check no active hand
    require!(
        table.status != TableStatus::Playing,
        ZkPokerError::HandInProgress
    );

    // Transfer USDC from player to vault
    let cpi_accounts = Transfer {
        from: ctx.accounts.player_token_account.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        authority: ctx.accounts.player.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, buy_in_amount)?;

    // Assign seat
    if table.player_one.is_none() {
        table.player_one = Some(player);
        table.player_one_chips = buy_in_amount;
        msg!("Player {} joined seat 0 with {} chips", player, buy_in_amount);
    } else {
        table.player_two = Some(player);
        table.player_two_chips = buy_in_amount;
        msg!("Player {} joined seat 1 with {} chips", player, buy_in_amount);
    }

    // Update status if table is now full
    if table.is_full() {
        table.status = TableStatus::Between;
        msg!("Table is full, ready to start hand");
    }

    Ok(())
}

/// Leave table handler
pub fn handle_leave_table(ctx: Context<LeaveTable>) -> Result<()> {
    let table = &mut ctx.accounts.table;
    let player = ctx.accounts.player.key();

    // Get player's seat
    let seat = table.get_seat(&player).ok_or(ZkPokerError::PlayerNotAtTable)?;

    // Check no active hand
    require!(
        table.status != TableStatus::Playing,
        ZkPokerError::HandInProgress
    );

    // Get chips to return
    let chips_to_return = table.get_chips(seat);

    // Transfer chips back to player if any
    if chips_to_return > 0 {
        let table_key = table.key();
        let seeds = &[
            VAULT_SEED,
            table_key.as_ref(),
            &[ctx.bumps.vault],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.player_token_account.to_account_info(),
            authority: table.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        token::transfer(cpi_ctx, chips_to_return)?;
    }

    // Clear seat
    match seat {
        0 => {
            table.player_one = None;
            table.player_one_chips = 0;
        }
        1 => {
            table.player_two = None;
            table.player_two_chips = 0;
        }
        _ => {}
    }

    // Update status
    table.status = TableStatus::Waiting;

    msg!("Player {} left table with {} chips", player, chips_to_return);

    Ok(())
}

/// Add chips handler
pub fn handle_add_chips(ctx: Context<AddChips>, amount: u64) -> Result<()> {
    let table = &mut ctx.accounts.table;
    let player = ctx.accounts.player.key();

    // Get player's seat
    let seat = table.get_seat(&player).ok_or(ZkPokerError::PlayerNotAtTable)?;

    // Check no active hand
    require!(
        table.status != TableStatus::Playing,
        ZkPokerError::HandInProgress
    );

    // Check total doesn't exceed max buy-in
    let current_chips = table.get_chips(seat);
    let new_total = current_chips.checked_add(amount).ok_or(ZkPokerError::ArithmeticOverflow)?;
    require!(new_total <= table.max_buy_in, ZkPokerError::InvalidBuyIn);

    // Transfer USDC from player to vault
    let cpi_accounts = Transfer {
        from: ctx.accounts.player_token_account.to_account_info(),
        to: ctx.accounts.vault.to_account_info(),
        authority: ctx.accounts.player.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    // Add chips to stack
    table.add_chips(seat, amount);

    msg!("Player {} added {} chips, new total: {}", player, amount, new_total);

    Ok(())
}
