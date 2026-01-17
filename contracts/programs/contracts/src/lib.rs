use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod state;
pub mod instructions;
pub mod utils;

use instructions::*;

declare_id!("GnDHa3pfhiqEG5xVTjtnTYue33ceX6disU8F2YJymqYr");

#[program]
pub mod contracts {
    use super::*;

    // ============================================
    // ADMIN INSTRUCTIONS
    // ============================================

    /// Initialize the ZkPoker protocol
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::admin::handle_initialize(ctx)
    }

    /// Pause the protocol (emergency)
    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        instructions::admin::handle_pause(ctx)
    }

    /// Unpause the protocol
    pub fn unpause(ctx: Context<Unpause>) -> Result<()> {
        instructions::admin::handle_unpause(ctx)
    }

    // ============================================
    // TABLE INSTRUCTIONS
    // ============================================

    /// Create a new poker table
    pub fn create_table(
        ctx: Context<CreateTable>,
        small_blind: u64,
        big_blind: u64,
        min_buy_in: u64,
        max_buy_in: u64,
        action_timeout: Option<i64>,
    ) -> Result<()> {
        instructions::table::handle_create_table(
            ctx,
            small_blind,
            big_blind,
            min_buy_in,
            max_buy_in,
            action_timeout,
        )
    }

    /// Join a table with USDC buy-in
    pub fn join_table(ctx: Context<JoinTable>, buy_in_amount: u64) -> Result<()> {
        instructions::table::handle_join_table(ctx, buy_in_amount)
    }

    /// Leave a table and cash out
    pub fn leave_table(ctx: Context<LeaveTable>) -> Result<()> {
        instructions::table::handle_leave_table(ctx)
    }

    /// Add chips to your stack
    pub fn add_chips(ctx: Context<AddChips>, amount: u64) -> Result<()> {
        instructions::table::handle_add_chips(ctx, amount)
    }

    // ============================================
    // HAND INSTRUCTIONS
    // ============================================

    /// Start a new hand
    pub fn start_hand(ctx: Context<StartHand>) -> Result<()> {
        instructions::hand::handle_start_hand(ctx)
    }

    /// Commit shuffle seed hash
    pub fn commit_seed(ctx: Context<CommitSeed>, seed_hash: [u8; 32]) -> Result<()> {
        instructions::hand::handle_commit_seed(ctx, seed_hash)
    }

    /// Reveal shuffle seed
    pub fn reveal_seed(ctx: Context<RevealSeed>, seed: [u8; 32]) -> Result<()> {
        instructions::hand::handle_reveal_seed(ctx, seed)
    }

    /// Commit hole cards with ZK proof
    pub fn commit_hole_cards(
        ctx: Context<CommitHoleCards>,
        commitments: [[u8; 32]; 2],
        proof: Vec<u8>,
    ) -> Result<()> {
        instructions::hand::handle_commit_hole_cards(ctx, commitments, proof)
    }

    /// Claim win due to opponent timeout
    pub fn timeout(ctx: Context<Timeout>) -> Result<()> {
        instructions::hand::handle_timeout(ctx)
    }

    // ============================================
    // BETTING INSTRUCTIONS
    // ============================================

    /// Check (pass without betting)
    pub fn check(ctx: Context<BettingAction>) -> Result<()> {
        instructions::betting::handle_check(ctx)
    }

    /// Bet (open betting)
    pub fn bet(ctx: Context<BettingAction>, amount: u64) -> Result<()> {
        instructions::betting::handle_bet(ctx, amount)
    }

    /// Call (match current bet)
    pub fn call(ctx: Context<BettingAction>) -> Result<()> {
        instructions::betting::handle_call(ctx)
    }

    /// Raise to a total amount
    pub fn raise_to(ctx: Context<BettingAction>, amount: u64) -> Result<()> {
        instructions::betting::handle_raise_to(ctx, amount)
    }

    /// Fold (surrender hand)
    pub fn fold(ctx: Context<BettingAction>) -> Result<()> {
        instructions::betting::handle_fold(ctx)
    }

    /// Go all-in
    pub fn all_in(ctx: Context<BettingAction>) -> Result<()> {
        instructions::betting::handle_all_in(ctx)
    }

    // ============================================
    // REVEAL INSTRUCTIONS
    // ============================================

    /// Reveal flop cards with ZK proof
    pub fn reveal_flop(
        ctx: Context<RevealCommunity>,
        cards: [u8; 3],
        proof: Vec<u8>,
    ) -> Result<()> {
        instructions::reveal::handle_reveal_flop(ctx, cards, proof)
    }

    /// Reveal turn card with ZK proof
    pub fn reveal_turn(
        ctx: Context<RevealCommunity>,
        card: u8,
        proof: Vec<u8>,
    ) -> Result<()> {
        instructions::reveal::handle_reveal_turn(ctx, card, proof)
    }

    /// Reveal river card with ZK proof
    pub fn reveal_river(
        ctx: Context<RevealCommunity>,
        card: u8,
        proof: Vec<u8>,
    ) -> Result<()> {
        instructions::reveal::handle_reveal_river(ctx, card, proof)
    }

    // ============================================
    // SHOWDOWN INSTRUCTIONS
    // ============================================

    /// Reveal hand at showdown with ZK proof
    pub fn reveal_hand(
        ctx: Context<RevealHand>,
        hand_rank: u64,
        proof: Vec<u8>,
    ) -> Result<()> {
        instructions::showdown::handle_reveal_hand(ctx, hand_rank, proof)
    }

    /// Claim the pot after winning
    pub fn claim_pot(ctx: Context<ClaimPot>) -> Result<()> {
        instructions::showdown::handle_claim_pot(ctx)
    }
}
