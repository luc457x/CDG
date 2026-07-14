use crate::{api, cache, pipeline};
use anyhow::Result;

#[cfg(windows)]
pub fn clear_terminal() {
    if std::process::Command::new("cmd")
        .args(["/c", "cls"])
        .status()
        .is_err()
    {
        print!("\x1B[2J\x1B[1;1H");
        let _ = std::io::Write::flush(&mut std::io::stdout());
    }
}

#[cfg(not(windows))]
pub fn clear_terminal() {
    print!("\x1B[2J\x1B[1;1H");
    let _ = std::io::Write::flush(&mut std::io::stdout());
}

pub fn wait_for_back() {
    println!();
    let options = &["[Back]"];
    let _ = dialoguer::Select::new()
        .with_prompt("Press enter/select option to go back")
        .default(0)
        .items(options)
        .interact_opt();
}

pub async fn run_interactive_menu(
    db_path: &str,
    output_dir: &str,
    output_prefix: &str,
    raw_format: &str,
    mut cache_ttl: i64,
) -> Result<()> {
    let cache = std::sync::Arc::new(cache::Cache::new(db_path).await?);
    let mut cg_client = api::coingecko::CoinGeckoClient::new(cache.clone())?.with_ttl(cache_ttl);
    let mut yahoo_client = api::yahoo::YahooClient::new(cache.clone())?.with_ttl(cache_ttl);

    loop {
        clear_terminal();

        let options = &[
            "Run Portfolio Pipeline",
            "Ping Servers",
            "List Supported Coins",
            "Show Trending Coins",
            "Get Raw OHLCV Data",
            "Check Coin ID",
            "Settings",
            "Exit",
        ];

        let selection = dialoguer::Select::new()
            .with_prompt("Select an action")
            .default(0)
            .items(options)
            .interact_opt()?;

        let choice = match selection {
            Some(idx) => options[idx],
            None => {
                println!("Operation cancelled.");
                break;
            }
        };

        if choice != "Exit" {
            clear_terminal();
        }

        match choice {
            "Run Portfolio Pipeline" => {
                let coin: String = dialoguer::Input::new()
                    .with_prompt("Enter Coin ID(s) (comma-separated)")
                    .default("bitcoin".to_string())
                    .interact_text()?;

                let currency: String = dialoguer::Input::new()
                    .with_prompt("Enter Currency")
                    .default("usd".to_string())
                    .interact_text()?;

                let days: u32 = dialoguer::Input::new()
                    .with_prompt("Enter Timeframe (days)")
                    .default(90)
                    .interact_text()?;

                let prep_ml = dialoguer::Confirm::new()
                    .with_prompt("Enable ML Preprocessing?")
                    .default(false)
                    .interact()?;

                let light = dialoguer::Confirm::new()
                    .with_prompt("Enable Lightweight Mode?")
                    .default(false)
                    .interact()?;

                let drop_weekends = dialoguer::Confirm::new()
                    .with_prompt("Drop Weekends?")
                    .default(false)
                    .interact()?;

                let seed_str: String = dialoguer::Input::new()
                    .with_prompt("Enter Seed (optional, press Enter to skip)")
                    .allow_empty(true)
                    .interact_text()?;
                let seed = if seed_str.is_empty() {
                    None
                } else {
                    seed_str.parse::<u64>().ok()
                };

                let default_concurrency = {
                    let is_pro = std::env::var("COINGECKO_PRO_API_KEY").is_ok()
                        || (std::env::var("COINGECKO_API_KEY").is_ok()
                            && std::env::var("COINGECKO_API_KEY_TYPE")
                                .unwrap_or_default()
                                .to_lowercase()
                                == "pro");
                    if is_pro {
                        3
                    } else {
                        1
                    }
                };

                let concurrency: usize = dialoguer::Input::new()
                    .with_prompt("Enter Concurrency Limit")
                    .default(default_concurrency)
                    .interact_text()?;

                let ann_factor_str: String = dialoguer::Input::new()
                    .with_prompt(
                        "Enter Annualization Factor Override (optional, press Enter to skip)",
                    )
                    .allow_empty(true)
                    .interact_text()?;
                let annualization_factor = if ann_factor_str.is_empty() {
                    None
                } else {
                    ann_factor_str.parse::<f64>().ok()
                };

                let backtest = dialoguer::Confirm::new()
                    .with_prompt("Run strategy backtesting?")
                    .default(false)
                    .interact()?;

                let (strategy, fee, slippage, rebalance_frequency) = if backtest {
                    let strat_options = &["RSI", "MACD", "Bollinger Bands", "All (Compare)"];
                    let selection = dialoguer::Select::new()
                        .with_prompt("Select backtest strategy")
                        .default(0)
                        .items(strat_options)
                        .interact()?;
                    let strategy_str = match selection {
                        0 => "rsi",
                        1 => "macd",
                        2 => "bollinger",
                        3 => "all",
                        _ => "rsi",
                    };

                    let fee: f64 = dialoguer::Input::new()
                        .with_prompt("Enter transaction fee (decimal)")
                        .default(0.001)
                        .interact_text()?;

                    let slippage: f64 = dialoguer::Input::new()
                        .with_prompt("Enter slippage (decimal)")
                        .default(0.0005)
                        .interact_text()?;

                    let freq_options = &["daily", "weekly", "monthly"];
                    let freq_selection = dialoguer::Select::new()
                        .with_prompt("Select rebalancing frequency")
                        .default(0)
                        .items(freq_options)
                        .interact()?;
                    let freq_str = match freq_selection {
                        0 => "daily",
                        1 => "weekly",
                        2 => "monthly",
                        _ => "daily",
                    };

                    (
                        strategy_str.to_string(),
                        fee,
                        slippage,
                        freq_str.to_string(),
                    )
                } else {
                    ("rsi".to_string(), 0.001, 0.0005, "daily".to_string())
                };

                println!("\nRunning pipeline...\n");
                if let Err(e) = pipeline::run_pipeline_flow(pipeline::PipelineConfig {
                    coin: &coin,
                    currency: &currency,
                    days,
                    prep_ml,
                    light,
                    drop_weekends,
                    db_path,
                    output_dir,
                    output_prefix,
                    raw_format,
                    seed,
                    cache_ttl,
                    concurrency: Some(concurrency),
                    annualization_factor,
                    backtest,
                    strategy: &strategy,
                    fee,
                    slippage,
                    rebalance_frequency: &rebalance_frequency,
                    coingecko_base_url: None,
                    yahoo_base_url: None,
                    plots: true,
                    optimize: true,
                    candle_stdout: false,
                })
                .await
                {
                    println!("Error running pipeline: {}", e);
                }
            }
            "Ping Servers" => {
                println!("Pinging CoinGecko API...");
                match cg_client.ping().await {
                    Ok(val) => println!("CoinGecko connection successful: {:?}", val),
                    Err(e) => println!("CoinGecko ping failed: {}", e),
                }
                println!("Pinging Yahoo Finance API...");
                match yahoo_client.ping().await {
                    Ok(_) => println!("Yahoo Finance connection successful."),
                    Err(e) => println!("Yahoo Finance ping failed: {}", e),
                }
            }
            "List Supported Coins" => {
                println!("Fetching top 50 coins by market cap (USD)...");
                match cg_client.get_coins_markets("usd", 50).await {
                    Ok(coins) => {
                        println!(
                            "{:<20} | {:<10} | {:<25} | {:<15} | {:<15}",
                            "ID", "Symbol", "Name", "Price (USD)", "Market Cap"
                        );
                        println!("{}", "-".repeat(95));
                        for c in &coins {
                            let id = c.get("id").and_then(|v| v.as_str()).unwrap_or("");
                            let symbol = c.get("symbol").and_then(|v| v.as_str()).unwrap_or("");
                            let name = c.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            let price = c
                                .get("current_price")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0);
                            let market_cap =
                                c.get("market_cap").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            println!(
                                "{:<20} | {:<10} | {:<25} | {:<15.2} | {:<15.0}",
                                id,
                                symbol.to_uppercase(),
                                name,
                                price,
                                market_cap
                            );
                        }
                    }
                    Err(e) => println!("Failed to fetch coins list: {}", e),
                }
            }
            "Show Trending Coins" => {
                println!("Fetching trending coins...");
                match cg_client.get_search_trending().await {
                    Ok(val) => {
                        if let Some(coins) = val.get("coins").and_then(|v| v.as_array()) {
                            println!(
                                "{:<5} | {:<30} | {:<10} | {:<30}",
                                "Rank", "ID", "Symbol", "Name"
                            );
                            println!("{}", "-".repeat(81));
                            for (i, c) in coins.iter().enumerate() {
                                if let Some(item) = c.get("item") {
                                    let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
                                    let name =
                                        item.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                    let symbol =
                                        item.get("symbol").and_then(|v| v.as_str()).unwrap_or("");
                                    println!(
                                        "{:<5} | {:<30} | {:<10} | {:<30}",
                                        i + 1,
                                        id,
                                        symbol,
                                        name
                                    );
                                }
                            }
                        } else {
                            println!("No trending coins found in response.");
                        }
                    }
                    Err(e) => println!("Failed to fetch trending coins: {}", e),
                }
            }
            "Get Raw OHLCV Data" => {
                let coin: String = dialoguer::Input::new()
                    .with_prompt("Enter Coin ID")
                    .default("bitcoin".to_string())
                    .interact_text()?;

                let currency: String = dialoguer::Input::new()
                    .with_prompt("Enter Currency")
                    .default("usd".to_string())
                    .interact_text()?;

                let days: u32 = dialoguer::Input::new()
                    .with_prompt("Enter Timeframe (days)")
                    .default(90)
                    .interact_text()?;

                let format: String = dialoguer::Input::new()
                    .with_prompt("Enter Output Format (stdout, csv, json)")
                    .default("stdout".to_string())
                    .interact_text()?;

                if let Err(e) = pipeline::run_ohlcv_flow(
                    &cg_client,
                    &coin,
                    &currency,
                    days,
                    &format,
                    output_dir,
                    output_prefix,
                    raw_format,
                    false,
                )
                .await
                {
                    println!("Error fetching/exporting OHLCV data: {}", e);
                }
            }
            "Check Coin ID" => {
                let coin_to_check: String = dialoguer::Input::new()
                    .with_prompt("Enter Coin ID or symbol to check")
                    .interact_text()?;

                println!("Checking CoinGecko ID for '{}'...", coin_to_check);
                use crate::api::coingecko::CoinResolution;
                match cg_client.check_coin_id(&coin_to_check).await {
                    Ok(CoinResolution::Exact(resolved_id)) => {
                        println!(
                            "Success: '{}' is a valid CoinGecko ID (resolves to '{}').",
                            coin_to_check, resolved_id
                        );
                    }
                    Ok(CoinResolution::Ambiguous(suggestions)) => {
                        println!("Warning: '{}' is ambiguous.", coin_to_check);
                        println!("\nSuggested IDs:");
                        for sug in suggestions {
                            println!("  - {}", sug);
                        }
                    }
                    Ok(CoinResolution::NotFound) => {
                        println!("Error: '{}' is not a valid CoinGecko ID and no suggestions were found.", coin_to_check);
                    }
                    Err(e) => {
                        println!("Error: Failed to query CoinGecko coins list: {}", e);
                    }
                }
            }
            "Settings" => loop {
                clear_terminal();
                println!("Note: Interactive selections are not saved to disk. To set permanent defaults, edit your .env file.\n");
                let settings_options = &["Configure Cache TTL", "Back"];
                let settings_selection = dialoguer::Select::new()
                    .with_prompt("Select a setting")
                    .default(0)
                    .items(settings_options)
                    .interact_opt()?;

                let settings_choice = match settings_selection {
                    Some(idx) => settings_options[idx],
                    None => break,
                };

                if settings_choice == "Back" {
                    break;
                }

                match settings_choice {
                    "Configure Cache TTL" => {
                        clear_terminal();
                        let new_ttl: i64 = dialoguer::Input::new()
                            .with_prompt("Enter new Cache TTL in seconds")
                            .default(cache_ttl)
                            .interact_text()?;
                        cache_ttl = new_ttl;
                        cg_client = cg_client.with_ttl(cache_ttl);
                        yahoo_client = yahoo_client.with_ttl(cache_ttl);
                        println!("Cache TTL set to {} seconds.", cache_ttl);
                        wait_for_back();
                    }
                    _ => unreachable!(),
                }
            },
            "Exit" => {
                println!("Goodbye!");
                break;
            }
            _ => unreachable!(),
        }

        if choice != "Exit" && choice != "Settings" {
            wait_for_back();
        }
    }
    Ok(())
}
