use crate::lib::assert_valid_cards;
use crate::deck::commit::{commit_hole_cards, verify_hole_commitments};

/// Verify that a player was dealt specific cards from the deck.
pub fn verify_deal(
    hole_cards: [Field; 2],
    hole_salts: [Field; 2],
    hole_commitments: [Field; 2],
    deck_position: u32,
    shuffled_deck: [Field; 52]
) {
    verify_hole_commitments(hole_cards, hole_salts, hole_commitments);

    assert(
        shuffled_deck[deck_position] == hole_cards[0],
        "First hole card does not match deck position"
    );
    assert(
        shuffled_deck[deck_position + 1] == hole_cards[1],
        "Second hole card does not match deck position"
    );
}

/// Verify player owns the claimed hole cards.
pub fn verify_player_hand(
    hole_cards: [Field; 2],
    hole_salts: [Field; 2],
    hole_commitments: [Field; 2]
) {
    assert_valid_cards(hole_cards);
    verify_hole_commitments(hole_cards, hole_salts, hole_commitments);
}

/// Deal cards to multiple players from a shuffled deck.
pub fn deal_to_players<let N: u32>(
    shuffled_deck: [Field; 52],
    player_salts: [[Field; 2]; N]
) -> [[Field; 2]; N] {
    let mut commitments: [[Field; 2]; N] = [[0; 2]; N];

    for i in 0..N {
        let pos = (i * 2) as u32;
        let cards: [Field; 2] = [shuffled_deck[pos], shuffled_deck[pos + 1]];
        commitments[i] = commit_hole_cards(cards, player_salts[i]);
    }

    commitments
}

// ============================================
// TESTS
// ============================================

#[test]
fn test_verify_player_hand() {
    let cards: [Field; 2] = [12, 25];
    let salts: [Field; 2] = [111, 222];
    let commitments = commit_hole_cards(cards, salts);
    verify_player_hand(cards, salts, commitments);
}

#[test]
fn test_verify_deal() {
    let mut deck: [Field; 52] = [0; 52];
    for i in 0..52 {
        deck[i] = i as Field;
    }

    let cards: [Field; 2] = [0, 1];
    let salts: [Field; 2] = [111, 222];
    let commitments = commit_hole_cards(cards, salts);
    verify_deal(cards, salts, commitments, 0, deck);
}

#[test(should_fail_with = "First hole card does not match deck position")]
fn test_verify_deal_wrong_position() {
    let mut deck: [Field; 52] = [0; 52];
    for i in 0..52 {
        deck[i] = i as Field;
    }

    let cards: [Field; 2] = [0, 1];
    let salts: [Field; 2] = [111, 222];
    let commitments = commit_hole_cards(cards, salts);
    verify_deal(cards, salts, commitments, 2, deck);
}

#[test]
fn test_deal_to_multiple_players() {
    let mut deck: [Field; 52] = [0; 52];
    for i in 0..52 {
        deck[i] = i as Field;
    }

    let salts: [[Field; 2]; 3] = [
        [111, 112],
        [221, 222],
        [331, 332]
    ];

    let commitments = deal_to_players(deck, salts);

    verify_player_hand([0, 1], salts[0], commitments[0]);
    verify_player_hand([2, 3], salts[1], commitments[1]);
    verify_player_hand([4, 5], salts[2], commitments[2]);
}
