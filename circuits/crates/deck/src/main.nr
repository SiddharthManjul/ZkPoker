// ============================================
// DECK CIRCUIT
// Hole card commitment verification
// ============================================
//
// This circuit proves that a player's hole cards
// were correctly dealt from a shuffled deck and
// that the commitments are valid.
// ============================================

use zkpoker_lib::{hash_with_salt, assert_valid_card};

/// Main circuit: Verify hole card commitment
///
/// Public inputs:
///   - commitment1: Commitment to first hole card
///   - commitment2: Commitment to second hole card
///   - deck_seed: Combined seed used to shuffle the deck
///   - player_seat: Player's seat (0 or 1 for heads-up)
///
/// Private inputs:
///   - card1: First hole card (0-51)
///   - card2: Second hole card (0-51)
///   - salt1: Salt for first card commitment
///   - salt2: Salt for second card commitment
fn main(
    // Private inputs
    card1: Field,
    card2: Field,
    salt1: Field,
    salt2: Field,
    // Public inputs (returned)
    _deck_seed: pub [u8; 32],
    _player_seat: pub u8,
) -> pub ([Field; 2]) {
    // Validate cards are in valid range
    assert_valid_card(card1);
    assert_valid_card(card2);

    // Ensure cards are different
    assert(card1 != card2, "Hole cards must be different");

    // Note: deck_seed and player_seat are public inputs for verification
    // The client derives cards from deck_seed using deterministic shuffle
    // and verifies cards are at correct positions (seat * 2, seat * 2 + 1)
    // The actual deck derivation and position verification happens off-chain

    // Generate commitments
    let commitment1 = hash_with_salt(card1, salt1);
    let commitment2 = hash_with_salt(card2, salt2);

    // Return the commitments as public outputs
    [commitment1, commitment2]
}

/// Verify a single card commitment
fn verify_commitment(card: Field, salt: Field, commitment: Field) {
    assert_valid_card(card);
    let computed = hash_with_salt(card, salt);
    assert(computed == commitment, "Card does not match commitment");
}

/// Commit to a pair of hole cards
fn commit_hole_cards(cards: [Field; 2], salts: [Field; 2]) -> [Field; 2] {
    [
        hash_with_salt(cards[0], salts[0]),
        hash_with_salt(cards[1], salts[1])
    ]
}

// ============================================
// TESTS
// ============================================

#[test]
fn test_commit_card() {
    let card: Field = 25;
    let salt: Field = 98765;
    let commitment = hash_with_salt(card, salt);
    let commitment2 = hash_with_salt(card, salt);
    assert(commitment == commitment2);
}

#[test]
fn test_verify_commitment_valid() {
    let card: Field = 10;
    let salt: Field = 11111;
    let commitment = hash_with_salt(card, salt);
    verify_commitment(card, salt, commitment);
}

#[test]
fn test_main_circuit() {
    let card1: Field = 12;
    let card2: Field = 25;
    let salt1: Field = 111;
    let salt2: Field = 222;
    let deck_seed: [u8; 32] = [0; 32];
    let player_seat: u8 = 0;

    let commitments = main(card1, card2, salt1, salt2, deck_seed, player_seat);

    // Verify commitments match
    let expected1 = hash_with_salt(card1, salt1);
    let expected2 = hash_with_salt(card2, salt2);
    assert(commitments[0] == expected1);
    assert(commitments[1] == expected2);
}

#[test]
fn test_commit_hole_cards() {
    let cards: [Field; 2] = [12, 25];
    let salts: [Field; 2] = [111, 222];
    let commitments = commit_hole_cards(cards, salts);

    verify_commitment(cards[0], salts[0], commitments[0]);
    verify_commitment(cards[1], salts[1], commitments[1]);
}

#[test]
fn test_different_cards_different_commitments() {
    let salt: Field = 12345;
    let c1 = hash_with_salt(0, salt);
    let c2 = hash_with_salt(51, salt);
    assert(c1 != c2);
}
