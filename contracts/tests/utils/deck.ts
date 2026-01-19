/**
 * Deck shuffling utilities matching circuit expectations
 * Implements Fisher-Yates shuffle with Poseidon2-based PRNG
 */

import { randomFromSeed, fieldToNumber, hashArray, bufferToField } from "./crypto";

/**
 * Deterministically shuffle a deck using Fisher-Yates algorithm
 * with Poseidon2-based PRNG
 *
 * This must produce the same shuffled deck from the same seed
 *
 * @param deckSeed - 32-byte seed (from keccak256(seed1 || seed2))
 * @returns Array of 52 card indices (0-51)
 */
export function shuffleDeck(deckSeed: Buffer): number[] {
  if (deckSeed.length !== 32) {
    throw new Error("Deck seed must be exactly 32 bytes");
  }

  // Convert seed to field element for hashing
  const seedField = bufferToField(deckSeed);

  // Initialize deck with cards 0-51
  const deck: number[] = [];
  for (let i = 0; i < 52; i++) {
    deck.push(i);
  }

  // Fisher-Yates shuffle with deterministic PRNG
  for (let i = 51; i >= 1; i--) {
    // Generate random value from seed and current index
    const randomValue = randomFromSeed(seedField, i);

    // Convert to number in range [0, i]
    const j = fieldToNumber(randomValue, i + 1);

    // Swap deck[i] and deck[j]
    const temp = deck[i];
    deck[i] = deck[j];
    deck[j] = temp;
  }

  return deck;
}

/**
 * Get deck commitment (Poseidon2 hash of shuffled deck)
 * Matches: commit_deck(deck: [Field; 52]) -> Field
 */
export function commitDeck(shuffledDeck: number[]): bigint {
  if (shuffledDeck.length !== 52) {
    throw new Error("Deck must contain exactly 52 cards");
  }
  return hashArray(shuffledDeck);
}

/**
 * Verify deck integrity (all cards 0-51 present exactly once)
 * Matches: verify_deck_integrity(deck: [Field; 52])
 */
export function verifyDeckIntegrity(deck: number[]): boolean {
  if (deck.length !== 52) {
    return false;
  }

  const seen = new Array(52).fill(false);

  for (const card of deck) {
    if (card < 0 || card > 51) {
      return false;
    }
    if (seen[card]) {
      return false; // Duplicate
    }
    seen[card] = true;
  }

  // Check all cards present
  for (let i = 0; i < 52; i++) {
    if (!seen[i]) {
      return false; // Missing card
    }
  }

  return true;
}

/**
 * Card positions in shuffled deck for heads-up poker
 */
export const CARD_POSITIONS = {
  // Player hole cards (2 cards each)
  player0HoleCard1: 0,
  player0HoleCard2: 1,
  player1HoleCard1: 2,
  player1HoleCard2: 3,

  // Burn card positions (not used in our implementation)
  burnBeforeFlop: 4,

  // Community cards start at position 18 (after dealing to up to 9 players)
  COMMUNITY_START: 18,
  flop1: 18,
  flop2: 19,
  flop3: 20,
  turn: 21,
  river: 22,
} as const;

/**
 * Get hole cards for a player from shuffled deck
 * @param shuffledDeck - Full 52-card shuffled deck
 * @param seat - Player seat (0 or 1)
 * @returns Array of 2 card indices
 */
export function getHoleCards(shuffledDeck: number[], seat: number): [number, number] {
  if (seat !== 0 && seat !== 1) {
    throw new Error("Seat must be 0 or 1 for heads-up");
  }

  const pos1 = seat * 2;
  const pos2 = seat * 2 + 1;

  return [shuffledDeck[pos1], shuffledDeck[pos2]];
}

/**
 * Get flop cards from shuffled deck
 */
export function getFlopCards(shuffledDeck: number[]): [number, number, number] {
  return [
    shuffledDeck[CARD_POSITIONS.flop1],
    shuffledDeck[CARD_POSITIONS.flop2],
    shuffledDeck[CARD_POSITIONS.flop3],
  ];
}

/**
 * Get turn card from shuffled deck
 */
export function getTurnCard(shuffledDeck: number[]): number {
  return shuffledDeck[CARD_POSITIONS.turn];
}

/**
 * Get river card from shuffled deck
 */
export function getRiverCard(shuffledDeck: number[]): number {
  return shuffledDeck[CARD_POSITIONS.river];
}

/**
 * Get all community cards from shuffled deck
 */
export function getCommunityCards(shuffledDeck: number[]): {
  flop: [number, number, number];
  turn: number;
  river: number;
} {
  return {
    flop: getFlopCards(shuffledDeck),
    turn: getTurnCard(shuffledDeck),
    river: getRiverCard(shuffledDeck),
  };
}
