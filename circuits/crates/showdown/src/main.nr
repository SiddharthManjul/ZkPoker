// ============================================
// SHOWDOWN CIRCUIT
// Hand reveal and ranking verification
// ============================================
//
// This circuit proves that a player's claimed
// hand rank is correct based on their hole cards
// and the community cards.
// ============================================

use zkpoker_lib::{
    Card, hash_with_salt, assert_valid_cards, evaluate_hand_rank,
    RANK_TWO_PAIR, RANK_FLUSH, RANK_FULL_HOUSE
};

/// Main circuit: Verify hand reveal at showdown
///
/// Public inputs:
///   - commitment1: Commitment to first hole card
///   - commitment2: Commitment to second hole card
///   - community_cards: The 5 community cards
///   - claimed_rank: The claimed hand rank
///
/// Private inputs:
///   - hole_card1: First hole card (0-51)
///   - hole_card2: Second hole card (0-51)
///   - salt1: Salt for first card commitment
///   - salt2: Salt for second card commitment
fn main(
    // Private inputs
    hole_card1: Field,
    hole_card2: Field,
    salt1: Field,
    salt2: Field,
    // Public inputs
    commitment1: pub Field,
    commitment2: pub Field,
    community_cards: pub [Field; 5],
) -> pub Field {
    // Validate all cards
    let hole_cards: [Field; 2] = [hole_card1, hole_card2];
    assert_valid_cards(hole_cards);
    assert_valid_cards(community_cards);

    // Verify hole card commitments
    let computed1 = hash_with_salt(hole_card1, salt1);
    let computed2 = hash_with_salt(hole_card2, salt2);
    assert(computed1 == commitment1, "Hole card 1 commitment mismatch");
    assert(computed2 == commitment2, "Hole card 2 commitment mismatch");

    // Evaluate the hand rank
    let rank = evaluate_player_hand(hole_cards, community_cards);

    // Return the computed rank as public output
    rank
}

/// Evaluate a player's hand and return the rank
fn evaluate_player_hand(
    hole_cards: [Field; 2],
    community_cards: [Field; 5]
) -> Field {
    let mut all_cards: [Card; 7] = [Card { value: 0, suit: 0 }; 7];

    all_cards[0] = Card::from_index(hole_cards[0]);
    all_cards[1] = Card::from_index(hole_cards[1]);
    for i in 0..5 {
        all_cards[i + 2] = Card::from_index(community_cards[i]);
    }

    evaluate_hand_rank(all_cards)
}

/// Verify a player's claimed hand rank
fn verify_hand_rank(
    hole_cards: [Field; 2],
    hole_salts: [Field; 2],
    community_cards: [Field; 5],
    hole_commitments: [Field; 2],
    claimed_rank: Field
) {
    assert_valid_cards(hole_cards);
    assert_valid_cards(community_cards);

    for i in 0..2 {
        let commitment = hash_with_salt(hole_cards[i], hole_salts[i]);
        assert(commitment == hole_commitments[i], "Hole card commitment mismatch");
    }

    let computed_rank = evaluate_player_hand(hole_cards, community_cards);
    assert(computed_rank == claimed_rank, "Hand rank does not match claimed rank");
}

/// Simplified 2-player winner verification (for heads-up)
fn verify_winner_heads_up(
    player1_cards: [Field; 2],
    player1_salts: [Field; 2],
    player1_commitments: [Field; 2],
    player2_cards: [Field; 2],
    player2_salts: [Field; 2],
    player2_commitments: [Field; 2],
    community_cards: [Field; 5],
    claimed_winner: Field // 0 = player1, 1 = player2, 2 = tie
) {
    for i in 0..2 {
        let c1 = hash_with_salt(player1_cards[i], player1_salts[i]);
        assert(c1 == player1_commitments[i], "Player 1 commitment mismatch");

        let c2 = hash_with_salt(player2_cards[i], player2_salts[i]);
        assert(c2 == player2_commitments[i], "Player 2 commitment mismatch");
    }

    let rank1 = evaluate_player_hand(player1_cards, community_cards);
    let rank2 = evaluate_player_hand(player2_cards, community_cards);

    let actual_winner = if rank1 as u64 > rank2 as u64 {
        0 // Player 1 wins
    } else if rank2 as u64 > rank1 as u64 {
        1 // Player 2 wins
    } else {
        2 // Tie
    };

    assert(actual_winner == claimed_winner, "Winner mismatch");
}

