use anyhow::{Context, Result};
use clap::Parser;
use log::{error, info, warn};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    compute_budget::ComputeBudgetInstruction,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_token::{
    instruction::{burn, close_account},
    state::Account as TokenAccount,
};
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// RPC endpoint URL
    #[arg(long, env = "RPC_ENDPOINT")]
    rpc_endpoint: String,

    /// Private key (base58 encoded)
    #[arg(long, env = "PRIVATE_KEY")]
    private_key: String,

    /// Skip USDC token accounts
    #[arg(long, default_value = "true")]
    skip_usdc: bool,

    /// Maximum instructions per transaction
    #[arg(long, default_value = "22")]
    max_instructions: usize,

    /// Compute unit price in micro-lamports
    #[arg(long, default_value = "220000")]
    compute_unit_price: u64,

    /// Compute unit limit
    #[arg(long, default_value = "350000")]
    compute_unit_limit: u32,
}

const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const SPL_TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let args = Args::parse();
    
    info!("Starting Solana token account burn and close tool");
    info!("RPC Endpoint: {}", args.rpc_endpoint);
    
    let rpc_client = RpcClient::new_with_commitment(
        args.rpc_endpoint.clone(),
        CommitmentConfig::confirmed(),
    );

    // Parse private key
    let keypair = parse_private_key(&args.private_key)?;
    info!("Wallet address: {}", keypair.pubkey());

    // Burn and close all token accounts
    burn_and_close_all_tokens(
        &rpc_client,
        &keypair,
        args.skip_usdc,
        args.max_instructions,
        args.compute_unit_price,
        args.compute_unit_limit,
    )
    .await?;

    info!("Token account cleanup completed successfully");
    Ok(())
}

fn parse_private_key(private_key_str: &str) -> Result<Keypair> {
    let private_key_bytes = bs58::decode(private_key_str)
        .into_vec()
        .context("Failed to decode base58 private key")?;
    
    Keypair::from_bytes(&private_key_bytes)
        .context("Failed to create keypair from private key")
}

async fn burn_and_close_all_tokens(
    rpc_client: &RpcClient,
    keypair: &Keypair,
    skip_usdc: bool,
    max_instructions: usize,
    compute_unit_price: u64,
    compute_unit_limit: u32,
) -> Result<()> {
    info!("Fetching token accounts for wallet: {}", keypair.pubkey());

    // Get all token accounts owned by the wallet
    let token_accounts = rpc_client
        .get_token_accounts_by_owner(
            &keypair.pubkey(),
            solana_client::rpc_request::TokenAccountsFilter::ProgramId(
                Pubkey::from_str(SPL_TOKEN_PROGRAM_ID)?,
            ),
        )
        .context("Failed to fetch token accounts")?;

    if token_accounts.is_empty() {
        info!("No token accounts found for this wallet");
        return Ok(());
    }

    info!("Found {} token accounts", token_accounts.len());

    let mut instructions = Vec::new();
    let mut accounts_processed = 0;

    for (pubkey, account) in token_accounts {
        let token_account_data = TokenAccount::unpack(&account.data)
            .context("Failed to unpack token account data")?;

        // Skip USDC if requested
        if skip_usdc && token_account_data.mint.to_string() == USDC_MINT {
            info!("Skipping USDC account: {}", pubkey);
            continue;
        }

        // Check if account has tokens to burn
        if token_account_data.amount > 0 {
            info!(
                "Burning {} tokens from account: {} (mint: {})",
                token_account_data.amount, pubkey, token_account_data.mint
            );

            let burn_instruction = burn(
                &spl_token::id(),
                &pubkey,
                &token_account_data.mint,
                &keypair.pubkey(),
                &[],
                token_account_data.amount,
            )?;

            instructions.push(burn_instruction);
        }

        // Always close the account to recover SOL
        info!("Closing token account: {}", pubkey);
        let close_instruction = close_account(
            &spl_token::id(),
            &pubkey,
            &keypair.pubkey(),
            &keypair.pubkey(),
            &[],
        )?;

        instructions.push(close_instruction);
        accounts_processed += 1;
    }

    if instructions.is_empty() {
        info!("No token accounts to process");
        return Ok(());
    }

    info!("Processing {} instructions for {} accounts", instructions.len(), accounts_processed);

    // Process instructions in batches
    let mut processed_instructions = 0;
    while processed_instructions < instructions.len() {
        let end_index = std::cmp::min(
            processed_instructions + max_instructions,
            instructions.len(),
        );

        let batch_instructions = &instructions[processed_instructions..end_index];
        
        info!(
            "Processing batch: instructions {} to {} (total: {})",
            processed_instructions + 1,
            end_index,
            instructions.len()
        );

        process_instruction_batch(
            rpc_client,
            keypair,
            batch_instructions,
            compute_unit_price,
            compute_unit_limit,
        )
        .await?;

        processed_instructions = end_index;
    }

    Ok(())
}

async fn process_instruction_batch(
    rpc_client: &RpcClient,
    keypair: &Keypair,
    instructions: &[Instruction],
    compute_unit_price: u64,
    compute_unit_limit: u32,
) -> Result<()> {
    let mut transaction_instructions = Vec::new();

    // Add compute budget instructions
    transaction_instructions.push(
        ComputeBudgetInstruction::set_compute_unit_price(compute_unit_price),
    );
    transaction_instructions.push(
        ComputeBudgetInstruction::set_compute_unit_limit(compute_unit_limit),
    );

    // Add the actual instructions
    transaction_instructions.extend_from_slice(instructions);

    // Create and send transaction
    let recent_blockhash = rpc_client
        .get_latest_blockhash()
        .context("Failed to get recent blockhash")?;

    let mut transaction = Transaction::new_with_payer(
        &transaction_instructions,
        Some(&keypair.pubkey()),
    );

    transaction.sign(&[keypair], recent_blockhash);

    // Simulate transaction first
    match rpc_client.simulate_transaction(&transaction) {
        Ok(simulation_result) => {
            if let Some(err) = simulation_result.value.err {
                error!("Transaction simulation failed: {:?}", err);
                return Err(anyhow::anyhow!("Transaction simulation failed: {:?}", err));
            }
            info!("Transaction simulation successful");
        }
        Err(e) => {
            warn!("Failed to simulate transaction: {:?}", e);
        }
    }

    // Send and confirm transaction
    let signature = rpc_client
        .send_and_confirm_transaction(&transaction)
        .context("Failed to send and confirm transaction")?;

    info!(
        "Transaction successful! Signature: {}",
        signature
    );
    info!(
        "View on Solscan: https://solscan.io/tx/{}",
        signature
    );

    Ok(())
}
