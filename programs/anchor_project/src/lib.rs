pub use anchor_lang::prelude::*;
pub use anchor_spl::token::{Mint, Token, TokenAccount, Transfer};

pub mod instructions;
use instructions::*;
// Accounts:
// this program
//
// Swap State PDA
//  - owned by: this program
//
// Token vault account x_token
//  - owned by: State PDA
//
// Token vault account y_token
//  - owned by: State PDA
//

declare_id!("HkyxYeSTPPpVbJHyYZh53w5sLZ7ZHsNrgDT2DPGQmQQp");

#[program]
pub mod contract {
    use super::*;

    pub fn init(ctx: Context<Init>, spread_bps: u64) -> Result<()> {
        instructions::init(ctx, spread_bps)
    }
    pub fn update_price(ctx: Context<UpdatePrice>, x_to_y_scaled_price: u64) -> Result<()> {
        instructions::update_price(ctx, x_to_y_scaled_price)
    }
    pub fn swap_exact_in(
        ctx: Context<SwapExactIn>,
        input_amount: u64,
        input_is_x: bool,
        min_out_amount: u64,
    ) -> Result<()> {
        instructions::swap_exact_in(ctx, input_amount, input_is_x, min_out_amount)
    }
    // pub fn swap_exact_out(ctx: Context<SwapExactOut>) -> Result<()> {
    //     instructions::swap_exact_out(ctx, output_amount)
    // }
}

#[account]
#[derive(InitSpace)]
pub struct StateAccount {
    // Used to enforce admin rights in UpdatePrice
    authority: Pubkey,

    // Scaled up by PRICE_SCALE_FACTOR
    x_to_y_scaled_price: u64,

    // Units are basis points (100bps == 1%)
    spread_bps: u64,
}
impl StateAccount {
    // Increase precision of price ratio
    const PRICE_SCALE_FACTOR: u128 = 1_000_000;

    // Basis points conversion
    const SPREAD_SCALE_FACTOR: u128 = 10_000;
}

#[error_code]
pub enum WasabiError {
    #[msg("Output is below the minimum expected")]
    OutputTooSmall,
}
