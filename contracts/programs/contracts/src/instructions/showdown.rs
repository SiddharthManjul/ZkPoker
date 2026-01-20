use anchor_lang::prelude::*;
use crate::state::{GlobalConfig, Table, Hand, TableStatus, HandStage};
use crate::errors::ZkPokerError;
use crate::constants::{GLOBAL_SEED, TABLE_SEED, HAND_SEED};
use crate::utils::verify_hand_reveal;

/// Reveal hand at showdown
#[derive(Accounts)]
pub struct RevealHand<'info> {
    pub player: Signer<'info>,

    #[account(
        seeds = [GLOBAL_SEED],
        bump = global_config.bump
    )]
    pub global_config: Account<'info, GlobalConfig>,

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

    /// CHECK: SHOWDOWN verifier program - verified in verification function
    #[account(constraint = verifier_program.key() == global_config.showdown_verifier @ ZkPokerError::ProofVerificationFailed)]
    pub verifier_program: AccountInfo<'info>,
}

/// Claim pot after showdown
#[derive(Accounts)]
pub struct ClaimPot<'info> {
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

/// Reveal hand handler
pub fn handle_reveal_hand(
    ctx: Context<RevealHand>,
    hand_rank: u64,
    proof: Vec<u8>,
) -> Result<()> {
    let table = &mut ctx.accounts.table;
    let hand = &mut ctx.accounts.hand;
    let player = ctx.accounts.player.key();

    // Verify player is at table
    let seat = table.get_seat(&player).ok_or(ZkPokerError::PlayerNotAtTable)?;

    // Verify stage
    require!(hand.stage == HandStage::Showdown, ZkPokerError::InvalidStage);

    // Verify player hasn't folded
    require!(!hand.has_folded(seat), ZkPokerError::AlreadyFolded);

    // Verify player hasn't already revealed
    let already_revealed = match seat {
        0 => hand.p1_revealed,
        1 => hand.p2_revealed,
        _ => return Err(ZkPokerError::PlayerNotAtTable.into()),
    };
    require!(!already_revealed, ZkPokerError::HandAlreadyRevealed);

    // Verify ZK proof
    // Note: 'proof' parameter contains both proof + public witness from Sunspot
    verify_hand_reveal(
        &ctx.accounts.verifier_program,
        &proof,
    )?;

    // Store verified hand rank
    match seat {
        0 => {
            hand.p1_hand_rank = hand_rank;
            hand.p1_revealed = true;
        }
        1 => {
            hand.p2_hand_rank = hand_rank;
            hand.p2_revealed = true;
        }
        _ => {}
    }

    // Update timestamp
    let clock = Clock::get()?;
    hand.last_action_at = clock.unix_timestamp;

    msg!("Seat {} revealed hand with rank {}", seat, hand_rank);

    // Check if both players revealed, determine winner
    if hand.p1_revealed && hand.p2_revealed {
        determine_winner(table, hand)?;
    }

    Ok(())
}

/// Determine winner after both players reveal
fn determine_winner(_table: &mut Table, hand: &mut Hand) -> Result<()> {
    // Compare hand ranks (higher is better)
    // The hand_rank is a composite score: rank * 100 + primary_value
    // This ensures proper comparison including kickers

    if hand.p1_hand_rank > hand.p2_hand_rank {
        hand.winner = 0;
        msg!("Seat 0 wins with rank {} vs {}", hand.p1_hand_rank, hand.p2_hand_rank);
    } else if hand.p2_hand_rank > hand.p1_hand_rank {
        hand.winner = 1;
        msg!("Seat 1 wins with rank {} vs {}", hand.p2_hand_rank, hand.p1_hand_rank);
    } else {
        // Split pot
        hand.winner = 2;
        msg!("Split pot - both ranks equal at {}", hand.p1_hand_rank);
    }

    Ok(())
}

/// Claim pot handler
pub fn handle_claim_pot(ctx: Context<ClaimPot>) -> Result<()> {
    let table = &mut ctx.accounts.table;
    let hand = &mut ctx.accounts.hand;
    let player = ctx.accounts.player.key();

    // Verify player is at table
    let seat = table.get_seat(&player).ok_or(ZkPokerError::PlayerNotAtTable)?;

    // Verify hand is in showdown or complete stage
    require!(
        hand.stage == HandStage::Showdown || hand.stage == HandStage::Complete,
        ZkPokerError::InvalidStage
    );

    // Verify pot not already claimed
    require!(!hand.pot_claimed, ZkPokerError::PotAlreadyClaimed);

    // Check if this was a fold win (only one player remaining)
    let fold_win = hand.remaining_players() == 1;

    if fold_win {
        // Fold win - non-folded player claims
        let winner = hand.non_folded_seat().ok_or(ZkPokerError::NotTheWinner)?;
        require!(seat == winner, ZkPokerError::NotTheWinner);

        // Transfer pot
        table.add_chips(winner, hand.pot);
        msg!("Seat {} claims pot of {} (fold)", winner, hand.pot);
    } else {
        // Showdown - verify both revealed and winner determined
        require!(hand.p1_revealed && hand.p2_revealed, ZkPokerError::PlayersNotRevealed);
        require!(hand.winner != 255, ZkPokerError::ShowdownNotReady);

        if hand.winner == 2 {
            // Split pot
            let half = hand.pot / 2;
            let remainder = hand.pot % 2;

            table.add_chips(0, half + remainder); // P1 gets odd chip
            table.add_chips(1, half);

            msg!("Split pot: Seat 0 gets {}, Seat 1 gets {}", half + remainder, half);
        } else {
            // Single winner
            require!(seat == hand.winner, ZkPokerError::NotTheWinner);
            table.add_chips(hand.winner, hand.pot);
            msg!("Seat {} claims pot of {}", hand.winner, hand.pot);
        }
    }

    // Mark pot as claimed
    hand.pot = 0;
    hand.pot_claimed = true;

    // Complete the hand
    hand.stage = HandStage::Complete;
    table.status = TableStatus::Between;
    table.current_hand = None;
    table.increment_hands_played();
    table.rotate_button();

    msg!("Hand {} complete", hand.hand_number);

    Ok(())
}
