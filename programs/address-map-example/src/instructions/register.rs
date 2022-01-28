use crate::error::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use std::mem::size_of;
use crate::solana_address_lookup_table_instruction as solana_address_lookup_table_instruction;
use solana_program::slot_history::Slot;

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
    pub address_lookup_table: UncheckedAccount<'info>,
}

/// Returns if the anchor discriminator on the account is still unset
pub fn is_freshly_initialized(account_info: &AccountInfo) -> Result<bool> {
    let data = account_info.try_borrow_data()?;
    let mut disc_bytes = [0u8; 8];
    disc_bytes.copy_from_slice(&data[..8]);
    let discriminator = u64::from_le_bytes(disc_bytes);
    Ok(discriminator == 0)
}

pub fn register(ctx: Context<Register>, bump: u8, recent_slot: Slot) -> Result<()> {
    let registrar_address = ctx.accounts.registrar.key();
    let user_address = ctx.accounts.user.key();
    let new_registrar = is_freshly_initialized(ctx.accounts.registrar.as_ref())?;
    let registrar = if new_registrar {
        // I need a non-mutable reference to Registrar later, so there can be
        // more borrows during CPI. Anchor only allows me to call load_init() though.
        // Hack: manually set the discriminator _now_!
        ctx.accounts.registrar.exit(&crate::id())?;

        {
            let mut registrar = ctx.accounts.registrar.load_mut()?;
            registrar.user = user_address;
            registrar.mint = ctx.accounts.token_account.mint;
            registrar.address_map = ctx.accounts.address_map.key();
            registrar.bump = bump;
        }

        // create address map
        let (instruction, expected_adress_map_address) =
            solana_address_lookup_table_instruction::create_lookup_table(
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
        let registrar = ctx.accounts.registrar.load()?;
        let seeds = registrar_seeds!(registrar);
        solana_program::program::invoke_signed(&instruction, &account_infos, &[seeds])?;

        registrar
    } else {
        let registrar = ctx.accounts.registrar.load()?;
        require!(registrar.mint == ctx.accounts.token_account.mint, Invalid);
        require!(registrar.address_map == ctx.accounts.address_map.key(), Invalid);
        registrar
    };

    // TODO: check if the account already exists in the address map

    // extend address map with new account
    let instruction = solana_address_lookup_table_instruction::extend_lookup_table(
        registrar.address_map, registrar_address, user_address, vec![ctx.accounts.token_account.key()]);
    let account_infos = [
        ctx.accounts.address_map.to_account_info(),
        ctx.accounts.registrar.to_account_info(),
        ctx.accounts.user.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
    ];
    let seeds = registrar_seeds!(registrar);
    solana_program::program::invoke_signed(&instruction, &account_infos, &[seeds])?;

    Ok(())
}
