use anchor_lang::prelude::*;

/// Type of ZK proof being stored in the buffer
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProofType {
    /// DECK circuit - hole card commitments
    Deck = 0,
    /// REVEAL circuit - community card reveals
    Reveal = 1,
    /// SHOWDOWN circuit - hand reveal at showdown
    Showdown = 2,
}

impl Default for ProofType {
    fn default() -> Self {
        ProofType::Deck
    }
}

/// Proof buffer account for storing ZK proofs across multiple transactions
/// Seeds: ["proof_buffer", hand.key(), player.key(), proof_type]
#[account]
pub struct ProofBuffer {
    /// Parent hand public key
    pub hand: Pubkey,

    /// Player who owns this buffer
    pub player: Pubkey,

    /// Type of proof (Deck/Reveal/Showdown)
    pub proof_type: ProofType,

    /// Total expected size of proof data
    pub size: u16,

    /// Bytes uploaded so far
    pub uploaded: u16,

    /// Is the buffer complete (all bytes uploaded)?
    pub complete: bool,

    /// PDA bump seed
    pub bump: u8,

    /// The proof + public witness data (variable length)
    pub data: Vec<u8>,
}

impl ProofBuffer {
    /// Base size without data vector (discriminator + fixed fields)
    pub const BASE_LEN: usize = 8     // discriminator
        + 32                           // hand
        + 32                           // player
        + 1                            // proof_type
        + 2                            // size
        + 2                            // uploaded
        + 1                            // complete
        + 1                            // bump
        + 4;                           // vec length prefix

    /// Calculate full account size for given proof size
    pub fn space(proof_size: u16) -> usize {
        Self::BASE_LEN + proof_size as usize
    }

    /// Initialize a new proof buffer
    pub fn init(
        &mut self,
        hand: Pubkey,
        player: Pubkey,
        proof_type: ProofType,
        size: u16,
        bump: u8,
    ) {
        self.hand = hand;
        self.player = player;
        self.proof_type = proof_type;
        self.size = size;
        self.uploaded = 0;
        self.complete = false;
        self.bump = bump;
        // Data vec is pre-allocated based on account space
        self.data = vec![0u8; size as usize];
    }

    /// Upload a chunk of proof data at given offset
    pub fn upload_chunk(&mut self, offset: u16, chunk: &[u8]) -> Result<()> {
        let start = offset as usize;
        let end = start + chunk.len();

        // Validate bounds
        require!(end <= self.size as usize, ProofBufferError::ChunkOverflow);

        // Copy data
        self.data[start..end].copy_from_slice(chunk);

        // Update uploaded bytes (track furthest point written)
        let new_uploaded = end as u16;
        if new_uploaded > self.uploaded {
            self.uploaded = new_uploaded;
        }

        // Check if complete
        if self.uploaded >= self.size {
            self.complete = true;
        }

        Ok(())
    }

    /// Get the proof data (only valid when complete)
    pub fn get_proof_data(&self) -> Result<&[u8]> {
        require!(self.complete, ProofBufferError::IncompleteBuffer);
        Ok(&self.data)
    }
}

/// Proof buffer specific errors
#[error_code]
pub enum ProofBufferError {
    #[msg("Chunk write would overflow buffer")]
    ChunkOverflow,
    #[msg("Buffer upload not complete")]
    IncompleteBuffer,
}
