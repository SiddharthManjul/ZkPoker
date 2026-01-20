/**
 * ZK Proof generation using Sunspot (Groth16 for Solana)
 * Sunspot requires Noir 1.0.0-beta.13
 */

import { exec } from "child_process";
import { promisify } from "util";
import * as fs from "fs";
import * as path from "path";

const execAsync = promisify(exec);

// Paths (relative to project root, which is CWD when tests run)
const SUNSPOT_BIN = "/Users/brooklyn/Desktop/SchrodingerLabs/ZkPoker/sunspot/go/sunspot";
const CIRCUITS_TARGET = path.resolve(process.cwd(), "../circuits/target");
const CIRCUITS_CRATES = path.resolve(process.cwd(), "../circuits/crates");

/**
 * Generate witness file using nargo execute
 * Returns both witness path and circuit outputs (public values)
 */
async function generateWitness(
  circuitName: string,
  inputs: Record<string, any>
): Promise<{ witnessPath: string; publicOutputs: string | null }> {
  const circuitDir = path.join(CIRCUITS_CRATES, circuitName);
  const proverTomlPath = path.join(circuitDir, "Prover.toml");

  // Build Prover.toml content
  const tomlLines: string[] = [];
  for (const [key, value] of Object.entries(inputs)) {
    if (Array.isArray(value)) {
      const formatted = value.map((v) => `"${v}"`).join(", ");
      tomlLines.push(`${key} = [${formatted}]`);
    } else if (typeof value === "string" && value.startsWith("0x")) {
      tomlLines.push(`${key} = "${value}"`);
    } else {
      tomlLines.push(`${key} = "${value}"`);
    }
  }

  // Write Prover.toml
  fs.writeFileSync(proverTomlPath, tomlLines.join("\n") + "\n");

  // Execute circuit to generate witness and capture output
  const { stdout } = await execAsync(`nargo execute ${circuitName}`, { cwd: circuitDir });

  // Extract public outputs from circuit execution
  // Format: [circuitName] Circuit output: [0x..., 0x...]
  const outputMatch = stdout.match(/Circuit output: (.+)/);
  const publicOutputs = outputMatch ? outputMatch[1].trim() : null;

  // Witness is saved to workspace target directory
  const witnessPath = path.join(CIRCUITS_TARGET, `${circuitName}.gz`);

  if (!fs.existsSync(witnessPath)) {
    throw new Error(`Witness file not generated at ${witnessPath}`);
  }

  return { witnessPath, publicOutputs };
}

/**
 * Generate proof using Sunspot
 * Returns both proof and public witness
 */
async function generateProofWithSunspot(
  circuitName: string,
  witnessPath: string
): Promise<{ proof: Buffer; publicWitness: Buffer }> {
  const acirPath = path.join(CIRCUITS_TARGET, `${circuitName}.json`);
  const ccsPath = path.join(CIRCUITS_TARGET, `${circuitName}.ccs`);
  const pkPath = path.join(CIRCUITS_TARGET, `${circuitName}.pk`);
  const proofPath = path.join(CIRCUITS_TARGET, `${circuitName}.proof`);
  const publicWitnessPath = path.join(CIRCUITS_TARGET, `${circuitName}.pw`);

  // Verify files exist
  if (!fs.existsSync(acirPath)) {
    throw new Error(`ACIR file not found: ${acirPath}`);
  }
  if (!fs.existsSync(ccsPath)) {
    throw new Error(`CCS file not found: ${ccsPath}`);
  }
  if (!fs.existsSync(pkPath)) {
    throw new Error(`Proving key not found: ${pkPath}`);
  }

  // Generate proof using sunspot
  const cmd = `${SUNSPOT_BIN} prove ${acirPath} ${witnessPath} ${ccsPath} ${pkPath}`;
  await execAsync(cmd);

  // Sunspot writes proof and public witness to target directory
  if (!fs.existsSync(proofPath)) {
    throw new Error(`Proof not generated at ${proofPath}`);
  }
  if (!fs.existsSync(publicWitnessPath)) {
    throw new Error(`Public witness not generated at ${publicWitnessPath}`);
  }

  // Read proof and public witness
  const proof = fs.readFileSync(proofPath);
  const publicWitness = fs.readFileSync(publicWitnessPath);

  return { proof, publicWitness };
}

/**
 * Generate DECK circuit proof
 * Returns proof and commitments (from circuit public outputs)
 */
