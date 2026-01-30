use anchor_lang::prelude::*;
use crate::state::{ProofBuffer, ProofType};
use crate::errors::ZkPokerError;

/// Initialize a proof buffer for uploading a ZK proof
#[derive(Accounts)]
#[instruction(proof_type: u8, proof_size: u16)]
pub struct InitProofBuffer<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    /// The hand this proof is for
    /// CHECK: Validated by seeds constraint
    pub hand: AccountInfo<'info>,

    /// Proof buffer PDA
    #[account(
        init,
        payer = player,
        space = ProofBuffer::space(proof_size),
        seeds = [
            b"proof_buffer",
            hand.key().as_ref(),
            player.key().as_ref(),
            &[proof_type]
        ],
        bump
    )]
    pub proof_buffer: Account<'info, ProofBuffer>,

    pub system_program: Program<'info, System>,
}

/// Upload a chunk of proof data to the buffer
#[derive(Accounts)]
pub struct UploadProofChunk<'info> {
    pub player: Signer<'info>,

    #[account(
        mut,
        has_one = player @ ZkPokerError::Unauthorized,
        constraint = !proof_buffer.complete @ ZkPokerError::BufferAlreadyComplete
    )]
    pub proof_buffer: Account<'info, ProofBuffer>,
}

/// Close a proof buffer and reclaim rent (after verification)
#[derive(Accounts)]
pub struct CloseProofBuffer<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    #[account(
        mut,
        close = player,
        has_one = player @ ZkPokerError::Unauthorized
    )]
    pub proof_buffer: Account<'info, ProofBuffer>,
}

/// Initialize a proof buffer
pub fn handle_init_proof_buffer(
    ctx: Context<InitProofBuffer>,
    proof_type: u8,
    proof_size: u16,
) -> Result<()> {
    let buffer = &mut ctx.accounts.proof_buffer;
    
    let pt = match proof_type {
        0 => ProofType::Deck,
        1 => ProofType::Reveal,
        2 => ProofType::Showdown,
        _ => return Err(ZkPokerError::InvalidProofType.into()),
    };

    buffer.init(
        ctx.accounts.hand.key(),
        ctx.accounts.player.key(),
        pt,
        proof_size,
        ctx.bumps.proof_buffer,
    );

    msg!("Proof buffer initialized: {} bytes for {:?}", proof_size, pt);
    Ok(())
}

/// Upload a chunk of proof data
pub fn handle_upload_proof_chunk(
    ctx: Context<UploadProofChunk>,
    offset: u16,
    data: Vec<u8>,
) -> Result<()> {
    let buffer = &mut ctx.accounts.proof_buffer;
    
    buffer.upload_chunk(offset, &data)?;
    
    msg!(
        "Uploaded {} bytes at offset {}, total: {}/{}",
        data.len(),
        offset,
        buffer.uploaded,
        buffer.size
    );
    
    Ok(())
}

/// Close a proof buffer and reclaim rent
pub fn handle_close_proof_buffer(_ctx: Context<CloseProofBuffer>) -> Result<()> {
    msg!("Proof buffer closed, rent reclaimed");
    Ok(())
}
