# ZkPoker - Part 1: Circuits & Contracts (Foundation Layer)

> **Development Order:** This file covers **Days 1-3** of the implementation roadmap.
> - Day 1: Noir Circuits
> - Day 2: Solana Program Core
> - Day 3: Token & Escrow System
>
> See `CLAUDE-PART2.md` for Client (Backend API + Frontend) - Days 4-7.

---

## Project Overview

ZkPoker is a decentralized, privacy-preserving Texas Hold'em poker platform built using Zero-Knowledge Proofs (Noir circuits) and deployed on Solana via Sunspot. The platform enables provably fair poker games where card privacy is cryptographically guaranteed, eliminating trust requirements for both the platform and other players.

**Hackathon:** Solana Privacy Hack
**Timeline:** 7 days
**Primary Game Mode:** Texas Hold'em (2-9 players)
**MVP Goals:** Multi-player support, multiple concurrent tables, token escrow, ZK-verified game outcomes

---

## Project Structure

```
ZkPoker/
├── circuits/                    # Noir ZK circuits
│   ├── Nargo.toml              # Noir package manifest
│   └── src/
│       └── main.nr             # Circuit implementations
├── contracts/                   # Solana/Anchor programs
│   ├── Anchor.toml             # Anchor configuration
│   ├── Cargo.toml              # Rust workspace
│   ├── programs/
│   │   └── contracts/
│   │       └── src/
│   │           └── lib.rs      # Program entry point
│   ├── tests/
│   │   └── contracts.ts        # Integration tests
│   └── migrations/
│       └── deploy.ts           # Deployment script
├── client/                      # Next.js (Frontend + Backend API)
│   ├── app/                    # Next.js App Router
│   │   ├── api/               # Backend API routes
│   │   ├── layout.tsx
│   │   └── page.tsx
│   ├── package.json
│   └── tsconfig.json
├── CLAUDE-PART1.md             # This file (Circuits + Contracts)
└── CLAUDE-PART2.md             # Client documentation
```

---

## Architecture Overview

### Three-Layer Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Client (Next.js)                          │
│  ┌─────────────────────┐  ┌─────────────────────────────┐   │
│  │  Frontend (React)    │  │  Backend (API Routes)       │   │
│  │  - Poker UI          │  │  - REST endpoints           │   │
│  │  - Wallet Adapter    │  │  - WebSocket server         │   │
│  │  - Proof Generation  │  │  - Database queries         │   │
│  │  - Animations        │  │  - Game orchestration       │   │
│  └─────────────────────┘  └─────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                            ↕
┌─────────────────────────────────────────────────────────────┐
│                   ZK Layer (Noir Circuits)                   │
│  - Card commitment schemes                                   │
│  - Hand evaluation circuits                                  │
│  - Shuffle verification                                      │
│  - Reveal verification                                       │
└─────────────────────────────────────────────────────────────┘
                            ↕
┌─────────────────────────────────────────────────────────────┐
│              Blockchain Layer (Solana + Sunspot)             │
│  ┌─────────────────────┐  ┌─────────────────────────────┐   │
│  │  Game Program        │  │  Token Layer                │   │
│  │  - State management  │  │  - SPL Token escrow         │   │
│  │  - Proof verification│  │  - Buy-in / Payout          │   │
│  │  - Player actions    │  │  - Rake collection          │   │
│  └─────────────────────┘  └─────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Key Design Principles

1. **Privacy First:** No player can see others' hole cards until showdown
2. **Trustless:** No central authority holds privileged information
3. **Verifiable:** Every action is cryptographically proven
4. **Performant:** Optimized for Solana's high throughput
5. **Composable:** Token-based chips enable DeFi integration

---

## Technical Stack (Foundation)

### Zero-Knowledge Layer (`circuits/`)
- **Noir** (v0.39.0+): Circuit development language
- **Barretenberg**: Proof generation backend
- **Nargo**: Noir compiler and testing framework