export async function generateDeckProof(params: {
  deckSeed: Buffer;
  playerSeat: number;
  card1: number;
  card2: number;
  salt1: bigint;
  salt2: bigint;
}): Promise<{ proof: Buffer; commitments: [bigint, bigint] }> {
  const { deckSeed, playerSeat, card1, card2, salt1, salt2 } = params;

  try {
    const inputs = {
      card1,
      card2,
      salt1: salt1.toString(),
      salt2: salt2.toString(),
      _deck_seed: Array.from(deckSeed),
      _player_seat: playerSeat,
    };

    const { witnessPath, publicOutputs } = await generateWitness("deck", inputs);

    // Parse commitments from circuit output: [0x..., 0x...]
    let commitments: [bigint, bigint] = [0n, 0n];
    if (publicOutputs) {
      const matches = publicOutputs.match(/0x[0-9a-f]+/gi);
      if (matches && matches.length >= 2) {
        commitments = [BigInt(matches[0]), BigInt(matches[1])];
      }
    }

    const { proof, publicWitness } = await generateProofWithSunspot("deck", witnessPath);
    // Sunspot verifier expects proof + public witness concatenated
    const proofAndWitness = Buffer.concat([proof, publicWitness]);
    return { proof: proofAndWitness, commitments };
  } catch (error: any) {
    throw new Error(`DECK proof generation failed: ${error.message}`);
  }
}

/**
 * Generate REVEAL circuit proof
 */
export async function generateRevealProof(params: {
  deckSeed: Buffer;
  cards: number[];
  numCards: number;
  shuffledDeck: number[];
}): Promise<Buffer> {
  const { deckSeed, cards, numCards, shuffledDeck } = params;

  try {
    // Pad cards array to 5 elements
    const paddedCards = [...cards];
    while (paddedCards.length < 5) {
      paddedCards.push(0);
    }

    const inputs = {
      shuffled_deck: shuffledDeck,
      _deck_seed: Array.from(deckSeed),
      cards: paddedCards,
      num_cards: numCards,
    };

    const { witnessPath } = await generateWitness("reveal", inputs);
    const { proof, publicWitness } = await generateProofWithSunspot("reveal", witnessPath);
    // Sunspot verifier expects proof + public witness concatenated
    return Buffer.concat([proof, publicWitness]);
  } catch (error: any) {
    throw new Error(`REVEAL proof generation failed: ${error.message}`);
  }
}

/**
 * Generate SHOWDOWN circuit proof
 * Returns proof and hand rank (circuit public output)
 */
export async function generateShowdownProof(params: {
  commitment1: bigint;
  commitment2: bigint;
  communityCards: [number, number, number, number, number];
  holeCard1: number;
  holeCard2: number;
  salt1: bigint;
  salt2: bigint;
}): Promise<{ proof: Buffer; handRank: bigint }> {
  const { commitment1, commitment2, communityCards, holeCard1, holeCard2, salt1, salt2 } = params;

  try {
    const inputs = {
      commitment1: "0x" + commitment1.toString(16).padStart(64, "0"),
      commitment2: "0x" + commitment2.toString(16).padStart(64, "0"),
      community_cards: Array.from(communityCards),
      hole_card1: holeCard1,
      hole_card2: holeCard2,
      salt1: salt1.toString(),
      salt2: salt2.toString(),
    };

    const { witnessPath, publicOutputs } = await generateWitness("showdown", inputs);

    // Parse hand rank from circuit output
    let handRank: bigint = 0n;
    if (publicOutputs) {
      const match = publicOutputs.match(/0x[0-9a-f]+/i);
      if (match) {
        handRank = BigInt(match[0]);
      }
    }

    const { proof, publicWitness } = await generateProofWithSunspot("showdown", witnessPath);
    // Sunspot verifier expects proof + public witness concatenated
    const proofAndWitness = Buffer.concat([proof, publicWitness]);
    return { proof: proofAndWitness, handRank };
  } catch (error: any) {
    throw new Error(`SHOWDOWN proof generation failed: ${error.message}`);
  }
}

/**
 * Convert proof Buffer to byte array for Anchor
 */
export function proofToBytes(proof: Buffer): number[] {
  return Array.from(proof);
}

/**
 * Verify proof size (Groth16 proof + public witness)
 * Proof is 388 bytes + variable-length public witness
 */
export function verifyProofSize(proof: Buffer, minSize: number = 388): boolean {
  // Proof should be at least 388 bytes (Groth16 proof)
  // Plus public witness (12 header bytes + N*32 bytes for fields)
  return proof.length >= minSize;
}
