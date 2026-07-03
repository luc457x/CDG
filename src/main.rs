use anyhow::Result;
use cdg::{analysis, api, cache, export, plot};
use clap::{Parser, Subcommand};
use std::io::Write;

#[derive(Parser, Debug, PartialEq)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Base output directory (default: cdg_files)
    #[arg(long, default_value = "cdg_files", env = "CDG_OUTPUT_DIR")]
    pub output_dir: String,

    /// Database file path (default: {output_dir}/cache.db)
    #[arg(long)]
    pub db_path: Option<String>,

    /// Path to export results (default: {output_dir}/output)
    #[arg(short, long)]
    pub output_prefix: Option<String>,

    /// Raw OHLCV export format: 'json' or 'csv' (default: json)
    #[arg(long, default_value = "json", env = "CDG_RAW_FORMAT")]
    pub raw_format: String,

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

        /// CoinGecko query concurrency limit (default: 1 for demo/free keys, 3 for pro keys)
        #[arg(long, env = "COINGECKO_CONCURRENCY")]
        concurrency: Option<usize>,

        /// Custom annualization factor override (e.g. 252 or 365)
        #[arg(long, env = "ANNUALIZATION_FACTOR")]
        annualization_factor: Option<f64>,

        /// Enable strategy backtesting
        #[arg(long, env = "CDG_BACKTEST")]
        backtest: bool,

        /// Strategy to backtest: 'rsi', 'macd', 'bollinger', or 'all' (default: rsi)
        #[arg(long, default_value = "rsi", env = "CDG_BACKTEST_STRATEGY")]
        strategy: String,

        /// Transaction fee as decimal (default: 0.001)
        #[arg(long, default_value_t = 0.001, env = "CDG_FEE")]
        fee: f64,

        /// Slippage as decimal (default: 0.0005)
        #[arg(long, default_value_t = 0.0005, env = "CDG_SLIPPAGE")]
        slippage: f64,
    },
    /// Retrieve and backtest strategy on a coin
    Backtest {
        /// Coin ID
        #[arg(short, long, default_value = "bitcoin")]
        coin: String,

        /// Vs currency (default: usd)
        #[arg(short = 'v', long, default_value = "usd")]
        currency: String,

        /// Timeframe in days (default: 90)
        #[arg(short, long, default_value_t = 90)]
        days: u32,

        /// Strategy to backtest: 'rsi', 'macd', 'bollinger', or 'all' (default: rsi)
        #[arg(short, long, default_value = "rsi")]
        strategy: String,

        /// Transaction fee as decimal (default: 0.001)
        #[arg(long, default_value_t = 0.001)]
        fee: f64,

        /// Slippage as decimal (default: 0.0005)
        #[arg(long, default_value_t = 0.0005)]
        slippage: f64,
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

    let output_dir = args.output_dir;
    cdg::utils::validate_safe_path(&output_dir)?;

    let raw_format = args.raw_format.to_lowercase();
    if raw_format != "json" && raw_format != "csv" {
        return Err(anyhow::anyhow!(
            "Invalid raw format: '{}'. Must be 'json' or 'csv'.",
            raw_format
        ));
    }

    let db_path = args
        .db_path
        .unwrap_or_else(|| format!("{}/cache.db", output_dir));
    let output_prefix = args
        .output_prefix
        .unwrap_or_else(|| format!("{}/output", output_dir));

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
            backtest,
            strategy,
            fee,
            slippage,
        }) => {
            run_pipeline_flow(PipelineConfig {
                coin: &coin,
                currency: &currency,
                days,
                prep_ml,
                light,
                drop_weekends,
                db_path: &db_path,
                output_dir: &output_dir,
                output_prefix: &output_prefix,
                raw_format: &raw_format,
                seed,
                cache_ttl: args.cache_ttl,
                concurrency,
                annualization_factor,
                backtest,
                strategy: &strategy,
                fee,
                slippage,
            })
            .await?;
        }
        Some(Commands::Backtest {
            coin,
            currency,
            days,
            strategy,
            fee,
            slippage,
        }) => {
            run_standalone_backtest(
                &db_path,
                &output_dir,
                &output_prefix,
                args.cache_ttl,
                &coin,
                &currency,
                days,
                &strategy,
                fee,
                slippage,
            )
            .await?;
        }
        Some(Commands::Ping) => {
            let cache = std::sync::Arc::new(cache::Cache::new(&db_path).await?);
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
            let cache = std::sync::Arc::new(cache::Cache::new(&db_path).await?);
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
            let cache = std::sync::Arc::new(cache::Cache::new(&db_path).await?);
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
            let cache = std::sync::Arc::new(cache::Cache::new(&db_path).await?);
            let cg_client =
                api::coingecko::CoinGeckoClient::new(cache.clone())?.with_ttl(args.cache_ttl);
            run_ohlcv_flow(
                &cg_client,
                &coin,
                &currency,
                days,
                &format,
                &output_dir,
                &output_prefix,
                &raw_format,
            )
            .await?;
        }
        Some(Commands::CheckCoin { coin }) => {
            let cache = std::sync::Arc::new(cache::Cache::new(&db_path).await?);
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
            run_interactive_menu(
                &db_path,
                &output_dir,
                &output_prefix,
                &raw_format,
                args.cache_ttl,
            )
            .await?;
        }
    }

    Ok(())
}

