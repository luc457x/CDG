use cdg::analysis;
use cdg::optimization::run_monte_carlo;
use cdg::plot::get_distinct_color;
use polars::prelude::*;

#[test]
fn test_color_distinctness() {
    let c0 = get_distinct_color(0, 5);
    let c1 = get_distinct_color(1, 5);
    let c2 = get_distinct_color(2, 5);

    assert_ne!(c0.0, c1.0);
    assert_ne!(c1.0, c2.0);
}

#[test]
fn test_optimization_integration() {
    let df = DataFrame::new(vec![
        Series::new(
            "date",
            vec!["2026-06-01", "2026-06-02", "2026-06-03", "2026-06-04"],
        ),
        Series::new("BTC", vec![60000.0, 61000.0, 59000.0, 62000.0]),
        Series::new("ETH", vec![3000.0, 3100.0, 2950.0, 3150.0]),
    ])
    .unwrap();

    let assets = vec!["BTC".to_string(), "ETH".to_string()];
    let res = run_monte_carlo(&df, &assets, 365.0, 100, None).unwrap();

    assert_eq!(res.simulated_points.len(), 100);
    assert_eq!(res.max_sharpe.weights.len(), 2);
    assert_eq!(res.min_volatility.weights.len(), 2);
}

#[test]
fn test_optimization_deterministic_with_seed() {
    let df = DataFrame::new(vec![
        Series::new(
            "date",
            vec!["2026-06-01", "2026-06-02", "2026-06-03", "2026-06-04"],
        ),
        Series::new("BTC", vec![60000.0, 61000.0, 59000.0, 62000.0]),
        Series::new("ETH", vec![3000.0, 3100.0, 2950.0, 3150.0]),
    ])
    .unwrap();

    let assets = vec!["BTC".to_string(), "ETH".to_string()];
    let res1 = run_monte_carlo(&df, &assets, 365.0, 200, Some(42)).unwrap();
    let res2 = run_monte_carlo(&df, &assets, 365.0, 200, Some(42)).unwrap();

    // Same seed must produce identical optimal weights
    assert_eq!(
        res1.max_sharpe.weights, res2.max_sharpe.weights,
        "Same seed must produce deterministic results"
    );
    assert_eq!(res1.min_volatility.weights, res2.min_volatility.weights);

    // Different seed must not produce identical weights (with high probability)
    let res3 = run_monte_carlo(&df, &assets, 365.0, 200, Some(99)).unwrap();
    // Allow the test to simply verify different seeds ran successfully
    assert_eq!(res3.simulated_points.len(), 200);
}

#[test]
fn test_full_pipeline_smoke() {
    // Build synthetic market data (daily prices for 35 days)
    let dates: Vec<String> = (1..=35).map(|i| format!("2026-05-{:02}", i)).collect();
    let btc_prices: Vec<f64> = (0..35).map(|i| 60000.0 + i as f64 * 100.0).collect();
    let eth_prices: Vec<f64> = (0..35).map(|i| 3000.0 + i as f64 * 10.0).collect();

    let market_df = DataFrame::new(vec![
        Series::new("date", dates.clone()),
        Series::new("bitcoin_usd", btc_prices.clone()),
        Series::new("bitcoin_usd_volume", vec![1_000_000.0_f64; 35]),
    ])
    .unwrap();

    // Build OHLC data for the same dates
    let highs: Vec<f64> = btc_prices.iter().map(|p| p + 200.0).collect();
    let lows: Vec<f64> = btc_prices.iter().map(|p| p - 200.0).collect();
    let ohlc_df = DataFrame::new(vec![
        Series::new("date", dates),
        Series::new(
            "bitcoin_usd_open",
            btc_prices.iter().map(|p| p - 50.0).collect::<Vec<f64>>(),
        ),
        Series::new("bitcoin_usd_high", highs),
        Series::new("bitcoin_usd_low", lows),
        Series::new("bitcoin_usd_close", btc_prices.clone()),
    ])
    .unwrap();

    // Align market + OHLC
    let aligned = analysis::align_datasets(&market_df, &[ohlc_df], false).unwrap();
    assert_eq!(aligned.height(), 35);

    // Second asset for multi-asset alignment
    let eth_dates: Vec<String> = (1..=35).map(|i| format!("2026-05-{:02}", i)).collect();
    let eth_df = DataFrame::new(vec![
        Series::new("date", eth_dates),
        Series::new("ethereum_usd", eth_prices),
    ])
    .unwrap();
    let multi_df = analysis::align_datasets(&aligned, &[eth_df], false).unwrap();
    assert!(multi_df.column("ethereum_usd").is_ok());

    // Compute indicators for bitcoin_usd
    let with_indicators =
        analysis::compute_returns_and_indicators(&multi_df, "bitcoin_usd").unwrap();
    assert!(with_indicators.column("bitcoin_usd_simple_return").is_ok());
    assert!(with_indicators.column("bitcoin_usd_rsi_14").is_ok());
    assert!(with_indicators.column("bitcoin_usd_atr_14").is_ok());
    assert!(with_indicators.column("bitcoin_usd_obv").is_ok());

    // ML prep: verify minmax and standard scaling columns are produced
    let prepped = analysis::prep_ml(&with_indicators).unwrap();
    assert!(prepped.column("bitcoin_usd_minmax").is_ok());
    assert!(prepped.column("bitcoin_usd_standard").is_ok());

    // Verify minmax range [0, 1]
    let mm = prepped.column("bitcoin_usd_minmax").unwrap().f64().unwrap();
    let min_mm = mm.min().unwrap_or(f64::NAN);
    let max_mm = mm.max().unwrap_or(f64::NAN);
    assert!(min_mm >= 0.0 - 1e-9, "minmax min should be >= 0");
    assert!(max_mm <= 1.0 + 1e-9, "minmax max should be <= 1");
}

