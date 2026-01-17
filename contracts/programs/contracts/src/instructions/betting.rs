use anchor_lang::prelude::*;
use crate::state::{Table, Hand, TableStatus, HandStage};
use crate::errors::ZkPokerError;
use crate::constants::{TABLE_SEED, HAND_SEED};

/// Betting action context (shared by all betting instructions)
#[derive(Accounts)]
pub struct BettingAction<'info> {
    pub player: Signer<'info>,

    #[account(
        mut,
        seeds = [TABLE_SEED, &table.table_id.to_le_bytes()],
        bump = table.bump
    )]
    pub table: Account<'info, Table>,

    #[account(
        mut,
        seeds = [HAND_SEED, table.key().as_ref(), &hand.hand_number.to_le_bytes()],
        bump = hand.bump,
        constraint = hand.table == table.key()
    )]
    pub hand: Account<'info, Hand>,
}

/// Validate that the player can take a betting action
fn validate_betting_action(table: &Table, hand: &Hand, player: &Pubkey) -> Result<u8> {
    // Get player's seat
    let seat = table.get_seat(player).ok_or(ZkPokerError::PlayerNotAtTable)?;

    // Verify it's a betting stage
    require!(hand.stage.is_betting_stage(), ZkPokerError::InvalidStage);

    // Verify it's player's turn
    require!(hand.action_on == seat, ZkPokerError::NotYourTurn);

    // Verify player hasn't folded
    require!(!hand.has_folded(seat), ZkPokerError::AlreadyFolded);

    // Verify player isn't already all-in
    require!(!hand.is_all_in(seat), ZkPokerError::AlreadyAllIn);

    Ok(seat)
}

/// Handle street transition after betting completes
fn handle_street_transition(table: &mut Table, hand: &mut Hand) -> Result<()> {
    // Check if someone folded
    if hand.remaining_players() == 1 {
        // Award pot to remaining player
        if let Some(winner) = hand.non_folded_seat() {
            hand.winner = winner;
            table.add_chips(winner, hand.pot);
            hand.pot = 0;
            hand.stage = HandStage::Complete;
            table.status = TableStatus::Between;
            table.current_hand = None;
            table.increment_hands_played();
            table.rotate_button();
            msg!("Player folded, seat {} wins pot", winner);
        }
        return Ok(());
    }

    // Check if betting round is complete
    if hand.is_betting_complete() {
        // Advance to next stage
        if let Some(next_stage) = hand.stage.next_betting_stage() {
            hand.stage = next_stage;
            hand.reset_street();

            // Set action to big blind (first to act post-flop in heads-up)
            let bb_seat = table.big_blind_seat();
            hand.action_on = bb_seat;

            msg!("Advancing to {:?}", next_stage);

            // If showdown, both players need to reveal
            if next_stage == HandStage::Showdown {
                msg!("Showdown reached!");
            }
        }
    }

    Ok(())
}

/// Check handler - pass without betting
pub fn handle_check(ctx: Context<BettingAction>) -> Result<()> {
    let table = &mut ctx.accounts.table;
    let hand = &mut ctx.accounts.hand;
    let player = ctx.accounts.player.key();

    let seat = validate_betting_action(table, hand, &player)?;

    // Can only check if current bet equals player's bet
    let player_bet = hand.get_bet_this_street(seat);
    require!(hand.current_bet == player_bet, ZkPokerError::CannotCheck);

    // Mark as acted
    hand.set_acted_this_street(seat);

    // Update timestamp
    let clock = Clock::get()?;
    hand.last_action_at = clock.unix_timestamp;

    // Switch action
    hand.switch_action();

    msg!("Seat {} checks", seat);

    // Handle potential street transition
    handle_street_transition(table, hand)?;

    Ok(())
}

/// Bet handler - open betting
pub fn handle_bet(ctx: Context<BettingAction>, amount: u64) -> Result<()> {
    let table = &mut ctx.accounts.table;
    let hand = &mut ctx.accounts.hand;
    let player = ctx.accounts.player.key();

    let seat = validate_betting_action(table, hand, &player)?;

    // Can only bet if no current bet
    require!(hand.current_bet == 0, ZkPokerError::InvalidBetAmount);

    // Bet must be at least big blind
    require!(amount >= table.big_blind, ZkPokerError::BetTooSmall);

    // Get player's available chips
    let available_chips = table.get_chips(seat);
    require!(amount <= available_chips, ZkPokerError::InsufficientChips);

    // Remove chips from player
    table.remove_chips(seat, amount);

    // Add to pot and track bet
    hand.add_bet(seat, amount);
    hand.current_bet = amount;
    hand.last_aggressor = seat;

    // Mark as acted
    hand.set_acted_this_street(seat);

    // Check if all-in
    if table.get_chips(seat) == 0 {
        hand.set_all_in(seat);
        msg!("Seat {} bets {} (ALL-IN)", seat, amount);
    } else {
        msg!("Seat {} bets {}", seat, amount);
    }

    // Update timestamp
    let clock = Clock::get()?;
    hand.last_action_at = clock.unix_timestamp;

    // Switch action
    hand.switch_action();

    // Handle potential street transition
    handle_street_transition(table, hand)?;

    Ok(())
}

