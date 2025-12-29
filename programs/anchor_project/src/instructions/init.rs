use crate::*;

pub fn init(ctx: Context<Initialize>, spread_bps: u64) -> Result<()> {
    ctx.accounts.state_account.spread_bps = spread_bps;
    ctx.accounts.state_account.x_to_y_scaled_price = 0;
    ctx.accounts.state_account.authority = ctx.accounts.user.key();
    Ok(())
}

#[derive(Accounts)]
pub struct Init<'a> {
    #[account(mut)]
    pub user: Signer<'a>,

    pub token_x_mint: Account<'a, Mint>,
    pub token_y_mint: Account<'a, Mint>,

    #[account(
        init,
        payer = user,
        token::mint = token_x_mint,
        token::authority = state_account,
    )]
    pub token_x_account: Account<'a, TokenAccount>,

    #[account(
        init,
        payer = user,
        token::mint = token_x_mint,
        token::authority = state_account,
    )]
    pub token_y_account: Account<'a, TokenAccount>,

    #[account(
        init,
        payer = user,
        space = StateAccount::INIT_SPACE + 8,
        seeds = [b"state", token_x_mint.key().as_ref(), token_y_mint.key().as_ref()],
        bump
    )]
    pub state_account: Account<'a, StateAccount>,

    // need this for creating accounts
    pub system_program: Program<'a, System>,
    pub token_program: Program<'a, Token>,
}
