#![allow(deprecated)]

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;

pub use instructions::*;

declare_id!("5gdV4b4cPnnRkVSvBq8WxCxRfyq7i5z9R5scwm3BA4ps");

#[program]
pub mod escrow_app {
    use super::*;

    pub fn make_offer(
        ctx: Context<MakeOffer>,
        offer_id: u64,
        token_a_offered_amount: u64,
        token_b_amount_wanted: u64,
    ) -> Result<()> {
        instructions::send_offered_tokens_to_vault(&ctx, token_a_offered_amount)?;
        instructions::save_offer(ctx, offer_id, token_b_amount_wanted)
    }
}
