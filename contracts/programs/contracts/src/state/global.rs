use anchor_lang::prelude::*;

/// Global configuration account for the ZkPoker protocol
/// Seeds: ["global"]
#[account]
pub struct GlobalConfig {
    /// Admin authority who can pause/unpause the protocol
    pub authority: Pubkey,

    /// USDC token mint address
    pub usdc_mint: Pubkey,

    /// ZK verifier program ID
    pub verifier_program: Pubkey,

    /// Total number of tables created
    pub table_count: u64,

    /// Emergency pause flag
    pub is_paused: bool,

    /// PDA bump seed
    pub bump: u8,
}

impl GlobalConfig {
    /// Account size for rent calculation
    /// 8 (discriminator) + 32 + 32 + 32 + 8 + 1 + 1 = 114 bytes
    pub const LEN: usize = 8 + 32 + 32 + 32 + 8 + 1 + 1;

    /// Initialize a new GlobalConfig
    pub fn init(
        &mut self,
        authority: Pubkey,
        usdc_mint: Pubkey,
        verifier_program: Pubkey,
        bump: u8,
    ) {
        self.authority = authority;
        self.usdc_mint = usdc_mint;
        self.verifier_program = verifier_program;
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
