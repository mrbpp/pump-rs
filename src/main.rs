use clap::Parser;
use solana_account_decoder::UiAccountData;
use {
    dialoguer::{theme::ColorfulTheme, Confirm},
    futures::StreamExt,
    jito_protos::searcher::SubscribeBundleResultsRequest,
    jito_searcher_client::get_searcher_client,
    pump_rs::{
        analyze::run_analysis,
        app::{App, Command},
        ata, bench,
        constants::{TOKEN_PROGRAM, WSOL},
        data::look_for_rpc_nodes,
        jito::{
            get_bundle_status, make_searcher_client,
            start_bundle_results_listener, subscribe_tips,
        },
        jup::Jupiter,
        launcher::{self, IPFSMetaForm},
        pump::{self},
        pump::{get_bonding_curve, get_token_amount},
        pump_service,
        seller::{self, get_tx_with_retries},
        slot::{make_deadline_tx, update_slot},
        snipe, snipe_portal,
        util::{env, init_logger, parse_holding},
        wallet::make_manager,
    },
    solana_client::{
        nonblocking::rpc_client::RpcClient,
        rpc_config::RpcSendTransactionConfig,
        rpc_request::TokenAccountsFilter,
    },
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{Keypair, Signature},
        signer::{EncodableKey, Signer},
    },
    std::{error::Error, str::FromStr, sync::Arc, time::Duration},
    tokio::sync::{Mutex, RwLock},
};

use log::{info, warn};

