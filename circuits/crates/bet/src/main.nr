// ============================================
// BET CIRCUIT
// Balance and bet verification
// ============================================

use zkpoker_lib::hash_with_salt;

/// Main circuit: Verify balance and bet validity
fn main(
    // Private inputs
    balance: Field,
    salt: Field,
    // Public inputs
    commitment: pub Field,
    bet_amount: pub Field,
) {
    verify_balance_and_bet(balance, salt, commitment, bet_amount);
}

// ============================================
// BALANCE FUNCTIONS
// ============================================

/// Verify player has sufficient balance for a bet.
pub fn verify_sufficient_balance(balance: Field, bet_amount: Field) {
    assert(
        balance as u64 >= bet_amount as u64,
        "Insufficient balance for bet"
    );
}

/// Verify a bet is valid according to table rules.
pub fn verify_bet_valid(
    balance: Field,
    bet_amount: Field,
    min_bet: Field,
    current_bet: Field,
    player_current_bet: Field
) {
    let balance_u64 = balance as u64;
    let bet_u64 = bet_amount as u64;
    let min_u64 = min_bet as u64;
    let current_u64 = current_bet as u64;
    let player_current_u64 = player_current_bet as u64;

    let to_call = current_u64 - player_current_u64;

    assert(balance_u64 >= bet_u64, "Insufficient balance");

    let is_all_in = (bet_u64 == balance_u64);
    let is_call = (bet_u64 == to_call);
    let is_valid_raise = (bet_u64 >= to_call + min_u64);

    assert(
        is_all_in | is_call | is_valid_raise,
        "Bet must be all-in, call, or valid raise"
    );
}

/// Verify check is valid (no outstanding bet to call).
pub fn verify_check_valid(current_bet: Field, player_current_bet: Field) {
    assert(
        current_bet == player_current_bet,
        "Cannot check when there is a bet to call"
    );
}

/// Commit to a player's balance (for privacy).
pub fn commit_balance(balance: Field, salt: Field) -> Field {
    hash_with_salt(balance, salt)
}

/// Verify balance commitment and sufficient funds.
pub fn verify_balance_and_bet(
    balance: Field,
    salt: Field,
    commitment: Field,
    bet_amount: Field
) {
    let computed = hash_with_salt(balance, salt);
    assert(computed == commitment, "Balance commitment mismatch");
    verify_sufficient_balance(balance, bet_amount);
}

// ============================================
// TESTS (9 tests)
// ============================================

#[test]
fn test_sufficient_balance() {
    verify_sufficient_balance(1000, 500);
    verify_sufficient_balance(1000, 1000);
}

#[test(should_fail_with = "Insufficient balance for bet")]
fn test_insufficient_balance() {
    verify_sufficient_balance(500, 1000);
}

#[test]
fn test_valid_call() {
    verify_bet_valid(1000, 100, 50, 100, 0);
}

#[test]
fn test_valid_raise() {
    verify_bet_valid(1000, 150, 50, 100, 0);
}

#[test]
fn test_valid_all_in() {
    verify_bet_valid(80, 80, 50, 100, 0);
}

#[test(should_fail_with = "Bet must be all-in, call, or valid raise")]
fn test_invalid_bet_too_small() {
    verify_bet_valid(1000, 75, 50, 100, 0);
}

#[test]
fn test_check_valid() {
    verify_check_valid(100, 100);
}

#[test(should_fail_with = "Cannot check when there is a bet to call")]
fn test_check_invalid() {
    verify_check_valid(100, 50);
}

#[test]
fn test_balance_commitment() {
    let balance: Field = 10000;
    let salt: Field = 99999;
    let commitment = commit_balance(balance, salt);
    verify_balance_and_bet(balance, salt, commitment, 5000);
}
