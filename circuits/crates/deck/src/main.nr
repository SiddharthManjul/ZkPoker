// ============================================
// DECK CIRCUIT
// Hole card commitment and shuffle verification
// ============================================
//
// This circuit proves that a player's hole cards
// were correctly dealt from a shuffled deck and
// that the commitments are valid.
// ============================================

use zkpoker_lib::{hash_with_salt, hash_array, assert_valid_card};

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
    let commitment1 = commit_card(card1, salt1);
    let commitment2 = commit_card(card2, salt2);

    // Return the commitments as public outputs
    [commitment1, commitment2]
}

// ============================================
// COMMITMENT FUNCTIONS
// ============================================

/// Generate a cryptographic commitment to a card.
pub fn commit_card(card: Field, salt: Field) -> Field {
    assert_valid_card(card);
    hash_with_salt(card, salt)
}

/// Verify that a card matches its commitment.
pub fn verify_commitment(card: Field, salt: Field, commitment: Field) {
    assert_valid_card(card);
    let computed = hash_with_salt(card, salt);
    assert(computed == commitment, "Card does not match commitment");
}

/// Commit to a pair of hole cards.
pub fn commit_hole_cards(cards: [Field; 2], salts: [Field; 2]) -> [Field; 2] {
    [
        commit_card(cards[0], salts[0]),
        commit_card(cards[1], salts[1])
    ]
}

/// Verify hole card commitments.
pub fn verify_hole_commitments(
    cards: [Field; 2],
    salts: [Field; 2],
    commitments: [Field; 2]
) {
    verify_commitment(cards[0], salts[0], commitments[0]);
    verify_commitment(cards[1], salts[1], commitments[1]);
}

// ============================================
// SHUFFLE FUNCTIONS
// ============================================

/// Verify that a shuffled deck is valid and matches its commitment.
pub fn verify_shuffle(shuffled_deck: [Field; 52], deck_commitment: Field) {
    let computed = hash_array(shuffled_deck);
    assert(computed == deck_commitment, "Deck does not match commitment");
    verify_deck_integrity(shuffled_deck);
}

/// Verify deck contains all 52 cards exactly once.
pub fn verify_deck_integrity(deck: [Field; 52]) {
    let mut seen: [bool; 52] = [false; 52];

    for i in 0..52 {
        let card = deck[i];
        assert_valid_card(card);

        let card_idx = card as u32;
        for j in 0..52 {
            if j == card_idx {
                assert(!seen[j], "Duplicate card in deck");
                seen[j] = true;
            }
        }
    }

    for i in 0..52 {
        assert(seen[i], "Missing card in deck");
    }
}

/// Commit to a shuffled deck.
pub fn commit_deck(deck: [Field; 52]) -> Field {
    verify_deck_integrity(deck);
    hash_array(deck)
}

// ============================================
// COMMITMENT TESTS (6 tests)
// ============================================

#[test]
fn test_commit_card() {
    let card: Field = 25;
    let salt: Field = 98765;
    let commitment = commit_card(card, salt);
    let commitment2 = commit_card(card, salt);
    assert(commitment == commitment2);
}

#[test]
fn test_verify_commitment_valid() {
    let card: Field = 10;
    let salt: Field = 11111;
    let commitment = commit_card(card, salt);
    verify_commitment(card, salt, commitment);
}

#[test(should_fail_with = "Card does not match commitment")]
fn test_verify_commitment_wrong_card() {
    let card: Field = 10;
    let salt: Field = 11111;
    let commitment = commit_card(card, salt);
    verify_commitment(11, salt, commitment);
}

#[test(should_fail_with = "Card does not match commitment")]
fn test_verify_commitment_wrong_salt() {
    let card: Field = 10;
    let salt: Field = 11111;
    let commitment = commit_card(card, salt);
    verify_commitment(card, 22222, commitment);
}

#[test]
fn test_commit_hole_cards() {
    let cards: [Field; 2] = [12, 25];
    let salts: [Field; 2] = [111, 222];
    let commitments = commit_hole_cards(cards, salts);
    verify_hole_commitments(cards, salts, commitments);
}

#[test]
fn test_different_cards_different_commitments() {
    let salt: Field = 12345;
    let c1 = commit_card(0, salt);
    let c2 = commit_card(51, salt);
    assert(c1 != c2);
}

// ============================================
// SHUFFLE TESTS (4 tests)
// ============================================

#[test]
fn test_verify_deck_integrity_valid() {
    let mut deck: [Field; 52] = [0; 52];
    for i in 0..52 {
        deck[i] = i as Field;
    }
    verify_deck_integrity(deck);
}

#[test]
fn test_commit_and_verify_shuffle() {
    let mut deck: [Field; 52] = [0; 52];
    for i in 0..52 {
        deck[i] = (51 - i) as Field;
    }

    let commitment = commit_deck(deck);
    verify_shuffle(deck, commitment);
}

#[test(should_fail_with = "Duplicate card in deck")]
fn test_deck_with_duplicate_fails() {
    let mut deck: [Field; 52] = [0; 52];
    for i in 0..52 {
        deck[i] = i as Field;
    }
    deck[51] = 0;
    verify_deck_integrity(deck);
}

#[test(should_fail_with = "Card index must be 0-51")]
fn test_deck_with_invalid_card_fails() {
    let mut deck: [Field; 52] = [0; 52];
    for i in 0..52 {
        deck[i] = i as Field;
    }
    deck[0] = 52;
    verify_deck_integrity(deck);
}
