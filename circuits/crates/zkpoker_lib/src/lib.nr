// ============================================
// ZKPOKER LIBRARY
// Shared types and functions for all circuits
// ============================================

// ============================================
// CONSTANTS
// ============================================

pub global DECK_SIZE: u32 = 52;
pub global HAND_SIZE: u32 = 2;
pub global COMMUNITY_SIZE: u32 = 5;
pub global MAX_PLAYERS: u32 = 9;

// Hand rankings (higher is better)
pub global RANK_HIGH_CARD: Field = 0;
pub global RANK_ONE_PAIR: Field = 1;
pub global RANK_TWO_PAIR: Field = 2;
pub global RANK_THREE_OF_KIND: Field = 3;
pub global RANK_STRAIGHT: Field = 4;
pub global RANK_FLUSH: Field = 5;
pub global RANK_FULL_HOUSE: Field = 6;
pub global RANK_FOUR_OF_KIND: Field = 7;
pub global RANK_STRAIGHT_FLUSH: Field = 8;
pub global RANK_ROYAL_FLUSH: Field = 9;

// Community cards start after dealing to 9 players (18 cards)
pub global COMMUNITY_START: u32 = 18;

// ============================================
// TYPES
// ============================================

/// Represents a playing card with value and suit
pub struct Card {
    pub value: Field,  // 0=2, 1=3, ..., 8=10, 9=J, 10=Q, 11=K, 12=A
    pub suit: Field,   // 0=Clubs, 1=Diamonds, 2=Hearts, 3=Spades
}

impl Card {
    /// Creates a Card from a deck index (0-51)
    pub fn from_index(index: Field) -> Self {
        let idx = index as u32;
        Card {
            value: (idx % 13) as Field,
            suit: (idx / 13) as Field,
        }
    }

    /// Converts Card back to deck index
    pub fn to_index(self) -> Field {
        self.suit * 13 + self.value
    }
}

// ============================================
// HASH HELPERS (using Poseidon2)
// ============================================

use poseidon::poseidon2::Poseidon2;

/// Hash a single field element with a salt using Poseidon2
pub fn hash_with_salt(value: Field, salt: Field) -> Field {
    Poseidon2::hash([value, salt], 2)
}

/// Hash two field elements
pub fn hash_pair(a: Field, b: Field) -> Field {
    Poseidon2::hash([a, b], 2)
}

/// Hash an array of fields
pub fn hash_array<let N: u32>(arr: [Field; N]) -> Field {
    Poseidon2::hash(arr, N)
}

// ============================================
// VALIDATION HELPERS
// ============================================

/// Validates a card index is within valid range (0-51)
pub fn assert_valid_card(card: Field) {
    assert(card as u64 < DECK_SIZE as u64, "Card index must be 0-51");
}

/// Validates all cards in an array
pub fn assert_valid_cards<let N: u32>(cards: [Field; N]) {
    for i in 0..N {
        assert_valid_card(cards[i]);
    }
}

/// Checks if all cards in a deck are unique (no duplicates)
pub fn assert_no_duplicates<let N: u32>(cards: [Field; N]) {
    for i in 0..N {
        for j in (i + 1)..N {
            assert(cards[i] != cards[j], "Duplicate card found");
        }
    }
}

// ============================================
// HAND EVALUATION HELPERS
// ============================================

/// Count occurrences of each card value in a 7-card hand
pub fn count_values(cards: [Card; 7]) -> [Field; 13] {
    let mut counts: [Field; 13] = [0; 13];
    for i in 0..7 {
        let v = cards[i].value;
        for j in 0..13 {
            if v == j as Field {
                counts[j] = counts[j] + 1;
            }
        }
    }
    counts
}

/// Count occurrences of each suit in a 7-card hand
pub fn count_suits(cards: [Card; 7]) -> [Field; 4] {
    let mut counts: [Field; 4] = [0; 4];
    for i in 0..7 {
        let s = cards[i].suit;
        for j in 0..4 {
            if s == j as Field {
                counts[j] = counts[j] + 1;
            }
        }
    }
    counts
}

/// Check if hand contains a flush (5+ cards of same suit)
pub fn has_flush(cards: [Card; 7]) -> bool {
    let suit_counts = count_suits(cards);
    let mut found = false;
    for i in 0..4 {
        if suit_counts[i] as u64 >= 5 {
            found = true;
        }
    }
    found
}

/// Sort values array (bubble sort for simplicity in ZK)
pub fn sort_values(values: [Field; 7]) -> [Field; 7] {
    let mut sorted = values;
    for i in 0..6 {
        for j in 0..(6 - i) {
            if sorted[j] as u64 > sorted[j + 1] as u64 {
                let temp = sorted[j];
                sorted[j] = sorted[j + 1];
                sorted[j + 1] = temp;
            }
        }
    }
    sorted
}

