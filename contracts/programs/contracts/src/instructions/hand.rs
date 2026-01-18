use anchor_lang::prelude::*;
use solana_keccak_hasher as keccak;
use crate::state::{GlobalConfig, Table, Hand, TableStatus, HandStage};
use crate::errors::ZkPokerError;
use crate::constants::{GLOBAL_SEED, TABLE_SEED, HAND_SEED};
use crate::utils::verify_hole_card_commitments;

/// Start a new hand
#[derive(Accounts)]
pub struct StartHand<'info> {
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

    #[account(
        init,
        payer = player,
        space = Hand::LEN,
        seeds = [HAND_SEED, table.key().as_ref(), &table.hands_played.to_le_bytes()],
        bump
    )]
    pub hand: Account<'info, Hand>,

    pub system_program: Program<'info, System>,
}

/// Commit shuffle seed
#[derive(Accounts)]
pub struct CommitSeed<'info> {
    pub player: Signer<'info>,

    #[account(
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

/// Reveal shuffle seed
#[derive(Accounts)]
pub struct RevealSeed<'info> {
    pub player: Signer<'info>,

    #[account(
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

/// Commit hole cards (with ZK proof)
#[derive(Accounts)]
pub struct CommitHoleCards<'info> {
    pub player: Signer<'info>,

    #[account(
        seeds = [GLOBAL_SEED],
        bump = global_config.bump
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(
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

    /// CHECK: DECK verifier program - verified in verification function
    #[account(constraint = verifier_program.key() == global_config.deck_verifier @ ZkPokerError::ProofVerificationFailed)]
    pub verifier_program: AccountInfo<'info>,
}

/// Timeout claim
#[derive(Accounts)]
pub struct Timeout<'info> {
    pub caller: Signer<'info>,

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

/// Start hand handler
pub fn handle_start_hand(ctx: Context<StartHand>) -> Result<()> {
    let table = &mut ctx.accounts.table;
    let hand = &mut ctx.accounts.hand;
    let player = ctx.accounts.player.key();

    // Verify player is at table
    let _seat = table.get_seat(&player).ok_or(ZkPokerError::PlayerNotAtTable)?;

    // Verify table is full and between hands
    require!(table.is_full(), ZkPokerError::NotEnoughPlayers);
    require!(table.status == TableStatus::Between, ZkPokerError::InvalidStage);

    // Verify both players have enough chips for blinds
    let sb_seat = table.small_blind_seat();
    let bb_seat = table.big_blind_seat();
    require!(
        table.get_chips(sb_seat) >= table.small_blind,
        ZkPokerError::InsufficientChips
    );
    require!(
        table.get_chips(bb_seat) >= table.big_blind,
        ZkPokerError::InsufficientChips
    );

    let clock = Clock::get()?;
    let bump = ctx.bumps.hand;
    let hand_number = table.hands_played;

    // Initialize hand
    hand.init(table.key(), hand_number, clock.unix_timestamp, bump);

    // Post blinds (copy values first to avoid borrow conflict)
    let small_blind = table.small_blind;
    let big_blind = table.big_blind;
    let sb_amount = table.remove_chips(sb_seat, small_blind);
    let bb_amount = table.remove_chips(bb_seat, big_blind);

    hand.add_bet(sb_seat, sb_amount);
    hand.add_bet(bb_seat, bb_amount);
    hand.current_bet = bb_amount;

    // Set action to small blind player first for seed commit
    hand.action_on = 0; // Either player can commit first

    // Update table state
    table.status = TableStatus::Playing;
    table.current_hand = Some(hand.key());

    msg!("Hand {} started", hand_number);
    msg!("Small blind: {} from seat {}", sb_amount, sb_seat);
    msg!("Big blind: {} from seat {}", bb_amount, bb_seat);
    msg!("Pot: {}", hand.pot);

    Ok(())
}

