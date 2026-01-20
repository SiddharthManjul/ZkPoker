use anchor_lang::prelude::*;

/// ZK Circuit Verifier Program IDs (deployed on Solana devnet)
/// Each circuit has its own verifier program

/// Deck Circuit Verifier - for hole card commitments
pub const DECK_VERIFIER_PROGRAM_ID: Pubkey = pubkey!("5mWDL7NZwacC8fxVouwEwUgvJGQMpcaAfjmyMNkwzWEd");

/// Deal Circuit Verifier - for dealing cards from deck
pub const DEAL_VERIFIER_PROGRAM_ID: Pubkey = pubkey!("DewUCARGDNMyp2yWwn69VF5upEuchW7pfUMAAznFiJzy");

/// Reveal Circuit Verifier - for revealing community cards
pub const REVEAL_VERIFIER_PROGRAM_ID: Pubkey = pubkey!("9Yp14dZ4ZVY9ckWn5tzyEaymy4r1dH5VwCbCwKSRgvTx");

/// Showdown Circuit Verifier - for hand reveals at showdown
pub const SHOWDOWN_VERIFIER_PROGRAM_ID: Pubkey = pubkey!("7urWEDFxTrKSE6X6zGdd9wgkCEieAWHXSCxEd8zxcTgh");

/// Bet Circuit Verifier - for bet/balance verification
pub const BET_VERIFIER_PROGRAM_ID: Pubkey = pubkey!("6kucgYYg8q9PVWTxvzH1sA9vgg5onhmSYUcuMD3zkwai");

/// PDA Seeds
pub const GLOBAL_SEED: &[u8] = b"global";
pub const TABLE_SEED: &[u8] = b"table";
pub const HAND_SEED: &[u8] = b"hand";
pub const VAULT_SEED: &[u8] = b"vault";

/// Default action timeout (seconds)
pub const DEFAULT_ACTION_TIMEOUT: i64 = 30;

/// Minimum action timeout (seconds)
pub const MIN_ACTION_TIMEOUT: i64 = 10;

/// Maximum action timeout (seconds)
pub const MAX_ACTION_TIMEOUT: i64 = 120;

/// Groth16 proof size (bytes)
pub const PROOF_SIZE: usize = 388;

/// Card commitment size (bytes)
pub const COMMITMENT_SIZE: usize = 32;

/// Number of cards in deck
pub const DECK_SIZE: u8 = 52;

/// Number of hole cards per player
pub const HOLE_CARDS: u8 = 2;

/// Number of community cards
pub const COMMUNITY_CARDS: u8 = 5;

/// Card positions in shuffled deck
pub const P1_CARD_1_POS: u8 = 0;
pub const P1_CARD_2_POS: u8 = 1;
pub const P2_CARD_1_POS: u8 = 2;
pub const P2_CARD_2_POS: u8 = 3;
pub const FLOP_1_POS: u8 = 4;
pub const FLOP_2_POS: u8 = 5;
pub const FLOP_3_POS: u8 = 6;
pub const TURN_POS: u8 = 7;
pub const RIVER_POS: u8 = 8;
