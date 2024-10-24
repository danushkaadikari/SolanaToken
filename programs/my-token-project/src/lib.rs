use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, Burn, Transfer, FreezeAccount, ThawAccount};

declare_id!("FJEYa3zCE6uQ6EpFdK1ad8513WGMRFPm6ikcGE3bZSex");

#[program]
pub mod rwap_meme_token {
    use super::*;

    // Initialize the token with the name, symbol, and initial supply, and set the pause state to false.
    pub fn initialize_token(ctx: Context<InitializeToken>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.is_paused = false;  // Start unpaused
        state.admin = *ctx.accounts.admin.key;  // Set the admin who can pause/unpause the contract

        // Mint the initial supply of 100,000,000 tokens (9 decimal places)
        let total_supply: u64 = 100_000_000 * 10_u64.pow(9);
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_ctx, total_supply)?;

        Ok(())
    }

    // Mint additional tokens (only callable by mint authority and if not paused)
    pub fn mint_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
        let state = &ctx.accounts.state;
        require!(!state.is_paused, ErrorCode::ContractPaused);  // Check if the contract is paused

        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_ctx, amount)?;

        Ok(())
    }

    // Burn tokens from the sender's account (only if not paused)
    pub fn burn_tokens(ctx: Context<BurnTokens>, amount: u64) -> Result<()> {
        let state = &ctx.accounts.state;
        require!(!state.is_paused, ErrorCode::ContractPaused);  // Check if the contract is paused

        let cpi_accounts = Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.from.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::burn(cpi_ctx, amount)?;

        Ok(())
    }

    // Transfer tokens from one account to another (only if not paused)
    pub fn transfer_tokens(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
        let state = &ctx.accounts.state;
        require!(!state.is_paused, ErrorCode::ContractPaused);  // Check if the contract is paused

        let cpi_accounts = Transfer {
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    // Freeze an account (only callable by freeze authority and if not paused)
    pub fn freeze_account(ctx: Context<FreezeAccountContext>) -> Result<()> {
        let state = &ctx.accounts.state;
        require!(!state.is_paused, ErrorCode::ContractPaused);  // Check if the contract is paused

        let cpi_accounts = FreezeAccount {
            account: ctx.accounts.account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            authority: ctx.accounts.freeze_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::freeze_account(cpi_ctx)?;

        Ok(())
    }

    // Thaw a frozen account (only callable by freeze authority and if not paused)
    pub fn thaw_account(ctx: Context<ThawAccountContext>) -> Result<()> {
        let state = &ctx.accounts.state;
        require!(!state.is_paused, ErrorCode::ContractPaused);  // Check if the contract is paused

        let cpi_accounts = ThawAccount {
            account: ctx.accounts.account.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
            authority: ctx.accounts.freeze_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::thaw_account(cpi_ctx)?;

        Ok(())
    }

    // Pause the entire contract (only callable by admin)
    pub fn pause_contract(ctx: Context<PauseContract>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        require!(ctx.accounts.admin.key() == state.admin, ErrorCode::Unauthorized);  // Check if the caller is the admin
        state.is_paused = true;  // Set paused to true
        Ok(())
    }

    // Unpause the contract (only callable by admin)
    pub fn unpause_contract(ctx: Context<PauseContract>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        require!(ctx.accounts.admin.key() == state.admin, ErrorCode::Unauthorized);  // Check if the caller is the admin
        state.is_paused = false;  // Set paused to false
        Ok(())
    }
}

// Contexts for each instruction
#[derive(Accounts)]
pub struct InitializeToken<'info> {
    #[account(
        init,
        payer = mint_authority,
        mint::decimals = 9,
        mint::authority = mint_authority,
        mint::freeze_authority = freeze_authority
    )]
    pub mint: Account<'info, Mint>,

    #[account(mut)]  // Mark the payer as mutable
    pub mint_authority: Signer<'info>,

    #[account(signer)]
    pub freeze_authority: Signer<'info>,

    #[account(mut, signer)]
    pub admin: Signer<'info>,

    #[account(init, payer = admin, space = 8 + State::SIZE)]  // Add space for the state account
    pub state: Account<'info, State>,

    #[account(mut)]  // The token account to receive the initial supply of tokens
    pub to: Account<'info, TokenAccount>,  // ADD this to represent the recipient token account

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    #[account(signer)]
    pub mint_authority: Signer<'info>,
    pub state: Account<'info, State>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BurnTokens<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(signer)]
    pub authority: Signer<'info>,
    pub state: Account<'info, State>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct TransferTokens<'info> {
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    #[account(signer)]
    pub authority: Signer<'info>,
    pub state: Account<'info, State>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct FreezeAccountContext<'info> {
    #[account(mut)]
    pub account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(signer)]
    pub freeze_authority: Signer<'info>,
    pub state: Account<'info, State>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ThawAccountContext<'info> {
    #[account(mut)]
    pub account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(signer)]
    pub freeze_authority: Signer<'info>,
    pub state: Account<'info, State>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct PauseContract<'info> {
    #[account(signer)]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub state: Account<'info, State>,
}

// State struct for managing paused state and admin
#[account]
pub struct State {
    pub is_paused: bool, // Contract pause status
    pub admin: Pubkey,   // Address of the admin (who can pause/unpause)
}

impl State {
    pub const SIZE: usize = 1 + 32; // Size for paused status (1 byte) and admin pubkey (32 bytes)
}

// Custom error codes
#[error_code]
pub enum ErrorCode {
    #[msg("The contract is currently paused.")]
    ContractPaused,
    #[msg("You are not authorized to perform this action.")]
    Unauthorized,
}
