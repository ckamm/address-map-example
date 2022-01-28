use crate::error::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use std::mem::size_of;

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct Register<'info> {
    #[account(
        init_if_needed,
        seeds = [b"registrar".as_ref(), user.key().as_ref(), token_account.mint.as_ref()],
        bump = bump,
        payer = user,
    )]
    pub registrar: AccountLoader<'info, Registrar>,

    #[account(mut)]
    pub address_map: UncheckedAccount<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_account: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

/// Returns if the anchor discriminator on the account is still unset
pub fn is_freshly_initialized(account_info: &AccountInfo) -> Result<bool> {
    let data = account_info.try_borrow_data()?;
    let mut disc_bytes = [0u8; 8];
    disc_bytes.copy_from_slice(&data[..8]);
    let discriminator = u64::from_le_bytes(disc_bytes);
    Ok(discriminator == 0)
}

pub fn register(ctx: Context<Register>, bump: u8) -> Result<()> {
    let registrar_address = ctx.accounts.registrar.key();
    let user_address = ctx.accounts.user.key();
    let recent_slot = Clock::get()?.slot;
    let new_registrar = is_freshly_initialized(ctx.accounts.registrar.as_ref())?;
    let mut registrar = if new_registrar {
        let mut registrar = ctx.accounts.registrar.load_init()?;
        registrar.mint = ctx.accounts.token_account.mint;
        registrar.address_map = ctx.accounts.address_map.key();
        registrar.bump = bump;

        // create address map
        let (instruction, expected_adress_map_address) =
            solana_address_lookup_table_program::instruction::create_lookup_table(
                registrar_address,
                user_address,
                recent_slot,
            );
        require!(
            expected_adress_map_address == ctx.accounts.address_map.key(),
            Invalid
        );
        let account_infos = [
            ctx.accounts.address_map.to_account_info(),
            ctx.accounts.registrar.to_account_info(),
            ctx.accounts.user.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ];
        let seeds = registrar_seeds!(registrar);
        solana_program::program::invoke_signed(&instruction, &account_infos, &[seeds])?;

        registrar
    } else {
        let mut registrar = ctx.accounts.registrar.load_mut()?;
        require!(registrar.mint == ctx.accounts.token_account.mint, Invalid);
        require!(registrar.address_map == ctx.accounts.address_map.key(), Invalid);
        registrar
    };

    // extend address map
    // TODO

    Ok(())
}
