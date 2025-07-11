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

    // Deposit instruction
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        
        ctx.accounts.deposit(amount)
    }

    // Withdraw instruction
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        
        ctx.accounts.withdraw(amount)
    }

    // Close instruction
    pub fn close(ctx: Context<Close>) -> Result<()> {
        ctx.accounts.close()
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

        self.vault_state.state_bump = bumps.vault_state;
        self.vault_state.vault_bump = bumps.vault;

        Ok(())
    }
}

#[derive(Accounts)]
// Deposit context
pub struct Deposit<'info> {
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump, // already calculated at init
    )]
    pub vault_state: Account<'info, VaultState>,
    
    #[account(
        mut, // 'cuz adding lamports (SOL)
        seeds = [b"vault", vault_state.key().as_ref()], 
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,
    
    // Allows the lamport transfer
    pub system_program: Program<'info, System>,
}

// Deposit context functions
impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {

        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.user.to_account_info(),
            to: self.vault.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer(cpi_ctx, amount)
    }
}

#[derive(Accounts)]
// Withdraw context
pub struct Withdraw<'info> {
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump, // already calculated at init
    )]
    pub vault_state: Account<'info, VaultState>,
    
    #[account(
        mut, // 'cuz deducting lamports (SOL)
        seeds = [b"vault", vault_state.key().as_ref()], 
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,
    
    // Allows the lamport transfer
    pub system_program: Program<'info, System>,
}

// Withdraw context functions
impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {

        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        // Create seeds for the context to know the signer
        // Use the vault (from) seeds for derivation (in same order) and bump
        let seeds = &[
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ];

        // Needed to sign the ix
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        transfer(cpi_ctx, amount)
    }
}

#[derive(Accounts)]
// Close context
pub struct Close<'info> {

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump,
        close = user, // Empties data from the acc
    )]
    pub vault_state: Account<'info, VaultState>,
    
    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

// Close context functions
impl<'info> Close<'info> {
    pub fn close(&mut self) -> Result<()> {

        let cpi_program = self.system_program.to_account_info();

        let cpi_accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.user.to_account_info(),
        };

        let seeds = &[
            b"vault",
            self.vault_state.to_account_info().key.as_ref(),
            &[self.vault_state.vault_bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

        // Empty vault to close it
        transfer(cpi_ctx, self.vault.lamports())
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
