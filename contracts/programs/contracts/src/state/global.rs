use anchor_lang::prelude::*;

/// Global configuration account for the ZkPoker protocol
/// Seeds: ["global"]
#[account]
pub struct GlobalConfig {
    /// Admin authority who can pause/unpause the protocol
    pub authority: Pubkey,

    /// USDC token mint address
    pub usdc_mint: Pubkey,

    /// ZK Circuit Verifier Program IDs
    pub deck_verifier: Pubkey,      // Hole card commitments
    pub deal_verifier: Pubkey,      // Dealing cards
    pub reveal_verifier: Pubkey,    // Community card reveals
    pub showdown_verifier: Pubkey,  // Hand reveals at showdown
    pub bet_verifier: Pubkey,       // Bet/balance verification

    /// Total number of tables created
    pub table_count: u64,

    /// Emergency pause flag
    pub is_paused: bool,

    /// PDA bump seed
    pub bump: u8,
}

impl GlobalConfig {
    /// Account size for rent calculation
    /// 8 (discriminator) + 32 (authority) + 32 (usdc_mint) + 32*5 (verifiers) + 8 (table_count) + 1 (is_paused) + 1 (bump)
    /// = 8 + 32 + 32 + 160 + 8 + 1 + 1 = 242 bytes
    pub const LEN: usize = 8 + 32 + 32 + 160 + 8 + 1 + 1;

    /// Initialize a new GlobalConfig
    pub fn init(
        &mut self,
        authority: Pubkey,
        usdc_mint: Pubkey,
        deck_verifier: Pubkey,
        deal_verifier: Pubkey,
        reveal_verifier: Pubkey,
        showdown_verifier: Pubkey,
        bet_verifier: Pubkey,
        bump: u8,
    ) {
        self.authority = authority;
        self.usdc_mint = usdc_mint;
        self.deck_verifier = deck_verifier;
        self.deal_verifier = deal_verifier;
        self.reveal_verifier = reveal_verifier;
        self.showdown_verifier = showdown_verifier;
        self.bet_verifier = bet_verifier;
        self.table_count = 0;
        self.is_paused = false;
        self.bump = bump;
    }

    /// Increment table count and return new table ID
    pub fn next_table_id(&mut self) -> u64 {
        let id = self.table_count;
        self.table_count = self.table_count.saturating_add(1);
        id
    }
}
