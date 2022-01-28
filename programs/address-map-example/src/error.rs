use anchor_lang::prelude::*;

#[error]
pub enum ErrorCode {
    // 6000 / 0x1770
    #[msg("")]
    Invalid,
}
