/**
 * Cryptographic utilities matching circuit implementations
 * Uses Poseidon2 hash over BN254 curve (same as Noir circuits)
 */

import { poseidon2, poseidon5 } from "poseidon-lite";

/**
 * Convert a value to BigInt for hashing
 */
function toBigInt(value: number | bigint | string): bigint {
  return BigInt(value);
}

/**
 * Convert a 32-byte buffer to a BigInt field element
 * Used for converting deck_seed to field element
 */
export function bufferToField(buffer: Buffer): bigint {
  if (buffer.length !== 32) {
    throw new Error("Buffer must be exactly 32 bytes");
  }
  // Convert buffer to bigint (big-endian)
  let result = 0n;
  for (let i = 0; i < buffer.length; i++) {
    result = (result << 8n) | BigInt(buffer[i]);
  }
  // Ensure it fits in BN254 field (mod the field prime)
  // BN254 prime: 21888242871839275222246405745257275088548364400416034343698204186575808495617
  const BN254_FIELD_PRIME = 21888242871839275222246405745257275088548364400416034343698204186575808495617n;
  return result % BN254_FIELD_PRIME;
}

/**
 * Hash a single field element with a salt using Poseidon2
 * Matches: hash_with_salt(value: Field, salt: Field) -> Field
 */
export function hashWithSalt(value: number | bigint, salt: number | bigint): bigint {
  const v = toBigInt(value);
  const s = toBigInt(salt);
  return poseidon2([v, s]);
}

/**
 * Hash two field elements
 * Matches: hash_pair(a: Field, b: Field) -> Field
 */
export function hashPair(a: number | bigint, b: number | bigint): bigint {
  const a_ = toBigInt(a);
  const b_ = toBigInt(b);
  return poseidon2([a_, b_]);
}

/**
 * Hash an array of field elements
 * Matches: hash_array<let N: u32>(arr: [Field; N]) -> Field
 */
export function hashArray(arr: (number | bigint)[]): bigint {
  const bigIntArr = arr.map(toBigInt);

  // For arrays larger than 5, we need to use a sponge construction
  // or hash in chunks. For now, we support up to 52 elements (deck size)
  if (bigIntArr.length <= 2) {
    return poseidon2(bigIntArr);
  } else if (bigIntArr.length <= 5) {
    return poseidon5(bigIntArr);
  } else {
    // For larger arrays (like 52-card deck), use sponge construction
    // Hash in chunks of 5 and accumulate
    let state = 0n;
    for (let i = 0; i < bigIntArr.length; i += 5) {
      const chunk = bigIntArr.slice(i, Math.min(i + 5, bigIntArr.length));
      // Pad chunk to 5 elements with zeros if needed
      while (chunk.length < 5) {
        chunk.push(0n);
      }
      // Mix in previous state
      chunk[0] = poseidon2([chunk[0], state]);
      state = poseidon5(chunk);
    }
    return state;
  }
}

/**
 * Generate deterministic random value from seed and index
 * Used for Fisher-Yates shuffle
 */
export function randomFromSeed(seed: bigint, index: number): bigint {
  return poseidon2([seed, BigInt(index)]);
}

/**
 * Convert field element to number (mod n)
 * Used for shuffle algorithm
 */
export function fieldToNumber(field: bigint, n: number): number {
  return Number(field % BigInt(n));
}

/**
 * Commit to a card using Poseidon2
 * Matches: commit_card(card: Field, salt: Field) -> Field
 */
export function commitCard(card: number, salt: bigint): bigint {
  if (card < 0 || card > 51) {
    throw new Error("Card index must be 0-51");
  }
  return hashWithSalt(card, salt);
}

/**
 * Commit to multiple cards
 */
export function commitCards(cards: number[], salts: bigint[]): bigint[] {
  if (cards.length !== salts.length) {
    throw new Error("Cards and salts arrays must have same length");
  }
  return cards.map((card, i) => commitCard(card, salts[i]));
}

/**
 * Generate a random salt for commitments
 */
export function generateSalt(): bigint {
  // Generate 32 random bytes and convert to field element
  const bytes = new Uint8Array(32);
  crypto.getRandomValues(bytes);
  return bufferToField(Buffer.from(bytes));
}

/**
 * Card structure matching circuit
 * value: 0=2, 1=3, ..., 8=10, 9=J, 10=Q, 11=K, 12=A
 * suit: 0=Clubs, 1=Diamonds, 2=Hearts, 3=Spades
 */
export interface Card {
  value: number; // 0-12
  suit: number;  // 0-3
}

/**
 * Convert card index (0-51) to Card structure
 * Matches: Card::from_index(index: Field) -> Card
 */
export function cardFromIndex(index: number): Card {
  if (index < 0 || index > 51) {
    throw new Error("Card index must be 0-51");
  }
  return {
    value: index % 13,
    suit: Math.floor(index / 13),
  };
}

/**
 * Convert Card structure to index (0-51)
 * Matches: Card::to_index(self) -> Field
 */
export function cardToIndex(card: Card): number {
  return card.suit * 13 + card.value;
}

/**
 * Convert card to human-readable string
 */
export function cardToString(card: Card | number): string {
  const c = typeof card === "number" ? cardFromIndex(card) : card;
  const values = ["2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K", "A"];
  const suits = ["♣", "♦", "♥", "♠"];
  return values[c.value] + suits[c.suit];
}
