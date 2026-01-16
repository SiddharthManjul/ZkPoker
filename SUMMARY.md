# ZkPoker - Circuit Deployment Summary

## Overview

This document contains all deployment details for the ZkPoker ZK circuits on Solana devnet.

---

## Noir Version

```
nargo version = 1.0.0-beta.13
noirc version = 1.0.0-beta.13+6e469c3004209a8b107e7707306e25c80a110fd6
```

**Note:** Sunspot requires Noir 1.0.0-beta.13. To switch versions:
```bash
noirup -v 1.0.0-beta.13
```

---

## Sunspot Installation

```bash
# Clone and build
git clone https://github.com/reilabs/sunspot.git
cd sunspot/go
go build -o sunspot .

# Install to PATH
mkdir -p ~/bin
mv sunspot ~/bin/
export PATH="$HOME/bin:$PATH"

# Set environment variable for deploy
export GNARK_VERIFIER_BIN=/path/to/sunspot/gnark-solana/crates/verifier-bin
```

---

## Circuit Compilation Workflow

### 1. Compile Noir to ACIR
```bash
cd circuits
nargo compile
# Output: target/zkpoker.json
```

### 2. Compile ACIR to CCS (Sunspot)
```bash
sunspot compile ./target/zkpoker.json
# Output: target/zkpoker.ccs
```

### 3. Generate Proving & Verifying Keys
```bash
sunspot setup ./target/zkpoker.ccs
# Output: target/zkpoker.pk, target/zkpoker.vk
```

### 4. Generate Witness
```bash
nargo execute
# Output: target/zkpoker.gz
```

### 5. Generate Groth16 Proof
```bash
sunspot prove ./target/zkpoker.json ./target/zkpoker.gz ./target/zkpoker.ccs ./target/zkpoker.pk
# Output: target/zkpoker.proof, target/zkpoker.pw
```

### 6. Verify Locally
```bash
sunspot verify ./target/zkpoker.vk ./target/zkpoker.proof ./target/zkpoker.pw
# Output: "Verification successful!"
```

### 7. Build Solana Verifier Program
```bash
export GNARK_VERIFIER_BIN=/Users/brooklyn/Desktop/SchrodingerLabs/ZkPoker/sunspot/gnark-solana/crates/verifier-bin
sunspot deploy ./target/zkpoker.vk
# Output: target/zkpoker.so, target/zkpoker-keypair.json
```

### 8. Deploy to Solana
```bash
solana config set --url devnet
solana program deploy ./target/zkpoker.so --program-id ./target/zkpoker-keypair.json
```

---

## Deployed Verifier Program

