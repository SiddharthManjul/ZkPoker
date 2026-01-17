// ============================================
// REVEAL CIRCUIT
// Community card reveal verification
// ============================================
//
// This circuit proves that community cards
// are correctly revealed from the shuffled deck.
// ============================================

use zkpoker_lib::{hash_array, assert_valid_cards, COMMUNITY_START};

/// Main circuit: Verify community card reveal
///
/// Public inputs:
///   - cards: The revealed community cards (up to 5)
///   - positions: The expected positions in the deck
///   - deck_seed: Combined seed used to shuffle the deck
///
/// Private inputs:
///   - shuffled_deck: The full shuffled deck
fn main(
    // Private inputs
    shuffled_deck: [Field; 52],
    // Public inputs
    _deck_seed: pub [u8; 32],  // Used for off-chain verification of shuffle derivation
    cards: pub [Field; 5],
    num_cards: pub u8,  // How many cards to verify (3 for flop, 4 for turn, 5 for river)
) -> pub Field {
    // Validate revealed cards
    assert_valid_cards(cards);

    // Compute deck commitment from shuffled deck
    let deck_commitment = hash_array(shuffled_deck);

    // Verify each card matches its position in the shuffled deck
    // Community cards are at positions COMMUNITY_START + 0..4
    for i in 0..5 {
        if (i as u8) < num_cards {
            let expected_pos = COMMUNITY_START + i;
            assert(
                shuffled_deck[expected_pos] == cards[i],
                "Community card does not match deck position"
            );
        }
    }

    // Return deck commitment as public output
    deck_commitment
}

/// Verify flop (first 3 community cards)
fn verify_flop(
    flop: [Field; 3],
    shuffled_deck: [Field; 52],
    deck_commitment: Field
) {
    let computed = hash_array(shuffled_deck);
    assert(computed == deck_commitment, "Deck commitment mismatch");

    for i in 0..3 {
        assert(
            shuffled_deck[COMMUNITY_START + i] == flop[i],
            "Flop card does not match deck"
        );
    }
}

/// Verify turn (4th community card)
fn verify_turn(
    turn: Field,
    shuffled_deck: [Field; 52],
    deck_commitment: Field,
    expected_flop: [Field; 3]
) {
    let computed = hash_array(shuffled_deck);
    assert(computed == deck_commitment, "Deck commitment mismatch");

    for i in 0..3 {
        assert(
            shuffled_deck[COMMUNITY_START + i] == expected_flop[i],
            "Flop mismatch"
        );
    }

    assert(
        shuffled_deck[COMMUNITY_START + 3] == turn,
        "Turn card does not match deck"
    );
}

/// Verify river (5th community card)
fn verify_river(
    river: Field,
    shuffled_deck: [Field; 52],
    deck_commitment: Field,
    expected_flop: [Field; 3],
    expected_turn: Field
) {
    let computed = hash_array(shuffled_deck);
    assert(computed == deck_commitment, "Deck commitment mismatch");

    for i in 0..3 {
        assert(
            shuffled_deck[COMMUNITY_START + i] == expected_flop[i],
            "Flop mismatch"
        );
    }
    assert(
        shuffled_deck[COMMUNITY_START + 3] == expected_turn,
        "Turn mismatch"
    );

    assert(
        shuffled_deck[COMMUNITY_START + 4] == river,
        "River card does not match deck"
    );
}

// ============================================
// TESTS
// ============================================

#[test]
fn test_verify_flop() {
    let mut deck: [Field; 52] = [0; 52];
    for i in 0..52 {
        deck[i] = i as Field;
    }
    let commitment = hash_array(deck);

    let flop: [Field; 3] = [18, 19, 20];
    verify_flop(flop, deck, commitment);
}

#[test]
fn test_verify_turn() {
    let mut deck: [Field; 52] = [0; 52];
    for i in 0..52 {
        deck[i] = i as Field;
    }
    let commitment = hash_array(deck);

    let flop: [Field; 3] = [18, 19, 20];
    let turn: Field = 21;
    verify_turn(turn, deck, commitment, flop);
}

#[test]
fn test_verify_river() {
    let mut deck: [Field; 52] = [0; 52];
    for i in 0..52 {
        deck[i] = i as Field;
    }
    let commitment = hash_array(deck);

    let flop: [Field; 3] = [18, 19, 20];
    let turn: Field = 21;
    let river: Field = 22;
    verify_river(river, deck, commitment, flop, turn);
}

#[test]
fn test_main_circuit() {
    let mut deck: [Field; 52] = [0; 52];
    for i in 0..52 {
        deck[i] = i as Field;
    }
    let deck_seed: [u8; 32] = [0; 32];
    let cards: [Field; 5] = [18, 19, 20, 21, 22];
    let num_cards: u8 = 5;

    let commitment = main(deck, deck_seed, cards, num_cards);
    let expected_commitment = hash_array(deck);
    assert(commitment == expected_commitment);
}

#[test(should_fail_with = "Flop card does not match deck")]
fn test_verify_flop_wrong_cards() {
    let mut deck: [Field; 52] = [0; 52];
    for i in 0..52 {
        deck[i] = i as Field;
    }
    let commitment = hash_array(deck);

    let flop: [Field; 3] = [0, 1, 2]; // Wrong positions
    verify_flop(flop, deck, commitment);
}
