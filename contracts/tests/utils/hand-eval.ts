/**
 * Poker hand evaluation matching circuit logic
 * Matches: evaluate_hand_rank() in zkpoker_lib
 */

import { Card, cardFromIndex } from "./crypto";

// Hand rankings (higher is better) - matching circuit constants
export const RANK_HIGH_CARD = 0;
export const RANK_ONE_PAIR = 1;
export const RANK_TWO_PAIR = 2;
export const RANK_THREE_OF_KIND = 3;
export const RANK_STRAIGHT = 4;
export const RANK_FLUSH = 5;
export const RANK_FULL_HOUSE = 6;
export const RANK_FOUR_OF_KIND = 7;
export const RANK_STRAIGHT_FLUSH = 8;
export const RANK_ROYAL_FLUSH = 9;

/**
 * Count occurrences of each card value in a 7-card hand
 */
function countValues(cards: Card[]): number[] {
  const counts = new Array(13).fill(0);
  for (const card of cards) {
    counts[card.value]++;
  }
  return counts;
}

/**
 * Count occurrences of each suit in a 7-card hand
 */
function countSuits(cards: Card[]): number[] {
  const counts = new Array(4).fill(0);
  for (const card of cards) {
    counts[card.suit]++;
  }
  return counts;
}

/**
 * Check if hand contains a flush (5+ cards of same suit)
 */
function hasFlush(cards: Card[]): boolean {
  const suitCounts = countSuits(cards);
  return suitCounts.some((count) => count >= 5);
}

/**
 * Sort card values (bubble sort matching circuit)
 */
function sortValues(values: number[]): number[] {
  const sorted = [...values];
  for (let i = 0; i < sorted.length - 1; i++) {
    for (let j = 0; j < sorted.length - 1 - i; j++) {
      if (sorted[j] > sorted[j + 1]) {
        const temp = sorted[j];
        sorted[j] = sorted[j + 1];
        sorted[j + 1] = temp;
      }
    }
  }
  return sorted;
}

/**
 * Check if hand contains a straight (5 consecutive values)
 */
function hasStraight(cards: Card[]): boolean {
  const values = cards.map((c) => c.value);
  const sorted = sortValues(values);

  // Check for normal straights
  for (let start = 0; start < 3; start++) {
    let consecutive = true;
    for (let i = 0; i < 4; i++) {
      const curr = sorted[start + i];
      const next = sorted[start + i + 1];
      if (next !== curr && next !== curr + 1) {
        consecutive = false;
      }
    }
    if (consecutive) {
      const span = sorted[start + 4] - sorted[start];
      if (span === 4) {
        return true;
      }
    }
  }

  // Check for wheel straight (A-2-3-4-5)
  const hasAce = sorted[6] === 12;
  const hasTwo = sorted[0] === 0;
  const hasThree = sorted[1] === 1 || sorted[0] === 1;
  const hasFour = sorted[2] === 2 || sorted[1] === 2 || sorted[0] === 2;
  const hasFive = sorted[3] === 3 || sorted[2] === 3 || sorted[1] === 3 || sorted[0] === 3;

  return hasAce && hasTwo && hasThree && hasFour && hasFive;
}

/**
 * Evaluate a 7-card poker hand and return composite score
 * Returns: rank * 100 + primary_value
 *
 * This matches the circuit implementation exactly
 */
export function evaluateHandRank(cards: Card[]): number {
  if (cards.length !== 7) {
    throw new Error("Hand evaluation requires exactly 7 cards");
  }

  const valueCounts = countValues(cards);
  const flush = hasFlush(cards);
  const straight = hasStraight(cards);

  let pairs = 0;
  let trips = 0;
  let quads = 0;
  let pairValue = 0;
  let pairValue2 = 0;
  let tripValue = 0;
  let quadValue = 0;
  let highCard = 0;

  for (let i = 0; i < 13; i++) {
    const count = valueCounts[i];

    if (count >= 1) {
      highCard = i;
    }
    if (count === 2) {
      pairValue2 = pairValue;
      pairValue = i;
      pairs++;
    }
    if (count === 3) {
      tripValue = i;
      trips++;
    }
    if (count === 4) {
      quadValue = i;
      quads++;
    }
  }

  let rank = RANK_HIGH_CARD;
  let primaryValue = highCard;

  if (pairs > 0) {
    rank = RANK_ONE_PAIR;
    primaryValue = pairValue;
  }
  if (pairs > 1) {
    rank = RANK_TWO_PAIR;
    primaryValue = pairValue;
  }
  if (trips > 0) {
    rank = RANK_THREE_OF_KIND;
    primaryValue = tripValue;
  }
  if (straight) {
    rank = RANK_STRAIGHT;
    primaryValue = highCard;
  }
  if (flush) {
    rank = RANK_FLUSH;
    primaryValue = highCard;
  }
  if (trips > 0 && pairs > 0) {
    rank = RANK_FULL_HOUSE;
    primaryValue = tripValue;
  }
  if (quads > 0) {
    rank = RANK_FOUR_OF_KIND;
    primaryValue = quadValue;
  }
  if (straight && flush) {
    const hasTen = cards.some((c) => c.value === 8); // 10 is index 8
    const hasAce = cards.some((c) => c.value === 12);
    if (hasTen && hasAce) {
      rank = RANK_ROYAL_FLUSH;
      primaryValue = 12;
    } else {
      rank = RANK_STRAIGHT_FLUSH;
      primaryValue = highCard;
    }
  }

  return rank * 100 + primaryValue;
}

/**
 * Evaluate hand from card indices (0-51)
 */
export function evaluateHandFromIndices(cardIndices: number[]): number {
  if (cardIndices.length !== 7) {
    throw new Error("Hand evaluation requires exactly 7 cards");
  }
  const cards = cardIndices.map(cardFromIndex);
  return evaluateHandRank(cards);
}

/**
 * Create a 7-card hand from 2 hole cards + 5 community cards
 */
export function createSevenCardHand(
  holeCards: [number, number],
  communityCards: [number, number, number, number, number]
): number[] {
  return [...holeCards, ...communityCards];
}

/**
 * Get hand rank name
 */
export function getHandRankName(compositeScore: number): string {
  const rank = Math.floor(compositeScore / 100);
  const names = [
    "High Card",
    "One Pair",
    "Two Pair",
    "Three of a Kind",
    "Straight",
    "Flush",
    "Full House",
    "Four of a Kind",
    "Straight Flush",
    "Royal Flush",
  ];
  return names[rank] || "Unknown";
}

/**
 * Compare two hand ranks
 * Returns: 1 if hand1 wins, -1 if hand2 wins, 0 if tie
 */
export function compareHands(rank1: number, rank2: number): number {
  if (rank1 > rank2) return 1;
  if (rank2 > rank1) return -1;
  return 0;
}
