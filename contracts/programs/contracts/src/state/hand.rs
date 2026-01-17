use anchor_lang::prelude::*;

/// Hand stage enum representing the current phase of the hand
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum HandStage {
    /// Waiting for both players to commit shuffle seeds
    SeedCommit = 0,
    /// Waiting for both players to reveal shuffle seeds
    SeedReveal = 1,
    /// Waiting for both players to commit hole cards
    CardCommit = 2,
    /// Pre-flop betting round
    Preflop = 3,
    /// Flop betting round
    Flop = 4,
    /// Turn betting round
    Turn = 5,
    /// River betting round
    River = 6,
    /// Showdown - players reveal hands
    Showdown = 7,
    /// Hand complete
    Complete = 8,
}

impl Default for HandStage {
    fn default() -> Self {
        HandStage::SeedCommit
    }
}

impl HandStage {
    /// Check if this is a betting stage
    pub fn is_betting_stage(&self) -> bool {
        matches!(self, HandStage::Preflop | HandStage::Flop | HandStage::Turn | HandStage::River)
    }

    /// Get the next betting stage (for street transitions)
    pub fn next_betting_stage(&self) -> Option<HandStage> {
        match self {
            HandStage::Preflop => Some(HandStage::Flop),
            HandStage::Flop => Some(HandStage::Turn),
            HandStage::Turn => Some(HandStage::River),
            HandStage::River => Some(HandStage::Showdown),
            _ => None,
        }
    }
}

/// Hand account representing a single poker hand
/// Seeds: ["hand", table.key(), hand_number.to_le_bytes()]
#[account]
pub struct Hand {
    /// Parent table public key
    pub table: Pubkey,

    /// Sequential hand number at this table
    pub hand_number: u64,

    /// Current hand stage
    pub stage: HandStage,

    // ============================================
    // PHASE 1: Shuffle Seeds
    // ============================================

    /// Player 1 seed commitment (hash of seed)
    pub seed_commit_one: [u8; 32],

    /// Player 2 seed commitment (hash of seed)
    pub seed_commit_two: [u8; 32],

    /// Player 1 revealed seed (set after reveal)
    pub seed_one: [u8; 32],

    /// Player 2 revealed seed (set after reveal)
    pub seed_two: [u8; 32],

    /// Combined deck seed: hash(seed_1 || seed_2)
    pub deck_seed: [u8; 32],

    /// Tracking: has player 1 committed seed?
    pub p1_seed_committed: bool,

    /// Tracking: has player 2 committed seed?
    pub p2_seed_committed: bool,

    /// Tracking: has player 1 revealed seed?
    pub p1_seed_revealed: bool,

    /// Tracking: has player 2 revealed seed?
    pub p2_seed_revealed: bool,

    // ============================================
    // PHASE 2: Hole Card Commitments
    // ============================================

    /// Player 1 hole card commitments [commit_card1, commit_card2]
    pub p1_hole_commits: [[u8; 32]; 2],

    /// Player 2 hole card commitments [commit_card1, commit_card2]
    pub p2_hole_commits: [[u8; 32]; 2],

    /// Tracking: has player 1 committed cards?
    pub p1_cards_committed: bool,

    /// Tracking: has player 2 committed cards?
    pub p2_cards_committed: bool,

    // ============================================
    // PHASE 4: Community Cards (ZK revealed)
    // ============================================

    /// Flop cards (3 cards, indices 0-51), 255 = not revealed
    pub flop: [u8; 3],

    /// Turn card (index 0-51), 255 = not revealed
    pub turn: u8,

    /// River card (index 0-51), 255 = not revealed
    pub river: u8,

    /// Tracking reveal status
    pub flop_revealed: bool,
    pub turn_revealed: bool,
    pub river_revealed: bool,

    // ============================================
    // PHASE 5: Showdown
    // ============================================

    /// Player 1 hand rank (ZK verified composite score)
    pub p1_hand_rank: u64,

    /// Player 2 hand rank (ZK verified composite score)
    pub p2_hand_rank: u64,

    /// Tracking: has player 1 revealed hand?
    pub p1_revealed: bool,

    /// Tracking: has player 2 revealed hand?
    pub p2_revealed: bool,

    /// Winner seat (0, 1, or 2 for split)
    pub winner: u8,

    /// Has pot been claimed?
    pub pot_claimed: bool,

    // ============================================
    // BETTING STATE
    // ============================================

