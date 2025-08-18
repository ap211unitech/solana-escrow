use std::str::FromStr;

use anchor_client::{
    solana_sdk::{self, commitment_config::CommitmentConfig, signature::Signer},
    Cluster,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::{
    solana_program::{program_pack::Pack, pubkey::Pubkey},
    state::Account as TokenAccount,
    ui_amount_to_amount,
};

use crate::utils::{initialize, SetupStruct};

use escrow_app;

#[tokio::test]
pub async fn make_and_take_offer() {
    println!("\n//// take_offer instruction ////");

    // Setup environment: funded accounts, minted tokens, ATAs, balances
    let SetupStruct {
        rpc_client,
        maker,
        taker,
        token_mint_a,
        token_mint_b,
        token_mint_a_decimals,
        token_mint_b_decimals,
        maker_ata_a,
        taker_ata_b,
    } = initialize().await;

    let program_id = Pubkey::from_str("5gdV4b4cPnnRkVSvBq8WxCxRfyq7i5z9R5scwm3BA4ps").unwrap();
    let program = anchor_client::Client::new_with_options(
        Cluster::Localnet,
        &maker,
        CommitmentConfig::confirmed(),
    )
    .program(program_id)
    .unwrap();

    let maker_pubkey = maker.pubkey();
    let taker_pubkey = taker.pubkey();

    // Instruction parameters
    let offer_id: u64 = 1;
    let token_a_offered_amount: u64 = ui_amount_to_amount(100.0, token_mint_a_decimals); // 50 tokens (depends on decimals)
    let token_b_amount_wanted: u64 = ui_amount_to_amount(80.0, token_mint_b_decimals);

    let (offer_pda, _) = Pubkey::find_program_address(
        &[b"offer", maker_pubkey.as_ref(), &offer_id.to_le_bytes()],
        &program_id,
    );

    let vault_ata = get_associated_token_address(&offer_pda, &token_mint_a);

    let taker_ata_a = get_associated_token_address(&taker_pubkey, &token_mint_a);
    let maker_ata_b = get_associated_token_address(&maker_pubkey, &token_mint_b);

    // Send transaction via Anchor client (Make Offer)
    program
        .request()
        .accounts(escrow_app::accounts::MakeOffer {
            maker: maker_pubkey,
            token_mint_a,
            token_mint_b,
            maker_token_account_a: maker_ata_a,
            offer: offer_pda,
            vault: vault_ata,
            token_program: spl_token::id(),
            associated_token_program: spl_associated_token_account::ID,
            system_program: solana_sdk::system_program::id(),
        })
        .args(escrow_app::instruction::MakeOffer {
            offer_id,
            token_a_offered_amount,
            token_b_amount_wanted,
        })
        .send()
        .await
        .unwrap();

    ///////////// Take Offer /////////////
    let maker_account_balance_before = rpc_client.get_balance(&maker_pubkey).await.unwrap();

    let signature = program
        .request()
        .accounts(escrow_app::accounts::TakeOffer {
            taker: taker_pubkey,
            maker: maker_pubkey,
            token_mint_a,
            token_mint_b,
            taker_token_account_a: taker_ata_a,
            taker_token_account_b: taker_ata_b,
            maker_token_account_b: maker_ata_b,
            offer: offer_pda,
            vault: vault_ata,
            token_program: spl_token::id(),
            associated_token_program: spl_associated_token_account::ID,
            system_program: solana_sdk::system_program::id(),
        })
        .args(escrow_app::instruction::TakeOffer {})
        .signer(taker)
        .send()
        .await
        .unwrap();

    println!("TakeOffer Successful with signature: {}", signature);

    // Asset maker's account balance
    let maker_account_balance_after = rpc_client.get_balance(&maker_pubkey).await.unwrap();
    assert!(maker_account_balance_after > maker_account_balance_before);

    // Assert maker's token's balance of token mint a
    let maker_ata_account_a = rpc_client.get_account(&maker_ata_a).await.unwrap();
    let maker_data_a = TokenAccount::unpack(&maker_ata_account_a.data).unwrap();
    assert_eq!(maker_data_a.amount, 0);

    // Assert maker's token's balance of token mint b
    let maker_ata_account_b = rpc_client.get_account(&maker_ata_b).await.unwrap();
    let maker_data_b = TokenAccount::unpack(&maker_ata_account_b.data).unwrap();
    assert_eq!(maker_data_b.amount, token_b_amount_wanted);

    // Assert taker's token's balance of token mint a
    let taker_ata_account_a = rpc_client.get_account(&taker_ata_a).await.unwrap();
    let taker_data_a = TokenAccount::unpack(&taker_ata_account_a.data).unwrap();
    assert_eq!(taker_data_a.amount, token_a_offered_amount);

    // Assert taker's token's balance of token mint b
    let taker_ata_account_b = rpc_client.get_account(&taker_ata_b).await.unwrap();
    let taker_data_b = TokenAccount::unpack(&taker_ata_account_b.data).unwrap();
    assert_eq!(taker_data_b.amount, 0);

    // Asset offer PDA
    let offer_closed = rpc_client.get_account(&offer_pda).await;
    assert!(
        offer_closed.is_err(),
        "Offer account should be closed after take"
    );

    // Asset vault PDA
    let vault_closed = rpc_client.get_account(&vault_ata).await;
    assert!(
        vault_closed.is_err(),
        "Vault ATA should be closed after take"
    );

    println!();
}