| Field | Value |
|-------|-------|
| **Program ID** | `5fkbNoQZykoAz4SmKKkGg6ajKfRArwKq7Y9yUcgRANBe` |
| **Network** | Devnet |
| **Deployment Date** | 2026-01-16 |
| **Explorer** | [View on Solana Explorer](https://explorer.solana.com/address/5fkbNoQZykoAz4SmKKkGg6ajKfRArwKq7Y9yUcgRANBe?cluster=devnet) |

---

## Generated Artifacts

| File | Size | Description |
|------|------|-------------|
| `zkpoker.json` | 16 KB | Noir ACIR (compiled circuit) |
| `zkpoker.ccs` | 55 KB | Constraint system (Sunspot format) |
| `zkpoker.pk` | 174 KB | Proving key (client-side, keep secure) |
| `zkpoker.vk` | 1 KB | Verifying key (embedded in Solana program) |
| `zkpoker.gz` | 332 B | Witness (private inputs) |
| `zkpoker.proof` | 388 B | Groth16 proof (sent on-chain) |
| `zkpoker.pw` | 12 B | Public witness |
| `zkpoker.so` | 200 KB | Solana verifier program (BPF) |
| `zkpoker-keypair.json` | 228 B | Program keypair |

---

## Circuit Details

### Main Circuit Function
```noir
fn main(card: Field, salt: Field) -> pub Field {
    assert_valid_card(card);
    hash_with_salt(card, salt)
}
```

### Inputs
| Input | Type | Description |
|-------|------|-------------|
| `card` | Field | Card index (0-51) |
| `salt` | Field | Random salt for hiding commitment |

### Output
| Output | Type | Description |
|--------|------|-------------|
| `commitment` | Field (public) | Poseidon2 hash of card + salt |

### Test Values Used
```toml
# Prover.toml
card = "12"                      # Ace of Clubs
salt = "98765432101234567890"    # Random salt
```

### Expected Output
```
commitment = 0x03d49c089322cbb993bc5ff172af13d902fc2a843314874f48c5058e089d334a
```

---

## Circuit Modules

| Module | File | Purpose |
|--------|------|---------|
| `lib` | `src/lib.nr` | Card struct, hash helpers, hand evaluation |
| `deck/commit` | `src/deck/commit.nr` | Card commitment functions |
| `deck/shuffle` | `src/deck/shuffle.nr` | Deck integrity verification |
| `deal` | `src/deal/deal.nr` | Deal verification, player hand ownership |
| `bet` | `src/bet/balance.nr` | Balance commitments, bet validation |
| `reveal` | `src/reveal/reveal.nr` | Community card verification |
| `showdown` | `src/showdown/showdown.nr` | Hand ranking, winner determination |

### Test Results
```
40 tests passed
```

---

## Solana Wallet

| Field | Value |
|-------|-------|
| **Pubkey** | `ZePH1U86mgTx2ZaWKwpBAVvXH2RhGAGMFrvXutdAPth` |
| **Keypair** | `/Users/brooklyn/.config/solana/id.json` |
| **Network** | Devnet |

---

## Integration with Game Contracts

### CPI Call to Verifier
```rust
// In Anchor program
use anchor_lang::prelude::*;

pub const VERIFIER_PROGRAM_ID: Pubkey = pubkey!("5fkbNoQZykoAz4SmKKkGg6ajKfRArwKq7Y9yUcgRANBe");

// CPI to verify proof
let cpi_accounts = VerifyProof {
    // ... accounts as required by verifier
};

let proof: [u8; 388] = /* from client */;
let public_inputs: [u8; 32] = /* commitment hash */;

// Invoke verifier
verify_groth16_proof(proof, public_inputs)?;
```

### Client-Side Proof Generation
```typescript
// 1. Generate witness
const witness = await generateWitness(card, salt);

// 2. Generate Groth16 proof using proving key
const { proof, publicInputs } = await prove(
  witness,
  provingKey,
  ccs
);

// 3. Send to Solana
await program.methods
  .verifyCardCommitment(proof, publicInputs)
  .accounts({ /* ... */ })
  .rpc();
```

---

## Proof Sizes (On-Chain Costs)

| Component | Size | Est. Compute Units |
|-----------|------|-------------------|
| Groth16 Proof | 388 bytes | ~200,000 CU |
| Public Inputs | 32 bytes | - |
| Total TX Size | ~420 bytes | ~200,000 CU |
| Est. Cost | - | ~0.00005 SOL |

---

## Troubleshooting

### Noir Version Mismatch
```bash
# Error: ACIR parsing fails
# Solution: Downgrade Noir
noirup -v 1.0.0-beta.13
nargo compile
```

### Sunspot Not Found
```bash
export PATH="$HOME/bin:$PATH"
```

### Deploy Fails (Insufficient Funds)
```bash
solana airdrop 5
```

### Verifier Build Fails
```bash
# Ensure GNARK_VERIFIER_BIN is set
export GNARK_VERIFIER_BIN=/path/to/sunspot/gnark-solana/crates/verifier-bin
```

---

## File Locations

```
ZkPoker/
├── circuits/
│   ├── Nargo.toml
│   ├── Prover.toml
│   ├── src/
│   │   ├── lib.nr
│   │   ├── main.nr
│   │   ├── deck/
│   │   ├── deal/
│   │   ├── bet/
│   │   ├── reveal/
│   │   └── showdown/
│   └── target/
│       ├── zkpoker.json      # ACIR
│       ├── zkpoker.ccs       # CCS
│       ├── zkpoker.pk        # Proving key
│       ├── zkpoker.vk        # Verifying key
│       ├── zkpoker.proof     # Groth16 proof
│       ├── zkpoker.pw        # Public witness
│       ├── zkpoker.so        # Solana program
│       └── zkpoker-keypair.json
├── sunspot/                   # Cloned repo
│   ├── go/
│   └── gnark-solana/
└── SUMMARY.md                 # This file
```

---

## Quick Reference Commands

```bash
# Recompile everything
cd circuits
nargo compile
sunspot compile ./target/zkpoker.json
sunspot setup ./target/zkpoker.ccs

# Generate and verify proof
nargo execute
sunspot prove ./target/zkpoker.json ./target/zkpoker.gz ./target/zkpoker.ccs ./target/zkpoker.pk
sunspot verify ./target/zkpoker.vk ./target/zkpoker.proof ./target/zkpoker.pw

# Rebuild and redeploy verifier
export GNARK_VERIFIER_BIN=/Users/brooklyn/Desktop/SchrodingerLabs/ZkPoker/sunspot/gnark-solana/crates/verifier-bin
sunspot deploy ./target/zkpoker.vk
solana program deploy ./target/zkpoker.so --program-id ./target/zkpoker-keypair.json
```
