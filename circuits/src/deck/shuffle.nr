use crate::lib::{hash_array, assert_valid_card};

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
// TESTS
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
