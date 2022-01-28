use crate::error::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use std::mem::size_of;

#[derive(Accounts)]
pub struct Balance {
    // We only care about remainingAccounts which will be filled
    // from the user's address map
}

pub fn balance(ctx: Context<Balance>) -> Result<()> {
    let sum = ctx.remaining_accounts.iter().map(
        |account| {
            let token = Account::<TokenAccount>::try_from(&account.clone()).unwrap();
            token.amount
        })
        .sum::<u64>();

    msg!("the total balance is {}", sum);

    Ok(())
}