/// Commit seed handler
pub fn handle_commit_seed(ctx: Context<CommitSeed>, seed_hash: [u8; 32]) -> Result<()> {
    let table = &ctx.accounts.table;
    let hand = &mut ctx.accounts.hand;
    let player = ctx.accounts.player.key();

    // Verify player is at table
    let seat = table.get_seat(&player).ok_or(ZkPokerError::PlayerNotAtTable)?;

    // Verify stage
    require!(hand.stage == HandStage::SeedCommit, ZkPokerError::InvalidStage);

    // Check if player already committed
    let already_committed = match seat {
        0 => hand.p1_seed_committed,
        1 => hand.p2_seed_committed,
        _ => return Err(ZkPokerError::PlayerNotAtTable.into()),
    };
    require!(!already_committed, ZkPokerError::SeedAlreadyCommitted);

    // Store commitment
    match seat {
        0 => {
            hand.seed_commit_one = seed_hash;
            hand.p1_seed_committed = true;
        }
        1 => {
            hand.seed_commit_two = seed_hash;
            hand.p2_seed_committed = true;
        }
        _ => {}
    }

    // Update timestamp
    let clock = Clock::get()?;
    hand.last_action_at = clock.unix_timestamp;

    msg!("Player {} (seat {}) committed seed", player, seat);

    // Check if both committed, advance stage
    if hand.p1_seed_committed && hand.p2_seed_committed {
        hand.stage = HandStage::SeedReveal;
        msg!("Both seeds committed, advancing to SeedReveal");
    }

    Ok(())
}

/// Reveal seed handler
pub fn handle_reveal_seed(ctx: Context<RevealSeed>, seed: [u8; 32]) -> Result<()> {
    let table = &ctx.accounts.table;
    let hand = &mut ctx.accounts.hand;
    let player = ctx.accounts.player.key();

    // Verify player is at table
    let seat = table.get_seat(&player).ok_or(ZkPokerError::PlayerNotAtTable)?;

    // Verify stage
    require!(hand.stage == HandStage::SeedReveal, ZkPokerError::InvalidStage);

    // Check if player has committed
    let committed = match seat {
        0 => hand.p1_seed_committed,
        1 => hand.p2_seed_committed,
        _ => return Err(ZkPokerError::PlayerNotAtTable.into()),
    };
    require!(committed, ZkPokerError::SeedNotCommitted);

    // Check if player already revealed
    let already_revealed = match seat {
        0 => hand.p1_seed_revealed,
        1 => hand.p2_seed_revealed,
        _ => return Err(ZkPokerError::PlayerNotAtTable.into()),
    };
    require!(!already_revealed, ZkPokerError::SeedAlreadyCommitted);

    // Verify hash matches commitment
    let computed_hash = keccak::hashv(&[&seed]);
    let expected_hash = match seat {
        0 => hand.seed_commit_one,
        1 => hand.seed_commit_two,
        _ => return Err(ZkPokerError::PlayerNotAtTable.into()),
    };

    require!(
        computed_hash.to_bytes() == expected_hash,
        ZkPokerError::InvalidSeedReveal
    );

    // Store revealed seed
    match seat {
        0 => {
            hand.seed_one = seed;
            hand.p1_seed_revealed = true;
        }
        1 => {
            hand.seed_two = seed;
            hand.p2_seed_revealed = true;
        }
        _ => {}
    }

    // Update timestamp
    let clock = Clock::get()?;
    hand.last_action_at = clock.unix_timestamp;

    msg!("Player {} (seat {}) revealed seed", player, seat);

    // Check if both revealed, compute deck seed and advance stage
    if hand.p1_seed_revealed && hand.p2_seed_revealed {
        // Compute deck_seed = keccak(seed_1 || seed_2)
        let deck_seed = keccak::hashv(&[&hand.seed_one, &hand.seed_two]);
        hand.deck_seed = deck_seed.to_bytes();

        hand.stage = HandStage::CardCommit;
        msg!("Both seeds revealed, deck_seed computed");
        msg!("Advancing to CardCommit stage");
    }

    Ok(())
}