### Blockchain Layer (`contracts/`)
- **Solana** (mainnet-beta): Layer 1 blockchain
- **Sunspot**: ZK proof verifier for Solana
- **Anchor Framework** (v0.32.1): Solana program development
- **SPL Token**: Fungible token standard for chips
- **Rust** (v1.89.0): Smart contract language

### Current Configuration

**contracts/Anchor.toml:**
```toml
[programs.localnet]
contracts = "GnDHa3pfhiqEG5xVTjtnTYue33ceX6disU8F2YJymqYr"

[provider]
cluster = "localnet"
wallet = "~/.config/solana/id.json"
```

---

## Circuits (`circuits/`)

### Current Structure

```
circuits/
├── Nargo.toml                   # Package manifest
└── src/
    └── main.nr                  # Circuit implementations
```

**Current Nargo.toml:**
```toml
[package]
name = "circuits"
type = "bin"
authors = [""]

[dependencies]
```

### Required Circuit Implementations

The following circuits need to be implemented in `circuits/src/`:

---

### Circuit 1: Card Commitment

**Purpose:** Generate cryptographic commitment to a card without revealing it.

**File:** `circuits/src/commit.nr`

```rust
use dep::std;

// Poseidon hash for commitment
fn poseidon_hash(card: Field, salt: Field) -> Field {
    std::hash::poseidon2::Poseidon2::hash([card, salt], 2)
}

fn main(
    card: Field,        // Card value (0-51)
    salt: Field         // Random salt for hiding
) -> pub Field {       // Public commitment
    // Validate card is in range
    assert(card < 52);

    // Generate commitment
    poseidon_hash(card, salt)
}

#[test]
fn test_commitment() {
    let card = 10;
    let salt = 12345;
    let commitment = main(card, salt);
    // Commitment should be deterministic
    assert(commitment == main(card, salt));
}
```

---

### Circuit 2: Card Reveal

**Purpose:** Prove you know the card behind a commitment without trusting the prover.

**File:** `circuits/src/reveal.nr`

```rust
use dep::std;

fn poseidon_hash(card: Field, salt: Field) -> Field {
    std::hash::poseidon2::Poseidon2::hash([card, salt], 2)
}

fn main(
    card: Field,                    // Private: card being revealed
    salt: Field,                    // Private: salt used in commitment
    pub expected_commitment: Field  // Public: commitment to verify against
) {
    // Validate card range
    assert(card < 52);

    // Recompute commitment
    let commitment = poseidon_hash(card, salt);

    // Verify it matches public commitment
    assert(commitment == expected_commitment);
}

#[test]
fn test_reveal() {
    let card = 10;
    let salt = 12345;
    let commitment = std::hash::poseidon2::Poseidon2::hash([card, salt], 2);
    main(card, salt, commitment); // Should not panic
}
```

**Public Inputs:** `[expected_commitment]`
**Private Inputs:** `[card, salt]`

---

### Circuit 3: Hand Evaluation

**Purpose:** Compute hand rank (pair, flush, straight, etc.) in zero-knowledge.

**File:** `circuits/src/evaluate_hand.nr`