// Helper function to identify known stablecoins and wrapped tokens
fn get_known_token_symbol(mint: &Pubkey) -> Option<&'static str> {
    let mint_str = mint.to_string();
    match mint_str.as_str() {
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => Some("USDC"),
        "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => Some("USDT"),
        "USD1ttGY1N17NEEHLmELoaybftRBUSErhqYiQzvEmuB" => Some("USD1"),
        "3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh" => Some("wBTC"),
        "7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs" => Some("wETH"),
        _ => None,
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::from_filename(".env")?;
    init_logger()?;

    let app = App::parse();

    match app.command {
        Command::WalletsFund { lamports } => {
            let wallet_manager = make_manager().await?;
            wallet_manager.fund_idempotent(lamports).await?;
        }
        Command::BundleStatusListener {} => {
            let searcher_client = make_searcher_client().await?;
            start_bundle_results_listener(Arc::new(Mutex::new(
                searcher_client,
            )))
            .await;
            tokio::signal::ctrl_c().await?;
        }
        Command::Launch {
            name,
            symbol,
            description,
            telegram,
            twitter,
            image_path,
            website,
            dev_buy,
            snipe_buy,
        } => {
            let wallet_manager = make_manager().await?;
            let signer = Keypair::read_from_file(env("FUND_KEYPAIR_PATH"))
                .expect("read wallet");

            launcher::launch(
                &IPFSMetaForm {
                    name,
                    symbol,
                    description,
                    telegram,
                    twitter,
                    website,
                    show_name: true,
                },
                Some(image_path),
                &signer,
                Some(dev_buy),
                Some(&wallet_manager),
                Some(snipe_buy),
            )
            .await?;
        }
        Command::WalletsDrain {} => {
            let manager = make_manager().await?;
            manager.drain().await?;
        }
        Command::Wallets {
            token_balances,
            create,
        } => {
            let mut manager = make_manager().await?;
            if let Some(count) = create {
                info!("Creating {} wallets...", count);
                manager.create_wallets(count)?;
                info!("Wallets created in: {}", manager.wallet_directory);
            } else if token_balances {
                manager.token_balances().await;
            } else {
                manager.balances().await;
            }
        }
        Command::LookForGeyser {} => {
            look_for_rpc_nodes().await;
        }
        Command::BundleStatus { bundle_id } => {
            get_bundle_status(bundle_id).await?;
        }
        Command::SubscribeTip {} => {
            let tip = Arc::new(RwLock::new(0u64));
            subscribe_tips(tip).await?;
        }
        Command::GetTx { sig } => {
            let signature = Signature::from_str(&sig).expect("parse sig");
            let rpc_client = RpcClient::new(env("RPC_URL").to_string());
            let tx = get_tx_with_retries(&rpc_client, &signature)
                .await
                .expect("tx");
            info!("{:#?}", tx);
        }
        Command::SlotCreated { mint } => {
            let rpc_client = RpcClient::new(env("RPC_URL").to_string());
            let slot_created = pump::get_slot_created(
                &rpc_client,
                &Pubkey::from_str(&mint)?,
            )
            .await?;
            info!("{}: created {}", mint, slot_created);
        }
        Command::SubscribePump {} => {
            let slot = Arc::new(RwLock::new(0));
            update_slot(slot.clone());
            let _ = pump::subscribe_to_pump(slot.clone()).await;
        }
        Command::TestSlotProgram {} => {
            let rpc_client = RpcClient::new(env("RPC_URL").to_string());
            let keypair = Keypair::read_from_file(env("FUND_KEYPAIR_PATH"))
                .expect("read wallet");
            let current_slot = rpc_client
                .get_slot_with_commitment(CommitmentConfig::confirmed())
                .await?;
            info!("Current slot: {}", current_slot);
            let tx = make_deadline_tx(
                current_slot + 20,
                rpc_client.get_latest_blockhash().await?,
                &keypair,
            );
            let sig = rpc_client
                .send_and_confirm_transaction_with_spinner_and_config(
                    &tx,
                    CommitmentConfig::confirmed(),
                    RpcSendTransactionConfig {
                        skip_preflight: true,
                        ..Default::default()
                    },
                )
                .await?;
            info!("{sig:#?}");
        }
        Command::SlotSubscribe {} => {
            let current_slot = Arc::new(RwLock::new(0));
            println!("Current slot: {}", *current_slot.read().await);
            tracing::info!(msg = "Subscribing to slot updates");
            let _ = update_slot(current_slot).await;
        }
        Command::IsOnCurve { pubkey } => {
            let pubkey = Pubkey::from_str(&pubkey).expect("parse pubkey");
            println!("Pubkey: {}", pubkey);
            println!("Is on curve: {}", pubkey.is_on_curve());
        }
        Command::Subscribe {} => {
            let auth = Arc::new(
                Keypair::read_from_file(env("AUTH_KEYPAIR_PATH")).unwrap(),
            );
            let _searcher_client =
                get_searcher_client(env("BLOCK_ENGINE_URL").as_str(), &auth)
                    .await
                    .expect("makes searcher client");
            return Err("Unimplemented".into());
        }
        Command::Seller {} => {
            info!("Running seller");
            seller::run_seller().await?;
        }
        Command::BenchPortal {} => {
            info!("Benching portal connection");
            bench::bench_pump_portal_connection().await?;
        }
        Command::BenchPump {} => {
            info!("Benching pump connection");
            bench::bench_pump_connection().await?;
        }
        Command::SnipePortal { lamports } => {
            info!("Sniping portal with {} lamports", lamports);
            snipe_portal::snipe_portal(lamports).await?;
        }
        Command::SnipePump { lamports } => {
            info!("Sniping pump with {} lamports", lamports);
            snipe::snipe_pump(lamports).await?;
        }
        Command::Analyze {
            wallet_path,
            address,
        } => {
            run_analysis(wallet_path, address).await?;
        }
        Command::Sanity {} => {
            let keypair = Keypair::read_from_file(env("FUND_KEYPAIR_PATH"))
                .expect("read wallet");
            let auth = Keypair::read_from_file(env("AUTH_KEYPAIR_PATH"))
                .expect("read auth");
            let rpc_client = RpcClient::new(env("RPC_URL").to_string());
            info!("Wallet: {}", keypair.pubkey());
            let balance = rpc_client.get_balance(&keypair.pubkey()).await?;
            info!("Balance: {}", balance);
            info!("Auth: {}", auth.pubkey());
            info!("RPC: {}", env("RPC_URL"));
            info!("Block Engine: {}", env("BLOCK_ENGINE_URL"));
        }
        Command::CloseTokenAccounts { wallet_path, burn } => {
            info!("Burn: {}", burn);
            let keypair =
                Keypair::read_from_file(wallet_path).expect("read wallet");
            info!("Wallet: {}", keypair.pubkey());
            let rpc_client =
                Arc::new(RpcClient::new(env("RPC_URL").to_string()));
            let auth = Arc::new(
                Keypair::read_from_file(env("AUTH_KEYPAIR_PATH")).unwrap(),
            );
            let mut searcher_client =
                get_searcher_client(env("BLOCK_ENGINE_URL").as_str(), &auth)
                    .await
                    .expect("makes searcher client");
            let mut bundle_results_stream = searcher_client
                .subscribe_bundle_results(SubscribeBundleResultsRequest {})
                .await
                .expect("subscribe bundle results")
                .into_inner();
            tokio::spawn(async move {
                while let Some(res) = bundle_results_stream.next().await {
                    info!("Received bundle result: {:?}", res);
                }
            });
            ata::close_all_atas(
                rpc_client,
                &keypair,
                burn,
                &mut searcher_client,
            )
            .await?;
        }
        Command::PumpService { lamports } => {
            pump_service::run_pump_service(lamports).await?;
        }
        Command::BumpPump { mint } => {
            let keypair = Keypair::read_from_file(env("BUMP_KEYPAIR_PATH"))
                .expect("read wallet");
            let rpc_client = RpcClient::new(env("RPC_URL").to_string());
            let auth = Arc::new(
                Keypair::read_from_file(env("AUTH_KEYPAIR_PATH")).unwrap(),
            );
            let mut searcher_client = Arc::new(RwLock::new(
                get_searcher_client(env("BLOCK_ENGINE_URL").as_str(), &auth)
                    .await
                    .expect("makes searcher client"),
            ));
            loop {
                match pump::send_pump_bump(
                    &keypair,
                    &rpc_client,
                    &Pubkey::from_str(&mint)?,
                    &mut searcher_client,
                    true,
                )
                .await
                {
                    Ok(_) => {
                        info!("Bump success");
                    }
                    Err(e) => {
                        warn!("Bump failed: {}", e);
                    }
                };

                tokio::time::sleep(Duration::from_secs(6)).await;
            }
        }
        Command::SweepPump { wallet_path } => {
            let keypair =
                Keypair::read_from_file(wallet_path).expect("read wallet");
            info!("Wallet: {}", keypair.pubkey());
            let rpc_client = RpcClient::new(env("RPC_URL").to_string());
            let pump_tokens =
                pump::get_tokens_held_pump(&keypair.pubkey()).await?;
            info!("Pump tokens held: {}", pump_tokens.len());

            // LOG 1: Show all pump.fun tokens from API
            for token in &pump_tokens {
                info!("Pump API token: {}", token.mint);
            }

            // Create a HashSet of pump.fun token mints for fast lookup
            let pump_mints: std::collections::HashSet<Pubkey> = pump_tokens
                .iter()
                .filter_map(|t| Pubkey::from_str(&t.mint).ok())
                .collect();

            // LOG 2: Show HashSet size
            info!("HashSet contains {} pump tokens", pump_mints.len());

            let token_accounts = rpc_client
                .get_token_accounts_by_owner(
                    &keypair.pubkey(),
                    TokenAccountsFilter::ProgramId(Pubkey::from_str(
                        TOKEN_PROGRAM,
                    )?),
                )
                .await?;
            for token_account in token_accounts {
                let holding =
                    parse_holding(token_account).expect("parse holding");
                let mint = holding.mint;

                // LOG 3: Check every token in wallet
                info!("Checking token: {}, amount: {}", mint, holding.amount);

                // FILTER 1: Skip if NOT a pump.fun token
                if !pump_mints.contains(&mint) {
                    // Check if it's a known stablecoin/wrapped token
                    if let Some(symbol) = get_known_token_symbol(&mint) {
                        info!("SKIPPED ({})", symbol);
                    } else {
                        info!("SKIPPED (not pump.fun): {}", mint);
                    }
                    continue;
                }

                info!("Token IS pump.fun: {}", mint);

                let pump_accounts = pump::mint_to_pump_accounts(&mint);
                if holding.amount > 0 {
                    // FILTER 2: Skip if bonding curve doesn't exist (graduated tokens)
                    if pump::get_bonding_curve(&rpc_client, pump_accounts.bonding_curve).await.is_err() {
                        warn!("Skipping {}: Token has graduated to Pump AMM", mint);
                        continue;
                    }

                    info!("Selling {} of {}", holding.amount, mint);
                    match pump::sell_pump_token(
                        &keypair,
                        rpc_client.get_latest_blockhash().await?,
                        pump_accounts,
                        holding.amount,
                        &rpc_client,
                    )
                    .await
                    {
                        Ok(_) => {
                            info!("Successfully sold {} of {}", holding.amount, mint);
                        }
                        Err(e) => {
                            warn!("Failed to sell {} of {}: {}", holding.amount, mint, e);
                            warn!("Continuing to next token...");
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(300)).await;
                }
            }
        }
        Command::SweepJup { wallet_path } => {
            let keypair =
                Keypair::read_from_file(wallet_path).expect("read wallet");
            let rpc_client = RpcClient::new(env("RPC_URL").to_string());
            let all_atas = rpc_client
                .get_token_accounts_by_owner(
                    &keypair.pubkey(),
                    TokenAccountsFilter::ProgramId(
                        Pubkey::from_str(TOKEN_PROGRAM).unwrap(),
                    ),
                )
                .await?;

            for acc in all_atas {
                // Extract the mint address from the parsed data
                let (mint, balance) =
                    if let UiAccountData::Json(parsed) = &acc.account.data {
                        if let serde_json::Value::Object(parsed_obj) =
                            &parsed.parsed
                        {
                            if let Some(info) = parsed_obj.get("info") {
                                if let Some(mint_str) =
                                    info.get("mint").and_then(|m| m.as_str())
                                {
                                    // Get the amount directly instead of uiAmountString
                                    if let Some(amount) = info
                                        .get("tokenAmount")
                                        .and_then(|ta| ta.get("amount"))
                                        .and_then(|a| a.as_str())
                                    {
                                        match amount.parse::<u64>() {
                                            Ok(parsed_amount) => (
                                                mint_str.to_string(),
                                                parsed_amount,
                                            ),
                                            Err(_) => continue, // Skip if parsing fails
                                        }
                                    } else {
                                        continue; // Skip if we can't get the amount
                                    }
                                } else {
                                    continue; // Skip if we can't get the mint
                                }
                            } else {
                                continue; // Skip if we can't get the info
                            }
                        } else {
                            continue; // Skip if parsed is not an object
                        }
                    } else {
                        continue; // Skip if data is not JSON
                    };

                if balance == 0 {
                    info!("Skipping {} because balance is 0", mint);
                    continue; // Skip if balance is 0
                }

                let slippage = 50u16; // 50 bps, 0.5%
                let quote =
                    Jupiter::fetch_quote(&mint, WSOL, balance, slippage)
                        .await?;
                println!("Quote for {}: {:#?}", mint, quote);

                if Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt(format!("Sell {} tokens?", mint))
                    .default(false)
                    .show_default(true)
                    .wait_for_newline(true)
                    .interact()
                    .unwrap()
                {
                    Jupiter::swap(quote, &keypair).await?;
                }
            }
        }
        Command::SwapMode { lamports, sell } => {
            let keypair = Keypair::read_from_file(env("FUND_KEYPAIR_PATH"))
                .expect("read wallet");
            let rpc_client =
                Arc::new(RpcClient::new(env("RPC_URL").to_string()));
            let tip = 50_000;

            loop {
                println!("Enter a mint address (or 'q' to quit):");
                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read line");
                let input = input.trim();

                if input.to_lowercase() == "q" {
                    break;
                }

                match Pubkey::from_str(input) {
                    Ok(mint) => {
                        let pump_accounts =
                            pump::mint_to_pump_accounts(&mint);
                        let latest_blockhash =
                            rpc_client.get_latest_blockhash().await?;

                        let bonding_curve = get_bonding_curve(
                            &rpc_client,
                            pump_accounts.bonding_curve,
                        )
                        .await?;

                        if sell {
                            let ata = spl_associated_token_account::get_associated_token_address(
                                &keypair.pubkey(),
                                &mint,
                            );
                            let token_amount = rpc_client
                                .get_token_account_balance(&ata)
                                .await?
                                .amount
                                .parse::<u64>()
                                .unwrap();
                            match pump::sell_pump_token(
                                &keypair,
                                latest_blockhash,
                                pump_accounts,
                                token_amount,
                                &rpc_client,
                            )
                            .await
                            {
                                Ok(_) => {
                                    info!(
                                        "Sell successful for mint: {}",
                                        mint
                                    )
                                }
                                Err(e) => {
                                    warn!(
                                        "Sell failed for mint {}: {}",
                                        mint, e
                                    )
                                }
                            }
                        } else {
                            info!("bonding curve: {:#?}", bonding_curve);
                            let token_amount = get_token_amount(
                                bonding_curve.virtual_sol_reserves,
                                bonding_curve.virtual_token_reserves,
                                None,
                                lamports,
                            )?;
                            match pump::buy_pump_token(
                                &keypair,
                                latest_blockhash,
                                pump_accounts,
                                token_amount,
                                lamports * 105 / 100, // slippage
                                tip,
                                &rpc_client,
                            )
                            .await
                            {
                                Ok(_) => {
                                    info!("Buy successful for mint: {}", mint)
                                }
                                Err(e) => {
                                    warn!(
                                        "Buy failed for mint {}: {}",
                                        mint, e
                                    )
                                }
                            }
                        }
                    }
                    Err(_) => {
                        println!("Invalid mint address. Please try again.")
                    }
                }
            }
        }
    }

    Ok(())
}
