use anchor_lang::prelude::*;

declare_id!("8gqiE9DoKXxFSzM7uVvd81C6jMUFiACN6UpGM34eogtd");

#[program]
pub mod sol_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