```rust
use dep::std;

// Card representation: value (0-12) and suit (0-3)
struct Card {
    value: Field,  // 0=2, 1=3, ..., 12=Ace
    suit: Field    // 0=Clubs, 1=Diamonds, 2=Hearts, 3=Spades
}

// Hand rankings (higher is better)
global RANK_HIGH_CARD: Field = 0;
global RANK_ONE_PAIR: Field = 1;
global RANK_TWO_PAIR: Field = 2;
global RANK_THREE_OF_KIND: Field = 3;
global RANK_STRAIGHT: Field = 4;
global RANK_FLUSH: Field = 5;
global RANK_FULL_HOUSE: Field = 6;
global RANK_FOUR_OF_KIND: Field = 7;
global RANK_STRAIGHT_FLUSH: Field = 8;
global RANK_ROYAL_FLUSH: Field = 9;

// Convert card index (0-51) to Card struct
fn index_to_card(index: Field) -> Card {
    let value = index % 13;
    let suit = index / 13;
    Card { value, suit }
}

// Check if 5+ cards have same suit (flush)
fn is_flush(cards: [Card; 7]) -> bool {
    let mut suit_counts: [Field; 4] = [0; 4];
    for i in 0..7 {
        let suit = cards[i].suit as u32;
        suit_counts[suit] = suit_counts[suit] + 1;
    }
    let mut has_flush = false;
    for i in 0..4 {
        if suit_counts[i] >= 5 {
            has_flush = true;
        }
    }
    has_flush
}

// Check for straight (5 consecutive values)
fn is_straight(values: [Field; 7]) -> bool {
    // Sort values (simple bubble sort)
    let mut sorted = values;
    for i in 0..6 {
        for j in 0..(6-i) {
            if sorted[j] > sorted[j+1] {
                let temp = sorted[j];
                sorted[j] = sorted[j+1];
                sorted[j+1] = temp;
            }
        }
    }

    // Check for 5 consecutive
    for i in 0..3 {
        let mut consecutive = true;
        for j in 0..4 {
            if sorted[i+j+1] != sorted[i+j] + 1 {
                consecutive = false;
            }
        }
        if consecutive {
            return true;
        }
    }

    // Check for wheel (A-2-3-4-5)
    if sorted[0] == 0 && sorted[1] == 1 && sorted[2] == 2 &&
       sorted[3] == 3 && sorted[6] == 12 {
        return true;
    }

    false
}

// Count occurrences of each value
fn count_values(cards: [Card; 7]) -> [Field; 13] {
    let mut counts: [Field; 13] = [0; 13];
    for i in 0..7 {
        let value = cards[i].value as u32;
        counts[value] = counts[value] + 1;
    }
    counts
}

// Determine hand rank
fn evaluate_hand(cards: [Card; 7]) -> Field {
    let counts = count_values(cards);
    let values = [cards[0].value, cards[1].value, cards[2].value,
                  cards[3].value, cards[4].value, cards[5].value,
                  cards[6].value];

    let flush = is_flush(cards);
    let straight = is_straight(values);

    // Count pairs, trips, quads
    let mut pairs = 0;
    let mut trips = 0;
    let mut quads = 0;

    for i in 0..13 {
        if counts[i] == 2 { pairs = pairs + 1; }
        if counts[i] == 3 { trips = trips + 1; }
        if counts[i] == 4 { quads = quads + 1; }
    }

    // Determine rank
    if straight && flush {
        // Check for royal flush (10-J-Q-K-A)
        let mut has_ten = false;
        let mut has_ace = false;
        for i in 0..7 {
            if cards[i].value == 8 { has_ten = true; }
            if cards[i].value == 12 { has_ace = true; }
        }
        if has_ten && has_ace {
            return RANK_ROYAL_FLUSH;
        }
        return RANK_STRAIGHT_FLUSH;
    }

    if quads > 0 { return RANK_FOUR_OF_KIND; }
    if trips > 0 && pairs > 0 { return RANK_FULL_HOUSE; }
    if flush { return RANK_FLUSH; }
    if straight { return RANK_STRAIGHT; }
    if trips > 0 { return RANK_THREE_OF_KIND; }
    if pairs > 1 { return RANK_TWO_PAIR; }
    if pairs > 0 { return RANK_ONE_PAIR; }

    RANK_HIGH_CARD
}

fn main(
    // Private inputs
    hole_cards: [Field; 2],           // Player's 2 hole cards (indices 0-51)
    community_cards: [Field; 5],      // 5 community cards (indices 0-51)
    hole_salts: [Field; 2],           // Salts for commitments

    // Public inputs
    pub hole_commitments: [Field; 2], // Commitments to hole cards
    pub hand_rank: Field              // Claimed hand rank
) {
    // Verify hole card commitments
    for i in 0..2 {
        let commitment = std::hash::poseidon2::Poseidon2::hash(
            [hole_cards[i], hole_salts[i]], 2
        );
        assert(commitment == hole_commitments[i]);
    }

    // Convert all cards to Card structs
    let mut all_cards: [Card; 7] = [Card { value: 0, suit: 0 }; 7];
    all_cards[0] = index_to_card(hole_cards[0]);
    all_cards[1] = index_to_card(hole_cards[1]);
    for i in 0..5 {
        all_cards[i+2] = index_to_card(community_cards[i]);
    }

    // Evaluate hand
    let computed_rank = evaluate_hand(all_cards);

    // Verify claimed rank matches
    assert(computed_rank == hand_rank);
}

#[test]
fn test_one_pair() {
    // Two aces
    let hole_cards = [12, 25]; // A♣, A♦
    let community = [0, 14, 28, 41, 3]; // 2♣, 2♦, 2♥, 4♠, 5♣
    let salts = [111, 222];
    let commitments = [
        std::hash::poseidon2::Poseidon2::hash([hole_cards[0], salts[0]], 2),
        std::hash::poseidon2::Poseidon2::hash([hole_cards[1], salts[1]], 2)
    ];
    main(hole_cards, community, salts, commitments, RANK_FULL_HOUSE);
}
```

