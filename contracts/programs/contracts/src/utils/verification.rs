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
/// * `proof_and_witness` - The proof + public witness bytes (proof is 388 bytes + variable witness size)
///
/// # Returns
/// * `Ok(())` if proof is valid
/// * `Err(ZkPokerError::ProofVerificationFailed)` if proof is invalid
pub fn verify_groth16_proof(
    verifier_program: &AccountInfo,
    expected_verifier_id: &Pubkey,
    proof_and_witness: &[u8],
) -> Result<()> {
    // Verify verifier program ID matches expected circuit verifier
    require!(
        *verifier_program.key == *expected_verifier_id,
        ZkPokerError::ProofVerificationFailed
    );

    // Verify minimum size (proof must be at least PROOF_SIZE bytes)
    require!(
        proof_and_witness.len() >= PROOF_SIZE,
        ZkPokerError::InvalidProofFormat
    );

    msg!("Verifying ZK proof via CPI");
    msg!("Verifier program: {}", verifier_program.key());
    msg!("Proof + witness size: {} bytes", proof_and_witness.len());

    // The instruction data is already in the correct format: proof || public_witness
    // Sunspot generates this format automatically
    let verify_ix = Instruction {
        program_id: *expected_verifier_id,
        accounts: vec![], // Verifier programs are stateless
        data: proof_and_witness.to_vec(),
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
/// * `proof_and_witness` - The proof + public witness from Sunspot
pub fn verify_hole_card_commitments(
    verifier_program: &AccountInfo,
    proof_and_witness: &[u8],
) -> Result<()> {
    verify_groth16_proof(verifier_program, &DECK_VERIFIER_PROGRAM_ID, proof_and_witness)
}

/// Verify community card reveal (uses REVEAL circuit)
///
/// Verifies that the revealed cards are at the correct positions
/// in the shuffled deck derived from deck_seed.
///
/// # Arguments
/// * `verifier_program` - The verifier program account (must be REVEAL verifier)
/// * `proof_and_witness` - The proof + public witness from Sunspot
pub fn verify_community_cards(
    verifier_program: &AccountInfo,
    proof_and_witness: &[u8],
) -> Result<()> {
    verify_groth16_proof(verifier_program, &REVEAL_VERIFIER_PROGRAM_ID, proof_and_witness)
}

/// Verify hand reveal at showdown (uses SHOWDOWN circuit)
///
/// Verifies that:
/// 1. The revealed cards match the commitments
/// 2. The hand rank is correctly computed
///
/// # Arguments
/// * `verifier_program` - The verifier program account (must be SHOWDOWN verifier)
/// * `proof_and_witness` - The proof + public witness from Sunspot
pub fn verify_hand_reveal(
    verifier_program: &AccountInfo,
    proof_and_witness: &[u8],
) -> Result<()> {
    verify_groth16_proof(verifier_program, &SHOWDOWN_VERIFIER_PROGRAM_ID, proof_and_witness)
}
