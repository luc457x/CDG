use anyhow::{anyhow, Result};
use chrono::{Datelike, NaiveDate, TimeZone, Utc};
use polars::prelude::*;

#[derive(serde::Deserialize)]
struct MarketChart {
    prices: Vec<(i64, f64)>,
}

pub fn parse_coingecko_market_chart(json_str: &str, price_col_name: &str) -> Result<DataFrame> {
    let chart: MarketChart = serde_json::from_str(json_str)?;
    let mut dates = Vec::with_capacity(chart.prices.len());
    let mut values = Vec::with_capacity(chart.prices.len());

    for (ts, val) in chart.prices {
        let datetime = Utc
            .timestamp_millis_opt(ts)
            .single()
            .ok_or_else(|| anyhow!("Invalid timestamp: {}", ts))?;
        dates.push(datetime.format("%Y-%m-%d").to_string());
        values.push(val);
    }

    let df = DataFrame::new(vec![
        Series::new("date", dates),
        Series::new(price_col_name, values),
    ])?;

    // Group by date and take mean to aggregate to daily
    let df = df
        .lazy()
        .group_by([col("date")])
        .agg([col(price_col_name).mean()])
        .sort("date", SortOptions::default())
        .collect()?;

    Ok(df)
}

#[derive(serde::Deserialize)]
struct YahooChartResponse {
    chart: YahooChartData,
}

#[derive(serde::Deserialize)]
struct YahooChartData {
    result: Option<Vec<YahooChartResult>>,
    error: Option<serde_json::Value>,
}

#[derive(serde::Deserialize)]
struct YahooChartResult {
    timestamp: Vec<i64>,
    indicators: YahooIndicators,
}

#[derive(serde::Deserialize)]
struct YahooIndicators {
    adjclose: Option<Vec<YahooAdjClose>>,
    quote: Option<Vec<YahooQuote>>,
}

#[derive(serde::Deserialize)]
struct YahooAdjClose {
    adjclose: Vec<Option<f64>>,
}

#[derive(serde::Deserialize)]
struct YahooQuote {
    close: Vec<Option<f64>>,
}

pub fn parse_yahoo_json(json_str: &str, ticker: &str) -> Result<DataFrame> {
    let resp: YahooChartResponse = serde_json::from_str(json_str)?;
    let result_vec = resp.chart.result.ok_or_else(|| {
        anyhow!(
            "Yahoo Finance error: {:?}",
            resp.chart.error.unwrap_or(serde_json::Value::Null)
        )
    })?;

    if result_vec.is_empty() {
        return Err(anyhow!("Yahoo Finance returned empty result list"));
    }

    let res = &result_vec[0];
    let mut dates = Vec::new();
    let mut prices = Vec::new();

    let adjclose_values = res
        .indicators
        .adjclose
        .as_ref()
        .and_then(|v| v.first())
        .map(|ac| &ac.adjclose);

    let close_values = res
        .indicators
        .quote
        .as_ref()
        .and_then(|v| v.first())
        .map(|q| &q.close);

    for (i, &ts) in res.timestamp.iter().enumerate() {
        let datetime = Utc
            .timestamp_opt(ts, 0)
            .single()
            .ok_or_else(|| anyhow!("Invalid Yahoo timestamp: {}", ts))?;
        let date_str = datetime.format("%Y-%m-%d").to_string();

        let price_opt = adjclose_values
            .and_then(|v| v.get(i).copied().flatten())
            .or_else(|| close_values.and_then(|v| v.get(i).copied().flatten()));

        if let Some(price) = price_opt {
            dates.push(date_str);
            prices.push(price);
        }
    }

    let df = DataFrame::new(vec![
        Series::new("date", dates),
        Series::new(ticker, prices),
    ])?;

    Ok(df)
}

pub fn align_datasets(
    base_df: &DataFrame,
    other_dfs: &[DataFrame],
    drop_weekends: bool,
) -> Result<DataFrame> {
    let mut lf = base_df.clone().lazy();

    for other_df in other_dfs {
        lf = lf.join(
            other_df.clone().lazy(),
            [col("date")],
            [col("date")],
            JoinType::Left.into(),
        );
    }

    let mut df = lf.collect()?;

    // Perform forward fill and then backward fill on all columns except date
    let column_names: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|&s| s.to_string())
        .collect();

    for name in &column_names {
        if name != "date" {
            let filled = df
                .column(name)?
                .fill_null(FillNullStrategy::Forward(None))?
                .fill_null(FillNullStrategy::Backward(None))?;
            df.replace(name, filled)?;
        }
    }

    if drop_weekends {
        let date_series = df.column("date")?.str()?;
        let mask_vec: Vec<bool> = date_series
            .into_iter()
            .map(|opt_date| {
                if let Some(date_str) = opt_date {
                    if let Ok(nd) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                        let weekday = nd.weekday();
                        return weekday != chrono::Weekday::Sat && weekday != chrono::Weekday::Sun;
                    }
                }
                true
            })
            .collect();
        let mask = BooleanChunked::from_slice("mask", &mask_vec);
        df = df.filter(&mask)?;
    }

    Ok(df)
}