    /// Total pot amount
    pub pot: u64,

    /// Current bet amount to match
    pub current_bet: u64,

    /// Player 1 bet this street
    pub p1_bet_this_street: u64,

    /// Player 2 bet this street
    pub p2_bet_this_street: u64,

    /// Player 1 total bet this hand
    pub p1_total_bet: u64,

    /// Player 2 total bet this hand
    pub p2_total_bet: u64,

    // ============================================
    // GAME STATE
    // ============================================

    /// Whose turn (0 = player_one, 1 = player_two)
    pub action_on: u8,

    /// Last action timestamp (for timeout tracking)
    pub last_action_at: i64,

    /// Last aggressor (who bet/raised last)
    pub last_aggressor: u8,

    /// Has player 1 folded?
    pub p1_folded: bool,

    /// Has player 2 folded?
    pub p2_folded: bool,

    /// Is player 1 all-in?
    pub p1_all_in: bool,

    /// Is player 2 all-in?
    pub p2_all_in: bool,

    /// Has player 1 acted this street?
    pub p1_acted_this_street: bool,

    /// Has player 2 acted this street?
    pub p2_acted_this_street: bool,

    /// PDA bump seed
    pub bump: u8,
}

impl Hand {
    /// Account size for rent calculation
    pub const LEN: usize = 8     // discriminator
        + 32                      // table
        + 8                       // hand_number
        + 1                       // stage
        + 32                      // seed_commit_one
        + 32                      // seed_commit_two
        + 32                      // seed_one
        + 32                      // seed_two
        + 32                      // deck_seed
        + 1                       // p1_seed_committed
        + 1                       // p2_seed_committed
        + 1                       // p1_seed_revealed
        + 1                       // p2_seed_revealed
        + 64                      // p1_hole_commits
        + 64                      // p2_hole_commits
        + 1                       // p1_cards_committed
        + 1                       // p2_cards_committed
        + 3                       // flop
        + 1                       // turn
        + 1                       // river
        + 1                       // flop_revealed
        + 1                       // turn_revealed
        + 1                       // river_revealed
        + 8                       // p1_hand_rank
        + 8                       // p2_hand_rank
        + 1                       // p1_revealed
        + 1                       // p2_revealed
        + 1                       // winner
        + 1                       // pot_claimed
        + 8                       // pot
        + 8                       // current_bet
        + 8                       // p1_bet_this_street
        + 8                       // p2_bet_this_street
        + 8                       // p1_total_bet
        + 8                       // p2_total_bet
        + 1                       // action_on
        + 8                       // last_action_at
        + 1                       // last_aggressor
        + 1                       // p1_folded
        + 1                       // p2_folded
        + 1                       // p1_all_in
        + 1                       // p2_all_in
        + 1                       // p1_acted_this_street
        + 1                       // p2_acted_this_street
        + 1;                      // bump

    /// Initialize a new hand
    pub fn init(
        &mut self,
        table: Pubkey,
        hand_number: u64,
        timestamp: i64,
        bump: u8,
    ) {
        self.table = table;
        self.hand_number = hand_number;
        self.stage = HandStage::SeedCommit;

        // Initialize seeds as zeros
        self.seed_commit_one = [0u8; 32];
        self.seed_commit_two = [0u8; 32];
        self.seed_one = [0u8; 32];
        self.seed_two = [0u8; 32];
        self.deck_seed = [0u8; 32];
        self.p1_seed_committed = false;
        self.p2_seed_committed = false;
        self.p1_seed_revealed = false;
        self.p2_seed_revealed = false;

        // Initialize card commits as zeros
        self.p1_hole_commits = [[0u8; 32]; 2];
        self.p2_hole_commits = [[0u8; 32]; 2];
        self.p1_cards_committed = false;
        self.p2_cards_committed = false;

        // Initialize community cards as not revealed (255)
        self.flop = [255u8; 3];
        self.turn = 255;
        self.river = 255;
        self.flop_revealed = false;
        self.turn_revealed = false;
        self.river_revealed = false;

        // Initialize showdown
        self.p1_hand_rank = 0;
        self.p2_hand_rank = 0;
        self.p1_revealed = false;
        self.p2_revealed = false;
        self.winner = 255; // No winner yet
        self.pot_claimed = false;

        // Initialize betting
        self.pot = 0;
        self.current_bet = 0;
        self.p1_bet_this_street = 0;
        self.p2_bet_this_street = 0;
        self.p1_total_bet = 0;
        self.p2_total_bet = 0;

        // Initialize game state
        self.action_on = 0;
        self.last_action_at = timestamp;
        self.last_aggressor = 255; // No aggressor yet
        self.p1_folded = false;
        self.p2_folded = false;
        self.p1_all_in = false;
        self.p2_all_in = false;
        self.p1_acted_this_street = false;
        self.p2_acted_this_street = false;

        self.bump = bump;
    }