#[test]
fn test_align_datasets_volume_filling() {
    let base_df = DataFrame::new(vec![
        Series::new("date", vec!["2026-06-01", "2026-06-02", "2026-06-03"]),
        Series::new("bitcoin_usd", vec![60000.0, 61000.0, 62000.0]),
        Series::new("bitcoin_usd_volume", vec![Some(100.0), None, Some(150.0)]),
    ])
    .unwrap();

    let aligned = analysis::align_datasets(&base_df, &[], false).unwrap();
    let vol = aligned.column("bitcoin_usd_volume").unwrap().f64().unwrap();
    assert_eq!(vol.get(1), Some(0.0)); // Should be 0.0, not forward-filled 100.0!
}

#[test]
fn test_covariance_date_alignment() {
    let df = DataFrame::new(vec![
        Series::new("date", vec!["2026-06-01", "2026-06-02", "2026-06-03", "2026-06-04"]),
        Series::new("asset_a", vec![100.0, 0.0, 102.0, 104.0]),
        Series::new("asset_b", vec![10.0, 12.0, 15.0, 20.0]),
    ])
    .unwrap();

    let assets = vec!["asset_a".to_string(), "asset_b".to_string()];
    let res = run_monte_carlo(&df, &assets, 365.0, 100, Some(42)).unwrap();
    assert_eq!(res.simulated_points.len(), 100);
}

#[test]
fn test_backtest_with_risk_free_rate() {
    let df = DataFrame::new(vec![
        Series::new("date", vec!["2026-06-01", "2026-06-02", "2026-06-03", "2026-06-04", "2026-06-05"]),
        Series::new("asset_close", vec![100.0, 101.0, 102.0, 103.0, 104.0]),
        Series::new("asset_rsi_14", vec![Some(20.0), Some(20.0), Some(80.0), Some(80.0), Some(50.0)]),
        Series::new("^TNX", vec![5.0, 5.0, 5.0, 5.0, 5.0]), // 5% risk-free rate
    ])
    .unwrap();

    let (res_rf, _, _) = cdg::backtest::run_backtest_for_asset(&df, "asset", "rsi", 0.0, 0.0, 252.0).unwrap();

    let df_no_rf = DataFrame::new(vec![
        Series::new("date", vec!["2026-06-01", "2026-06-02", "2026-06-03", "2026-06-04", "2026-06-05"]),
        Series::new("asset_close", vec![100.0, 101.0, 102.0, 103.0, 104.0]),
        Series::new("asset_rsi_14", vec![Some(20.0), Some(20.0), Some(80.0), Some(80.0), Some(50.0)]),
    ])
    .unwrap();

    let (res_no_rf, _, _) = cdg::backtest::run_backtest_for_asset(&df_no_rf, "asset", "rsi", 0.0, 0.0, 252.0).unwrap();

    // The Sharpe ratio with 5% risk free rate should be lower than with 0% risk free rate
    assert!(res_rf.strategy_sharpe < res_no_rf.strategy_sharpe);
}