fn calculate_sma(prices: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut sma = vec![None; prices.len()];
    if prices.len() < period {
        return sma;
    }
    let mut sum: f64 = prices[0..period].iter().sum();
    sma[period - 1] = Some(sum / period as f64);
    for i in period..prices.len() {
        sum += prices[i] - prices[i - period];
        sma[i] = Some(sum / period as f64);
    }
    sma
}

fn calculate_ema(prices: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut ema = vec![None; prices.len()];
    if prices.len() < period {
        return ema;
    }
    let k = 2.0 / (period + 1) as f64;
    let sma_sum: f64 = prices[0..period].iter().sum();
    let mut current_ema = sma_sum / period as f64;
    ema[period - 1] = Some(current_ema);
    for i in period..prices.len() {
        current_ema = prices[i] * k + current_ema * (1.0 - k);
        ema[i] = Some(current_ema);
    }
    ema
}

fn calculate_rsi(prices: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut rsi = vec![None; prices.len()];
    if prices.len() <= period {
        return rsi;
    }
    let mut gains = 0.0;
    let mut losses = 0.0;
    for i in 1..=period {
        let diff = prices[i] - prices[i - 1];
        if diff > 0.0 {
            gains += diff;
        } else {
            losses -= diff;
        }
    }
    let mut avg_gain = gains / period as f64;
    let mut avg_loss = losses / period as f64;

    if avg_loss == 0.0 {
        rsi[period] = Some(100.0);
    } else {
        let rs = avg_gain / avg_loss;
        rsi[period] = Some(100.0 - (100.0 / (1.0 + rs)));
    }

    for i in (period + 1)..prices.len() {
        let diff = prices[i] - prices[i - 1];
        let gain = if diff > 0.0 { diff } else { 0.0 };
        let loss = if diff < 0.0 { -diff } else { 0.0 };

        avg_gain = (avg_gain * (period - 1) as f64 + gain) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + loss) / period as f64;

        if avg_loss == 0.0 {
            rsi[i] = Some(100.0);
        } else {
            let rs = avg_gain / avg_loss;
            rsi[i] = Some(100.0 - (100.0 / (1.0 + rs)));
        }
    }
    rsi
}

#[allow(clippy::type_complexity)]
fn calculate_macd(prices: &[f64]) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
    let ema12 = calculate_ema(prices, 12);
    let ema26 = calculate_ema(prices, 26);

    let mut macd_line = vec![None; prices.len()];
    for i in 0..prices.len() {
        if let (Some(e12), Some(e26)) = (ema12[i], ema26[i]) {
            macd_line[i] = Some(e12 - e26);
        }
    }

    let mut signal_line = vec![None; prices.len()];
    let first_valid_macd = macd_line.iter().position(|x| x.is_some());
    if let Some(start_idx) = first_valid_macd {
        let macd_slice: Vec<f64> = macd_line[start_idx..].iter().map(|x| x.unwrap()).collect();
        let signal_slice = calculate_ema(&macd_slice, 9);
        signal_line[start_idx..start_idx + signal_slice.len()].clone_from_slice(&signal_slice);
    }

    let mut histogram = vec![None; prices.len()];
    for i in 0..prices.len() {
        if let (Some(m), Some(s)) = (macd_line[i], signal_line[i]) {
            histogram[i] = Some(m - s);
        }
    }

    (macd_line, signal_line, histogram)
}

#[allow(clippy::type_complexity)]
fn calculate_bollinger_bands(
    prices: &[f64],
    period: usize,
    k: f64,
) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
    let sma = calculate_sma(prices, period);
    let mut upper = vec![None; prices.len()];
    let mut lower = vec![None; prices.len()];

    for i in (period - 1)..prices.len() {
        if let Some(mean) = sma[i] {
            let variance: f64 = prices[(i + 1 - period)..=i]
                .iter()
                .map(|p| {
                    let diff = p - mean;
                    diff * diff
                })
                .sum::<f64>()
                / period as f64;
            let std_dev = variance.sqrt();
            upper[i] = Some(mean + k * std_dev);
            lower[i] = Some(mean - k * std_dev);
        }
    }
    (sma, upper, lower)
}

