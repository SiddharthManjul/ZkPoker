use anchor_lang::prelude::*;

/// ZK Verifier Program ID (deployed on devnet)
pub const VERIFIER_PROGRAM_ID: Pubkey = pubkey!("5fkbNoQZykoAz4SmKKkGg6ajKfRArwKq7Y9yUcgRANBe");

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