// ============================================
// TESTS
// ============================================

#[test]
fn test_evaluate_flush() {
    let hole_cards: [Field; 2] = [12, 25]; // Ace clubs, Ace diamonds
    let community: [Field; 5] = [0, 1, 2, 4, 6]; // 2c, 3c, 4c, 6c, 8c

    let rank = evaluate_player_hand(hole_cards, community);
    assert(rank == RANK_FLUSH * 100 + 12);
}

#[test]
fn test_evaluate_two_pair() {
    let hole_cards: [Field; 2] = [12, 24]; // Ac, Kd
    let community: [Field; 5] = [25, 11, 26, 40, 29]; // Ad, Kc, 2h, 3s, 5h

    let rank = evaluate_player_hand(hole_cards, community);
    assert(rank == RANK_TWO_PAIR * 100 + 12);
}

#[test]
fn test_verify_hand_rank() {
    let hole_cards: [Field; 2] = [12, 25]; // Ac, Ad
    let salts: [Field; 2] = [111, 222];
    let commitments: [Field; 2] = [
        hash_with_salt(hole_cards[0], salts[0]),
        hash_with_salt(hole_cards[1], salts[1])
    ];
    let community: [Field; 5] = [0, 13, 26, 42, 16]; // 2c, 2d, 2h, 4s, 4d

    let expected_rank = RANK_FULL_HOUSE * 100 + 0;
    verify_hand_rank(hole_cards, salts, community, commitments, expected_rank);
}

#[test]
fn test_main_circuit() {
    let hole_card1: Field = 12;
    let hole_card2: Field = 25;
    let salt1: Field = 111;
    let salt2: Field = 222;
    let commitment1 = hash_with_salt(hole_card1, salt1);
    let commitment2 = hash_with_salt(hole_card2, salt2);
    let community: [Field; 5] = [0, 13, 26, 42, 16];

    let rank = main(hole_card1, hole_card2, salt1, salt2, commitment1, commitment2, community);
    let expected_rank = RANK_FULL_HOUSE * 100 + 0;
    assert(rank == expected_rank);
}

#[test]
fn test_verify_winner_heads_up() {
    // Player 1: Ac Ad (pair of aces)
    let p1_cards: [Field; 2] = [12, 25];
    let p1_salts: [Field; 2] = [111, 112];
    let p1_commits: [Field; 2] = [
        hash_with_salt(p1_cards[0], p1_salts[0]),
        hash_with_salt(p1_cards[1], p1_salts[1])
    ];

    // Player 2: Kc Kd (pair of kings)
    let p2_cards: [Field; 2] = [11, 24];
    let p2_salts: [Field; 2] = [221, 222];
    let p2_commits: [Field; 2] = [
        hash_with_salt(p2_cards[0], p2_salts[0]),
        hash_with_salt(p2_cards[1], p2_salts[1])
    ];

    let community: [Field; 5] = [26, 27, 42, 44, 7];

    verify_winner_heads_up(
        p1_cards, p1_salts, p1_commits,
        p2_cards, p2_salts, p2_commits,
        community,
        0 // Player 1 wins
    );
}

#[test(should_fail_with = "Winner mismatch")]
fn test_verify_winner_wrong_claim() {
    let p1_cards: [Field; 2] = [12, 25];
    let p1_salts: [Field; 2] = [111, 112];
    let p1_commits: [Field; 2] = [
        hash_with_salt(p1_cards[0], p1_salts[0]),
        hash_with_salt(p1_cards[1], p1_salts[1])
    ];

    let p2_cards: [Field; 2] = [11, 24];
    let p2_salts: [Field; 2] = [221, 222];
    let p2_commits: [Field; 2] = [
        hash_with_salt(p2_cards[0], p2_salts[0]),
        hash_with_salt(p2_cards[1], p2_salts[1])
    ];

    let community: [Field; 5] = [26, 27, 42, 44, 7];

    verify_winner_heads_up(
        p1_cards, p1_salts, p1_commits,
        p2_cards, p2_salts, p2_commits,
        community,
        1 // Wrong: Player 1 actually wins
    );
}
