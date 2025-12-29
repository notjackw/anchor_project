use crate::*;
/**
 * swap_exact_in (I want to buy 100 Solana of USD)
 * Parameters:
 * token_in		    USDC
 * token_out		SOL
 * amount_in		100
 * min_out_amount	0.95
 * expiration
 */
pub fn swap_exact_in(
    ctx: Context<SwapExactIn>,
    input_amount: u64,
    input_is_x: bool,
    min_out_amount: u64,
) -> Result<()> {
    // This was a common check across codebases that I saw:
    // If user tries to input more than it has, transfer entire user balance (instead of failing)
    let input_amount = if input_is_x && input_amount > ctx.accounts.user_token_x_acct.amount {
        ctx.accounts.user_token_x_acct.amount
    } else if !input_is_x && input_amount > ctx.accounts.user_token_y_acct.amount {
        ctx.accounts.user_token_y_acct.amount
    } else {
        input_amount
    };

    /********************************************************************************************
    Normally, this is where you would use the Constant Product Formula to calculate output_amount
    -- by solving for New_A in this invariant:     Old_A * Old_B = New_A * New_B
    The fee/spread set by the admin would be applied to the input_amount instead of the
    output_amount, so that fees are not lessened by slippage.

    In this demo, we calculate output using a fixed price set by the admin. The fee will be
    applied to the output because there is no slippage.
    ********************************************************************************************/

    if input_is_x {
        // transfer <provided_amount> of x from user to vault
        let transfer_accounts = anchor_spl::token::Transfer {
            from: ctx.accounts.user_token_x_acct.to_account_info(),
            to: ctx.accounts.vault_token_x_acct.to_account_info(),
            authority: ctx.accounts.user_wallet.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_accounts,
        );
        anchor_spl::token::transfer(cpi_ctx, input_amount)?;

        // calculate amount of y
        let mut calculated_y_amount: u64 = (input_amount as u128)
            .checked_mul(StateAccount::PRICE_SCALE_FACTOR)
            .unwrap()
            .checked_div(ctx.accounts.state_account.x_to_y_scaled_price as u128)
            .unwrap() as u64;
        // apply spread to y
        calculated_y_amount -= (calculated_y_amount as u128)
            .checked_mul(ctx.accounts.state_account.spread_bps as u128)
            .unwrap()
            .checked_div(StateAccount::SPREAD_SCALE_FACTOR)
            .unwrap() as u64;

        // check user's expectations
        if calculated_y_amount < min_out_amount {
            return err!(WasabiError::OutputTooSmall);
        }

        // transfer <calculated_y_amount> from vault to user
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
        anchor_spl::token::transfer(cpi_ctx, calculated_y_amount)?;
    } else {
        // transfer <provided_amount> of y from user to vault
        let transfer_accounts = anchor_spl::token::Transfer {
            from: ctx.accounts.user_token_y_acct.to_account_info(),
            to: ctx.accounts.vault_token_y_acct.to_account_info(),
            authority: ctx.accounts.user_wallet.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_accounts,
        );
        anchor_spl::token::transfer(cpi_ctx, input_amount)?;

        // calculate amount of x
        let mut calculated_x_amount: u64 = (input_amount as u128)
            .checked_mul(ctx.accounts.state_account.x_to_y_scaled_price as u128)
            .unwrap()
            .checked_div(StateAccount::PRICE_SCALE_FACTOR)
            .unwrap() as u64;
        // apply spread to x
        calculated_x_amount -= (calculated_x_amount as u128)
            .checked_mul(ctx.accounts.state_account.spread_bps as u128)
            .unwrap()
            .checked_div(StateAccount::SPREAD_SCALE_FACTOR)
            .unwrap() as u64;

        // check user's expectations
        if calculated_x_amount < min_out_amount {
            return err!(WasabiError::OutputTooSmall);
        }

        // transfer <calculated_x_amount> from vault to user
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
        anchor_spl::token::transfer(cpi_ctx, calculated_x_amount)?;
    }

    // return
    Ok(())
}

#[derive(Accounts)]
pub struct SwapExactIn<'a> {
    user_wallet: Signer<'a>,
    #[account(
        token::mint = token_x_mint,
        token::authority = user_wallet,
    )]
    user_token_x_acct: Account<'a, TokenAccount>,
    #[account(
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
        token::mint = token_x_mint,
        token::authority = state_account,
    )]
    vault_token_x_acct: Account<'a, TokenAccount>,
    #[account(
        token::mint = token_y_mint,
        token::authority = state_account,
    )]
    vault_token_y_acct: Account<'a, TokenAccount>,

    token_x_mint: Account<'a, Mint>,
    token_y_mint: Account<'a, Mint>,
    system_program: Program<'a, System>,
    token_program: Program<'a, Token>,
}
