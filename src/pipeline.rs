use crate::{analysis, api, backtest, cache, export, plot};
use anyhow::{anyhow, Result};
use std::io::Write;

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
    pub rebalance_frequency: &'a str,
    pub coingecko_base_url: Option<&'a str>,
    pub yahoo_base_url: Option<&'a str>,
}

struct AbortOnDrop {
    handle: tokio::task::JoinHandle<()>,
}

impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

pub async fn run_pipeline_flow(mut config: PipelineConfig<'_>) -> Result<()> {
    let cancel_token = tokio_util::sync::CancellationToken::new();
    let cancel_token_clone = cancel_token.clone();
    let ctrlc_handle = tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        println!("\nOperation cancelled by user.");
        cancel_token_clone.cancel();
    });
    let _ctrlc_guard = AbortOnDrop {
        handle: ctrlc_handle,
    };

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
    let rebalance_frequency = config.rebalance_frequency;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let run_dir = format!("{}/run_{}", output_dir, timestamp);
    crate::utils::validate_safe_path(&run_dir)?;
    std::fs::create_dir_all(&run_dir)?;
    let ohlcv_dir = format!("{}/raw_ohlcv", run_dir);
    crate::utils::validate_safe_path(&ohlcv_dir)?;
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
    let mut cg_client = api::coingecko::CoinGeckoClient::new(cache.clone())?.with_ttl(cache_ttl);
    if let Some(url) = config.coingecko_base_url {
        cg_client = cg_client.with_base_url(url.to_string());
    }
    let mut yahoo_client = api::yahoo::YahooClient::new(cache.clone())?.with_ttl(cache_ttl);
    if let Some(url) = config.yahoo_base_url {
        yahoo_client = yahoo_client.with_base_url(url.to_string());
    }

    // 3. Ping CoinGecko
    match cg_client.ping().await {
        Ok(_) => println!("CoinGecko API Connection: OK"),
        Err(e) => println!("Warning: CoinGecko API Connection Failed: {}", e),
    }

    // 4. Calculate Timestamps (aligned to start of day for caching)
    let now = chrono::Utc::now().timestamp();
    let rounded_now = (now / 86400) * 86400;
    let fetch_days = if backtest {
        if days <= 40 {
            90
        } else if days <= 130 {
            180
        } else if days <= 315 {
            365
        } else {
            days + 50
        }
    } else {
        days
    };
    let days_num: i64 = fetch_days as i64;
    let from_timestamp = rounded_now - (days_num * 24 * 60 * 60);
    let to_timestamp = rounded_now;

    // 5. Parse coins and currencies
    let raw_coins: Vec<&str> = coin
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if raw_coins.is_empty() {
        return Err(anyhow!("No coins specified"));
    }

    let currencies: Vec<&str> = currency
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if currencies.is_empty() {
        return Err(anyhow!("No currencies specified"));
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
        return Err(anyhow!("No valid coins found to process"));
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
        let days_str = fetch_days.to_string();
        let raw_format_clone = raw_format.to_string();

        let token = cancel_token.clone();
        join_set.spawn(async move {
            if token.is_cancelled() {
                return Err(anyhow!("Operation cancelled"));
            }
            let _permit = sem.acquire().await?;
            if token.is_cancelled() {
                return Err(anyhow!("Operation cancelled"));
            }

            let cg_val = match client
                .get_coin_market_chart_range(&c, &curr, from_timestamp, to_timestamp)
                .await
            {
                Ok(val) => val,
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to fetch market chart for {}-{}: {}. Skipping coin.",
                        c, curr, e
                    );
                    return Ok(None);
                }
            };
            if token.is_cancelled() {
                return Err(anyhow!("Operation cancelled"));
            }
            let cg_json_str = serde_json::to_string(&cg_val)?;

            let c_safe = crate::utils::sanitize_name(&c);
            let curr_safe = crate::utils::sanitize_name(&curr);
            crate::utils::validate_safe_path(&c_safe)?;
            crate::utils::validate_safe_path(&curr_safe)?;

            let price_col_name = format!("{}_{}", c_safe, curr_safe);
            let df_market =
                match analysis::parse_coingecko_market_chart(&cg_json_str, &price_col_name) {
                    Ok(df) => df,
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to parse market chart for {}-{}: {}. Skipping coin.",
                            c, curr, e
                        );
                        return Ok(None);
                    }
                };
            if token.is_cancelled() {
                return Err(anyhow!("Operation cancelled"));
            }

            let ohlc_val = match client.get_coin_ohlc(&c, &curr, &days_str).await {
                Ok(val) => val,
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to fetch OHLC for {}-{}: {}. Skipping coin.",
                        c, curr, e
                    );
                    return Ok(None);
                }
            };
            if token.is_cancelled() {
                return Err(anyhow!("Operation cancelled"));
            }

            if raw_format_clone == "json" {
                let ohlc_json_pretty = serde_json::to_string_pretty(&ohlc_val)?;
                let json_file_path = format!("{}/{}_{}.json", ohlcv_dir_clone, c_safe, curr_safe);
                crate::utils::validate_safe_path(&json_file_path)?;
                std::fs::write(&json_file_path, &ohlc_json_pretty)?;
            } else if raw_format_clone == "csv" {
                let csv_file_path = format!("{}/{}_{}.csv", ohlcv_dir_clone, c_safe, curr_safe);
                crate::utils::validate_safe_path(&csv_file_path)?;
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
            if token.is_cancelled() {
                return Err(anyhow!("Operation cancelled"));
            }

            let ohlc_json_str = serde_json::to_string(&ohlc_val)?;
            let df_ohlc = match analysis::parse_coingecko_ohlc(&ohlc_json_str, &price_col_name) {
                Ok(df) => df,
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse OHLC for {}-{}: {}. Skipping coin.",
                        c, curr, e
                    );
                    return Ok(None);
                }
            };

            let df = match analysis::align_datasets(&df_market, &[df_ohlc], false) {
                Ok(df) => df,
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to align data for {}-{}: {}. Skipping coin.",
                        c, curr, e
                    );
                    return Ok(None);
                }
            };
            Ok::<Option<(polars::prelude::DataFrame, String)>, anyhow::Error>(Some((
                df,
                price_col_name,
            )))
        });
    }

    let mut currency_dfs = Vec::new();
    let mut currency_cols = Vec::new();

    loop {
        tokio::select! {
            res = join_set.join_next() => {
                match res {
                    Some(Ok(Ok(Some((df, col_name))))) => {
                        pb.set_message(format!("Loaded {} rows for {}", df.height(), &col_name));
                        currency_dfs.push(df);
                        currency_cols.push(col_name);
                    }
                    Some(Ok(Ok(None))) => {
                        // Skipped gracefully due to errors
                    }
                    Some(Ok(Err(e))) => {
                        join_set.shutdown().await;
                        return Err(e);
                    }
                    Some(Err(e)) => {
                        join_set.shutdown().await;
                        return Err(anyhow!("Join error: {}", e));
                    }
                    None => break,
                }
            }
            _ = cancel_token.cancelled() => {
                join_set.shutdown().await;
                return Err(anyhow!("Operation cancelled"));
            }
        }
    }

    // Merge all currency DataFrames
    if currency_dfs.is_empty() {
        return Err(anyhow!("No cryptocurrency data was successfully loaded"));
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
        let bench_tickers = vec!["^GSPC", "^DJI", "^IXIC", "^HSI", "^BVSP", "^TNX"];
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
                        if ticker != "^TNX" {
                            assets_to_plot.push(ticker.to_string());
                        }
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

        let final_ann_factor =
            annualization_factor.unwrap_or(if drop_weekends { 252.0 } else { 365.0 });

        match crate::optimization::run_monte_carlo(
            &final_df,
            &currency_cols,
            final_ann_factor,
            10000,
            seed,
        ) {
            Ok(opt_res) => {
                println!("\nOptimal Portfolio Formulations (Annualized):");
                let metrics_table = crate::optimization::format_portfolio_metrics_table(&opt_res);
                println!("{}", metrics_table);

                println!("\nOptimal Asset Weights:");
                let weights_table =
                    crate::optimization::format_optimal_weights_table(&currency_cols, &opt_res);
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
        let final_ann_factor =
            annualization_factor.unwrap_or(if drop_weekends { 252.0 } else { 365.0 });

        let mut backtest_metrics = Vec::new();
        let mut custom_configs = Vec::new();
        let mut strats = Vec::new();
        if strategy.ends_with(".json") {
            let configs = backtest::load_custom_strategies(strategy)?;
            for cfg in configs {
                strats.push(cfg.name.clone());
                custom_configs.push(Some(cfg));
            }
        } else if strategy.to_lowercase() == "all" {
            strats = vec![
                "rsi".to_string(),
                "macd".to_string(),
                "bollinger".to_string(),
            ];
            custom_configs = vec![None, None, None];
        } else {
            strats = vec![strategy.to_lowercase()];
            custom_configs = vec![None];
        }

        let backtest_dir = format!("{}/backtests", run_dir);
        crate::utils::validate_safe_path(&backtest_dir)?;
        std::fs::create_dir_all(&backtest_dir)?;

        let mut asset_bh_caches: std::collections::HashMap<String, Option<backtest::BhCache>> =
            std::collections::HashMap::new();

        // 1. Backtest individual assets
        for col in &currency_cols {
            for (strat, custom_cfg) in strats.iter().zip(custom_configs.iter()) {
                let cache_entry = asset_bh_caches.entry(col.to_string()).or_insert(None);
                match backtest::run_backtest_for_asset(
                    &final_df,
                    col,
                    strat,
                    custom_cfg.as_ref(),
                    fee,
                    slippage,
                    final_ann_factor,
                    days as usize,
                    cache_entry,
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
                        let plot_path = format!("{}/{}_{}_backtest.png", backtest_dir, col, strat);
                        let active_dates = dates[dates.len() - equity.len()..].to_vec();
                        if let Err(e) = plot::plot_backtest_equity(
                            &active_dates,
                            &equity,
                            &bh_equity,
                            col,
                            strat,
                            &plot_path,
                        ) {
                            println!(
                                "Warning: Failed to generate backtest equity plot for {} ({}): {}",
                                col, strat, e
                            );
                        }
                    }
                    Err(e) => {
                        println!("Warning: Backtest failed for {} ({}): {}", col, strat, e);
                    }
                }
            }
        }

        // 2. US Treasury 10Y Benchmark Calculation
        if !currency_cols.is_empty() {
            if let Err(e) = backtest::append_treasury_benchmark(
                &final_df,
                &currency_cols[0],
                days as usize,
                final_ann_factor,
                &mut backtest_metrics,
            ) {
                println!("Warning: Treasury benchmark failed: {}", e);
            }
        }

        // 3. Backtest optimized portfolios if available
        if let Some(ref opt_res) = opt_res_opt {
            let dates: Vec<String> = final_df
                .column("date")?
                .str()?
                .into_iter()
                .map(|opt| opt.unwrap_or("").to_string())
                .collect();

            // Max Sharpe Portfolio
            match backtest::backtest_portfolio(
                &final_df,
                &currency_cols,
                &opt_res.max_sharpe.weights,
                "max_sharpe",
                final_ann_factor,
                days as usize,
                fee,
                slippage,
                rebalance_frequency,
            ) {
                Ok((metrics, equity, bh_equity)) => {
                    backtest_metrics.push(metrics);

                    let plot_path = format!(
                        "{}/max_sharpe_portfolio_rebalanced_backtest.png",
                        backtest_dir
                    );
                    let active_dates = dates[dates.len() - equity.len()..].to_vec();
                    if let Err(e) = plot::plot_backtest_equity(
                        &active_dates,
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
            match backtest::backtest_portfolio(
                &final_df,
                &currency_cols,
                &opt_res.min_volatility.weights,
                "min_volatility",
                final_ann_factor,
                days as usize,
                fee,
                slippage,
                rebalance_frequency,
            ) {
                Ok((metrics, equity, bh_equity)) => {
                    backtest_metrics.push(metrics);

                    let plot_path =
                        format!("{}/min_vol_portfolio_rebalanced_backtest.png", backtest_dir);
                    let active_dates = dates[dates.len() - equity.len()..].to_vec();
                    if let Err(e) = plot::plot_backtest_equity(
                        &active_dates,
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
                    println!(
                        "Warning: Portfolio backtest failed for min_volatility: {}",
                        e
                    );
                }
            }
        }

        // 4. Print consolidated table and save reports
        if !backtest_metrics.is_empty() {
            let backtest_table = backtest::format_backtest_table(&backtest_metrics);
            println!("\nBacktest Summary Results:");
            println!("{}", backtest_table);

            if let Err(e) = backtest::generate_backtest_report(
                &backtest_metrics,
                std::path::Path::new(&backtest_dir),
            ) {
                println!("Warning: Failed to save backtest report: {}", e);
            }
        }
    }

    println!("CDG data pipeline completed successfully!");
    Ok(())
}

pub async fn run_ohlcv_flow(
    cg_client: &api::coingecko::CoinGeckoClient,
    coin: &str,
    currency: &str,
    days: u32,
    format: &str,
    output_dir: &str,
    output_prefix: &str,
    raw_format: &str,
) -> Result<()> {
    let sanitized_coin = crate::utils::sanitize_name(coin);
    let sanitized_currency = crate::utils::sanitize_name(currency);
    crate::utils::validate_safe_path(&sanitized_coin)?;
    crate::utils::validate_safe_path(&sanitized_currency)?;

    println!(
        "Retrieving OHLC data for {} in {} ({} days)...",
        sanitized_coin, sanitized_currency, days
    );
    let ohlc_data = cg_client
        .get_coin_ohlc(coin, currency, &days.to_string())
        .await?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let ohlcv_dir = format!("{}/can_{}", output_dir, timestamp);
    crate::utils::validate_safe_path(&ohlcv_dir)?;
    std::fs::create_dir_all(&ohlcv_dir)?;

    // Save raw OHLCV JSON or CSV files inside ohlcv_dir based on raw_format
    if raw_format == "json" {
        let ohlc_json_pretty = serde_json::to_string_pretty(&ohlc_data)?;
        let json_file_path = format!(
            "{}/{}_{}.json",
            ohlcv_dir, sanitized_coin, sanitized_currency
        );
        crate::utils::validate_safe_path(&json_file_path)?;
        std::fs::write(&json_file_path, &ohlc_json_pretty)?;
    } else if raw_format == "csv" {
        let csv_file_path = format!(
            "{}/{}_{}.csv",
            ohlcv_dir, sanitized_coin, sanitized_currency
        );
        crate::utils::validate_safe_path(&csv_file_path)?;
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
            crate::utils::validate_safe_path(&file_path)?;
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
            crate::utils::validate_safe_path(&file_path)?;
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

pub async fn run_standalone_backtest(
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
    _rebalance_frequency: &str,
    drop_weekends: bool,
) -> Result<()> {
    let cancel_token = tokio_util::sync::CancellationToken::new();
    let cancel_token_clone = cancel_token.clone();
    let ctrlc_handle = tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        println!("\nOperation cancelled by user.");
        cancel_token_clone.cancel();
    });
    let _ctrlc_guard = AbortOnDrop {
        handle: ctrlc_handle,
    };

    println!("Starting CDG Standalone Backtester...");
    println!("Target Coin(s): {}", coin);
    println!("Currency: {}", currency);
    println!("Days: {}", days);
    println!("Strategy: {}", strategy);
    println!("Fee: {}", fee);
    println!("Slippage: {}", slippage);
    println!("Drop Weekends: {}", drop_weekends);

    let ann_factor = if drop_weekends { 252.0 } else { 365.0 };

    // 1. Initialize Cache
    let cache = std::sync::Arc::new(cache::Cache::new(db_path).await?);

    // 2. Initialize Client
    let cg_client = api::coingecko::CoinGeckoClient::new(cache.clone())?.with_ttl(cache_ttl);

    // 3. Calculate Timestamps
    let now = chrono::Utc::now().timestamp();
    let rounded_now = (now / 86400) * 86400;
    let fetch_days = if days <= 40 {
        90
    } else if days <= 130 {
        180
    } else if days <= 315 {
        365
    } else {
        days + 50
    };
    let days_num: i64 = fetch_days as i64;
    let from_timestamp = rounded_now - (days_num * 24 * 60 * 60);
    let to_timestamp = rounded_now;

    // 4. Parse coins and currencies
    let coins: Vec<&str> = coin
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if coins.is_empty() {
        return Err(anyhow!("No coins specified"));
    }

    let currencies: Vec<&str> = currency
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if currencies.is_empty() {
        return Err(anyhow!("No currencies specified"));
    }

    // 5. Ingestion Loop
    let mut coin_dfs = Vec::new();
    let mut target_names = Vec::new();

    for c in &coins {
        for curr in &currencies {
            if cancel_token.is_cancelled() {
                return Err(anyhow!("Operation cancelled"));
            }
            println!("Ingesting historical data for {} in {}...", c, curr);
            let cg_val = cg_client
                .get_coin_market_chart_range(c, curr, from_timestamp, to_timestamp)
                .await?;
            if cancel_token.is_cancelled() {
                return Err(anyhow!("Operation cancelled"));
            }
            let cg_json_str = serde_json::to_string(&cg_val)?;

            let c_safe = crate::utils::sanitize_name(c);
            let curr_safe = crate::utils::sanitize_name(curr);
            crate::utils::validate_safe_path(&c_safe)?;
            crate::utils::validate_safe_path(&curr_safe)?;

            let price_col_name = format!("{}_{}", c_safe, curr_safe);
            let df_market = analysis::parse_coingecko_market_chart(&cg_json_str, &price_col_name)?;

            let days_str = fetch_days.to_string();
            let ohlc_val = cg_client.get_coin_ohlc(c, curr, &days_str).await?;
            if cancel_token.is_cancelled() {
                return Err(anyhow!("Operation cancelled"));
            }
            let ohlc_json_str = serde_json::to_string(&ohlc_val)?;
            let df_ohlc = analysis::parse_coingecko_ohlc(&ohlc_json_str, &price_col_name)?;

            let mut df = analysis::align_datasets(&df_market, &[df_ohlc], false)?;
            if cancel_token.is_cancelled() {
                return Err(anyhow!("Operation cancelled"));
            }
            df = analysis::compute_returns_and_indicators(&df, &price_col_name)?;

            coin_dfs.push(df);
            target_names.push(price_col_name);
        }
    }

    // 6. Run backtests
    let mut backtest_metrics = Vec::new();
    let mut custom_configs = Vec::new();
    let mut strats = Vec::new();
    if strategy.ends_with(".json") {
        let configs = backtest::load_custom_strategies(strategy)?;
        for cfg in configs {
            strats.push(cfg.name.clone());
            custom_configs.push(Some(cfg));
        }
    } else if strategy.to_lowercase() == "all" {
        strats = vec![
            "rsi".to_string(),
            "macd".to_string(),
            "bollinger".to_string(),
        ];
        custom_configs = vec![None, None, None];
    } else {
        strats = vec![strategy.to_lowercase()];
        custom_configs = vec![None];
    }

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let run_dir = format!("{}/backtest_run_{}", output_dir, timestamp);
    crate::utils::validate_safe_path(&run_dir)?;
    std::fs::create_dir_all(&run_dir)?;

    for (df, col) in coin_dfs.iter().zip(target_names.iter()) {
        let mut bh_cache = None;
        for (strat, custom_cfg) in strats.iter().zip(custom_configs.iter()) {
            if cancel_token.is_cancelled() {
                return Err(anyhow!("Operation cancelled"));
            }
            // Standalone run uses ann_factor derived from drop_weekends
            match backtest::run_backtest_for_asset(
                df,
                col,
                strat,
                custom_cfg.as_ref(),
                fee,
                slippage,
                ann_factor,
                days as usize,
                &mut bh_cache,
            ) {
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
                    let active_dates = dates[dates.len() - equity.len()..].to_vec();
                    if let Err(e) = plot::plot_backtest_equity(
                        &active_dates,
                        &equity,
                        &bh_equity,
                        col,
                        strat,
                        &plot_path,
                    ) {
                        println!(
                            "Warning: Failed to generate backtest equity plot for {} ({}): {}",
                            col, strat, e
                        );
                    }
                }
                Err(e) => {
                    println!("Warning: Backtest failed for {} ({}): {}", col, strat, e);
                }
            }
        }

        // Add US Treasury benchmark row
        if let Err(e) = backtest::append_treasury_benchmark(
            df,
            col,
            days as usize,
            ann_factor,
            &mut backtest_metrics,
        ) {
            println!("Warning: Treasury benchmark failed: {}", e);
        }
    }

    if !backtest_metrics.is_empty() {
        let backtest_table = backtest::format_backtest_table(&backtest_metrics);
        println!("\nBacktest Summary Results (Standalone Run):");
        println!("{}", backtest_table);

        if let Err(e) =
            backtest::generate_backtest_report(&backtest_metrics, std::path::Path::new(&run_dir))
        {
            println!("Warning: Failed to save backtest report: {}", e);
        }
    } else {
        println!("No backtests executed successfully.");
    }

    Ok(())
}