/// Check if hand contains a straight (5 consecutive values)
pub fn has_straight(cards: [Card; 7]) -> bool {
    let mut values: [Field; 7] = [0; 7];
    for i in 0..7 {
        values[i] = cards[i].value;
    }
    let sorted = sort_values(values);

    let mut found = false;
    for start in 0..3 {
        let mut consecutive = true;
        for i in 0..4 {
            let curr = sorted[start + i] as u64;
            let next = sorted[start + i + 1] as u64;
            if (next != curr) & (next != curr + 1) {
                consecutive = false;
            }
        }
        if consecutive {
            let span = sorted[start + 4] as u64 - sorted[start] as u64;
            if span == 4 {
                found = true;
            }
        }
    }

    // Check for wheel straight (A-2-3-4-5)
    let has_ace = (sorted[6] == 12);
    let has_two = (sorted[0] == 0);
    let has_three = (sorted[1] == 1) | (sorted[0] == 1);
    let has_four = (sorted[2] == 2) | (sorted[1] == 2) | (sorted[0] == 2);
    let has_five = (sorted[3] == 3) | (sorted[2] == 3) | (sorted[1] == 3) | (sorted[0] == 3);

    if has_ace & has_two & has_three & has_four & has_five {
        found = true;
    }

    found
}

/// Evaluate a 7-card hand and return composite score: rank * 100 + primary_value
pub fn evaluate_hand_rank(cards: [Card; 7]) -> Field {
    let value_counts = count_values(cards);
    let flush = has_flush(cards);
    let straight = has_straight(cards);

    let mut pairs: Field = 0;
    let mut trips: Field = 0;
    let mut quads: Field = 0;
    let mut pair_value: Field = 0;
    let mut pair_value_2: Field = 0;
    let mut trip_value: Field = 0;
    let mut quad_value: Field = 0;
    let mut high_card: Field = 0;

    for i in 0..13 {
        let count = value_counts[i] as u64;
        let val = i as Field;

        if count >= 1 {
            high_card = val;
        }
        if count == 2 {
            pair_value_2 = pair_value;
            pair_value = val;
            pairs = pairs + 1;
        }
        if count == 3 {
            trip_value = val;
            trips = trips + 1;
        }
        if count == 4 {
            quad_value = val;
            quads = quads + 1;
        }
    }

    let mut rank = RANK_HIGH_CARD;
    let mut primary_value = high_card;

    if pairs as u64 > 0 {
        rank = RANK_ONE_PAIR;
        primary_value = pair_value;
    }
    if pairs as u64 > 1 {
        rank = RANK_TWO_PAIR;
        primary_value = pair_value;
    }
    if trips as u64 > 0 {
        rank = RANK_THREE_OF_KIND;
        primary_value = trip_value;
    }
    if straight {
        rank = RANK_STRAIGHT;
        primary_value = high_card;
    }
    if flush {
        rank = RANK_FLUSH;
        primary_value = high_card;
    }
    if (trips as u64 > 0) & (pairs as u64 > 0) {
        rank = RANK_FULL_HOUSE;
        primary_value = trip_value;
    }
    if quads as u64 > 0 {
        rank = RANK_FOUR_OF_KIND;
        primary_value = quad_value;
    }
    if straight & flush {
        let mut has_ten = false;
        let mut has_ace = false;
        for i in 0..7 {
            if cards[i].value == 8 {
                has_ten = true;
            }
            if cards[i].value == 12 {
                has_ace = true;
            }
        }
        if has_ten & has_ace {
            rank = RANK_ROYAL_FLUSH;
            primary_value = 12;
        } else {
            rank = RANK_STRAIGHT_FLUSH;
            primary_value = high_card;
        }
    }

    rank * 100 + primary_value
}

// ============================================
// TESTS
// ============================================

#[test]
fn test_card_from_index() {
    let card = Card::from_index(0);
    assert(card.value == 0);
    assert(card.suit == 0);

    let card2 = Card::from_index(51);
    assert(card2.value == 12);
    assert(card2.suit == 3);
}

#[test]
fn test_hash_deterministic() {
    let h1 = hash_with_salt(10, 12345);
    let h2 = hash_with_salt(10, 12345);
    assert(h1 == h2);
}

#[test]
fn test_hash_different_inputs() {
    let h1 = hash_with_salt(10, 12345);
    let h2 = hash_with_salt(11, 12345);
    assert(h1 != h2);
}

#[test]
fn test_valid_card_assertion() {
    // Test valid cards at boundaries
    assert_valid_card(0);  // First card
    assert_valid_card(51); // Last card
    assert_valid_card(25); // Middle card
}

#[test]
fn test_hash_array_deterministic() {
    let arr: [Field; 5] = [1, 2, 3, 4, 5];
    let h1 = hash_array(arr);
    let h2 = hash_array(arr);
    assert(h1 == h2);
}
