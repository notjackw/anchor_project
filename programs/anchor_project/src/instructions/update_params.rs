use crate::*;

pub fn update_params(
    ctx: Context<UpdateParams>,
    x_to_y_scaled_price: Option<u64>,
    spread_bps: Option<u64>,
) -> Result<()> {
    // Update the provided fields
    if let Some(x_to_y_scaled_price) = x_to_y_scaled_price {
        ctx.accounts.state_account.x_to_y_scaled_price = x_to_y_scaled_price;
    }
    if let Some(spread_bps) = spread_bps {
        ctx.accounts.state_account.spread_bps = spread_bps;
    }
    // return
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateParams<'a> {
    // Signer must be the account authority (set during init). See the 'has_one' constraint.
    pub authority: Signer<'a>,

    pub token_x_mint: Account<'a, Mint>,
    pub token_y_mint: Account<'a, Mint>,

    #[account(
        mut,
        seeds = [b"state", token_x_mint.key().as_ref(), token_y_mint.key().as_ref()],
        has_one = authority,
        bump,
    )]
    pub state_account: Account<'a, StateAccount>,
}
