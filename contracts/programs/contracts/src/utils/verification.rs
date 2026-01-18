use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke;
use crate::constants::{
    DECK_VERIFIER_PROGRAM_ID,
    REVEAL_VERIFIER_PROGRAM_ID,
    SHOWDOWN_VERIFIER_PROGRAM_ID,
    PROOF_SIZE
};
use crate::errors::ZkPokerError;

/// Verify a Groth16 proof using the deployed verifier program
///
/// # Arguments
/// * `verifier_program` - The verifier program account
/// * `expected_verifier_id` - The expected verifier program ID for this circuit
/// * `proof` - The Groth16 proof bytes (388 bytes)
/// * `public_inputs` - The public inputs to the circuit
///
/// # Returns
/// * `Ok(())` if proof is valid
/// * `Err(ZkPokerError::ProofVerificationFailed)` if proof is invalid
pub fn verify_groth16_proof(
    verifier_program: &AccountInfo,
    expected_verifier_id: &Pubkey,
    proof: &[u8],
    public_inputs: &[u8],
) -> Result<()> {
    // Verify verifier program ID matches expected circuit verifier
    require!(
        *verifier_program.key == *expected_verifier_id,
        ZkPokerError::ProofVerificationFailed
    );

    // Verify proof size (Groth16 proofs are 388 bytes)
    require!(
        proof.len() == PROOF_SIZE,
        ZkPokerError::InvalidProofFormat
    );

    msg!("Verifying ZK proof via CPI");
    msg!("Verifier program: {}", verifier_program.key());
    msg!("Proof size: {} bytes", proof.len());
    msg!("Public inputs size: {} bytes", public_inputs.len());

    // Build instruction data: proof || public_inputs
    // The Sunspot/gnark-solana verifier expects the proof concatenated with public inputs
    let mut instruction_data = Vec::with_capacity(proof.len() + public_inputs.len());
    instruction_data.extend_from_slice(proof);
    instruction_data.extend_from_slice(public_inputs);

    // Create instruction for the verifier program
    let verify_ix = Instruction {
        program_id: *expected_verifier_id,
        accounts: vec![], // Verifier programs are stateless
        data: instruction_data,
    };

    // Execute CPI call to verifier program
    invoke(&verify_ix, &[verifier_program.clone()])
        .map_err(|_| ZkPokerError::ProofVerificationFailed)?;

    msg!("✓ ZK Proof verified successfully");

    Ok(())
}

/// Verify hole card commitments (uses DECK circuit)
///
/// Verifies that the commitments are valid for cards at the specified positions
/// in the shuffled deck derived from deck_seed.
///
/// # Arguments
/// * `verifier_program` - The verifier program account (must be DECK verifier)
/// * `deck_seed` - The combined deck seed
/// * `player_seat` - The player's seat (0 or 1)
/// * `commitments` - The two hole card commitments
/// * `proof` - The ZK proof
pub fn verify_hole_card_commitments(
    verifier_program: &AccountInfo,
    deck_seed: &[u8; 32],
    player_seat: u8,
    commitments: &[[u8; 32]; 2],
    proof: &[u8],
) -> Result<()> {
    // Build public inputs
    // Format: deck_seed || player_seat || commitment_1 || commitment_2
    let mut public_inputs = Vec::with_capacity(32 + 1 + 32 + 32);
    public_inputs.extend_from_slice(deck_seed);
    public_inputs.push(player_seat);
    public_inputs.extend_from_slice(&commitments[0]);
    public_inputs.extend_from_slice(&commitments[1]);

    verify_groth16_proof(verifier_program, &DECK_VERIFIER_PROGRAM_ID, proof, &public_inputs)
}

/// Verify community card reveal (uses REVEAL circuit)
///
/// Verifies that the revealed cards are at the correct positions
/// in the shuffled deck derived from deck_seed.
///
/// # Arguments
/// * `verifier_program` - The verifier program account (must be REVEAL verifier)
/// * `deck_seed` - The combined deck seed
/// * `cards` - The revealed card indices
/// * `positions` - The expected positions in the deck
/// * `proof` - The ZK proof
pub fn verify_community_cards(
    verifier_program: &AccountInfo,
    deck_seed: &[u8; 32],
    cards: &[u8],
    positions: &[u8],
    proof: &[u8],
) -> Result<()> {
    // Validate card indices
    for card in cards {
        require!(*card < 52, ZkPokerError::InvalidCardIndex);
    }

    // Build public inputs
    // Format: deck_seed || cards || positions
    let mut public_inputs = Vec::with_capacity(32 + cards.len() + positions.len());
    public_inputs.extend_from_slice(deck_seed);
    public_inputs.extend_from_slice(cards);
    public_inputs.extend_from_slice(positions);

    verify_groth16_proof(verifier_program, &REVEAL_VERIFIER_PROGRAM_ID, proof, &public_inputs)
}

/// Verify hand reveal at showdown (uses SHOWDOWN circuit)
///
/// Verifies that:
/// 1. The revealed cards match the commitments
/// 2. The hand rank is correctly computed
///
/// # Arguments
/// * `verifier_program` - The verifier program account (must be SHOWDOWN verifier)
/// * `commitments` - The stored hole card commitments
/// * `community_cards` - The revealed community cards
/// * `hand_rank` - The claimed hand rank
/// * `proof` - The ZK proof
pub fn verify_hand_reveal(
    verifier_program: &AccountInfo,
    commitments: &[[u8; 32]; 2],
    community_cards: &[u8; 5],
    hand_rank: u64,
    proof: &[u8],
) -> Result<()> {
    // Build public inputs
    // Format: commitment_1 || commitment_2 || community_cards || hand_rank
    let mut public_inputs = Vec::with_capacity(32 + 32 + 5 + 8);
    public_inputs.extend_from_slice(&commitments[0]);
    public_inputs.extend_from_slice(&commitments[1]);
    public_inputs.extend_from_slice(community_cards);
    public_inputs.extend_from_slice(&hand_rank.to_le_bytes());

    verify_groth16_proof(verifier_program, &SHOWDOWN_VERIFIER_PROGRAM_ID, proof, &public_inputs)
}
