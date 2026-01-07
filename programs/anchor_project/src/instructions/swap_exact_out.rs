use crate::*;
use anchor_lang::solana_program::clock::Clock;

pub fn swap_exact_out(
    ctx: Context<SwapExactOut>,
    output_amount: u64,
    output_is_x: bool,
    max_in_amount: u64,
) -> Result<()> {
    // Start timer
    let start_time = Clock::get()?.unix_timestamp;

    if output_is_x {
        // Output from vault is X
        // User puts in Y

        // 1. Calculate base input (Y) derived from output (X) * Price (Y/X)
        let taxed_input = (output_amount as u128)
            .checked_mul(ctx.accounts.state_account.x_to_y_scaled_price as u128)
            .unwrap()
            .checked_div(StateAccount::PRICE_SCALE_FACTOR)
            .unwrap() as u64;

        // 2. Add spread
        let required_input = (taxed_input as u128)
            .checked_mul(StateAccount::SPREAD_SCALE_FACTOR)
            .unwrap()
            .checked_div(
                StateAccount::SPREAD_SCALE_FACTOR
                    .checked_sub(ctx.accounts.state_account.spread_bps as u128)
                    .unwrap(),
            )
            .unwrap() as u64;

        // Confirm that this does not exceed user's max input
        if required_input > max_in_amount {
            return err!(WasabiError::InputTooLarge);
        }

        // Confirm that user has enough balance; if no, fail
        if required_input > ctx.accounts.user_token_y_acct.amount {
            return err!(WasabiError::UserInsufficientBalance);
        }

        // transfer <required_input> of y from user to vault
        transfer_user_to_vault(
            ctx.accounts.user_token_y_acct.to_account_info(),
            ctx.accounts.vault_token_y_acct.to_account_info(),
            ctx.accounts.user_wallet.to_account_info(),
            required_input,
            ctx.accounts.token_program.to_account_info(),
        )?;

        // transfer <output_amount> of x from vault to user
        let transfer_accounts = anchor_spl::token::Transfer {
            from: ctx.accounts.vault_token_x_acct.to_account_info(),
            to: ctx.accounts.user_token_x_acct.to_account_info(),
            authority: ctx.accounts.state_account.to_account_info(),
        };
        let bump = [ctx.bumps.state_account];
        let seeds = &[
            "state".as_bytes(),
            ctx.accounts.token_x_mint.to_account_info().key.as_ref(),
            ctx.accounts.token_y_mint.to_account_info().key.as_ref(),
            &bump,
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_accounts,
            signer_seeds,
        );
        anchor_spl::token::transfer(cpi_ctx, output_amount)?;
    } else {
        // Output from vault is Y
        // User puts in X

        // 1. Calculate base input (X) derived from output (Y) / Price (Y/X)
        let taxed_input = (output_amount as u128)
            .checked_mul(StateAccount::PRICE_SCALE_FACTOR)
            .unwrap()
            .checked_div(ctx.accounts.state_account.x_to_y_scaled_price as u128)
            .unwrap() as u64;

        // 2. Add spread
        let required_input = (taxed_input as u128)
            .checked_mul(StateAccount::SPREAD_SCALE_FACTOR)
            .unwrap()
            .checked_div(
                StateAccount::SPREAD_SCALE_FACTOR
                    .checked_sub(ctx.accounts.state_account.spread_bps as u128)
                    .unwrap(),
            )
            .unwrap() as u64;

        // Confirm that this does not exceed user's max input
        if required_input > max_in_amount {
            return err!(WasabiError::InputTooLarge);
        }

        // Confirm that user has enough balance; if no, fail
        if required_input > ctx.accounts.user_token_x_acct.amount {
            return err!(WasabiError::UserInsufficientBalance);
        }

        // transfer <required_input> of x from user to vault
        transfer_user_to_vault(
            ctx.accounts.user_token_x_acct.to_account_info(),
            ctx.accounts.vault_token_x_acct.to_account_info(),
            ctx.accounts.user_wallet.to_account_info(),
            required_input,
            ctx.accounts.token_program.to_account_info(),
        )?;

        // transfer <output_amount> of y from vault to user
        let transfer_accounts = anchor_spl::token::Transfer {
            from: ctx.accounts.vault_token_y_acct.to_account_info(),
            to: ctx.accounts.user_token_y_acct.to_account_info(),
            authority: ctx.accounts.state_account.to_account_info(),
        };
        let bump = [ctx.bumps.state_account];
        let seeds = &[
            "state".as_bytes(),
            ctx.accounts.token_x_mint.to_account_info().key.as_ref(),
            ctx.accounts.token_y_mint.to_account_info().key.as_ref(),
            &bump,
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_accounts,
            signer_seeds,
        );
        anchor_spl::token::transfer(cpi_ctx, output_amount)?;
    }

    // Check expiration
    let end_time = Clock::get()?.unix_timestamp;
    if end_time - start_time > ctx.accounts.state_account.tx_exp_duration {
        return err!(WasabiError::ExpirationError);
    }

    // return
    Ok(())
}

#[derive(Accounts)]
pub struct SwapExactOut<'a> {
    user_wallet: Signer<'a>,
    #[account(
        mut,
        token::mint = token_x_mint,
        token::authority = user_wallet,
    )]
    user_token_x_acct: Account<'a, TokenAccount>,
    #[account(
        mut,
        token::mint = token_y_mint,
        token::authority = user_wallet,
    )]
    user_token_y_acct: Account<'a, TokenAccount>,

    // Used for signing token xfers and holding price/fee data
    #[account(
        mut,
        seeds = [b"state", token_x_mint.key().as_ref(), token_y_mint.key().as_ref()],
        bump,
    )]
    state_account: Account<'a, StateAccount>,
    #[account(
        mut,
        token::mint = token_x_mint,
        token::authority = state_account,
    )]
    vault_token_x_acct: Account<'a, TokenAccount>,
    #[account(
        mut,
        token::mint = token_y_mint,
        token::authority = state_account,
    )]
    vault_token_y_acct: Account<'a, TokenAccount>,

    token_x_mint: Account<'a, Mint>,
    token_y_mint: Account<'a, Mint>,
    system_program: Program<'a, System>,
    token_program: Program<'a, Token>,
}
