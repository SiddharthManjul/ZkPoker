use anchor_lang::prelude::*;

/// Table status enum
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TableStatus {
    /// Waiting for second player to join
    Waiting = 0,
    /// Hand in progress
    Playing = 1,
    /// Between hands, ready to start new hand
    Between = 2,
}

impl Default for TableStatus {
    fn default() -> Self {
        TableStatus::Waiting
    }
}

/// Table account representing a poker table
/// Seeds: ["table", table_id.to_le_bytes()]
#[account]
pub struct Table {
    /// Unique table identifier
    pub table_id: u64,

    /// Small blind amount (in USDC base units)
    pub small_blind: u64,

    /// Big blind amount (in USDC base units)
    pub big_blind: u64,

    /// Minimum buy-in amount
    pub min_buy_in: u64,

    /// Maximum buy-in amount
    pub max_buy_in: u64,

    /// Action timeout in seconds
    pub action_timeout: i64,

    /// Player in seat 0 (None if empty)
    pub player_one: Option<Pubkey>,

    /// Player in seat 1 (None if empty)
    pub player_two: Option<Pubkey>,

    /// Player one chip stack
    pub player_one_chips: u64,

    /// Player two chip stack
    pub player_two_chips: u64,

    /// Dealer button position (0 or 1)
    pub button: u8,

    /// Current table status
    pub status: TableStatus,

    /// Current active hand account (None if between hands)
    pub current_hand: Option<Pubkey>,

    /// Total hands played at this table
    pub hands_played: u64,

    /// Table creation timestamp
    pub created_at: i64,

    /// PDA bump seed
    pub bump: u8,
}

impl Table {
    /// Account size for rent calculation
    /// 8 (discriminator) + 8 + 8 + 8 + 8 + 8 + 8 + 33 + 33 + 8 + 8 + 1 + 1 + 33 + 8 + 8 + 1 = 180 bytes
    pub const LEN: usize = 8 + 8 + 8 + 8 + 8 + 8 + 8 + 33 + 33 + 8 + 8 + 1 + 1 + 33 + 8 + 8 + 1;

    /// Initialize a new table
    pub fn init(
        &mut self,
        table_id: u64,
        small_blind: u64,
        big_blind: u64,
        min_buy_in: u64,
        max_buy_in: u64,
        action_timeout: i64,
        created_at: i64,
        bump: u8,
    ) {
        self.table_id = table_id;
        self.small_blind = small_blind;
        self.big_blind = big_blind;
        self.min_buy_in = min_buy_in;
        self.max_buy_in = max_buy_in;
        self.action_timeout = action_timeout;
        self.player_one = None;
        self.player_two = None;
        self.player_one_chips = 0;
        self.player_two_chips = 0;
        self.button = 0;
        self.status = TableStatus::Waiting;
        self.current_hand = None;
        self.hands_played = 0;
        self.created_at = created_at;
        self.bump = bump;
    }

    /// Check if table has an empty seat
    pub fn has_empty_seat(&self) -> bool {
        self.player_one.is_none() || self.player_two.is_none()
    }

    /// Check if table is full (2 players)
    pub fn is_full(&self) -> bool {
        self.player_one.is_some() && self.player_two.is_some()
    }

    /// Get the seat number for a player (0, 1, or None if not at table)
    pub fn get_seat(&self, player: &Pubkey) -> Option<u8> {
        if self.player_one.as_ref() == Some(player) {
            Some(0)
        } else if self.player_two.as_ref() == Some(player) {
            Some(1)
        } else {
            None
        }
    }

    /// Get player chips by seat number
    pub fn get_chips(&self, seat: u8) -> u64 {
        match seat {
            0 => self.player_one_chips,
            1 => self.player_two_chips,
            _ => 0,
        }
    }

    /// Set player chips by seat number
    pub fn set_chips(&mut self, seat: u8, chips: u64) {
        match seat {
            0 => self.player_one_chips = chips,
            1 => self.player_two_chips = chips,
            _ => {}
        }
    }

    /// Add chips to a player's stack
    pub fn add_chips(&mut self, seat: u8, amount: u64) {
        let current = self.get_chips(seat);
        self.set_chips(seat, current.saturating_add(amount));
    }

    /// Remove chips from a player's stack (returns actual amount removed)
    pub fn remove_chips(&mut self, seat: u8, amount: u64) -> u64 {
        let current = self.get_chips(seat);
        let removed = amount.min(current);
        self.set_chips(seat, current.saturating_sub(removed));
        removed
    }

    /// Get the other player's seat
    pub fn other_seat(&self, seat: u8) -> u8 {
        if seat == 0 { 1 } else { 0 }
    }

    /// Rotate the dealer button
    pub fn rotate_button(&mut self) {
        self.button = self.other_seat(self.button);
    }

    /// Get small blind seat (button in heads-up)
    pub fn small_blind_seat(&self) -> u8 {
        self.button
    }

    /// Get big blind seat (opposite of button in heads-up)
    pub fn big_blind_seat(&self) -> u8 {
        self.other_seat(self.button)
    }

    /// Increment hands played counter
    pub fn increment_hands_played(&mut self) {
        self.hands_played = self.hands_played.saturating_add(1);
    }
}
