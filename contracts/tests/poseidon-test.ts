/**
 * Test Poseidon2 hash to verify it matches Noir circuits
 */

import { poseidon2 } from "poseidon-lite";
import { commitCard } from "./utils/crypto";
import { exec } from "child_process";
import { promisify } from "util";
import * as fs from "fs";
import * as path from "path";

const execAsync = promisify(exec);

async function testCircuitHash() {
  // Test values
  const card = 12;
  const salt = 111n;

  console.log("Test values:");
  console.log(`  card: ${card}`);
  console.log(`  salt: ${salt}`);
  console.log();

  // TypeScript Poseidon2
  const tsHash = poseidon2([BigInt(card), salt]);
  console.log("TypeScript poseidon-lite result:");
  console.log(`  ${tsHash}`);
  console.log(`  0x${tsHash.toString(16)}`);
  console.log();

  // Run the DECK circuit to see what it produces
  const deckDir = "/Users/brooklyn/Desktop/SchrodingerLabs/ZkPoker/circuits/crates/deck";
  const proverToml = `card1 = "${card}"
card2 = "25"
salt1 = "${salt}"
salt2 = "222"
_deck_seed = ["0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0"]
_player_seat = "0"
`;

  fs.writeFileSync(path.join(deckDir, "Prover.toml"), proverToml);

  // Execute circuit
  const { stdout } = await execAsync("nargo execute deck", { cwd: deckDir });
  console.log("Circuit execution output:");
  console.log(stdout);

  // The circuit outputs the commitments, parse them
  const match = stdout.match(/Circuit output: \[(0x[0-9a-f]+), (0x[0-9a-f]+)\]/);
  if (match) {
    const commitment1 = BigInt(match[1]);
    const commitment2 = BigInt(match[2]);
    console.log("Circuit produced commitment1:", commitment1);
    console.log("  Hex:", match[1]);
    console.log();

    console.log("Comparison:");
    console.log(`  TypeScript: ${tsHash}`);
    console.log(`  Circuit:    ${commitment1}`);
    console.log(`  Match: ${tsHash === commitment1 ? "✅" : "❌"}`);

    if (tsHash !== commitment1) {
      console.log();
      console.log("⚠️  MISMATCH DETECTED!");
      console.log("The TypeScript Poseidon2 implementation does not match the circuit.");
    }
  }
}

testCircuitHash().catch(console.error);
