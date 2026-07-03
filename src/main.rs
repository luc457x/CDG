use anyhow::Result;
use cdg::{analysis, api, cache, export, plot};
use clap::{Parser, Subcommand};
use std::io::Write;

#[derive(Parser, Debug, PartialEq)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Database file path (default: cdg_files/cache.db)
    #[arg(long, default_value = "cdg_files/cache.db")]
    pub db_path: String,

    /// Path to export results (default: cdg_files/output)]
    #[arg(short, long, default_value = "cdg_files/output")]
    pub output_prefix: String,

    /// Cache TTL in seconds (default: 300)
    #[arg(long, default_value_t = 300)]
    pub cache_ttl: i64,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum Commands {
    /// Run full data processing pipeline
    RunPipeline {
        /// Coin ID or comma-separated list of coin IDs from CoinGecko (default: bitcoin)
        #[arg(short, long, default_value = "bitcoin")]
        coin: String,

        /// Vs currency for CoinGecko (default: usd)
        #[arg(short = 'v', long, default_value = "usd")]
        currency: String,

        /// Timeframe in days (default: 90)
        #[arg(short, long, default_value_t = 90)]
        days: u32,

        /// Enable ML preprocessing pipeline (scaling)
        #[arg(long)]
        prep_ml: bool,

        /// Enable lightweight mode (forces coin=bitcoin, days=30, skips benchmarks)
        #[arg(long)]
        light: bool,

        /// Drop weekends instead of forward-filling
        #[arg(long)]
        drop_weekends: bool,

        /// Optional RNG seed for Monte Carlo simulation (default: 1337)
        #[arg(long)]
        seed: Option<u64>,

        /// CoinGecko query concurrency limit (default: 3)
        #[arg(long, env = "COINGECKO_CONCURRENCY", default_value_t = 3)]
        concurrency: usize,

        /// Custom annualization factor override (e.g. 252 or 365)
        #[arg(long, env = "ANNUALIZATION_FACTOR")]
        annualization_factor: Option<f64>,
    },
    /// Ping CoinGecko and Yahoo Finance API servers
    Ping,
    /// List supported/available coins
    ListCoins,
    /// Get trending coins on CoinGecko
    Trending,
    /// Retrieve and show/export raw OHLCV data for a coin
    Ohlcv {
        /// Coin ID
        #[arg(short, long, default_value = "bitcoin")]
        coin: String,

        /// Vs currency (default: usd)
        #[arg(short = 'v', long, default_value = "usd")]
        currency: String,

        /// Timeframe in days (default: 90)
        #[arg(short, long, default_value_t = 90)]
        days: u32,

        /// Export format: 'stdout', 'csv', or 'json' (default: stdout)
        #[arg(short, long, default_value = "stdout")]
        format: String,
    },
    /// Check if a coin name is a valid ID and show suggestions
    CheckCoin {
        /// Coin ID/symbol to check
        #[arg(name = "COIN")]
        coin: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Graceful Ctrl+C handling
    tokio::spawn(async {
        tokio::signal::ctrl_c().await.ok();
        println!("\nOperation cancelled by user.");
        std::process::exit(0);
    });

    let args = Cli::parse();

    match args.command {
        Some(Commands::RunPipeline {
            coin,
            currency,
            days,
            prep_ml,
            light,
            drop_weekends,
            seed,
            concurrency,
            annualization_factor,
        }) => {
            run_pipeline_flow(
                &coin,
                &currency,
                days,
                prep_ml,
                light,
                drop_weekends,
                &args.db_path,
                &args.output_prefix,
                seed,
                args.cache_ttl,
                concurrency,
                annualization_factor,
            )
            .await?;
        }
        Some(Commands::Ping) => {
            let cache = std::sync::Arc::new(cache::Cache::new(&args.db_path).await?);
            let cg_client =
                api::coingecko::CoinGeckoClient::new(cache.clone())?.with_ttl(args.cache_ttl);
            let yahoo_client =
                api::yahoo::YahooClient::new(cache.clone())?.with_ttl(args.cache_ttl);

            println!("Pinging CoinGecko API...");
            match cg_client.ping().await {
                Ok(val) => println!("CoinGecko Connection: OK, response: {:?}", val),
                Err(e) => println!("CoinGecko Connection Failed: {}", e),
            }

            println!("Pinging Yahoo Finance API...");
            match yahoo_client.ping().await {
                Ok(_) => println!("Yahoo Finance Connection: OK"),
                Err(e) => println!("Yahoo Finance Connection Failed: {}", e),
            }
        }
        Some(Commands::ListCoins) => {
            let cache = std::sync::Arc::new(cache::Cache::new(&args.db_path).await?);
            let cg_client =
                api::coingecko::CoinGeckoClient::new(cache.clone())?.with_ttl(args.cache_ttl);
            println!("Fetching top 50 coins by market cap (USD)...");
            let coins = cg_client.get_coins_markets("usd", 50).await?;
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
                let market_cap = c.get("market_cap").and_then(|v| v.as_f64()).unwrap_or(0.0);
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
        Some(Commands::Trending) => {
            let cache = std::sync::Arc::new(cache::Cache::new(&args.db_path).await?);
            let cg_client =
                api::coingecko::CoinGeckoClient::new(cache.clone())?.with_ttl(args.cache_ttl);
            println!("Fetching trending coins...");
            let val = cg_client.get_search_trending().await?;
            if let Some(coins) = val.get("coins").and_then(|v| v.as_array()) {
                println!(
                    "{:<5} | {:<30} | {:<10} | {:<30}",
                    "Rank", "ID", "Symbol", "Name"
                );
                println!("{}", "-".repeat(81));
                for (i, c) in coins.iter().enumerate() {
                    if let Some(item) = c.get("item") {
                        let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let symbol = item.get("symbol").and_then(|v| v.as_str()).unwrap_or("");
                        println!("{:<5} | {:<30} | {:<10} | {:<30}", i + 1, id, symbol, name);
                    }
                }
            } else {
                println!("No trending coins found.");
            }
        }
        Some(Commands::Ohlcv {
            coin,
            currency,
            days,
            format,
        }) => {
            let cache = std::sync::Arc::new(cache::Cache::new(&args.db_path).await?);
            let cg_client =
                api::coingecko::CoinGeckoClient::new(cache.clone())?.with_ttl(args.cache_ttl);
            run_ohlcv_flow(
                &cg_client,
                &coin,
                &currency,
                days,
                &format,
                &args.output_prefix,
            )
            .await?;
        }
        Some(Commands::CheckCoin { coin }) => {
            let cache = std::sync::Arc::new(cache::Cache::new(&args.db_path).await?);
            let cg_client =
                api::coingecko::CoinGeckoClient::new(cache.clone())?.with_ttl(args.cache_ttl);
            println!("Checking CoinGecko ID for '{}'...", coin);
            match cg_client.check_coin_id(&coin).await {
                Ok(None) => {
                    println!("Success: '{}' is a valid CoinGecko ID.", coin);
                }
                Ok(Some(suggestions)) => {
                    println!("Error: '{}' is not a valid CoinGecko ID.", coin);
                    if suggestions.is_empty() {
                        println!("No suggestions found.");
                    } else {
                        println!("\nSuggested IDs:");
                        for sug in suggestions {
                            println!(
                                "  - {} (symbol: {}, name: {})",
                                sug.id, sug.symbol, sug.name
                            );
                        }
                    }
                }
                Err(e) => {
                    println!("Error: Failed to query CoinGecko coins list: {}", e);
                }
            }
        }
        None => {
            run_interactive_menu(&args.db_path, &args.output_prefix, args.cache_ttl).await?;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn run_pipeline_flow(
    coin: &str,
    currency: &str,
    mut days: u32,
    prep_ml: bool,
    light: bool,
    drop_weekends: bool,
    db_path: &str,
    output_prefix: &str,
    seed: Option<u64>,
    cache_ttl: i64,
    concurrency: usize,
    annualization_factor: Option<f64>,
) -> Result<()> {
    // Lightweight override
    if light {
        days = 30;
    }

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let run_dir = format!("cdg_files/run_{}", timestamp);
    std::fs::create_dir_all(&run_dir)?;
    let ohlcv_dir = format!("cdg_files/can_{}", timestamp);
    std::fs::create_dir_all(&ohlcv_dir)?;

    println!("Starting CDG Data Collector...");
    println!("Run Directory: {}", run_dir);
    println!("OHLCV Directory: {}", ohlcv_dir);
    println!("Target Coin: {}", coin);
    println!("Currency: {}", currency);
    println!("Days: {}", days);
    println!("ML Prep: {}", prep_ml);
    println!("Lightweight Mode: {}", light);
    println!("Drop Weekends: {}", drop_weekends);
    println!("DB Path: {}", db_path);
    println!("Output Prefix: {}", output_prefix);
    println!("Concurrency: {}", concurrency);
    println!(
        "Annualization Factor Override: {}",
        annualization_factor.map_or("None".to_string(), |f| f.to_string())
    );
    println!(
        "Seed: {}",
        seed.map_or("default (1337)".to_string(), |s| s.to_string())
    );

    // 1. Initialize Cache
    println!("Initializing SQLite Cache...");
    let cache = std::sync::Arc::new(cache::Cache::new(db_path).await?);

    // 2. Initialize Clients
    let cg_client = api::coingecko::CoinGeckoClient::new(cache.clone())?.with_ttl(cache_ttl);
    let yahoo_client = api::yahoo::YahooClient::new(cache.clone())?.with_ttl(cache_ttl);

    // 3. Ping CoinGecko
    match cg_client.ping().await {
        Ok(_) => println!("CoinGecko API Connection: OK"),
        Err(e) => println!("Warning: CoinGecko API Connection Failed: {}", e),
    }

    // 4. Calculate Timestamps (aligned to start of day for caching)
    let now = chrono::Utc::now().timestamp();
    let rounded_now = (now / 86400) * 86400;
    let days_num: i64 = days as i64;
    let from_timestamp = rounded_now - (days_num * 24 * 60 * 60);
    let to_timestamp = rounded_now;

    // 5. Parse coins and currencies
    let raw_coins: Vec<&str> = coin
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if raw_coins.is_empty() {
        return Err(anyhow::anyhow!("No coins specified"));
    }

    let currencies: Vec<&str> = currency
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if currencies.is_empty() {
        return Err(anyhow::anyhow!("No currencies specified"));
    }

    // Fetch and display current orderbook metrics for coins
    println!("\nFetching exchange tickers & orderbook metrics...");
    let mut coins = Vec::new();
    for c in raw_coins {
        match cg_client.get_coin_tickers(c, Some(1)).await {
            Ok(tickers_val) => {
                let tickers_json_str = serde_json::to_string(&tickers_val)?;
                match analysis::parse_coingecko_tickers(&tickers_json_str) {
                    Ok(tickers_df) => {
                        if let Ok(metrics_df) = analysis::calculate_orderbook_metrics(&tickers_df) {
                            let avg_spread = metrics_df
                                .column("average_spread")?
                                .f64()?
                                .get(0)
                                .unwrap_or(0.0);
                            let total_vol = metrics_df
                                .column("total_volume")?
                                .f64()?
                                .get(0)
                                .unwrap_or(0.0);
                            let std_dev = metrics_df
                                .column("price_std_dev")?
                                .f64()?
                                .get(0)
                                .unwrap_or(0.0);
                            println!("--------------------------------------------------");
                            println!("Orderbook Metrics for {}:", c.to_uppercase());
                            println!("  Average Bid-Ask Spread : {:.4}%", avg_spread * 100.0);
                            println!("  Total Ticker Volume     : {:.2}", total_vol);
                            println!("  Price Std Dev (Exchanges): {:.2}", std_dev);
                        }
                    }
                    Err(e) => println!("Warning: Failed to parse tickers for {}: {}", c, e),
                }
                coins.push(c.to_string());
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("404") {
                    println!(
                        "Error: Coin '{}' is not a valid CoinGecko ID. Use '--check-coin {}' to search for suggested IDs. Skipping.",
                        c, c
                    );
                } else {
                    println!(
                        "Warning: Failed to fetch tickers for {}: {}. Keeping in pipeline.",
                        c, e
                    );
                    coins.push(c.to_string());
                }
            }
        }
    }
    println!("--------------------------------------------------\n");

    if coins.is_empty() {
        return Err(anyhow::anyhow!("No valid coins found to process"));
    }

    // Initialize progress bar for historical data fetching
    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_style(
        indicatif::ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let mut tasks = Vec::new();
    for c in &coins {
        for &curr in &currencies {
            tasks.push((c.clone(), curr.to_string()));
        }
    }

    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(concurrency));
    let mut join_set = tokio::task::JoinSet::new();
    let cg_client_arc = std::sync::Arc::new(cg_client);

    for (c, curr) in tasks {
        let sem = semaphore.clone();
        let client = cg_client_arc.clone();
        let ohlcv_dir_clone = ohlcv_dir.clone();
        let days_str = days.to_string();

        join_set.spawn(async move {
            let _permit = sem.acquire().await?;

            let cg_val = client
                .get_coin_market_chart_range(&c, &curr, from_timestamp, to_timestamp)
                .await?;
            let cg_json_str = serde_json::to_string(&cg_val)?;

            let price_col_name = format!("{}_{}", c, curr);
            let df_market = analysis::parse_coingecko_market_chart(&cg_json_str, &price_col_name)?;

            let ohlc_val = client.get_coin_ohlc(&c, &curr, &days_str).await?;

            let ohlc_json_pretty = serde_json::to_string_pretty(&ohlc_val)?;
            let json_file_path = format!("{}/{}_{}.json", ohlcv_dir_clone, c, curr);
            std::fs::write(&json_file_path, &ohlc_json_pretty)?;

            let csv_file_path = format!("{}/{}_{}.csv", ohlcv_dir_clone, c, curr);
            let mut wtr_ohlcv = std::fs::File::create(&csv_file_path)?;
            writeln!(wtr_ohlcv, "timestamp,open,high,low,close")?;
            for row in &ohlc_val {
                if row.len() >= 5 {
                    writeln!(
                        wtr_ohlcv,
                        "{},{},{},{},{}",
                        row[0], row[1], row[2], row[3], row[4]
                    )?;
                }
            }

            let ohlc_json_str = serde_json::to_string(&ohlc_val)?;
            let df_ohlc = analysis::parse_coingecko_ohlc(&ohlc_json_str, &price_col_name)?;

            let df = analysis::align_datasets(&df_market, &[df_ohlc], false)?;
            Ok::<(polars::prelude::DataFrame, String), anyhow::Error>((df, price_col_name))
        });
    }

    let mut currency_dfs = Vec::new();
    let mut currency_cols = Vec::new();

    while let Some(res) = join_set.join_next().await {
        match res {
            Ok(Ok((df, col_name))) => {
                pb.set_message(format!("Loaded {} rows for {}", df.height(), &col_name));
                currency_dfs.push(df);
                currency_cols.push(col_name);
            }
            Ok(Err(e)) => {
                return Err(e);
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Join error: {}", e));
            }
        }
    }

    // Merge all currency DataFrames
    let mut main_df = currency_dfs[0].clone();
    if currency_dfs.len() > 1 {
        main_df = analysis::align_datasets(&main_df, &currency_dfs[1..], false)?;
    }

    // 6. Fetch Yahoo Benchmarks (if not in lightweight mode)
    let mut other_dfs = Vec::new();
    let mut assets_to_plot = currency_cols.clone();

    if !light {
        let bench_tickers = vec!["^GSPC", "^DJI", "^IXIC", "^HSI", "^BVSP"];
        for ticker in bench_tickers {
            pb.set_message(format!("Fetching Yahoo Finance data for {}...", ticker));
            match yahoo_client
                .fetch_ticker_chart(ticker, from_timestamp, to_timestamp)
                .await
            {
                Ok(json_data) => match analysis::parse_yahoo_json(&json_data, ticker) {
                    Ok(df) => {
                        pb.set_message(format!("Loaded {} rows for {}", df.height(), ticker));
                        other_dfs.push(df);
                        assets_to_plot.push(ticker.to_string());
                    }
                    Err(e) => pb.println(format!("Error parsing JSON for {}: {}", ticker, e)),
                },
                Err(e) => pb.println(format!(
                    "Error fetching Yahoo Finance data for {}: {}",
                    ticker, e
                )),
            }
        }
    }

    // 7. Align Datasets
    pb.set_message("Aligning data...");
    let aligned_df = analysis::align_datasets(&main_df, &other_dfs, drop_weekends)?;
    pb.set_message(format!("Aligned DataFrame shape: {:?}", aligned_df.shape()));

    // 8. Compute indicators on target coins conditionally
    pb.set_message("Computing technical indicators and returns...");
    let mut final_df = aligned_df;
    if light {
        final_df = analysis::compute_returns_and_indicators(&final_df, &currency_cols[0])?;
    } else {
        for col in &currency_cols {
            final_df = analysis::compute_returns_and_indicators(&final_df, col)?;
        }
    }

    // 9. Prep ML features if flagged
    if prep_ml {
        pb.set_message("Applying MinMax and Standard scaling for ML prep...");
        final_df = analysis::prep_ml(&final_df)?;
    }

    // 10. Export datasets
    let csv_path = format!("{}/data.csv", run_dir);
    let parquet_path = format!("{}/data.parquet", run_dir);

    pb.set_message(format!("Saving CSV to: {}", csv_path));
    export::export_csv(&mut final_df, &csv_path)?;

    pb.set_message(format!("Saving Parquet to: {}", parquet_path));
    export::export_parquet(&mut final_df, &parquet_path)?;

    // 11. Plotting
    if !light {
        for col in &currency_cols {
            let returns_cols = [
                format!("{}_simple_return", col),
                format!("{}_log_return", col),
            ];
            let returns_cols_refs: Vec<&str> = returns_cols.iter().map(|s| s.as_str()).collect();
            let returns_plot_path = format!("{}/{}_returns.png", run_dir, col);
            pb.set_message(format!(
                "Saving returns plot for {} to: {}",
                col, returns_plot_path
            ));
            if let Err(e) = plot::plot_line_chart(
                &final_df,
                &returns_cols_refs,
                &format!("{} Returns (%)", col),
                &returns_plot_path,
            ) {
                pb.println(format!(
                    "Warning: Failed to generate returns plot for {}: {}",
                    col, e
                ));
            }
        }

        let perf_plot_path = format!("{}/performance.png", run_dir);
        pb.set_message(format!("Saving performance plot to: {}", perf_plot_path));
        if let Err(e) = plot::plot_performance(&final_df, &assets_to_plot, &perf_plot_path) {
            pb.println(format!(
                "Warning: Failed to generate performance plot: {}",
                e
            ));
        }

        let rr_plot_path = format!("{}/risk_return.png", run_dir);
        pb.set_message(format!("Saving risk/return plot to: {}", rr_plot_path));
        if let Err(e) = plot::plot_risk_return(&final_df, &assets_to_plot, &rr_plot_path) {
            pb.println(format!(
                "Warning: Failed to generate risk/return plot: {}",
                e
            ));
        }
    } else {
        pb.println("Lightweight mode enabled: skipping plot generation.");
    }

    pb.finish_with_message("Data fetching and processing complete.");

    // 12. Portfolio Optimization (Markowitz Monte Carlo)
    if assets_to_plot.len() >= 2 {
        println!("\n==================================================");
        println!("RUNNING PORTFOLIO OPTIMIZATION (Markowitz Monte Carlo)");
        println!("==================================================");

        // Compute annualization factors dynamically
        let factors: Vec<f64> = assets_to_plot.iter().map(|asset| {
            if let Some(val) = annualization_factor {
                val
            } else if currency_cols.contains(asset) {
                365.0
            } else if asset.to_uppercase().ends_with("-USD") || asset.to_uppercase().ends_with("-EUR") {
                365.0
            } else {
                252.0
            }
        }).collect();

        match cdg::optimization::run_monte_carlo(&final_df, &assets_to_plot, &factors, 10000, seed) {
            Ok(opt_res) => {
                println!("\nOptimal Portfolio Formulations (Annualized):");
                let metrics_table = cdg::optimization::format_portfolio_metrics_table(&opt_res);
                println!("{}", metrics_table);

                println!("\nOptimal Asset Weights:");
                let weights_table =
                    cdg::optimization::format_optimal_weights_table(&assets_to_plot, &opt_res);
                println!("{}", weights_table);

                // Save weights CSV
                let weights_path = format!("{}/portfolio_weights.csv", run_dir);
                let mut w_file = std::fs::File::create(&weights_path)?;
                writeln!(w_file, "asset,max_sharpe_weight,min_vol_weight")?;
                for (i, asset) in assets_to_plot.iter().enumerate() {
                    writeln!(
                        w_file,
                        "{},{:.6},{:.6}",
                        asset, opt_res.max_sharpe.weights[i], opt_res.min_volatility.weights[i]
                    )?;
                }
                println!("Portfolio weights saved to: {}", weights_path);

                // Plot efficient frontier
                let ef_plot_path = format!("{}/efficient_frontier.png", run_dir);
                println!("Saving efficient frontier plot to: {}", ef_plot_path);
                if let Err(e) = plot::plot_efficient_frontier(
                    &opt_res.simulated_points,
                    &opt_res.max_sharpe,
                    &opt_res.min_volatility,
                    &ef_plot_path,
                ) {
                    println!("Warning: Failed to generate efficient frontier plot: {}", e);
                }
            }
            Err(e) => {
                println!("Error running portfolio optimization: {}", e);
            }
        }
    } else {
        println!("\nSkipping portfolio optimization (requires at least 2 assets).");
    }

    println!("CDG data pipeline completed successfully!");
    Ok(())
}

async fn run_ohlcv_flow(
    cg_client: &api::coingecko::CoinGeckoClient,
    coin: &str,
    currency: &str,
    days: u32,
    format: &str,
    output_prefix: &str,
) -> Result<()> {
    println!(
        "Retrieving OHLC data for {} in {} ({} days)...",
        coin, currency, days
    );
    let ohlc_data = cg_client
        .get_coin_ohlc(coin, currency, &days.to_string())
        .await?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let ohlcv_dir = format!("cdg_files/can_{}", timestamp);
    std::fs::create_dir_all(&ohlcv_dir)?;

    // Save raw OHLCV JSON and CSV files inside ohlcv_dir
    let ohlc_json_pretty = serde_json::to_string_pretty(&ohlc_data)?;
    let json_file_path = format!("{}/{}_{}.json", ohlcv_dir, coin, currency);
    std::fs::write(&json_file_path, &ohlc_json_pretty)?;

    let csv_file_path = format!("{}/{}_{}.csv", ohlcv_dir, coin, currency);
    let mut wtr_ohlcv = std::fs::File::create(&csv_file_path)?;
    writeln!(wtr_ohlcv, "timestamp,open,high,low,close")?;
    for row in &ohlc_data {
        if row.len() >= 5 {
            writeln!(
                wtr_ohlcv,
                "{},{},{},{},{}",
                row[0], row[1], row[2], row[3], row[4]
            )?;
        }
    }
    println!("Raw OHLCV files saved to: {}", ohlcv_dir);

    match format.to_lowercase().as_str() {
        "json" => {
            let json_str = serde_json::to_string_pretty(&ohlc_data)?;
            let file_path = format!("{}_{}_{}_ohlc.json", output_prefix, coin, currency);
            if let Some(parent) = std::path::Path::new(&file_path).parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&file_path, &json_str)?;
            println!("OHLC data exported to JSON file: {}", file_path);
        }
        "csv" => {
            let file_path = format!("{}_{}_{}_ohlc.csv", output_prefix, coin, currency);
            if let Some(parent) = std::path::Path::new(&file_path).parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut wtr = std::fs::File::create(&file_path)?;
            writeln!(wtr, "timestamp,open,high,low,close")?;
            for row in &ohlc_data {
                if row.len() >= 5 {
                    writeln!(
                        wtr,
                        "{},{},{},{},{}",
                        row[0], row[1], row[2], row[3], row[4]
                    )?;
                }
            }
            println!("OHLC data exported to CSV file: {}", file_path);
        }
        _ => {
            // stdout
            println!(
                "{:<20} | {:<10} | {:<10} | {:<10} | {:<10}",
                "Timestamp", "Open", "High", "Low", "Close"
            );
            println!("{}", "-".repeat(68));
            for row in ohlc_data.iter().take(50) {
                if row.len() >= 5 {
                    let ts_ms = row[0] as i64;
                    let date_str = chrono::DateTime::from_timestamp(ts_ms / 1000, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| ts_ms.to_string());
                    println!(
                        "{:<20} | {:<10.2} | {:<10.2} | {:<10.2} | {:<10.2}",
                        date_str, row[1], row[2], row[3], row[4]
                    );
                }
            }
            if ohlc_data.len() > 50 {
                println!("... and {} more rows.", ohlc_data.len() - 50);
            }
        }
    }
    Ok(())
}

fn clear_terminal() {
    print!("\x1B[2J\x1B[1;1H");
    let _ = std::io::Write::flush(&mut std::io::stdout());
}

fn wait_for_back() {
    println!();
    let options = &["[Back]"];
    let _ = dialoguer::Select::new()
        .with_prompt("Press enter/select option to go back")
        .default(0)
        .items(options)
        .interact_opt();
}

async fn run_interactive_menu(
    db_path: &str,
    output_prefix: &str,
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
            "Configure Cache TTL",
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

                println!("\nRunning pipeline...\n");
                if let Err(e) = run_pipeline_flow(
                    &coin,
                    &currency,
                    days,
                    prep_ml,
                    light,
                    drop_weekends,
                    db_path,
                    output_prefix,
                    seed,
                    cache_ttl,
                    3,
                    None,
                )
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

                if let Err(e) =
                    run_ohlcv_flow(&cg_client, &coin, &currency, days, &format, output_prefix).await
                {
                    println!("Error fetching/exporting OHLCV data: {}", e);
                }
            }
            "Check Coin ID" => {
                let coin_to_check: String = dialoguer::Input::new()
                    .with_prompt("Enter Coin ID or symbol to check")
                    .interact_text()?;

                println!("Checking CoinGecko ID for '{}'...", coin_to_check);
                match cg_client.check_coin_id(&coin_to_check).await {
                    Ok(None) => {
                        println!("Success: '{}' is a valid CoinGecko ID.", coin_to_check);
                    }
                    Ok(Some(suggestions)) => {
                        println!("Error: '{}' is not a valid CoinGecko ID.", coin_to_check);
                        if suggestions.is_empty() {
                            println!("No suggestions found.");
                        } else {
                            println!("\nSuggested IDs:");
                            for sug in suggestions {
                                println!(
                                    "  - {} (symbol: {}, name: {})",
                                    sug.id, sug.symbol, sug.name
                                );
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error: Failed to query CoinGecko coins list: {}", e);
                    }
                }
            }
            "Configure Cache TTL" => {
                let new_ttl: i64 = dialoguer::Input::new()
                    .with_prompt("Enter new Cache TTL in seconds")
                    .default(cache_ttl)
                    .interact_text()?;
                cache_ttl = new_ttl;
                cg_client = cg_client.with_ttl(cache_ttl);
                yahoo_client = yahoo_client.with_ttl(cache_ttl);
                println!("Cache TTL set to {} seconds.", cache_ttl);
            }
            "Exit" => {
                println!("Goodbye!");
                break;
            }
            _ => unreachable!(),
        }

        if choice != "Exit" {
            wait_for_back();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_parsing_ping() {
        let args = Cli::try_parse_from(&["cdg", "ping"]).unwrap();
        assert_eq!(args.command, Some(Commands::Ping));
    }

    #[test]
    fn test_cli_parsing_run_pipeline() {
        let args = Cli::try_parse_from(&[
            "cdg",
            "run-pipeline",
            "-c",
            "bitcoin,ethereum",
            "-v",
            "usd",
            "--days",
            "45",
        ])
        .unwrap();
        assert!(matches!(
            args.command,
            Some(Commands::RunPipeline {
                ref coin,
                ref currency,
                days: 45,
                ..
            }) if coin == "bitcoin,ethereum" && currency == "usd"
        ));
    }

    #[test]
    fn test_cli_parsing_cache_ttl() {
        let args = Cli::try_parse_from(&["cdg", "--cache-ttl", "600", "ping"]).unwrap();
        assert_eq!(args.cache_ttl, 600);
        assert_eq!(args.command, Some(Commands::Ping));
    }
}