**Public Inputs:** `[hole_commitments[2], hand_rank]`
**Private Inputs:** `[hole_cards[2], community_cards[5], hole_salts[2]]`
**Constraint Count:** ~15,000-20,000 (needs optimization)

---

### Circuit 4: Shuffle Verification (MVP - Simplified)

**Purpose:** Verify server's shuffle commitment.

**File:** `circuits/src/shuffle.nr`

```rust
use dep::std;

fn main(
    // Private
    shuffled_deck: [Field; 52],

    // Public
    pub deck_commitment: Field
) {
    // Verify deck commitment
    let computed_commitment = std::hash::poseidon2::Poseidon2::hash(shuffled_deck, 52);
    assert(computed_commitment == deck_commitment);

    // Verify all cards present (0-51)
    let mut card_present: [bool; 52] = [false; 52];
    for i in 0..52 {
        let card = shuffled_deck[i] as u32;
        assert(card < 52);
        assert(!card_present[card]); // No duplicates
        card_present[card] = true;
    }
}
```

**Note:** For MVP, we use a hybrid approach with trusted server shuffle. Full mental poker protocol can be added later.

---

### Circuit 5: Winner Determination

**Purpose:** Compare multiple players' hands and determine winner(s).

**File:** `circuits/src/determine_winner.nr`

```rust
use dep::std;

fn main(
    // Private: each player's cards
    player_hands: [[Field; 2]; 9],     // Up to 9 players, 2 hole cards each
    player_salts: [[Field; 2]; 9],     // Salts for commitments

    // Public
    pub community_cards: [Field; 5],   // Visible community cards
    pub player_commitments: [[Field; 2]; 9],
    pub player_ranks: [Field; 9],      // Each player's claimed rank
    pub active_players: [bool; 9],     // Who hasn't folded
    pub winner_index: Field            // Claimed winner
) {
    // Verify all commitments for active players
    for i in 0..9 {
        if active_players[i] {
            for j in 0..2 {
                let commitment = std::hash::poseidon2::Poseidon2::hash(
                    [player_hands[i][j], player_salts[i][j]], 2
                );
                assert(commitment == player_commitments[i][j]);
            }
        }
    }

    // Find best hand among active players
    let mut best_rank: Field = 0;
    let mut best_player: Field = 0;

    for i in 0..9 {
        if active_players[i] {
            if player_ranks[i] > best_rank {
                best_rank = player_ranks[i];
                best_player = i as Field;
            }
        }
    }

    // Verify claimed winner
    assert(best_player == winner_index);
}
```

---

### Circuit Organization

**Recommended Nargo.toml update:**
```toml
[package]
name = "zkpoker_circuits"
type = "bin"
authors = ["ZkPoker Team"]

[dependencies]
```

**File structure for multiple circuits:**
```
circuits/
├── Nargo.toml
└── src/
    ├── main.nr           # Re-exports or main circuit
    ├── commit.nr         # Card commitment
    ├── reveal.nr         # Card reveal
    ├── evaluate_hand.nr  # Hand evaluation
    ├── shuffle.nr        # Shuffle verification
    └── determine_winner.nr # Winner determination
```

---

## Contracts (`contracts/`)

### Current Structure