pub fn compute_returns_and_indicators(df: &DataFrame, target_column: &str) -> Result<DataFrame> {
    let prices_series = df.column(target_column)?;
    let prices: Vec<f64> = prices_series
        .f64()?
        .into_iter()
        .map(|opt| opt.unwrap_or(0.0))
        .collect();

    // Returns
    let mut smp_returns = vec![None; prices.len()];
    let mut log_returns = vec![None; prices.len()];
    for i in 1..prices.len() {
        let prev = prices[i - 1];
        if prev != 0.0 {
            smp_returns[i] = Some(((prices[i] / prev) - 1.0) * 100.0);
            log_returns[i] = Some((prices[i] / prev).ln() * 100.0);
        }
    }

    // Indicators
    let sma20 = calculate_sma(&prices, 20);
    let ema20 = calculate_ema(&prices, 20);
    let rsi14 = calculate_rsi(&prices, 14);
    let (macd_line, macd_signal, macd_hist) = calculate_macd(&prices);
    let (_, bb_upper, bb_lower) = calculate_bollinger_bands(&prices, 20, 2.0);

    let mut out_df = df.clone();
    out_df.insert_column(
        out_df.width(),
        Series::new(&format!("{}_simple_return", target_column), smp_returns),
    )?;
    out_df.insert_column(
        out_df.width(),
        Series::new(&format!("{}_log_return", target_column), log_returns),
    )?;
    out_df.insert_column(
        out_df.width(),
        Series::new(&format!("{}_sma_20", target_column), sma20),
    )?;
    out_df.insert_column(
        out_df.width(),
        Series::new(&format!("{}_ema_20", target_column), ema20),
    )?;
    out_df.insert_column(
        out_df.width(),
        Series::new(&format!("{}_rsi_14", target_column), rsi14),
    )?;
    out_df.insert_column(
        out_df.width(),
        Series::new(&format!("{}_macd_line", target_column), macd_line),
    )?;
    out_df.insert_column(
        out_df.width(),
        Series::new(&format!("{}_macd_signal", target_column), macd_signal),
    )?;
    out_df.insert_column(
        out_df.width(),
        Series::new(&format!("{}_macd_histogram", target_column), macd_hist),
    )?;
    out_df.insert_column(
        out_df.width(),
        Series::new(&format!("{}_bollinger_upper", target_column), bb_upper),
    )?;
    out_df.insert_column(
        out_df.width(),
        Series::new(&format!("{}_bollinger_lower", target_column), bb_lower),
    )?;

    Ok(out_df)
}