/// Commit hole cards handler (with ZK proof verification)
pub fn handle_commit_hole_cards(
    ctx: Context<CommitHoleCards>,
    commitments: [[u8; 32]; 2],
    proof: Vec<u8>,
) -> Result<()> {
    let table = &ctx.accounts.table;
    let hand = &mut ctx.accounts.hand;
    let player = ctx.accounts.player.key();

    // Verify player is at table
    let seat = table.get_seat(&player).ok_or(ZkPokerError::PlayerNotAtTable)?;

    // Verify stage
    require!(hand.stage == HandStage::CardCommit, ZkPokerError::InvalidStage);

    // Check if player already committed
    let already_committed = match seat {
        0 => hand.p1_cards_committed,
        1 => hand.p2_cards_committed,
        _ => return Err(ZkPokerError::PlayerNotAtTable.into()),
    };
    require!(!already_committed, ZkPokerError::CardsAlreadyCommitted);

    // Verify ZK proof via CPI to DECK verifier program
    // The proof verifies:
    // 1. Cards are at correct positions (0,1 for P1 or 2,3 for P2)
    // 2. Cards derived from deck_seed correctly
    // 3. Commitments are hash(card, salt)
    verify_hole_card_commitments(
        &ctx.accounts.verifier_program,
        &hand.deck_seed,
        seat,
        &commitments,
        &proof,
    )?;

    msg!("✓ Hole card commitments verified for seat {}", seat);

    // Store verified commitments
    match seat {
        0 => {
            hand.p1_hole_commits = commitments;
            hand.p1_cards_committed = true;
        }
        1 => {
            hand.p2_hole_commits = commitments;
            hand.p2_cards_committed = true;
        }
        _ => {}
    }

    // Update timestamp
    let clock = Clock::get()?;
    hand.last_action_at = clock.unix_timestamp;

    msg!("Player {} (seat {}) committed hole cards", player, seat);

    // Check if both committed, advance to preflop
    if hand.p1_cards_committed && hand.p2_cards_committed {
        hand.stage = HandStage::Preflop;

        // Reset street betting state
        hand.reset_street();

        // In heads-up, small blind (button) acts first preflop
        // But blinds are already posted, so action is on SB to call/raise/fold
        let sb_seat = table.small_blind_seat();
        hand.action_on = sb_seat;

        // Restore the bet amounts (blinds were already posted in start_hand)
        hand.p1_bet_this_street = if sb_seat == 0 { table.small_blind } else { table.big_blind };
        hand.p2_bet_this_street = if sb_seat == 0 { table.big_blind } else { table.small_blind };
        hand.current_bet = table.big_blind;

        msg!("Both cards committed, advancing to Preflop");
        msg!("Action on seat {}", hand.action_on);
    }

    Ok(())
}

/// Timeout handler
pub fn handle_timeout(ctx: Context<Timeout>) -> Result<()> {
    let table = &mut ctx.accounts.table;
    let hand = &mut ctx.accounts.hand;

    // Verify hand is not complete
    require!(hand.stage != HandStage::Complete, ZkPokerError::HandAlreadyComplete);

    // Get timeout setting
    let timeout = table.action_timeout;

    // Check if timeout has occurred
    let clock = Clock::get()?;
    let elapsed = clock.unix_timestamp - hand.last_action_at;
    require!(elapsed > timeout, ZkPokerError::NoTimeout);

    // Determine who timed out based on stage
    let timed_out_seat = match hand.stage {
        HandStage::SeedCommit => {
            // Whoever hasn't committed
            if !hand.p1_seed_committed {
                0
            } else if !hand.p2_seed_committed {
                1
            } else {
                return Err(ZkPokerError::NoTimeout.into());
            }
        }
        HandStage::SeedReveal => {
            // Whoever hasn't revealed
            if !hand.p1_seed_revealed {
                0
            } else if !hand.p2_seed_revealed {
                1
            } else {
                return Err(ZkPokerError::NoTimeout.into());
            }
        }
        HandStage::CardCommit => {
            // Whoever hasn't committed cards
            if !hand.p1_cards_committed {
                0
            } else if !hand.p2_cards_committed {
                1
            } else {
                return Err(ZkPokerError::NoTimeout.into());
            }
        }
        HandStage::Preflop | HandStage::Flop | HandStage::Turn | HandStage::River => {
            // Whoever's turn it is
            hand.action_on
        }
        HandStage::Showdown => {
            // Whoever hasn't revealed
            if !hand.p1_revealed {
                0
            } else if !hand.p2_revealed {
                1
            } else {
                return Err(ZkPokerError::NoTimeout.into());
            }
        }
        HandStage::Complete => {
            return Err(ZkPokerError::HandAlreadyComplete.into());
        }
    };

    // Award pot to non-timed-out player
    let winner_seat = hand.other_seat(timed_out_seat);
    hand.winner = winner_seat;
    hand.set_folded(timed_out_seat);

    // Transfer pot to winner
    table.add_chips(winner_seat, hand.pot);
    hand.pot = 0;

    // Complete the hand
    hand.stage = HandStage::Complete;
    table.status = TableStatus::Between;
    table.current_hand = None;
    table.increment_hands_played();
    table.rotate_button();

    msg!("Seat {} timed out", timed_out_seat);
    msg!("Seat {} wins pot", winner_seat);

    Ok(())
}
