use anchor_lang::prelude::*;
use error::*;
use instructions::*;
use state::*;

mod error;
mod instructions;
pub mod state;

#[macro_use]
extern crate static_assertions;

// The program address.
declare_id!("4Q6WW2ouZ6V3iaNm56MTd5n2tnTm4C5fiH8miFHnAFHo");

#[program]
pub mod address_map_example {
    use super::*;

    pub fn register(ctx: Context<Register>, bump: u8) -> Result<()> {
        instructions::register(ctx, bump)
    }
}