pub fn prep_ml(df: &DataFrame) -> Result<DataFrame> {
    let mut out_df = df.clone();
    let col_names: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|&s| s.to_string())
        .collect();

    for name in col_names {
        if name == "date" {
            continue;
        }

        let series = df.column(&name)?;
        let values: Vec<Option<f64>> = series.f64()?.into_iter().collect();

        // Calculate min, max, mean, std
        let valid_values: Vec<f64> = values.iter().filter_map(|&v| v).collect();

        if valid_values.is_empty() {
            continue;
        }

        let min = valid_values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = valid_values
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let mean: f64 = valid_values.iter().sum::<f64>() / valid_values.len() as f64;
        let variance: f64 = valid_values
            .iter()
            .map(|&x| {
                let diff = x - mean;
                diff * diff
            })
            .sum::<f64>()
            / valid_values.len() as f64;
        let std = if variance > 0.0 { variance.sqrt() } else { 1.0 };

        // MinMax
        let minmax: Vec<Option<f64>> = values
            .iter()
            .map(|&opt| {
                opt.map(|x| {
                    if max != min {
                        (x - min) / (max - min)
                    } else {
                        0.0
                    }
                })
            })
            .collect();

        // Standard Z-Score
        let standard: Vec<Option<f64>> = values
            .iter()
            .map(|&opt| opt.map(|x| (x - mean) / std))
            .collect();

        out_df.insert_column(
            out_df.width(),
            Series::new(&format!("{}_minmax", name), minmax),
        )?;
        out_df.insert_column(
            out_df.width(),
            Series::new(&format!("{}_standard", name), standard),
        )?;
    }

    Ok(out_df)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_coingecko() {
        let json_data = r#"{
            "prices": [
                [1700000000000, 50000.0],
                [1700000001000, 50002.0],
                [1700086400000, 51000.0]
            ]
        }"#;
        let df = parse_coingecko_market_chart(json_data, "bitcoin").unwrap();
        assert_eq!(df.height(), 2);
        assert!(df.column("bitcoin").is_ok());
    }

    #[test]
    fn test_parse_yahoo() {
        let json_data = r#"{
            "chart": {
                "result": [
                    {
                        "timestamp": [1700000000, 1700086400],
                        "indicators": {
                            "quote": [
                                {
                                    "close": [151.0, 152.5]
                                }
                            ],
                            "adjclose": [
                                {
                                    "adjclose": [151.0, 152.5]
                                }
                            ]
                        }
                    }
                ],
                "error": null
            }
        }"#;
        let df = parse_yahoo_json(json_data, "^GSPC").unwrap();
        assert_eq!(df.height(), 2);
        assert_eq!(
            df.column("^GSPC").unwrap().f64().unwrap().get(0),
            Some(151.0)
        );
    }

    #[test]
    fn test_align_datasets_logic() {
        let base_df = DataFrame::new(vec![
            Series::new(
                "date",
                vec!["2026-06-12", "2026-06-13", "2026-06-14", "2026-06-15"],
            ),
            Series::new("bitcoin", vec![100.0, 105.0, 110.0, 115.0]),
        ])
        .unwrap();

        let stock_df = DataFrame::new(vec![
            Series::new("date", vec!["2026-06-12", "2026-06-15"]),
            Series::new("^GSPC", vec![50.0, 52.0]),
        ])
        .unwrap();

        // 1. Test forward fill (drop_weekends = false)
        let aligned_ff = align_datasets(&base_df, std::slice::from_ref(&stock_df), false).unwrap();
        assert_eq!(aligned_ff.height(), 4);
        assert_eq!(
            aligned_ff.column("^GSPC").unwrap().f64().unwrap().get(1),
            Some(50.0)
        );
        assert_eq!(
            aligned_ff.column("^GSPC").unwrap().f64().unwrap().get(2),
            Some(50.0)
        );
        assert_eq!(
            aligned_ff.column("^GSPC").unwrap().f64().unwrap().get(3),
            Some(52.0)
        );

        // 2. Test drop weekends (drop_weekends = true)
        let aligned_drop = align_datasets(&base_df, &[stock_df], true).unwrap();
        assert_eq!(aligned_drop.height(), 2);
        assert_eq!(
            aligned_drop.column("date").unwrap().str().unwrap().get(0),
            Some("2026-06-12")
        );
        assert_eq!(
            aligned_drop.column("date").unwrap().str().unwrap().get(1),
            Some("2026-06-15")
        );
    }

    #[test]
    fn test_returns_and_indicators_computation() {
        let mut prices = Vec::new();
        for i in 0..30 {
            prices.push(100.0 + i as f64);
        }
        let dates: Vec<String> = (0..30).map(|i| format!("2026-06-{:02}", i + 1)).collect();

        let df = DataFrame::new(vec![
            Series::new("date", dates),
            Series::new("bitcoin", prices),
        ])
        .unwrap();

        let res = compute_returns_and_indicators(&df, "bitcoin").unwrap();
        assert!(res.column("bitcoin_simple_return").is_ok());
        assert!(res.column("bitcoin_log_return").is_ok());
        assert!(res.column("bitcoin_sma_20").is_ok());
        assert!(res.column("bitcoin_ema_20").is_ok());
        assert!(res.column("bitcoin_rsi_14").is_ok());
        assert!(res.column("bitcoin_macd_line").is_ok());
        assert!(res.column("bitcoin_bollinger_upper").is_ok());
    }

    #[test]
    fn test_prep_ml_normalization() {
        let df = DataFrame::new(vec![
            Series::new("date", vec!["2026-06-12", "2026-06-15"]),
            Series::new("bitcoin", vec![100.0, 200.0]),
        ])
        .unwrap();

        let res = prep_ml(&df).unwrap();
        assert_eq!(
            res.column("bitcoin_minmax").unwrap().f64().unwrap().get(0),
            Some(0.0)
        );
        assert_eq!(
            res.column("bitcoin_minmax").unwrap().f64().unwrap().get(1),
            Some(1.0)
        );
        assert!(res.column("bitcoin_standard").is_ok());
    }

    #[test]
    fn test_multi_currency_merging() {
        let df_usd = DataFrame::new(vec![
            Series::new("date", vec!["2026-06-12", "2026-06-13"]),
            Series::new("bitcoin_usd", vec![60000.0, 61000.0]),
        ])
        .unwrap();

        let df_eur = DataFrame::new(vec![
            Series::new("date", vec!["2026-06-12", "2026-06-13"]),
            Series::new("bitcoin_eur", vec![55000.0, 56000.0]),
        ])
        .unwrap();

        let aligned = align_datasets(&df_usd, &[df_eur], false).unwrap();
        assert_eq!(aligned.height(), 2);
        assert_eq!(
            aligned.column("bitcoin_usd").unwrap().f64().unwrap().get(0),
            Some(60000.0)
        );
        assert_eq!(
            aligned.column("bitcoin_eur").unwrap().f64().unwrap().get(0),
            Some(55000.0)
        );
    }
}
