use anchor_lang::prelude::*;
use crate::state::{GlobalConfig, Table, Hand, HandStage};
use crate::errors::ZkPokerError;
use crate::constants::{GLOBAL_SEED, TABLE_SEED, HAND_SEED, FLOP_1_POS, FLOP_2_POS, FLOP_3_POS, TURN_POS, RIVER_POS};
use crate::utils::verify_community_cards;

/// Reveal community cards context
#[derive(Accounts)]
pub struct RevealCommunity<'info> {
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

    /// CHECK: REVEAL verifier program - verified in verification function
    #[account(constraint = verifier_program.key() == global_config.reveal_verifier @ ZkPokerError::ProofVerificationFailed)]
    pub verifier_program: AccountInfo<'info>,
}

/// Reveal flop handler (3 cards)
pub fn handle_reveal_flop(
    ctx: Context<RevealCommunity>,
    cards: [u8; 3],
    proof: Vec<u8>,
) -> Result<()> {
    let table = &ctx.accounts.table;
    let hand = &mut ctx.accounts.hand;
    let player = ctx.accounts.player.key();

    // Verify player is at table
    let _seat = table.get_seat(&player).ok_or(ZkPokerError::PlayerNotAtTable)?;

    // Verify stage - must be in Flop stage (waiting for reveal)
    require!(hand.stage == HandStage::Flop, ZkPokerError::InvalidStage);

    // Verify flop not already revealed
    require!(!hand.flop_revealed, ZkPokerError::FlopAlreadyRevealed);

    // Validate card indices
    for card in &cards {
        require!(*card < 52, ZkPokerError::InvalidCardIndex);
    }

    // Verify ZK proof that cards are at correct positions
    let positions = [FLOP_1_POS, FLOP_2_POS, FLOP_3_POS];
    verify_community_cards(
        &ctx.accounts.verifier_program,
        &hand.deck_seed,
        &cards,
        &positions,
        &proof,
    )?;

    // Store revealed flop
    hand.flop = cards;
    hand.flop_revealed = true;

    // Update timestamp
    let clock = Clock::get()?;
    hand.last_action_at = clock.unix_timestamp;

    msg!("Flop revealed: [{}, {}, {}]", cards[0], cards[1], cards[2]);

    Ok(())
}

/// Reveal turn handler (1 card)
pub fn handle_reveal_turn(
    ctx: Context<RevealCommunity>,
    card: u8,
    proof: Vec<u8>,
) -> Result<()> {
    let table = &ctx.accounts.table;
    let hand = &mut ctx.accounts.hand;
    let player = ctx.accounts.player.key();

    // Verify player is at table
    let _seat = table.get_seat(&player).ok_or(ZkPokerError::PlayerNotAtTable)?;

    // Verify stage - must be in Turn stage (waiting for reveal)
    require!(hand.stage == HandStage::Turn, ZkPokerError::InvalidStage);

    // Verify flop was revealed (must reveal in order)
    require!(hand.flop_revealed, ZkPokerError::RevealOutOfOrder);

    // Verify turn not already revealed
    require!(!hand.turn_revealed, ZkPokerError::TurnAlreadyRevealed);

    // Validate card index
    require!(card < 52, ZkPokerError::InvalidCardIndex);

    // Verify ZK proof that card is at correct position
    verify_community_cards(
        &ctx.accounts.verifier_program,
        &hand.deck_seed,
        &[card],
        &[TURN_POS],
        &proof,
    )?;

    // Store revealed turn
    hand.turn = card;
    hand.turn_revealed = true;

    // Update timestamp
    let clock = Clock::get()?;
    hand.last_action_at = clock.unix_timestamp;

    msg!("Turn revealed: {}", card);

    Ok(())
}

/// Reveal river handler (1 card)
pub fn handle_reveal_river(
    ctx: Context<RevealCommunity>,
    card: u8,
    proof: Vec<u8>,
) -> Result<()> {
    let table = &ctx.accounts.table;
    let hand = &mut ctx.accounts.hand;
    let player = ctx.accounts.player.key();

    // Verify player is at table
    let _seat = table.get_seat(&player).ok_or(ZkPokerError::PlayerNotAtTable)?;

    // Verify stage - must be in River stage (waiting for reveal)
    require!(hand.stage == HandStage::River, ZkPokerError::InvalidStage);

    // Verify turn was revealed (must reveal in order)
    require!(hand.turn_revealed, ZkPokerError::RevealOutOfOrder);

    // Verify river not already revealed
    require!(!hand.river_revealed, ZkPokerError::RiverAlreadyRevealed);

    // Validate card index
    require!(card < 52, ZkPokerError::InvalidCardIndex);

    // Verify ZK proof that card is at correct position
    verify_community_cards(
        &ctx.accounts.verifier_program,
        &hand.deck_seed,
        &[card],
        &[RIVER_POS],
        &proof,
    )?;

    // Store revealed river
    hand.river = card;
    hand.river_revealed = true;

    // Update timestamp
    let clock = Clock::get()?;
    hand.last_action_at = clock.unix_timestamp;

    msg!("River revealed: {}", card);

    Ok(())
}