```
contracts/
├── Anchor.toml                  # Anchor configuration
├── Cargo.toml                   # Rust workspace
├── rust-toolchain.toml          # Rust 1.89.0
├── package.json                 # @coral-xyz/anchor v0.32.1
├── programs/
│   └── contracts/
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs           # Program entry (template)
├── tests/
│   └── contracts.ts             # Integration tests
└── migrations/
    └── deploy.ts                # Deployment script
```

### Required Program Implementation

Replace `contracts/programs/contracts/src/lib.rs` with:

```rust
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};

declare_id!("GnDHa3pfhiqEG5xVTjtnTYue33ceX6disU8F2YJymqYr");

// PDA Seeds
pub const TABLE_SEED: &[u8] = b"table";
pub const PLAYER_SEED: &[u8] = b"player";
pub const ESCROW_SEED: &[u8] = b"escrow";

#[program]
pub mod zkpoker {
    use super::*;

    pub fn create_table(
        ctx: Context<CreateTable>,
        table_id: u64,
        small_blind: u64,
        big_blind: u64,
        max_players: u8
    ) -> Result<()> {
        let game_table = &mut ctx.accounts.game_table;

        game_table.table_id = table_id;
        game_table.small_blind = small_blind;
        game_table.big_blind = big_blind;
        game_table.max_players = max_players;
        game_table.player_count = 0;
        game_table.stage = GameStage::Waiting;
        game_table.pot = 0;
        game_table.current_bet = 0;
        game_table.active = true;
        game_table.bump = ctx.bumps.game_table;

        emit!(TableCreatedEvent {
            table_id,
            small_blind,
            big_blind,
            creator: ctx.accounts.creator.key()
        });

        Ok(())
    }

    pub fn join_table(ctx: Context<JoinTable>, buy_in: u64) -> Result<()> {
        let game_table = &mut ctx.accounts.game_table;
        let player_state = &mut ctx.accounts.player_state;

        require!(
            game_table.player_count < game_table.max_players,
            ErrorCode::TableFull
        );
        require!(
            game_table.stage == GameStage::Waiting,
            ErrorCode::GameInProgress
        );

        // Transfer tokens to escrow
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.player_token_account.to_account_info(),
                    to: ctx.accounts.table_escrow.to_account_info(),
                    authority: ctx.accounts.player.to_account_info(),
                }
            ),
            buy_in
        )?;

        // Initialize player state
        player_state.player = ctx.accounts.player.key();
        player_state.table_id = game_table.table_id;
        player_state.chips = buy_in;
        player_state.current_bet = 0;
        player_state.has_folded = false;
        player_state.is_all_in = false;
        player_state.position = game_table.player_count;
        player_state.bump = ctx.bumps.player_state;

        game_table.player_count += 1;

        emit!(PlayerJoinedEvent {
            table_id: game_table.table_id,
            player: ctx.accounts.player.key(),
            buy_in,
            position: player_state.position
        });

        Ok(())
    }

    pub fn player_action(
        ctx: Context<PlayerAction>,
        action: ActionType,
        amount: u64
    ) -> Result<()> {
        let game_table = &mut ctx.accounts.game_table;
        let player_state = &mut ctx.accounts.player_state;

        require!(
            game_table.current_player_index == player_state.position,
            ErrorCode::NotPlayerTurn
        );
        require!(!player_state.has_folded, ErrorCode::PlayerFolded);

        match action {
            ActionType::Fold => {
                player_state.has_folded = true;
            },
            ActionType::Check => {
                require!(
                    player_state.current_bet == game_table.current_bet,
                    ErrorCode::CannotCheck
                );
            },
            ActionType::Call => {
                let call_amount = game_table.current_bet - player_state.current_bet;
                require!(
                    player_state.chips >= call_amount,
                    ErrorCode::InsufficientChips
                );
                player_state.chips -= call_amount;
                player_state.current_bet += call_amount;
                game_table.pot += call_amount;
            },
            ActionType::Bet | ActionType::Raise => {
                require!(
                    amount >= game_table.big_blind,
                    ErrorCode::BetTooSmall
                );
                require!(
                    player_state.chips >= amount,
                    ErrorCode::InsufficientChips
                );
                player_state.chips -= amount;
                player_state.current_bet += amount;
                game_table.pot += amount;
                game_table.current_bet = player_state.current_bet;
            },
            ActionType::AllIn => {
                let all_in_amount = player_state.chips;
                player_state.current_bet += all_in_amount;
                game_table.pot += all_in_amount;
                player_state.chips = 0;
                player_state.is_all_in = true;
                if player_state.current_bet > game_table.current_bet {
                    game_table.current_bet = player_state.current_bet;
                }
            }
        }

        // Move to next player
        game_table.current_player_index = (game_table.current_player_index + 1)
            % game_table.player_count;

        emit!(PlayerActionEvent {
            table_id: game_table.table_id,
            player: player_state.player,
            action,
            amount
        });

        Ok(())
    }

    pub fn distribute_pot(ctx: Context<DistributePot>) -> Result<()> {
        let game_table = &ctx.accounts.game_table;
        let winner_state = &mut ctx.accounts.winner_state;

        require!(
            game_table.stage == GameStage::Showdown,
            ErrorCode::NotShowdown
        );

        // Calculate rake (2.5% capped at 3 BB)
        let rake = calculate_rake(game_table.pot, game_table.big_blind);
        let payout = game_table.pot - rake;

        // Transfer pot to winner
        let seeds = &[
            ESCROW_SEED,
            &game_table.table_id.to_le_bytes(),
            &[ctx.accounts.table_escrow.bump]
        ];

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.table_escrow.to_account_info(),
                    to: ctx.accounts.winner_token_account.to_account_info(),
                    authority: ctx.accounts.table_escrow.to_account_info(),
                },
                &[seeds]
            ),
            payout
        )?;

        winner_state.chips += payout;

        emit!(PotDistributedEvent {
            table_id: game_table.table_id,
            winner: winner_state.player,
            amount: payout,
            rake
        });

        Ok(())
    }
}

// Helper function
fn calculate_rake(pot: u64, big_blind: u64) -> u64 {
    let rake = pot.checked_mul(25).unwrap().checked_div(1000).unwrap();
    std::cmp::min(rake, big_blind.checked_mul(3).unwrap())
}

// ============ ACCOUNTS ============

#[derive(Accounts)]
#[instruction(table_id: u64)]
pub struct CreateTable<'info> {
    #[account(
        init,
        payer = creator,
        space = 8 + GameTable::INIT_SPACE,
        seeds = [TABLE_SEED, &table_id.to_le_bytes()],
        bump
    )]
    pub game_table: Account<'info, GameTable>,

    #[account(mut)]
    pub creator: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct JoinTable<'info> {
    #[account(
        mut,
        seeds = [TABLE_SEED, &game_table.table_id.to_le_bytes()],
        bump = game_table.bump
    )]
    pub game_table: Account<'info, GameTable>,

    #[account(
        init,
        payer = player,
        space = 8 + PlayerState::INIT_SPACE,
        seeds = [PLAYER_SEED, player.key().as_ref(), &game_table.table_id.to_le_bytes()],
        bump
    )]
    pub player_state: Account<'info, PlayerState>,

    #[account(
        mut,
        seeds = [ESCROW_SEED, &game_table.table_id.to_le_bytes()],
        bump
    )]
    pub table_escrow: Account<'info, TokenAccount>,

    #[account(mut)]
    pub player_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub player: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlayerAction<'info> {
    #[account(
        mut,
        seeds = [TABLE_SEED, &game_table.table_id.to_le_bytes()],
        bump = game_table.bump
    )]
    pub game_table: Account<'info, GameTable>,

    #[account(
        mut,
        seeds = [PLAYER_SEED, player.key().as_ref(), &game_table.table_id.to_le_bytes()],
        bump = player_state.bump,
        constraint = player_state.player == player.key()
    )]
    pub player_state: Account<'info, PlayerState>,

    pub player: Signer<'info>,
}

#[derive(Accounts)]
pub struct DistributePot<'info> {
    #[account(
        mut,
        seeds = [TABLE_SEED, &game_table.table_id.to_le_bytes()],
        bump = game_table.bump
    )]
    pub game_table: Account<'info, GameTable>,

    #[account(
        mut,
        seeds = [PLAYER_SEED, winner_state.player.as_ref(), &game_table.table_id.to_le_bytes()],
        bump = winner_state.bump
    )]
    pub winner_state: Account<'info, PlayerState>,

    #[account(
        mut,
        seeds = [ESCROW_SEED, &game_table.table_id.to_le_bytes()],
        bump
    )]
    pub table_escrow: Account<'info, TokenAccount>,

    #[account(mut)]
    pub winner_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

// ============ STATE ============

#[account]
#[derive(InitSpace)]
pub struct GameTable {
    pub table_id: u64,
    pub small_blind: u64,
    pub big_blind: u64,
    pub max_players: u8,
    pub player_count: u8,
    pub current_player_index: u8,
    pub dealer_position: u8,
    pub stage: GameStage,
    pub pot: u64,
    pub current_bet: u64,
    #[max_len(5)]
    pub community_cards: Vec<u8>,
    pub deck_commitment: [u8; 32],
    pub active: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct PlayerState {
    pub player: Pubkey,
    pub table_id: u64,
    pub chips: u64,
    pub current_bet: u64,
    pub hole_card_commitments: [[u8; 32]; 2],
    pub has_folded: bool,
    pub is_all_in: bool,
    pub hand_verified: bool,
    pub hand_rank: u32,
    pub position: u8,
    pub bump: u8,
}

// ============ ENUMS ============

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum GameStage {
    Waiting,
    PreFlop,
    Flop,
    Turn,
    River,
    Showdown,
    Completed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ActionType {
    Fold,
    Check,
    Call,
    Bet,
    Raise,
    AllIn,
}

// ============ EVENTS ============

#[event]
pub struct TableCreatedEvent {
    pub table_id: u64,
    pub small_blind: u64,
    pub big_blind: u64,
    pub creator: Pubkey,
}

#[event]
pub struct PlayerJoinedEvent {
    pub table_id: u64,
    pub player: Pubkey,
    pub buy_in: u64,
    pub position: u8,
}

#[event]
pub struct PlayerActionEvent {
    pub table_id: u64,
    pub player: Pubkey,
    pub action: ActionType,
    pub amount: u64,
}

#[event]
pub struct PotDistributedEvent {
    pub table_id: u64,
    pub winner: Pubkey,
    pub amount: u64,
    pub rake: u64,
}

// ============ ERRORS ============

#[error_code]
pub enum ErrorCode {
    #[msg("Table is full")]
    TableFull,
    #[msg("Game already in progress")]
    GameInProgress,
    #[msg("Not player's turn")]
    NotPlayerTurn,
    #[msg("Player has folded")]
    PlayerFolded,
    #[msg("Cannot check - must call or raise")]
    CannotCheck,
    #[msg("Bet amount too small")]
    BetTooSmall,
    #[msg("Insufficient chips")]
    InsufficientChips,
    #[msg("Not in showdown stage")]
    NotShowdown,
    #[msg("Invalid proof")]
    InvalidProof,
}
```

