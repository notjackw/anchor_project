use crate::*;

pub fn update_price(ctx: Context<UpdatePrice>, x_to_y_scaled_price: u64) -> Result<()> {
    ctx.accounts.state_account.x_to_y_scaled_price = x_to_y_scaled_price;
    // return
    Ok(())
}

#[derive(Accounts)]
pub struct UpdatePrice<'a> {
    // The person signing this tx must be the listed authority on state_account
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
