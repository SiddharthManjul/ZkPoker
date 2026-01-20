use anchor_lang::prelude::*;

#[error_code]
pub enum ZkPokerError {
    // ============================================
    // Table Errors (6000-6099)
    // ============================================
    #[msg("Table is full")]
    TableFull,

    #[msg("Table requires two players to start")]
    NotEnoughPlayers,

    #[msg("Player not at this table")]
    PlayerNotAtTable,

    #[msg("Invalid buy-in amount")]
    InvalidBuyIn,

    #[msg("Cannot leave during active hand")]
    HandInProgress,

    #[msg("Player already at table")]
    PlayerAlreadyAtTable,

    #[msg("Invalid table configuration")]
    InvalidTableConfig,

    // ============================================
    // Hand Errors (6100-6199)
    // ============================================
    #[msg("Not your turn")]
    NotYourTurn,

    #[msg("Invalid stage for this action")]
    InvalidStage,

    #[msg("Already committed seed")]
    SeedAlreadyCommitted,

    #[msg("Seed not yet committed")]
    SeedNotCommitted,

    #[msg("Invalid seed reveal - hash mismatch")]
    InvalidSeedReveal,

    #[msg("Cards already committed")]
    CardsAlreadyCommitted,

    #[msg("Cards not yet committed")]
    CardsNotCommitted,

    #[msg("Hand not found")]
    HandNotFound,

    #[msg("Hand already complete")]
    HandAlreadyComplete,

    // ============================================
    // Betting Errors (6200-6299)
    // ============================================
    #[msg("Invalid bet amount")]
    InvalidBetAmount,

    #[msg("Insufficient chips")]
    InsufficientChips,

    #[msg("Cannot check - must call or raise")]
    CannotCheck,

    #[msg("Raise amount too small")]
    RaiseTooSmall,

    #[msg("Already folded")]
    AlreadyFolded,

    #[msg("Already all-in")]
    AlreadyAllIn,

    #[msg("Bet must be at least big blind")]
    BetTooSmall,

    // ============================================
    // ZK Errors (6300-6399)
    // ============================================
    #[msg("ZK proof verification failed")]
    ProofVerificationFailed,

    #[msg("Invalid proof format")]
    InvalidProofFormat,

    #[msg("Invalid commitment")]
    InvalidCommitment,

    #[msg("Invalid card index")]
    InvalidCardIndex,

    #[msg("Invalid hand rank")]
    InvalidHandRank,

    // ============================================
    // Timeout Errors (6400-6499)
    // ============================================
    #[msg("Action timed out")]
    ActionTimedOut,

    #[msg("No timeout has occurred")]
    NoTimeout,

    #[msg("Invalid timeout configuration")]
    InvalidTimeoutConfig,

    // ============================================
    // Reveal Errors (6500-6599)
    // ============================================
    #[msg("Flop already revealed")]
    FlopAlreadyRevealed,

    #[msg("Turn already revealed")]
    TurnAlreadyRevealed,

    #[msg("River already revealed")]
    RiverAlreadyRevealed,

    #[msg("Must reveal in order")]
    RevealOutOfOrder,

    #[msg("Hand already revealed")]
    HandAlreadyRevealed,

    // ============================================
    // Showdown Errors (6600-6699)
    // ============================================
    #[msg("Showdown not ready")]
    ShowdownNotReady,

    #[msg("Not the winner")]
    NotTheWinner,

    #[msg("Pot already claimed")]
    PotAlreadyClaimed,

    #[msg("Both players must reveal before claiming")]
    PlayersNotRevealed,

    // ============================================
    // Global Errors (6700-6799)
    // ============================================
    #[msg("Game is paused")]
    GamePaused,

    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("Invalid mint")]
    InvalidMint,

    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,

    // ============================================
    // Proof Buffer Errors (6800-6899)
    // ============================================
    #[msg("Invalid proof type")]
    InvalidProofType,

    #[msg("Proof buffer already complete")]
    BufferAlreadyComplete,

    #[msg("Proof buffer not complete")]
    BufferNotComplete,

    #[msg("Proof buffer mismatch")]
    BufferMismatch,
}