### Update Cargo.toml

**contracts/programs/contracts/Cargo.toml:**
```toml
[package]
name = "zkpoker"
version = "0.1.0"
description = "ZkPoker - Privacy-Preserving Poker on Solana"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]
name = "zkpoker"

[features]
default = []
cpi = ["no-entrypoint"]
no-entrypoint = []
no-idl = []
no-log-ix-name = []
idl-build = ["anchor-lang/idl-build"]

[dependencies]
anchor-lang = "0.32.1"
anchor-spl = "0.32.1"
```

---

## Game Flow & State Machine

### Game Stages

```rust
pub enum GameStage {
    Waiting,      // Waiting for players
    PreFlop,      // Hole cards dealt, betting round 1
    Flop,         // 3 community cards revealed, betting round 2
    Turn,         // 4th community card revealed, betting round 3
    River,        // 5th community card revealed, betting round 4
    Showdown,     // Players reveal, winner determined
    Completed     // Hand finished, pot distributed
}
```

### Hand Flow Sequence

```
1. WAITING STAGE
   ├─ Players join table (2-9 players)
   ├─ Players post buy-ins (tokens escrowed)
   └─ Dealer button positioned

2. START HAND (Transition to PreFlop)
   ├─ Small blind posts (auto-deducted)
   ├─ Big blind posts (auto-deducted)
   ├─ Server shuffles deck, generates commitment
   ├─ Hole cards dealt (encrypted, committed)
   └─ Players verify commitments on-chain

3. PRE-FLOP BETTING
   ├─ First player after BB acts
   ├─ Actions: Fold / Call / Raise
   └─ If only 1 player remaining → SHOWDOWN

4. FLOP → TURN → RIVER
   ├─ Community cards revealed with proofs
   └─ Betting rounds continue

5. SHOWDOWN
   ├─ Active players submit reveal proofs
   ├─ Active players submit hand evaluation proofs
   ├─ Smart contract compares hand ranks
   └─ Pot distributed to winner(s)
```

