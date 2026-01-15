use crate::lib::{hash_array, hash_with_salt, assert_valid_cards};

// Community cards start after dealing to 9 players (18 cards)
global COMMUNITY_START: u32 = 18;

/// Verify that revealed community cards match the committed deck.
pub fn verify_community_cards(
    community_cards: [Field; 5],
    shuffled_deck: [Field; 52],
    deck_commitment: Field
) {
    let computed_commitment = hash_array(shuffled_deck);
    assert(computed_commitment == deck_commitment, "Deck commitment mismatch");

    assert_valid_cards(community_cards);

    for i in 0..5 {
        assert(
            shuffled_deck[COMMUNITY_START + i] == community_cards[i],
            "Community card does not match deck"
        );
    }
}

/// Verify flop (first 3 community cards).
pub fn verify_flop(
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

/// Verify turn (4th community card).
pub fn verify_turn(
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

/// Verify river (5th community card).
pub fn verify_river(
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

/// Verify hole card reveal at showdown.
pub fn verify_hole_reveal(
    hole_cards: [Field; 2],
    hole_salts: [Field; 2],
    hole_commitments: [Field; 2]
) {
    for i in 0..2 {
        let computed = hash_with_salt(hole_cards[i], hole_salts[i]);
        assert(
            computed == hole_commitments[i],
            "Hole card does not match commitment"
        );
    }
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
fn test_verify_community_cards() {
    let mut deck: [Field; 52] = [0; 52];
    for i in 0..52 {
        deck[i] = i as Field;
    }
    let commitment = hash_array(deck);

    let community: [Field; 5] = [18, 19, 20, 21, 22];
    verify_community_cards(community, deck, commitment);
}

#[test(should_fail_with = "Flop card does not match deck")]
fn test_verify_flop_wrong_cards() {
    let mut deck: [Field; 52] = [0; 52];
    for i in 0..52 {
        deck[i] = i as Field;
    }
    let commitment = hash_array(deck);

    let flop: [Field; 3] = [0, 1, 2];
    verify_flop(flop, deck, commitment);
}

#[test]
fn test_verify_hole_reveal() {
    let cards: [Field; 2] = [12, 25];
    let salts: [Field; 2] = [111, 222];
    let commitments: [Field; 2] = [
        hash_with_salt(cards[0], salts[0]),
        hash_with_salt(cards[1], salts[1])
    ];

    verify_hole_reveal(cards, salts, commitments);
}
