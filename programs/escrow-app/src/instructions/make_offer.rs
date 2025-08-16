use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{constants::ANCHOR_DISCRIMINATOR, state::Offer, utils::transfer_tokens};

pub fn send_offered_tokens_to_vault(
    ctx: &Context<MakeOffer>,
    token_a_offered_amount: u64,
) -> Result<()> {
    transfer_tokens(
        &ctx.accounts.maker_token_account_a,
        &ctx.accounts.vault,
        &token_a_offered_amount,
        &ctx.accounts.token_mint_a,
        &ctx.accounts.maker,
        &ctx.accounts.token_program,
    )
}

/// The `MakeOffer` struct defines the accounts required to make a new trading offer.
///
/// It uses a Program Derived Address (PDA) to create a unique and verifiable `offer`
/// account, which will store the details of the trade. It also creates a separate
/// token account, a "vault," which is controlled by the `offer` PDA to securely
/// hold the tokens offered by the maker.
#[derive(Accounts)]
#[instruction(id: u64)] // `id` is a unique number for this specific offer.
pub struct MakeOffer<'info> {
    /// The person making the offer. They must sign the transaction and will pay for it.
    #[account(mut)]
    pub maker: Signer<'info>,

    /// The token the maker is offering
    #[account(mint::token_program = token_program)]
    pub token_mint_a: InterfaceAccount<'info, Mint>,

    /// The token the maker want in return.
    #[account(mint::token_program = token_program)]
    pub token_mint_b: InterfaceAccount<'info, Mint>,

    /// This is the maker's own Associated Token Account (ATA) for `token_mint_a`.
    /// It is marked as `mut` because tokens will be transferred out of it into the vault.
    ///
    /// `#[account(associated_token::mint = token_mint_a, ...)]`
    /// These constraints verify that this is indeed the correct ATA for the `maker`
    /// and `token_mint_a`, ensuring the transaction is acting on the intended account.
    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_token_account_a: InterfaceAccount<'info, TokenAccount>,

    /// This account will be created by the instruction to store the offer details. It's gonna be a PDA.
    ///
    /// Seeds are used to deterministically generate the PDA. This ensures
    /// the address is unique for each offer from a specific maker.
    #[account(
        init,
        payer = maker,
        space = (ANCHOR_DISCRIMINATOR as usize) + Offer::INIT_SPACE,
        seeds = [b"offer", maker.key().as_ref(), id.to_le_bytes().as_ref()],
        bump
    )]
    pub offer: Account<'info, Offer>,

    /// This is a new token account that will act as a "vault" or escrow for the
    /// tokens being offered.
    ///
    /// `associated_token::authority = offer`: This is the crucial constraint. It
    /// ensures this vault's authority is the `offer` PDA itself. This means that
    /// once tokens are deposited here, only the program (by using the `offer` PDA's
    /// signing authority) can move them out, securing the escrow.
    #[account(
        init,
        payer = maker,
        associated_token::mint = token_mint_a,
        associated_token::authority = offer,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    /// The Solana Token Program. This is required for all token-related operations,
    /// such as transferring tokens.
    pub token_program: Interface<'info, TokenInterface>,

    /// The Solana Associated Token Program. This is needed to create new ATAs
    /// for the `offer` PDA and the `maker_token_account_a`.
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// The Solana System Program. This is required to create new accounts (like
    /// the `offer` and `vault` accounts).
    pub system_program: Program<'info, System>,
}
