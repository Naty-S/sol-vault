#![allow(unexpected_cfgs)]
#![allow(deprecated)]
use anchor_lang::{
      prelude::*
    , system_program::{Transfer, transfer}
};

declare_id!("8gqiE9DoKXxFSzM7uVvd81C6jMUFiACN6UpGM34eogtd");

#[program]
pub mod sol_vault {
    use super::*;

    // Initialize instruction
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        
        ctx.accounts.initialize(&ctx.bumps)
    }
}

#[derive(Accounts)]
// Initialization context
pub struct Initialize<'info> {
    
    #[account(mut)] // mutable 'cuz pays for the state acc creation
    pub user: Signer<'info>,

    #[account(
        init, // 'cuz storing data (VaultState), also says is mutable
        payer = user, // pays the rent exception, needed if init
        seeds = [b"state", user.key().as_ref()], // ensures the user is the only that can sign ixs
        bump, // Anchor calculated canonical bump
        space = 8 + VaultState::INIT_SPACE // 8 for anchor discriminator
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut, // 'cuz the state changes
        seeds = [b"vault", vault_state.key().as_ref()],
        bump
    )]
    // Allows account initialization
    // Auto inited when it has enough lamports to be rent-excempt, at transfer
    pub vault: SystemAccount<'info>,

    // Allows account creation and initialization
    pub system_program: Program<'info, System>
}

// Initialize context functions
impl<'info> Initialize<'info> {
    pub fn initialize(
        &mut self, // To access the context
        bumps: &InitializeBumps
    ) -> Result<()> {

        // Get the amount of lamports needed to make the vault rent exempt
        let rent_exempt = (Rent::get()?)
            .minimum_balance(self.vault.to_account_info().data_len());

        // Transfer the rent-exempt amount from the user to the vault
        let cpi_program = self.system_program.to_account_info();
        let cpi_accounts = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, rent_exempt)?;

        // save the bumps for 
        self.vault_state.state_bump = bumps.vault_state;
        self.vault_state.vault_bump = bumps.vault;

        Ok(())
    }
}

#[account]
#[derive(InitSpace)]
// Account for holding the vault state
// Vault Authority
// Store the bumps for CU optimization. No recalculate for acc validation.
pub struct VaultState {
    pub vault_bump: u8, // bump seed for the vault acc that will store the user's funds
    pub state_bump: u8  // bump seed for the vault state acc itself
}

// Space trait
// impl Space for VaultState {
//     const INIT_SPACE: usize = 8 + 1 + 1;
// }
