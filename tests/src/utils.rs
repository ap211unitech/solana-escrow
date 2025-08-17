use std::{thread::sleep, time::Duration};

use anchor_client::solana_sdk::{
    commitment_config::CommitmentConfig, native_token::sol_to_lamports, program_pack::Pack,
    signature::Keypair, signer::Signer, transaction::Transaction,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use spl_associated_token_account::get_associated_token_address;
use spl_token::{id, solana_program::pubkey::Pubkey, state::Mint, ui_amount_to_amount};

pub async fn initialize() {
    let program_id = "5gdV4b4cPnnRkVSvBq8WxCxRfyq7i5z9R5scwm3BA4ps";
    let rpc_client = RpcClient::new_with_commitment(
        "http://localhost:8899".into(),
        CommitmentConfig::confirmed(),
    );

    let token_mint_authority = Keypair::new();
    rpc_client
        .request_airdrop(&token_mint_authority.pubkey(), sol_to_lamports(5.0))
        .await
        .unwrap();
    println!(
        "Airdropped 5 SOL to token mint authority: {}",
        token_mint_authority.pubkey()
    );

    sleep(Duration::from_secs(1));

    let maker = Keypair::new();
    rpc_client
        .request_airdrop(&maker.pubkey(), sol_to_lamports(5.0))
        .await
        .unwrap();
    println!("Airdropped 5 SOL to Maker: {}", maker.pubkey());

    let taker = Keypair::new();
    rpc_client
        .request_airdrop(&taker.pubkey(), sol_to_lamports(5.0))
        .await
        .unwrap();
    println!("Airdropped 5 SOL to Taker: {}", taker.pubkey());

    let (token_mint_a, token_mint_a_decimals) =
        create_token_mint(&rpc_client, &token_mint_authority).await;
    println!("Created Token Mint A: {}", token_mint_a);

    let (token_mint_b, token_mint_b_decimals) =
        create_token_mint(&rpc_client, &token_mint_authority).await;
    println!("Created Token Mint B: {}", token_mint_b);

    // Get or create maker's ATA for Token A
    let maker_ata_a = get_or_create_ata(&rpc_client, &maker, &token_mint_a).await;
    println!("Maker's ATA for Token A: {}", maker_ata_a);

    // Get or create taker's ATA for Token B
    let taker_ata_b = get_or_create_ata(&rpc_client, &taker, &token_mint_b).await;
    println!("Taker's ATA for Token B: {}", taker_ata_b);

    // Mint Token A to Maker's ATA
    mint_to_ata(
        &rpc_client,
        &token_mint_authority,
        &token_mint_a,
        &maker_ata_a,
        ui_amount_to_amount(100.0, token_mint_a_decimals),
    )
    .await;
    println!("Minted 100 Token A to Maker's ATA.");

    // Mint Token B to Taker's ATA
    mint_to_ata(
        &rpc_client,
        &token_mint_authority,
        &token_mint_b,
        &taker_ata_b,
        ui_amount_to_amount(80.0, token_mint_b_decimals),
    )
    .await;
    println!("Minted 80 Token B to Taker's ATA.");
}

async fn create_token_mint(rpc_client: &RpcClient, token_mint_authority: &Keypair) -> (Pubkey, u8) {
    let token_mint_authority_pubkey = token_mint_authority.pubkey();

    let (token_program_id, decimals) = (id(), 10 as u8);
    let space = Mint::LEN;
    let rent = rpc_client
        .get_minimum_balance_for_rent_exemption(space)
        .await
        .unwrap();

    let token_mint_account = Keypair::new();

    let token_mint_instruction = solana_system_interface::instruction::create_account(
        &token_mint_authority_pubkey,
        &token_mint_account.pubkey(),
        rent,
        space as u64,
        &token_program_id,
    );

    let token_mint_ix = spl_token::instruction::initialize_mint(
        &token_program_id,
        &token_mint_account.pubkey(),
        &token_mint_authority_pubkey,       // mint authority
        Some(&token_mint_authority_pubkey), // freeze authority
        decimals,
    )
    .unwrap();

    let recent_blockhash = rpc_client
        .get_latest_blockhash()
        .await
        .expect("Failed to get latest blockhash");

    let transaction = Transaction::new_signed_with_payer(
        &[token_mint_instruction.clone(), token_mint_ix],
        Some(&token_mint_authority_pubkey),
        &[&token_mint_authority, &token_mint_account],
        recent_blockhash,
    );

    rpc_client
        .send_and_confirm_transaction(&transaction)
        .await
        .unwrap();

    return (token_mint_account.pubkey(), decimals);
}

async fn get_or_create_ata(rpc_client: &RpcClient, owner: &Keypair, mint: &Pubkey) -> Pubkey {
    let owner_pubkey = owner.pubkey();
    let ata_pubkey = get_associated_token_address(&owner_pubkey, mint);

    // Check if the ATA already exists
    if rpc_client.get_account(&ata_pubkey).await.is_ok() {
        return ata_pubkey;
    }

    // If it doesn't exist, create it
    let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
        &owner_pubkey,
        &owner_pubkey,
        mint,
        &id(),
    );

    let recent_blockhash = rpc_client.get_latest_blockhash().await.unwrap();

    let transaction = Transaction::new_signed_with_payer(
        &[create_ata_ix],
        Some(&owner_pubkey),
        &[owner],
        recent_blockhash,
    );

    rpc_client
        .send_and_confirm_transaction(&transaction)
        .await
        .unwrap();

    ata_pubkey
}

async fn mint_to_ata(
    rpc_client: &RpcClient,
    mint_authority: &Keypair,
    mint: &Pubkey,
    destination: &Pubkey,
    amount: u64,
) {
    let mint_authority_pubkey = mint_authority.pubkey();

    let mint_ix = spl_token::instruction::mint_to(
        &id(),
        mint,
        destination,
        &mint_authority_pubkey,
        &[&mint_authority_pubkey],
        amount,
    )
    .unwrap();

    let recent_blockhash = rpc_client.get_latest_blockhash().await.unwrap();

    let transaction = Transaction::new_signed_with_payer(
        &[mint_ix],
        Some(&mint_authority_pubkey),
        &[mint_authority],
        recent_blockhash,
    );

    rpc_client
        .send_and_confirm_transaction(&transaction)
        .await
        .unwrap();
}
