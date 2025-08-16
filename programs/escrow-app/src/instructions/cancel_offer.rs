use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::state::Offer;

pub fn withdraw_from_vault_and_close_it(ctx: Context<CancelOffer>) -> Result<()> {
    // Transfer tokens held in vault back to maker's ATA for token_a
    let seeds = [
        b"offer",
        ctx.accounts.maker.key.as_ref(),
        &ctx.accounts.offer.id.to_le_bytes(),
        &[ctx.accounts.offer.bump],
    ];
    let signer_seeds = [&seeds[..]];

    let accounts = TransferChecked {
        from: ctx.accounts.vault.to_account_info(),
        to: ctx.accounts.maker_token_account_a.to_account_info(),
        mint: ctx.accounts.token_mint_a.to_account_info(),
        authority: ctx.accounts.offer.to_account_info(),
    };

    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        accounts,
        &signer_seeds,
    );

    ctx.accounts.vault.reload()?;

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

#[derive(Accounts)]
pub struct CancelOffer<'info> {
    /// The person created the offer. They must be a `Signer` to authorize the transaction.
    /// This account will pay for any new accounts created.
    #[account(mut)]
    pub maker: Signer<'info>,

    /// The token the maker was offering; taker will take this token essentially
    #[account(mint::token_program = token_program)]
    pub token_mint_a: InterfaceAccount<'info, Mint>,

    /// The maker's token account for `token_mint_a`. This is where the tokens from
    /// the vault will be transferred to. `init_if_needed` means Anchor will create
    /// this account if it doesn't already exist. The `maker` ofcourse pays for the rent.
    #[account(
        init_if_needed,
        payer = maker,
        associated_token::mint = token_mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_token_account_a: InterfaceAccount<'info, TokenAccount>,

    /// The offer account itself. It is marked `mut` because its state will change,
    /// and `close` will remove it from the blockchain, returning its rent to the `maker`.
    /// The `has_one` and `seeds` constraints are used to securely verify that this
    /// is the correct and valid offer PDA.
    #[account(
        mut,
        close = maker,
        has_one = maker,
        has_one = token_mint_a,
        seeds = [b"offer", maker.key().as_ref(), offer.id.to_le_bytes().as_ref()],
        bump = offer.bump,
    )]
    pub offer: Account<'info, Offer>,

    /// The vault token account holding the tokens from the maker. This is where
    /// the tokens will be taken from and returned back to maker.
    /// It is closed after the transfer, returning its rent to the maker.
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
