use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Maker itself can not take the offer")]
    TakerShouldNotBeMaker,
}
