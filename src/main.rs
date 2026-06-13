use anyhow::Result;
use cdg::{analysis, api, cache, export, plot};
use clap::Parser;
use std::io::Write;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Coin ID or comma-separated list of coin IDs from CoinGecko (default: bitcoin)
    #[arg(short, long, default_value = "bitcoin")]
    coin: String,

    /// Vs currency for CoinGecko (default: usd)
    #[arg(short = 'v', long, default_value = "usd")]
    currency: String,

    /// Timeframe in days (default: 90)
    #[arg(short, long, default_value = "90")]
    days: String,

    /// Enable ML preprocessing pipeline (scaling)
    #[arg(long)]
    prep_ml: bool,

    /// Enable lightweight mode (forces coin=bitcoin, days=30, skips benchmarks)
    #[arg(long)]
    light: bool,

    /// Drop weekends instead of forward-filling
    #[arg(long)]
    drop_weekends: bool,

    /// Database file path (default: cdg_files/cache.db)
    #[arg(long, default_value = "cdg_files/cache.db")]
    db_path: String,

    /// Path to export results (default: cdg_files/output)]
    #[arg(short, long, default_value = "cdg_files/output")]
    output_prefix: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = Args::parse();

    // Lightweight override
    if args.light {
        args.days = "30".to_string();
    }

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let run_dir = format!("cdg_files/run_{}", timestamp);
    std::fs::create_dir_all(&run_dir)?;

    println!("Starting CDG Data Collector...");
    println!("Run Directory: {}", run_dir);
    println!("Target Coin: {}", args.coin);
    println!("Currency: {}", args.currency);
    println!("Days: {}", args.days);
    println!("ML Prep: {}", args.prep_ml);
    println!("Lightweight Mode: {}", args.light);
    println!("Drop Weekends: {}", args.drop_weekends);
    println!("DB Path: {}", args.db_path);
    println!("Output Prefix: {}", args.output_prefix);

    // 1. Initialize Cache
    println!("Initializing SQLite Cache...");
    let cache = cache::Cache::new(&args.db_path).await?;

    // 2. Initialize Clients
    let cg_client = api::coingecko::CoinGeckoClient::new(cache.clone());
    let yahoo_client = api::yahoo::YahooClient::new(cache.clone());

    // 3. Ping CoinGecko
    match cg_client.ping().await {
        Ok(_) => println!("CoinGecko API Connection: OK"),
        Err(e) => println!("Warning: CoinGecko API Connection Failed: {}", e),
    }

    // 4. Calculate Timestamps (aligned to start of day for caching)
    let now = chrono::Utc::now().timestamp();
    let rounded_now = (now / 86400) * 86400;
    let days_num: i64 = args.days.parse().unwrap_or(90);
    let from_timestamp = rounded_now - (days_num * 24 * 60 * 60);
    let to_timestamp = rounded_now;

    // 5. Parse coins and currencies
    let coins: Vec<&str> = args
        .coin
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if coins.is_empty() {
        return Err(anyhow::anyhow!("No coins specified"));
    }

    let currencies: Vec<&str> = args
        .currency
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if currencies.is_empty() {
        return Err(anyhow::anyhow!("No currencies specified"));
    }

    // Fetch and display current orderbook metrics for coins
    println!("\nFetching exchange tickers & orderbook metrics...");
    for &coin in &coins {
        match cg_client.get_coin_tickers(coin, Some(1)).await {
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
                            println!("Orderbook Metrics for {}:", coin.to_uppercase());
                            println!("  Average Bid-Ask Spread : {:.4}%", avg_spread * 100.0);
                            println!("  Total Ticker Volume     : {:.2}", total_vol);
                            println!("  Price Std Dev (Exchanges): {:.2}", std_dev);
                        }
                    }
                    Err(e) => println!("Warning: Failed to parse tickers for {}: {}", coin, e),
                }
            }
            Err(e) => println!("Warning: Failed to fetch tickers for {}: {}", coin, e),
        }
    }
    println!("--------------------------------------------------\n");

    // Initialize progress bar for historical data fetching
    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_style(
        indicatif::ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let mut currency_dfs = Vec::new();
    let mut currency_cols = Vec::new();

    for &coin in &coins {
        for &curr in &currencies {
            pb.set_message(format!(
                "Fetching CoinGecko market chart for {} in {}...",
                coin, curr
            ));
            let cg_val = cg_client
                .get_coin_market_chart_range(coin, curr, from_timestamp, to_timestamp)
                .await?;
            let cg_json_str = serde_json::to_string(&cg_val)?;

            let price_col_name = format!("{}_{}", coin, curr);
            currency_cols.push(price_col_name.clone());

            let df_market = analysis::parse_coingecko_market_chart(&cg_json_str, &price_col_name)?;

            pb.set_message(format!(
                "Fetching CoinGecko OHLC for {} in {}...",
                coin, curr
            ));
            let ohlc_val = cg_client.get_coin_ohlc(coin, curr, &args.days).await?;
            let ohlc_json_str = serde_json::to_string(&ohlc_val)?;
            let df_ohlc = analysis::parse_coingecko_ohlc(&ohlc_json_str, &price_col_name)?;

            pb.set_message(format!(
                "Aligning market and OHLC data for {} in {}...",
                coin, curr
            ));
            let df = analysis::align_datasets(&df_market, &[df_ohlc], false)?;
            pb.set_message(format!(
                "Loaded {} rows for {} in {}",
                df.height(),
                coin,
                curr
            ));
            currency_dfs.push(df);
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

    if !args.light {
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
    let aligned_df = analysis::align_datasets(&main_df, &other_dfs, args.drop_weekends)?;
    pb.set_message(format!("Aligned DataFrame shape: {:?}", aligned_df.shape()));

    // 8. Compute indicators on target coins conditionally
    pb.set_message("Computing technical indicators and returns...");
    let mut final_df = aligned_df;
    if args.light {
        final_df = analysis::compute_returns_and_indicators(&final_df, &currency_cols[0])?;
    } else {
        for col in &currency_cols {
            final_df = analysis::compute_returns_and_indicators(&final_df, col)?;
        }
    }

    // 9. Prep ML features if flagged
    if args.prep_ml {
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
    if !args.light {
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
        match cdg::optimization::run_monte_carlo(&final_df, &assets_to_plot, 10000) {
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
