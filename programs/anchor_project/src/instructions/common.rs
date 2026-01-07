use crate::*;

pub fn transfer_user_to_vault<'a>(
    from: AccountInfo<'a>,
    to: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    amount: u64,
    token_program: AccountInfo<'a>,
) -> Result<()> {
    let transfer_accounts = anchor_spl::token::Transfer {
        from,
        to,
        authority,
    };
    let cpi_ctx = CpiContext::new(token_program.to_account_info(), transfer_accounts);
    anchor_spl::token::transfer(cpi_ctx, amount)?;
    Ok(())
}