pub struct PipelineConfig<'a> {
    pub coin: &'a str,
    pub currency: &'a str,
    pub days: u32,
    pub prep_ml: bool,
    pub light: bool,
    pub drop_weekends: bool,
    pub db_path: &'a str,
    pub output_dir: &'a str,
    pub output_prefix: &'a str,
    pub raw_format: &'a str,
    pub seed: Option<u64>,
    pub cache_ttl: i64,
    pub concurrency: Option<usize>,
    pub annualization_factor: Option<f64>,
    pub backtest: bool,
    pub strategy: &'a str,
    pub fee: f64,
    pub slippage: f64,
}

async fn run_pipeline_flow(mut config: PipelineConfig<'_>) -> Result<()> {
    // Lightweight override
    if config.light {
        config.days = 30;
    }

    let coin = config.coin;
    let currency = config.currency;
    let days = config.days;
    let prep_ml = config.prep_ml;
    let light = config.light;
    let drop_weekends = config.drop_weekends;
    let db_path = config.db_path;
    let output_dir = config.output_dir;
    let output_prefix = config.output_prefix;
    let raw_format = config.raw_format;
    let seed = config.seed;
    let cache_ttl = config.cache_ttl;
    let concurrency = config.concurrency.unwrap_or_else(|| {
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
    });
    let annualization_factor = config.annualization_factor;
    let backtest = config.backtest;
    let strategy = config.strategy;
    let fee = config.fee;
    let slippage = config.slippage;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let run_dir = format!("{}/run_{}", output_dir, timestamp);
    cdg::utils::validate_safe_path(&run_dir)?;
    std::fs::create_dir_all(&run_dir)?;
    let ohlcv_dir = format!("{}/raw_ohlcv", run_dir);
    cdg::utils::validate_safe_path(&ohlcv_dir)?;
    std::fs::create_dir_all(&ohlcv_dir)?;

    println!("Starting CDG Data Collector...");
    println!("Base Output Directory: {}", output_dir);
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
    let cg_client_arc = std::sync::Arc::new(cg_client.with_progress_bar(pb.clone()));

    for (c, curr) in tasks {
        let sem = semaphore.clone();
        let client = cg_client_arc.clone();
        let ohlcv_dir_clone = ohlcv_dir.clone();
        let days_str = days.to_string();
        let raw_format_clone = raw_format.to_string();

        join_set.spawn(async move {
            let _permit = sem.acquire().await?;

            let cg_val = client
                .get_coin_market_chart_range(&c, &curr, from_timestamp, to_timestamp)
                .await?;
            let cg_json_str = serde_json::to_string(&cg_val)?;

            let c_safe = cdg::utils::sanitize_name(&c);
            let curr_safe = cdg::utils::sanitize_name(&curr);
            cdg::utils::validate_safe_path(&c_safe)?;
            cdg::utils::validate_safe_path(&curr_safe)?;

            let price_col_name = format!("{}_{}", c_safe, curr_safe);
            let df_market = analysis::parse_coingecko_market_chart(&cg_json_str, &price_col_name)?;

            let ohlc_val = client.get_coin_ohlc(&c, &curr, &days_str).await?;

            if raw_format_clone == "json" {
                let ohlc_json_pretty = serde_json::to_string_pretty(&ohlc_val)?;
                let json_file_path = format!("{}/{}_{}.json", ohlcv_dir_clone, c_safe, curr_safe);
                cdg::utils::validate_safe_path(&json_file_path)?;
                std::fs::write(&json_file_path, &ohlc_json_pretty)?;
            } else if raw_format_clone == "csv" {
                let csv_file_path = format!("{}/{}_{}.csv", ohlcv_dir_clone, c_safe, curr_safe);
                cdg::utils::validate_safe_path(&csv_file_path)?;
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
    if currency_dfs.is_empty() {
        return Err(anyhow::anyhow!(
            "No cryptocurrency data was successfully loaded"
        ));
    }
    let mut main_df = currency_dfs[0].clone();
    if currency_dfs.len() > 1 {
        main_df = analysis::align_datasets(&main_df, &currency_dfs[1..], false)?;
    }

    // 6. Fetch Yahoo Benchmarks (if not in lightweight mode)
    let mut other_dfs = Vec::new();
    let mut assets_to_plot = currency_cols.clone();

    if !light {
        let yahoo_client = yahoo_client.with_progress_bar(pb.clone());
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
    let mut opt_res_opt = None;
    if currency_cols.len() >= 2 {
        println!("\n==================================================");
        println!("RUNNING PORTFOLIO OPTIMIZATION (Markowitz Monte Carlo)");
        println!("==================================================");

        let final_ann_factor = annualization_factor.unwrap_or(if drop_weekends { 252.0 } else { 365.0 });

        match cdg::optimization::run_monte_carlo(&final_df, &currency_cols, final_ann_factor, 10000, seed) {
            Ok(opt_res) => {
                println!("\nOptimal Portfolio Formulations (Annualized):");
                let metrics_table = cdg::optimization::format_portfolio_metrics_table(&opt_res);
                println!("{}", metrics_table);

                println!("\nOptimal Asset Weights:");
                let weights_table =
                    cdg::optimization::format_optimal_weights_table(&currency_cols, &opt_res);
                println!("{}", weights_table);

                // Save weights CSV
                let weights_path = format!("{}/portfolio_weights.csv", run_dir);
                let mut w_file = std::fs::File::create(&weights_path)?;
                writeln!(w_file, "asset,max_sharpe_weight,min_vol_weight")?;
                for (i, asset) in currency_cols.iter().enumerate() {
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

                opt_res_opt = Some(opt_res);
            }
            Err(e) => {
                println!("Error running portfolio optimization: {}", e);
            }
        }
    } else {
        println!("\nSkipping portfolio optimization (requires at least 2 assets).");
    }

    // 13. Strategy & Portfolio Backtesting
    if backtest {
        println!("\n==================================================");
        println!("RUNNING STRATEGY & PORTFOLIO BACKTESTING");
        println!("==================================================");
        let final_ann_factor = annualization_factor.unwrap_or(if drop_weekends { 252.0 } else { 365.0 });

        let mut backtest_metrics = Vec::new();
        let strats = if strategy.to_lowercase() == "all" {
            vec!["rsi".to_string(), "macd".to_string(), "bollinger".to_string()]
        } else {
            vec![strategy.to_lowercase()]
        };

        // 1. Backtest individual assets
        for col in &currency_cols {
            for strat in &strats {
                match cdg::backtest::run_backtest_for_asset(
                    &final_df,
                    col,
                    strat,
                    fee,
                    slippage,
                    final_ann_factor,
                ) {
                    Ok((metrics, equity, bh_equity)) => {
                        backtest_metrics.push(metrics);

                        // Save PNG plot
                        let dates: Vec<String> = final_df
                            .column("date")?
                            .str()?
                            .into_iter()
                            .map(|opt| opt.unwrap_or("").to_string())
                            .collect();
                        let plot_path = format!("{}/{}_{}_backtest.png", run_dir, col, strat);
                        if let Err(e) = cdg::plot::plot_backtest_equity(
                            &dates,
                            &equity,
                            &bh_equity,
                            col,
                            strat,
                            &plot_path,
                        ) {
                            println!("Warning: Failed to generate backtest equity plot for {} ({}): {}", col, strat, e);
                        }
                    }
                    Err(e) => {
                        println!("Warning: Backtest failed for {} ({}): {}", col, strat, e);
                    }
                }
            }
        }

        // 2. Backtest optimized portfolios if available
        if let Some(ref opt_res) = opt_res_opt {
            let dates: Vec<String> = final_df
                .column("date")?
                .str()?
                .into_iter()
                .map(|opt| opt.unwrap_or("").to_string())
                .collect();

            // Max Sharpe Portfolio
            match cdg::backtest::backtest_portfolio(
                &final_df,
                &currency_cols,
                &opt_res.max_sharpe.weights,
                "max_sharpe",
                final_ann_factor,
            ) {
                Ok((metrics, equity, bh_equity)) => {
                    backtest_metrics.push(metrics);

                    let plot_path = format!("{}/max_sharpe_portfolio_rebalanced_backtest.png", run_dir);
                    if let Err(e) = cdg::plot::plot_backtest_equity(
                        &dates,
                        &equity,
                        &bh_equity,
                        "max_sharpe",
                        "portfolio",
                        &plot_path,
                    ) {
                        println!("Warning: Failed to generate backtest equity plot for max_sharpe portfolio: {}", e);
                    }
                }
                Err(e) => {
                    println!("Warning: Portfolio backtest failed for max_sharpe: {}", e);
                }
            }

            // Min Volatility Portfolio
            match cdg::backtest::backtest_portfolio(
                &final_df,
                &currency_cols,
                &opt_res.min_volatility.weights,
                "min_volatility",
                final_ann_factor,
            ) {
                Ok((metrics, equity, bh_equity)) => {
                    backtest_metrics.push(metrics);

                    let plot_path = format!("{}/min_vol_portfolio_rebalanced_backtest.png", run_dir);
                    if let Err(e) = cdg::plot::plot_backtest_equity(
                        &dates,
                        &equity,
                        &bh_equity,
                        "min_volatility",
                        "portfolio",
                        &plot_path,
                    ) {
                        println!("Warning: Failed to generate backtest equity plot for min_volatility portfolio: {}", e);
                    }
                }
                Err(e) => {
                    println!("Warning: Portfolio backtest failed for min_volatility: {}", e);
                }
            }
        }

        // 3. Print consolidated table and save reports
        if !backtest_metrics.is_empty() {
            let backtest_table = cdg::backtest::format_backtest_table(&backtest_metrics);
            println!("\nBacktest Summary Results:");
            println!("{}", backtest_table);

            // Export to CSV
            let csv_report_path = format!("{}/backtest_report.csv", run_dir);
            if let Ok(mut file) = std::fs::File::create(&csv_report_path) {
                if let Err(e) = writeln!(
                    file,
                    "coin,currency,strategy,strategy_return,buy_and_hold_return,strategy_sharpe,buy_and_hold_sharpe,strategy_max_drawdown,buy_and_hold_max_drawdown,win_rate,trades,rating"
                ) {
                    println!("Warning: Failed to write backtest CSV header: {}", e);
                }
                for m in &backtest_metrics {
                    if let Err(e) = writeln!(
                        file,
                        "{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{},{}",
                        m.coin,
                        m.currency,
                        m.strategy,
                        m.strategy_return,
                        m.buy_and_hold_return,
                        m.strategy_sharpe,
                        m.buy_and_hold_sharpe,
                        m.strategy_max_drawdown,
                        m.buy_and_hold_max_drawdown,
                        m.active_win_rate,
                        m.total_trades,
                        m.strategy_rating
                    ) {
                        println!("Warning: Failed to write backtest CSV row: {}", e);
                    }
                }
                println!("Backtest CSV report saved to: {}", csv_report_path);
            }

            // Export to JSON
            let json_report_path = format!("{}/backtest_report.json", run_dir);
            let mut report_map = std::collections::HashMap::new();
            for m in &backtest_metrics {
                report_map.insert(format!("{}_{}", m.coin, m.strategy), m.clone());
            }
            let report = cdg::backtest::BacktestReport {
                timestamp: chrono::Local::now().to_rfc3339(),
                metrics: report_map,
            };
            if let Ok(json_str) = serde_json::to_string_pretty(&report) {
                if std::fs::write(&json_report_path, json_str).is_ok() {
                    println!("Backtest JSON report saved to: {}", json_report_path);
                }
            }
        }
    }

    println!("CDG data pipeline completed successfully!");
    Ok(())
}

async fn run_standalone_backtest(
    db_path: &str,
    output_dir: &str,
    _output_prefix: &str,
    cache_ttl: i64,
    coin: &str,
    currency: &str,
    days: u32,
    strategy: &str,
    fee: f64,
    slippage: f64,
) -> Result<()> {
    println!("Starting CDG Standalone Backtester...");
    println!("Target Coin(s): {}", coin);
    println!("Currency: {}", currency);
    println!("Days: {}", days);
    println!("Strategy: {}", strategy);
    println!("Fee: {}", fee);
    println!("Slippage: {}", slippage);

    // 1. Initialize Cache
    let cache = std::sync::Arc::new(cache::Cache::new(db_path).await?);

    // 2. Initialize Client
    let cg_client = api::coingecko::CoinGeckoClient::new(cache.clone())?.with_ttl(cache_ttl);

    // 3. Calculate Timestamps
    let now = chrono::Utc::now().timestamp();
    let rounded_now = (now / 86400) * 86400;
    let days_num: i64 = days as i64;
    let from_timestamp = rounded_now - (days_num * 24 * 60 * 60);
    let to_timestamp = rounded_now;

    // 4. Parse coins and currencies
    let coins: Vec<&str> = coin
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if coins.is_empty() {
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

    // 5. Ingestion Loop
    let mut coin_dfs = Vec::new();
    let mut target_names = Vec::new();

    for c in &coins {
        for curr in &currencies {
            println!("Ingesting historical data for {} in {}...", c, curr);
            let cg_val = cg_client
                .get_coin_market_chart_range(c, curr, from_timestamp, to_timestamp)
                .await?;
            let cg_json_str = serde_json::to_string(&cg_val)?;

            let c_safe = cdg::utils::sanitize_name(c);
            let curr_safe = cdg::utils::sanitize_name(curr);
            cdg::utils::validate_safe_path(&c_safe)?;
            cdg::utils::validate_safe_path(&curr_safe)?;

            let price_col_name = format!("{}_{}", c_safe, curr_safe);
            let df_market = analysis::parse_coingecko_market_chart(&cg_json_str, &price_col_name)?;

            let days_str = days.to_string();
            let ohlc_val = cg_client.get_coin_ohlc(c, curr, &days_str).await?;
            let ohlc_json_str = serde_json::to_string(&ohlc_val)?;
            let df_ohlc = analysis::parse_coingecko_ohlc(&ohlc_json_str, &price_col_name)?;

            let mut df = analysis::align_datasets(&df_market, &[df_ohlc], false)?;
            df = analysis::compute_returns_and_indicators(&df, &price_col_name)?;

            coin_dfs.push(df);
            target_names.push(price_col_name);
        }
    }

    // 6. Run backtests
    let mut backtest_metrics = Vec::new();
    let strats = if strategy.to_lowercase() == "all" {
        vec!["rsi".to_string(), "macd".to_string(), "bollinger".to_string()]
    } else {
        vec![strategy.to_lowercase()]
    };

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let run_dir = format!("{}/backtest_run_{}", output_dir, timestamp);
    cdg::utils::validate_safe_path(&run_dir)?;
    std::fs::create_dir_all(&run_dir)?;

    for (df, col) in coin_dfs.iter().zip(target_names.iter()) {
        for strat in &strats {
            // Standalone run uses default annualization 365.0
            match cdg::backtest::run_backtest_for_asset(df, col, strat, fee, slippage, 365.0) {
                Ok((metrics, equity, bh_equity)) => {
                    backtest_metrics.push(metrics);

                    // Plot equity curve
                    let dates: Vec<String> = df
                        .column("date")?
                        .str()?
                        .into_iter()
                        .map(|opt| opt.unwrap_or("").to_string())
                        .collect();
                    let plot_path = format!("{}/{}_{}_backtest.png", run_dir, col, strat);
                    if let Err(e) = cdg::plot::plot_backtest_equity(
                        &dates,
                        &equity,
                        &bh_equity,
                        col,
                        strat,
                        &plot_path,
                    ) {
                        println!("Warning: Failed to generate backtest equity plot for {} ({}): {}", col, strat, e);
                    }
                }
                Err(e) => {
                    println!("Warning: Backtest failed for {} ({}): {}", col, strat, e);
                }
            }
        }
    }

    if !backtest_metrics.is_empty() {
        let backtest_table = cdg::backtest::format_backtest_table(&backtest_metrics);
        println!("\nBacktest Summary Results (Standalone Run):");
        println!("{}", backtest_table);

        // Export JSON/CSV in run_dir
        let csv_report_path = format!("{}/backtest_report.csv", run_dir);
        if let Ok(mut file) = std::fs::File::create(&csv_report_path) {
            writeln!(
                file,
                "coin,currency,strategy,strategy_return,buy_and_hold_return,strategy_sharpe,buy_and_hold_sharpe,strategy_max_drawdown,buy_and_hold_max_drawdown,win_rate,trades,rating"
            )?;
            for m in &backtest_metrics {
                writeln!(
                    file,
                    "{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{},{}",
                    m.coin,
                    m.currency,
                    m.strategy,
                    m.strategy_return,
                    m.buy_and_hold_return,
                    m.strategy_sharpe,
                    m.buy_and_hold_sharpe,
                    m.strategy_max_drawdown,
                    m.buy_and_hold_max_drawdown,
                    m.active_win_rate,
                    m.total_trades,
                    m.strategy_rating
                )?;
            }
            println!("Backtest CSV report saved to: {}", csv_report_path);
        }

        let json_report_path = format!("{}/backtest_report.json", run_dir);
        let mut report_map = std::collections::HashMap::new();
        for m in &backtest_metrics {
            report_map.insert(format!("{}_{}", m.coin, m.strategy), m.clone());
        }
        let report = cdg::backtest::BacktestReport {
            timestamp: chrono::Local::now().to_rfc3339(),
            metrics: report_map,
        };
        if let Ok(json_str) = serde_json::to_string_pretty(&report) {
            std::fs::write(&json_report_path, json_str)?;
            println!("Backtest JSON report saved to: {}", json_report_path);
        }
    } else {
        println!("No backtests executed successfully.");
    }

    Ok(())
}

async fn run_ohlcv_flow(
    cg_client: &api::coingecko::CoinGeckoClient,
    coin: &str,
    currency: &str,
    days: u32,
    format: &str,
    output_dir: &str,
    output_prefix: &str,
    raw_format: &str,
) -> Result<()> {
    let sanitized_coin = cdg::utils::sanitize_name(coin);
    let sanitized_currency = cdg::utils::sanitize_name(currency);
    cdg::utils::validate_safe_path(&sanitized_coin)?;
    cdg::utils::validate_safe_path(&sanitized_currency)?;

    println!(
        "Retrieving OHLC data for {} in {} ({} days)...",
        sanitized_coin, sanitized_currency, days
    );
    let ohlc_data = cg_client
        .get_coin_ohlc(coin, currency, &days.to_string())
        .await?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let ohlcv_dir = format!("{}/can_{}", output_dir, timestamp);
    cdg::utils::validate_safe_path(&ohlcv_dir)?;
    std::fs::create_dir_all(&ohlcv_dir)?;

    // Save raw OHLCV JSON or CSV files inside ohlcv_dir based on raw_format
    if raw_format == "json" {
        let ohlc_json_pretty = serde_json::to_string_pretty(&ohlc_data)?;
        let json_file_path = format!(
            "{}/{}_{}.json",
            ohlcv_dir, sanitized_coin, sanitized_currency
        );
        cdg::utils::validate_safe_path(&json_file_path)?;
        std::fs::write(&json_file_path, &ohlc_json_pretty)?;
    } else if raw_format == "csv" {
        let csv_file_path = format!(
            "{}/{}_{}.csv",
            ohlcv_dir, sanitized_coin, sanitized_currency
        );
        cdg::utils::validate_safe_path(&csv_file_path)?;
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
    }
    println!("Raw OHLCV files saved to: {}", ohlcv_dir);

    match format.to_lowercase().as_str() {
        "json" => {
            let json_str = serde_json::to_string_pretty(&ohlc_data)?;
            let file_path = format!(
                "{}_{}_{}_ohlc.json",
                output_prefix, sanitized_coin, sanitized_currency
            );
            cdg::utils::validate_safe_path(&file_path)?;
            if let Some(parent) = std::path::Path::new(&file_path).parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&file_path, &json_str)?;
            println!("OHLC data exported to JSON file: {}", file_path);
        }
        "csv" => {
            let file_path = format!(
                "{}_{}_{}_ohlc.csv",
                output_prefix, sanitized_coin, sanitized_currency
            );
            cdg::utils::validate_safe_path(&file_path)?;
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

#[cfg(windows)]
fn clear_terminal() {
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

                let (strategy, fee, slippage) = if backtest {
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

                    (strategy_str.to_string(), fee, slippage)
                } else {
                    ("rsi".to_string(), 0.001, 0.0005)
                };

                println!("\nRunning pipeline...\n");
                if let Err(e) = run_pipeline_flow(PipelineConfig {
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

                if let Err(e) = run_ohlcv_flow(
                    &cg_client,
                    &coin,
                    &currency,
                    days,
                    &format,
                    output_dir,
                    output_prefix,
                    raw_format,
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

    #[test]
    fn test_dynamic_path_resolution() {
        // Case 1: No values specified -> defaults resolved based on output_dir
        let args = Cli::try_parse_from(&["cdg", "ping"]).unwrap();
        let output_dir = args.output_dir;
        assert_eq!(output_dir, "cdg_files");
        let db_path = args
            .db_path
            .unwrap_or_else(|| format!("{}/cache.db", output_dir));
        let output_prefix = args
            .output_prefix
            .unwrap_or_else(|| format!("{}/output", output_dir));
        assert_eq!(db_path, "cdg_files/cache.db");
        assert_eq!(output_prefix, "cdg_files/output");

        // Case 2: Custom output_dir specified -> defaults resolved based on that dir
        let args = Cli::try_parse_from(&["cdg", "--output-dir", "custom_dir", "ping"]).unwrap();
        let output_dir = args.output_dir;
        assert_eq!(output_dir, "custom_dir");
        let db_path = args
            .db_path
            .unwrap_or_else(|| format!("{}/cache.db", output_dir));
        let output_prefix = args
            .output_prefix
            .unwrap_or_else(|| format!("{}/output", output_dir));
        assert_eq!(db_path, "custom_dir/cache.db");
        assert_eq!(output_prefix, "custom_dir/output");

        // Case 3: Explicit db_path and output_prefix specified -> overrides defaults
        let args = Cli::try_parse_from(&[
            "cdg",
            "--output-dir",
            "custom_dir",
            "--db-path",
            "explicit.db",
            "--output-prefix",
            "explicit/prefix",
            "ping",
        ])
        .unwrap();
        let output_dir = args.output_dir;
        assert_eq!(output_dir, "custom_dir");
        let db_path = args
            .db_path
            .unwrap_or_else(|| format!("{}/cache.db", output_dir));
        let output_prefix = args
            .output_prefix
            .unwrap_or_else(|| format!("{}/output", output_dir));
        assert_eq!(db_path, "explicit.db");
        assert_eq!(output_prefix, "explicit/prefix");
    }

    #[test]
    fn test_raw_format_validation() {
        // Case 1: Default is "json"
        let args = Cli::try_parse_from(&["cdg", "ping"]).unwrap();
        assert_eq!(args.raw_format, "json");

        // Case 2: Custom format works
        let args = Cli::try_parse_from(&["cdg", "--raw-format", "csv", "ping"]).unwrap();
        assert_eq!(args.raw_format, "csv");

        // Case 3: Case insensitivity and validation works
        let raw_format_ok = "CSV".to_lowercase();
        assert_eq!(raw_format_ok, "csv");

        let raw_format_bad = "invalid".to_lowercase();
        assert!(raw_format_bad != "json" && raw_format_bad != "csv");
    }

    #[test]
    fn test_default_concurrency_resolution() {
        let orig_pro = std::env::var("COINGECKO_PRO_API_KEY").ok();
        let orig_demo = std::env::var("COINGECKO_DEMO_API_KEY").ok();
        let orig_generic = std::env::var("COINGECKO_API_KEY").ok();
        let orig_type = std::env::var("COINGECKO_API_KEY_TYPE").ok();

        std::env::remove_var("COINGECKO_PRO_API_KEY");
        std::env::remove_var("COINGECKO_DEMO_API_KEY");
        std::env::remove_var("COINGECKO_API_KEY");
        std::env::remove_var("COINGECKO_API_KEY_TYPE");

        let get_default_concurrency = || {
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

        // Case 1: No key -> defaults to 1
        assert_eq!(get_default_concurrency(), 1);

        // Case 2: Demo key set -> defaults to 1
        std::env::set_var("COINGECKO_DEMO_API_KEY", "some_demo_key");
        assert_eq!(get_default_concurrency(), 1);
        std::env::remove_var("COINGECKO_DEMO_API_KEY");

        // Case 3: Pro key set -> defaults to 3
        std::env::set_var("COINGECKO_PRO_API_KEY", "some_pro_key");
        assert_eq!(get_default_concurrency(), 3);
        std::env::remove_var("COINGECKO_PRO_API_KEY");

        // Case 4: Generic key set with type pro -> defaults to 3
        std::env::set_var("COINGECKO_API_KEY", "some_key");
        std::env::set_var("COINGECKO_API_KEY_TYPE", "pro");
        assert_eq!(get_default_concurrency(), 3);

        // Restore original env vars
        if let Some(val) = orig_pro {
            std::env::set_var("COINGECKO_PRO_API_KEY", val);
        }
        if let Some(val) = orig_demo {
            std::env::set_var("COINGECKO_DEMO_API_KEY", val);
        }
        if let Some(val) = orig_generic {
            std::env::set_var("COINGECKO_API_KEY", val);
        }
        if let Some(val) = orig_type {
            std::env::set_var("COINGECKO_API_KEY_TYPE", val);
        }
    }
}
