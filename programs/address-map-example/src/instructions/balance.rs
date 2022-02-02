use crate::error::*;
use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

#[derive(Accounts)]
pub struct Balance {
    // We only care about remainingAccounts which will be filled
// from the user's address map
}

pub fn balance(ctx: Context<Balance>, expected: u64) -> Result<()> {
    let sum = ctx
        .remaining_accounts
        .iter()
        .map(|account| {
            let token = Account::<TokenAccount>::try_from(&account.clone()).unwrap();
            token.amount
        })
        .sum::<u64>();

    msg!("the total balance is {}", sum);
    require!(sum == expected, Invalid);

    Ok(())
}
