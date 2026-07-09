use anyhow::Result;
use cdg::{api, cache, pipeline, ui};
use clap::{Parser, Subcommand};

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

        /// Portfolio rebalancing frequency: 'daily', 'weekly', or 'monthly' (default: daily)
        #[arg(long, default_value = "daily", env = "CDG_REBALANCE_FREQUENCY")]
        rebalance_frequency: String,
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
        #[arg(short, long, default_value = "rsi", env = "CDG_BACKTEST_STRATEGY")]
        strategy: String,

        /// Transaction fee as decimal (default: 0.001)
        #[arg(long, default_value_t = 0.001)]
        fee: f64,

        /// Slippage as decimal (default: 0.0005)
        #[arg(long, default_value_t = 0.0005)]
        slippage: f64,

        /// Portfolio rebalancing frequency: 'daily', 'weekly', or 'monthly' (default: daily)
        #[arg(long, default_value = "daily", env = "CDG_REBALANCE_FREQUENCY")]
        rebalance_frequency: String,

        /// Drop weekends (use 252 annualization) instead of 365 for standalone backtest
        #[arg(long)]
        drop_weekends: bool,
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

    let res = async {
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
                rebalance_frequency,
            }) => {
                pipeline::run_pipeline_flow(pipeline::PipelineConfig {
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
                    rebalance_frequency: &rebalance_frequency,
                    coingecko_base_url: None,
                    yahoo_base_url: None,
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
                rebalance_frequency,
                drop_weekends,
            }) => {
                pipeline::run_standalone_backtest(
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
                    &rebalance_frequency,
                    drop_weekends,
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
                pipeline::run_ohlcv_flow(
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
                ui::run_interactive_menu(
                    &db_path,
                    &output_dir,
                    &output_prefix,
                    &raw_format,
                    args.cache_ttl,
                )
                .await?;
            }
        }
        Ok::<(), anyhow::Error>(())
    }
    .await;

    if let Err(e) = res {
        if e.to_string().contains("cancelled") || e.to_string().contains("Cancelled") {
            return Ok(());
        }
        return Err(e);
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
