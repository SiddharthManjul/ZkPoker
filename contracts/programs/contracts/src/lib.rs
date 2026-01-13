use anchor_lang::prelude::*;

declare_id!("GnDHa3pfhiqEG5xVTjtnTYue33ceX6disU8F2YJymqYr");

#[program]
pub mod contracts {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
