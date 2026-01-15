// ============================================
// ZKPOKER CIRCUITS
// Privacy-Preserving Texas Hold'em on Solana
// ============================================

// Core library with shared types and functions
pub mod lib;
pub mod deck;
pub mod deal;
pub mod bet;
pub mod reveal;
pub mod showdown;

// Import from lib for main
use lib::{hash_with_salt, hash_array, assert_valid_card};

// ============================================
// MAIN ENTRYPOINT - Card Commitment Circuit
// ============================================

/// Main circuit entrypoint - commit to a card.
fn main(card: Field, salt: Field) -> pub Field {
    assert_valid_card(card);
    hash_with_salt(card, salt)
}

// ============================================
// TESTS
// ============================================

#[test]
fn test_main_commitment() {
    let card: Field = 42;
    let salt: Field = 12345;
    let commitment = main(card, salt);
    let commitment2 = main(card, salt);
    assert(commitment == commitment2);
}

#[test]
fn test_full_game_flow() {
    let mut deck_arr: [Field; 52] = [0; 52];
    for i in 0..52 {
        deck_arr[i] = i as Field;
    }
    let deck_commitment = hash_array(deck_arr);

    let p1_cards: [Field; 2] = [deck_arr[0], deck_arr[1]];
    let p1_salts: [Field; 2] = [111, 112];
    let p1_commits = deck::commit::commit_hole_cards(p1_cards, p1_salts);

    let p2_cards: [Field; 2] = [deck_arr[2], deck_arr[3]];
    let p2_salts: [Field; 2] = [221, 222];
    let p2_commits = deck::commit::commit_hole_cards(p2_cards, p2_salts);

    deal::deal::verify_player_hand(p1_cards, p1_salts, p1_commits);
    deal::deal::verify_player_hand(p2_cards, p2_salts, p2_commits);

    let community: [Field; 5] = [deck_arr[18], deck_arr[19], deck_arr[20], deck_arr[21], deck_arr[22]];

    reveal::reveal::verify_community_cards(community, deck_arr, deck_commitment);

    let p1_rank = showdown::showdown::evaluate_player_hand(p1_cards, community);
    let p2_rank = showdown::showdown::evaluate_player_hand(p2_cards, community);

    showdown::showdown::verify_hand_rank(p1_cards, p1_salts, community, p1_commits, p1_rank);
    showdown::showdown::verify_hand_rank(p2_cards, p2_salts, community, p2_commits, p2_rank);
}

#[test]
fn test_bet_verification() {
    let balance: Field = 10000;
    let salt: Field = 99999;
    let commitment = bet::balance::commit_balance(balance, salt);
    bet::balance::verify_balance_and_bet(balance, salt, commitment, 5000);
}
