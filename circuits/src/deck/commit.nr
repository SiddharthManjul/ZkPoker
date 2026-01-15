use crate::lib::{hash_with_salt, assert_valid_card};

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
// TESTS
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