    /// Check if player has folded
    pub fn has_folded(&self, seat: u8) -> bool {
        match seat {
            0 => self.p1_folded,
            1 => self.p2_folded,
            _ => true,
        }
    }

    /// Set player folded status
    pub fn set_folded(&mut self, seat: u8) {
        match seat {
            0 => self.p1_folded = true,
            1 => self.p2_folded = true,
            _ => {}
        }
    }

    /// Check if player is all-in
    pub fn is_all_in(&self, seat: u8) -> bool {
        match seat {
            0 => self.p1_all_in,
            1 => self.p2_all_in,
            _ => false,
        }
    }

    /// Set player all-in status
    pub fn set_all_in(&mut self, seat: u8) {
        match seat {
            0 => self.p1_all_in = true,
            1 => self.p2_all_in = true,
            _ => {}
        }
    }

    /// Get player's bet this street
    pub fn get_bet_this_street(&self, seat: u8) -> u64 {
        match seat {
            0 => self.p1_bet_this_street,
            1 => self.p2_bet_this_street,
            _ => 0,
        }
    }

    /// Add to player's bet this street
    pub fn add_bet(&mut self, seat: u8, amount: u64) {
        match seat {
            0 => {
                self.p1_bet_this_street = self.p1_bet_this_street.saturating_add(amount);
                self.p1_total_bet = self.p1_total_bet.saturating_add(amount);
            }
            1 => {
                self.p2_bet_this_street = self.p2_bet_this_street.saturating_add(amount);
                self.p2_total_bet = self.p2_total_bet.saturating_add(amount);
            }
            _ => {}
        }
        self.pot = self.pot.saturating_add(amount);
    }

    /// Check if player has acted this street
    pub fn has_acted_this_street(&self, seat: u8) -> bool {
        match seat {
            0 => self.p1_acted_this_street,
            1 => self.p2_acted_this_street,
            _ => false,
        }
    }

    /// Set player acted this street
    pub fn set_acted_this_street(&mut self, seat: u8) {
        match seat {
            0 => self.p1_acted_this_street = true,
            1 => self.p2_acted_this_street = true,
            _ => {}
        }
    }

    /// Reset street state for new betting round
    pub fn reset_street(&mut self) {
        self.p1_bet_this_street = 0;
        self.p2_bet_this_street = 0;
        self.current_bet = 0;
        self.p1_acted_this_street = false;
        self.p2_acted_this_street = false;
        self.last_aggressor = 255;
    }

    /// Get the other seat
    pub fn other_seat(&self, seat: u8) -> u8 {
        if seat == 0 { 1 } else { 0 }
    }

    /// Switch action to other player
    pub fn switch_action(&mut self) {
        self.action_on = self.other_seat(self.action_on);
    }

    /// Check if betting round is complete
    pub fn is_betting_complete(&self) -> bool {
        // If someone folded, betting is complete
        if self.p1_folded || self.p2_folded {
            return true;
        }

        // If both players are all-in, betting is complete
        if self.p1_all_in && self.p2_all_in {
            return true;
        }

        // Both must have acted and bets must be equal
        let both_acted = self.p1_acted_this_street && self.p2_acted_this_street;
        let bets_equal = self.p1_bet_this_street == self.p2_bet_this_street;

        both_acted && bets_equal
    }

    /// Count remaining players (not folded)
    pub fn remaining_players(&self) -> u8 {
        let mut count = 0;
        if !self.p1_folded {
            count += 1;
        }
        if !self.p2_folded {
            count += 1;
        }
        count
    }

    /// Get the non-folded player seat (only valid if exactly one player folded)
    pub fn non_folded_seat(&self) -> Option<u8> {
        if self.p1_folded && !self.p2_folded {
            Some(1)
        } else if !self.p1_folded && self.p2_folded {
            Some(0)
        } else {
            None
        }
    }
}
