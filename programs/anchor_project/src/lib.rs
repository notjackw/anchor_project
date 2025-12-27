use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, CloseAccount, Mint, Token, TokenAccount, Transfer},
};

/** Accounts:
 * this program
 *
 * Swap State PDA
 *  - owned by: this program
 *
 * Token vault account x_token
 *  - owned by: State PDA
 *
 * Token vault account y_token
 *  - owned by: State PDA
 *
 *
 * ** Instructions:
 *
swap_exact_in (I want to buy 100 Solana of USD)
Parameters:
token_in		USDC
token_out		SOL
amount_in		100
min_out_amount	0.95
expiration


swap_exact_out (I want to buy 1 Solana)
Parameters:
token_in		USDC
token_out		SOL
amount_out		1
max_in_amount	105
expiration

 *
*/
pub mod contract {
    use super::*;
    use anchor_lang::prelude::Context;

    use crate::UpdatePrice;

    pub fn init(ctx: Context<Init>) -> Result<()> {
        // return
        Ok(())
    }
    pub fn update_price(ctx: Context<UpdatePrice>, x_to_y_price: u64) -> Result<()> {
        ctx.accounts.state_account.x_to_y_price = x_to_y_price;
        // return
        Ok(())
    }
    pub fn swap_exact_in_x_to_y(ctx: Context<SwapExactIn>, in_amount: u64) -> Result<()> {
        // transfer <provided_amount> of IN from user to vault
        let transfer_accounts = anchor_spl::token::Transfer {
            from: ctx.accounts.user_token_x_acct.to_account_info(),
            to: ctx.accounts.token_x_account.to_account_info(),
            authority: ctx.accounts.user_wallet.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_accounts,
        );
        anchor_spl::token::transfer(cpi_ctx, x_amount);

        // transfer <calculated_amount> of y from vault to user
        let seeds = &[];
        let calculated_y_amount = in_amount / ctx.accounts.state_account.x_to_y_price;
        let transfer_accounts = anchor_spl::token::Transfer {
            from: ctx.accounts.token_y_account.to_account_info(),
            to: ctx.accounts.user_token_y_acct.to_account_info(),
            authority: ctx.accounts.state_account.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_accounts,
            seeds,
        );
        anchor_spl::token::transfer(cpi_ctx, calculated_y_amount);

        // return
        Ok(())
    }
    pub fn swap_exact_out(ctx: Context<SwapExactOut>) -> Result<()> {
        // return
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Init<'a> {
    #[account(mut)]
    user: Signer<'a>,

    token_x_mint: Account<'a, anchor_spl::token_interface::Mint>,
    token_y_mint: Account<'a, anchor_spl::token_interface::Mint>,

    #[account(
        init,
        payer = user,
        token::mint = token_x_mint,
        token::authority = state_account,
    )]
    token_x_account: Account<'a, TokenAccount>,

    #[account(
        init,
        payer = user,
        token::mint = token_x_mint,
        token::authority = state_account,
    )]
    token_y_account: Account<'a, TokenAccount>,

    #[account(
        init,
        payer = user,
        space = StateAccount::INIT_SPACE + 8,
        seeds = [b"state", user.key().as_ref()],
        bump
    )]
    state_account: Account<'a, StateAccount>,

    // need this for creating accounts
    system_program: Program<'a, System>,
    token_program: Program<'a, TokenInterface>,
}

#[derive(Accounts)]
pub struct UpdatePrice<'a> {
    user: Signer<'a>,

    #[account(
        mut,
        seeds = [b"state", user.key().as_ref()],
        bump,
    )]
    state_account: Account<'a, StateAccount>,
}

#[derive(Accounts)]
pub struct SwapExactIn<'a> {
    user_wallet: Signer<'a>,
    user_token_x_acct: Account<'a, TokenAccount>,
    user_token_y_acct: Account<'a, TokenAccount>,

    // need for pricing data
    state_account: Account<'a, StateAccount>,

    // for sending x to user
    token_x_mint: Account<'a, anchor_spl::token_interface::Mint>,
    token_y_mint: Account<'a, anchor_spl::token_interface::Mint>,
    #[account(
        token::mint = token_x_mint,
        token::authority = state_account,
    )]
    token_x_account: Account<'a, TokenAccount>,
    #[account(
        token::mint = token_y_mint,
        token::authority = state_account,
    )]
    token_y_account: Account<'a, TokenAccount>,

    // need this for creating accounts
    system_program: Program<'a, System>,
    token_program: Program<'a, TokenInterface>,
}

#[derive(Accounts)]
pub struct SwapExactOut {}

#[account]
#[derive(InitSpace)]
pub struct StateAccount {
    x_to_y_price: u64,
    authority: Pubkey,
}
