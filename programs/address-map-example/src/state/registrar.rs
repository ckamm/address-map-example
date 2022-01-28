use crate::error::*;
use anchor_lang::prelude::*;

#[account(zero_copy)]
#[derive(Default)]
pub struct Registrar {
    pub user: Pubkey,
    pub address_map: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
}
const_assert!(std::mem::size_of::<Registrar>() == 97);

#[macro_export]
macro_rules! registrar_seeds {
    ( $registrar:expr ) => {
        &[
            b"registrar".as_ref(),
            $registrar.user.as_ref(),
            $registrar.mint.as_ref(),
            &[$registrar.bump],
        ]
    };
}

pub use registrar_seeds;