---

## Days 1-3 Implementation Roadmap

### Day 1: Foundation & Circuits

**Tasks:**
```
[ ] Set up circuit structure in circuits/src/
[ ] Implement commit.nr circuit
[ ] Implement reveal.nr circuit
[ ] Implement evaluate_hand.nr circuit
[ ] Test circuits with `nargo test`
[ ] Generate verification keys with `nargo compile`
[ ] Benchmark proof generation times
```

**Commands:**
```bash
cd circuits
nargo check    # Verify syntax
nargo test     # Run tests
nargo compile  # Generate artifacts
```

### Day 2: Solana Program Core

**Tasks:**
```
[ ] Update lib.rs with full implementation
[ ] Add anchor-spl dependency for token support
[ ] Implement CreateTable instruction
[ ] Implement JoinTable instruction
[ ] Implement PlayerAction instruction
[ ] Write integration tests
[ ] Deploy to localnet
```

**Commands:**
```bash
cd contracts
anchor build
anchor test
anchor deploy --provider.cluster localnet
```

### Day 3: Token & Escrow System

**Tasks:**
```
[ ] Create TableEscrow PDA account
[ ] Implement buy_in token transfer
[ ] Implement distribute_pot instruction
[ ] Add rake calculation
[ ] Test escrow invariants
[ ] Test full game flow on localnet
```

