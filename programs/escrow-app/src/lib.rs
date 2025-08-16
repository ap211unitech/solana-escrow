#![allow(deprecated)]

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;

declare_id!("5gdV4b4cPnnRkVSvBq8WxCxRfyq7i5z9R5scwm3BA4ps");

#[program]
pub mod escrow_app {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }
}
