use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::{state::Offer, utils::transfer_tokens};

pub fn send_tokens_from_taker_to_maker(ctx: &Context<TakeOffer>) -> Result<()> {
    transfer_tokens(
        &ctx.accounts.taker_token_account_b,
        &ctx.accounts.maker_token_account_b,
        &ctx.accounts.offer.token_b_amount_wanted,
        &ctx.accounts.token_mint_b,
        &ctx.accounts.taker,
        &ctx.accounts.token_program,
    )
}

pub fn withdraw_from_vault_and_close_it(ctx: Context<TakeOffer>) -> Result<()> {
    // Transfer tokens held by vault token account (which is PDA for token_mint_a and maker) to taker's token account
    let seeds = &[
        b"offer",
        ctx.accounts.maker.key.as_ref(),
        &ctx.accounts.offer.id.to_le_bytes()[..],
        &[ctx.accounts.offer.bump],
    ];
    let signer_seeds = [&seeds[..]];

    let accounts = TransferChecked {
        from: ctx.accounts.vault.to_account_info(),
        to: ctx.accounts.taker_token_account_a.to_account_info(),
        authority: ctx.accounts.offer.to_account_info(),
        mint: ctx.accounts.token_mint_a.to_account_info(),
    };

    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        accounts,
        &signer_seeds,
    );

    transfer_checked(
        cpi_context,
        ctx.accounts.vault.amount,
        ctx.accounts.token_mint_a.decimals,
    )?;

    // Vault can be closed safely now
    let accounts = CloseAccount {
        account: ctx.accounts.vault.to_account_info(),
        authority: ctx.accounts.offer.to_account_info(),
        destination: ctx.accounts.maker.to_account_info(),
    };

    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        accounts,
        &signer_seeds,
    );

    close_account(cpi_context)
}

/// The `TakeOffer` struct defines the accounts required to accept an existing offer.
///
/// This instruction will transfer the tokens from the taker to the maker and
/// release the maker's tokens from the vault to the taker. It closes the offer
/// and vault accounts, returning the rent to the maker.
#[derive(Accounts)]
pub struct TakeOffer<'info> {
    /// The person accepting the offer. They must be a `Signer` to authorize the transaction.
    /// This account will pay for any new accounts created.
    #[account(mut)]
    pub taker: Signer<'info>,

    /// The person who originally made the offer. They are a `SystemAccount` because
    /// their primary role here is to receive a token transfer and the rent from
    /// the closed accounts. The `has_one = maker` constraint on the `offer` account
    /// ensures this is the correct maker.
    #[account(mut)]
    pub maker: SystemAccount<'info>,

    /// The token the maker was offering; taker will take this token essentially
    #[account(mint::token_program = token_program)]
    pub token_mint_a: InterfaceAccount<'info, Mint>,

    /// The token the maker want in return; maker will get this token essentially
    #[account(mint::token_program = token_program)]
    pub token_mint_b: InterfaceAccount<'info, Mint>,

    /// The taker's token account for `token_mint_a`. This is where the tokens from
    /// the vault will be transferred to. `init_if_needed` means Anchor will create
    /// this account if it doesn't already exist. The `taker` pays for the rent.
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = token_mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_token_account_a: InterfaceAccount<'info, TokenAccount>,

    /// The taker's token account for `token_mint_b`. This is where the taker's
    /// tokens will be transferred from to pay the maker.
    #[account(
        mut,
        associated_token::mint = token_mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program
    )]
    pub taker_token_account_b: InterfaceAccount<'info, TokenAccount>,

    /// The maker's token account for `token_mint_b`. This is where the taker's
    /// tokens `token_mint_b` will be transferred. `init_if_needed` ensures the maker doesn't need
    /// to have this account ready beforehand; it will be created if necessary.
    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = token_mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_token_account_b: InterfaceAccount<'info, TokenAccount>,

    /// The offer account itself. It is marked `mut` because its state might change,
    /// and `close` will remove it from the blockchain, returning its rent to the `maker`.
    /// The `has_one` and `seeds` constraints are used to securely verify that this
    /// is the correct and valid offer PDA.
    #[account(
        mut,
        close = maker,
        has_one = maker,
        has_one = token_mint_a,
        has_one = token_mint_b,
        seeds = [b"offer", maker.key().as_ref(), offer.id.to_le_bytes().as_ref()],
        bump = offer.bump,
    )]
    pub offer: Account<'info, Offer>,

    /// The vault token account holding the tokens from the maker. This is where
    /// the tokens will be taken from. It is closed after the transfer, returning its
    /// rent to the maker.
    #[account(
        mut,
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
