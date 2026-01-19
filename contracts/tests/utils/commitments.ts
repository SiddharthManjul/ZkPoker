/**
 * Card commitment utilities for ZK poker
 */

import { commitCard, commitCards, generateSalt, hashWithSalt } from "./crypto";

/**
 * Generate commitments for hole cards with random salts
 */
export function generateHoleCardCommitments(
  card1: number,
  card2: number
): {
  commitments: [bigint, bigint];
  salts: [bigint, bigint];
  cards: [number, number];
} {
  const salt1 = generateSalt();
  const salt2 = generateSalt();

  return {
    commitments: [commitCard(card1, salt1), commitCard(card2, salt2)],
    salts: [salt1, salt2],
    cards: [card1, card2],
  };
}

/**
 * Verify a card matches its commitment
 */
export function verifyCommitment(
  card: number,
  salt: bigint,
  commitment: bigint
): boolean {
  const computed = commitCard(card, salt);
  return computed === commitment;
}

/**
 * Verify hole card commitments
 */
export function verifyHoleCommitments(
  cards: [number, number],
  salts: [bigint, bigint],
  commitments: [bigint, bigint]
): boolean {
  return (
    verifyCommitment(cards[0], salts[0], commitments[0]) &&
    verifyCommitment(cards[1], salts[1], commitments[1])
  );
}

/**
 * Convert BigInt to Buffer for on-chain submission
 * Field elements in BN254 are 32 bytes
 */
export function fieldToBuffer(field: bigint): Buffer {
  const hex = field.toString(16).padStart(64, "0");
  return Buffer.from(hex, "hex");
}

/**
 * Convert BigInt array to Buffer array
 */
export function fieldsToBuffers(fields: bigint[]): Buffer[] {
  return fields.map(fieldToBuffer);
}

/**
 * Convert commitment to Uint8Array format expected by Anchor
 */
export function commitmentToBytes(commitment: bigint): number[] {
  const buffer = fieldToBuffer(commitment);
  return Array.from(buffer);
}

/**
 * Convert commitments array to bytes format
 */
export function commitmentsToBytes(commitments: bigint[]): number[][] {
  return commitments.map(commitmentToBytes);
}