---

## Testing Strategy

### Circuit Tests (Nargo)

```bash
cd circuits
nargo test --show-output
```

### Solana Program Tests

**contracts/tests/contracts.ts:**
```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Zkpoker } from "../target/types/zkpoker";
import { PublicKey } from "@solana/web3.js";

describe("zkpoker", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Zkpoker as Program<Zkpoker>;

  it("Creates a game table", async () => {
    const tableId = new anchor.BN(Date.now());

    const [tablePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("table"), tableId.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    await program.methods
      .createTable(tableId, new anchor.BN(5), new anchor.BN(10), 9)
      .accounts({
        gameTable: tablePda,
        creator: provider.wallet.publicKey,
      })
      .rpc();

    const table = await program.account.gameTable.fetch(tablePda);
    console.log("Table created:", table.tableId.toString());
  });
});
```

---

## Deployment

### Prerequisites

```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Install Anchor (already configured in contracts/)
# Rust 1.89.0 specified in rust-toolchain.toml

# Install Noir
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
noirup
```

### Deploy to Devnet

```bash
cd contracts
solana config set --url devnet
solana airdrop 2
anchor build
anchor deploy --provider.cluster devnet
```

---

## Performance Targets

### Proof Generation (Client-side)
- Commitment: <500ms
- Reveal: <2s
- Hand evaluation: <5s

### Blockchain (Solana)
- Transaction confirmation: <2s
- Proof verification: <1s
- Concurrent tables: 100+

---

> **Next:** See `CLAUDE-PART2.md` for Client implementation (Backend API + Frontend) - Days 4-7.
