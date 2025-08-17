use std::str::FromStr;

use anchor_client::{
    anchor_lang::AccountDeserialize,
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

use escrow_app::{self, state::Offer};

#[tokio::test]
async fn make_offer() {
    println!("\n//// make_offer instruction ////");

    // Setup environment: funded accounts, minted tokens, ATAs, balances
    let SetupStruct {
        rpc_client,
        maker,
        taker: _,
        token_mint_a,
        token_mint_b,
        token_mint_a_decimals,
        token_mint_b_decimals,
        maker_ata_a,
        taker_ata_b: _,
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

    // Instruction parameters
    let offer_id: u64 = 1;
    let token_a_offered_amount: u64 = ui_amount_to_amount(100.0, token_mint_a_decimals); // 50 tokens (depends on decimals)
    let token_b_amount_wanted: u64 = ui_amount_to_amount(80.0, token_mint_b_decimals);

    let (offer_pda, _) = Pubkey::find_program_address(
        &[b"offer", maker_pubkey.as_ref(), &offer_id.to_le_bytes()],
        &program_id,
    );

    let vault_ata = get_associated_token_address(&offer_pda, &token_mint_a);

    // Send transaction via Anchor client
    let signature = program
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

    println!("MakeOffer Successful with signature: {}", signature);

    // Assert vault balance == token_a_offered_amount
    let vault_acc = rpc_client.get_account(&vault_ata).await.unwrap();
    let vault_data = TokenAccount::unpack(&vault_acc.data).unwrap();
    assert_eq!(vault_data.amount, token_a_offered_amount);

    // Assert maker ATA balance reduced
    let maker_acc = rpc_client.get_account(&maker_ata_a).await.unwrap();
    let maker_data = TokenAccount::unpack(&maker_acc.data).unwrap();
    assert_eq!(
        maker_data.amount,
        ui_amount_to_amount(100.0, token_mint_a_decimals) - token_a_offered_amount
    );

    // Asset Offer info
    let offer_account = rpc_client.get_account(&offer_pda).await.unwrap();
    let offer = Offer::try_deserialize(&mut offer_account.data.as_slice()).unwrap();

    assert_eq!(offer.id, 1);
    assert_eq!(offer.maker, maker_pubkey);
    assert_eq!(offer.token_mint_a, token_mint_a);
    assert_eq!(offer.token_mint_b, token_mint_b);
    assert_eq!(offer.token_b_amount_wanted, token_b_amount_wanted);

    println!();
}
