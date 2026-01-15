use crate::lib::{
    Card, hash_with_salt, assert_valid_cards,
    evaluate_hand_rank, RANK_TWO_PAIR, RANK_FLUSH, RANK_FULL_HOUSE
};

/// Evaluate a player's hand and return the rank.
pub fn evaluate_player_hand(
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

/// Verify a player's claimed hand rank.
pub fn verify_hand_rank(
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

/// Verify the winner among multiple players.
pub fn verify_winner(
    player_hands: [[Field; 2]; 9],
    player_salts: [[Field; 2]; 9],
    player_commitments: [[Field; 2]; 9],
    player_active: [bool; 9],
    community_cards: [Field; 5],
    claimed_winner: Field
) {
    assert_valid_cards(community_cards);

    let mut best_rank: Field = 0;
    let mut best_player: Field = 0;
    let mut found_active = false;

    for i in 0..9 {
        if player_active[i] {
            for j in 0..2 {
                let commitment = hash_with_salt(player_hands[i][j], player_salts[i][j]);
                assert(
                    commitment == player_commitments[i][j],
                    "Player commitment mismatch"
                );
            }

            assert_valid_cards(player_hands[i]);

            let rank = evaluate_player_hand(player_hands[i], community_cards);

            if !found_active {
                best_rank = rank;
                best_player = i as Field;
                found_active = true;
            } else if rank as u64 > best_rank as u64 {
                best_rank = rank;
                best_player = i as Field;
            }
        }
    }

    assert(best_player == claimed_winner, "Winner index mismatch");
}

/// Simplified 2-player winner verification (for heads-up).
pub fn verify_winner_heads_up(
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
    // Cards that make a flush (5 clubs in community)
    let hole_cards: [Field; 2] = [12, 25]; // Ace clubs, Ace diamonds
    // Community: 2c, 3c, 4c, 5c, 7c (all clubs)
    let community: [Field; 5] = [0, 1, 2, 3, 5];

    let rank = evaluate_player_hand(hole_cards, community);
    assert(rank == RANK_FLUSH);
}

#[test]
fn test_evaluate_two_pair() {
    // Ac (12), Kd (24)
    let hole_cards: [Field; 2] = [12, 24];
    // Community: Ad (25), Kc (11), 2h (26), 3s (40), 5h (29)
    let community: [Field; 5] = [25, 11, 26, 40, 29];

    let rank = evaluate_player_hand(hole_cards, community);
    assert(rank == RANK_TWO_PAIR);
}

#[test]
fn test_verify_hand_rank() {
    let hole_cards: [Field; 2] = [12, 25]; // Ac, Ad
    let salts: [Field; 2] = [111, 222];
    let commitments: [Field; 2] = [
        hash_with_salt(hole_cards[0], salts[0]),
        hash_with_salt(hole_cards[1], salts[1])
    ];
    // Community: 2c, 2d, 2h, 4s, 4d - makes AAA22 = Full House
    let community: [Field; 5] = [0, 13, 26, 42, 16];

    let expected_rank = RANK_FULL_HOUSE;

    verify_hand_rank(hole_cards, salts, community, commitments, expected_rank);
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

    // Community: 2h, 3h, 5s, 7s, 9c (no help for either)
    let community: [Field; 5] = [26, 27, 42, 44, 7];

    // Player 1 should win (aces beat kings)
    verify_winner_heads_up(
        p1_cards, p1_salts, p1_commits,
        p2_cards, p2_salts, p2_commits,
        community,
        0 // Player 1 wins
    );
}

#[test(should_fail_with = "Winner mismatch")]
fn test_verify_winner_wrong_claim() {
    let p1_cards: [Field; 2] = [12, 25]; // Aces
    let p1_salts: [Field; 2] = [111, 112];
    let p1_commits: [Field; 2] = [
        hash_with_salt(p1_cards[0], p1_salts[0]),
        hash_with_salt(p1_cards[1], p1_salts[1])
    ];

    let p2_cards: [Field; 2] = [11, 24]; // Kings
    let p2_salts: [Field; 2] = [221, 222];
    let p2_commits: [Field; 2] = [
        hash_with_salt(p2_cards[0], p2_salts[0]),
        hash_with_salt(p2_cards[1], p2_salts[1])
    ];

    let community: [Field; 5] = [26, 27, 42, 44, 7];

    // Claim player 2 wins (should fail)
    verify_winner_heads_up(
        p1_cards, p1_salts, p1_commits,
        p2_cards, p2_salts, p2_commits,
        community,
        1 // Wrong: Player 1 actually wins
    );
}