/// Call handler - match current bet
pub fn handle_call(ctx: Context<BettingAction>) -> Result<()> {
    let table = &mut ctx.accounts.table;
    let hand = &mut ctx.accounts.hand;
    let player = ctx.accounts.player.key();

    let seat = validate_betting_action(table, hand, &player)?;

    // Calculate amount to call
    let player_bet = hand.get_bet_this_street(seat);
    let to_call = hand.current_bet.saturating_sub(player_bet);

    require!(to_call > 0, ZkPokerError::CannotCheck); // Should use check if nothing to call

    // Get player's available chips
    let available_chips = table.get_chips(seat);
    let actual_call = to_call.min(available_chips);

    // Remove chips from player
    table.remove_chips(seat, actual_call);

    // Add to pot and track bet
    hand.add_bet(seat, actual_call);

    // Mark as acted
    hand.set_acted_this_street(seat);

    // Check if all-in (couldn't fully call)
    if table.get_chips(seat) == 0 {
        hand.set_all_in(seat);
        msg!("Seat {} calls {} (ALL-IN)", seat, actual_call);
    } else {
        msg!("Seat {} calls {}", seat, actual_call);
    }

    // Update timestamp
    let clock = Clock::get()?;
    hand.last_action_at = clock.unix_timestamp;

    // Switch action
    hand.switch_action();

    // Handle potential street transition
    handle_street_transition(table, hand)?;

    Ok(())
}

/// Raise handler - raise to a total amount
pub fn handle_raise_to(ctx: Context<BettingAction>, amount: u64) -> Result<()> {
    let table = &mut ctx.accounts.table;
    let hand = &mut ctx.accounts.hand;
    let player = ctx.accounts.player.key();

    let seat = validate_betting_action(table, hand, &player)?;

    // Raise must be to an amount greater than current bet
    require!(amount > hand.current_bet, ZkPokerError::RaiseTooSmall);

    // Minimum raise is current_bet + big_blind (or current_bet * 2 for simplicity)
    let min_raise = hand.current_bet.saturating_add(table.big_blind);
    require!(amount >= min_raise, ZkPokerError::RaiseTooSmall);

    // Calculate how much more to put in
    let player_bet = hand.get_bet_this_street(seat);
    let additional = amount.saturating_sub(player_bet);

    // Get player's available chips
    let available_chips = table.get_chips(seat);
    require!(additional <= available_chips, ZkPokerError::InsufficientChips);

    // Remove chips from player
    table.remove_chips(seat, additional);

    // Add to pot and track bet
    hand.add_bet(seat, additional);
    hand.current_bet = amount;
    hand.last_aggressor = seat;

    // Mark as acted
    hand.set_acted_this_street(seat);

    // Reset opponent's acted flag (they need to act again)
    let opponent = hand.other_seat(seat);
    match opponent {
        0 => hand.p1_acted_this_street = false,
        1 => hand.p2_acted_this_street = false,
        _ => {}
    }

    // Check if all-in
    if table.get_chips(seat) == 0 {
        hand.set_all_in(seat);
        msg!("Seat {} raises to {} (ALL-IN)", seat, amount);
    } else {
        msg!("Seat {} raises to {}", seat, amount);
    }

    // Update timestamp
    let clock = Clock::get()?;
    hand.last_action_at = clock.unix_timestamp;

    // Switch action
    hand.switch_action();

    // Handle potential street transition
    handle_street_transition(table, hand)?;

    Ok(())
}

/// Fold handler - surrender the hand
pub fn handle_fold(ctx: Context<BettingAction>) -> Result<()> {
    let table = &mut ctx.accounts.table;
    let hand = &mut ctx.accounts.hand;
    let player = ctx.accounts.player.key();

    let seat = validate_betting_action(table, hand, &player)?;

    // Mark as folded
    hand.set_folded(seat);

    // Update timestamp
    let clock = Clock::get()?;
    hand.last_action_at = clock.unix_timestamp;

    msg!("Seat {} folds", seat);

    // Handle street transition (will award pot to winner)
    handle_street_transition(table, hand)?;

    Ok(())
}

/// All-in handler - bet entire stack
pub fn handle_all_in(ctx: Context<BettingAction>) -> Result<()> {
    let table = &mut ctx.accounts.table;
    let hand = &mut ctx.accounts.hand;
    let player = ctx.accounts.player.key();

    let seat = validate_betting_action(table, hand, &player)?;

    // Get player's entire stack
    let available_chips = table.get_chips(seat);
    require!(available_chips > 0, ZkPokerError::InsufficientChips);

    // Remove all chips from player
    table.remove_chips(seat, available_chips);

    // Calculate total bet this street
    let player_bet = hand.get_bet_this_street(seat);
    let new_total = player_bet.saturating_add(available_chips);

    // Add to pot and track bet
    hand.add_bet(seat, available_chips);

    // Update current bet if this is a raise
    if new_total > hand.current_bet {
        hand.current_bet = new_total;
        hand.last_aggressor = seat;

        // Reset opponent's acted flag
        let opponent = hand.other_seat(seat);
        match opponent {
            0 => hand.p1_acted_this_street = false,
            1 => hand.p2_acted_this_street = false,
            _ => {}
        }
    }

    // Mark as all-in and acted
    hand.set_all_in(seat);
    hand.set_acted_this_street(seat);

    msg!("Seat {} goes ALL-IN for {}", seat, available_chips);

    // Update timestamp
    let clock = Clock::get()?;
    hand.last_action_at = clock.unix_timestamp;

    // Switch action
    hand.switch_action();

    // Handle potential street transition
    handle_street_transition(table, hand)?;

    Ok(())
}
