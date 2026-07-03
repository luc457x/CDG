use anyhow::{anyhow, Result};
use chrono::{Datelike, NaiveDate, TimeZone, Utc};
use polars::prelude::*;

#[derive(serde::Deserialize)]
struct MarketChart {
    prices: Vec<(i64, f64)>,
    total_volumes: Option<Vec<(i64, f64)>>,
}

pub fn parse_coingecko_market_chart(json_str: &str, price_col_name: &str) -> Result<DataFrame> {
    let chart: MarketChart = serde_json::from_str(json_str)?;
    let mut dates = Vec::with_capacity(chart.prices.len());
    let mut values = Vec::with_capacity(chart.prices.len());
    let mut volumes = Vec::with_capacity(chart.prices.len());

    let total_volumes = chart.total_volumes.unwrap_or_default();

    for (i, (ts, val)) in chart.prices.iter().enumerate() {
        let datetime = Utc
            .timestamp_millis_opt(*ts)
            .single()
            .ok_or_else(|| anyhow!("Invalid timestamp: {}", ts))?;
        dates.push(datetime.format("%Y-%m-%d").to_string());
        values.push(*val);

        let vol = if i < total_volumes.len() {
            total_volumes[i].1
        } else {
            0.0
        };
        volumes.push(vol);
    }

    let df = DataFrame::new(vec![
        Series::new("date", dates),
        Series::new(price_col_name, values),
        Series::new(&format!("{}_volume", price_col_name), volumes),
    ])?;

    // Group by date and take mean to aggregate to daily
    let df = df
        .lazy()
        .group_by([col("date")])
        .agg([
            col(price_col_name).mean(),
            col(&format!("{}_volume", price_col_name)).mean(),
        ])
        .sort("date", SortOptions::default())
        .collect()?;

    Ok(df)
}

pub fn parse_coingecko_ohlc(json_str: &str, prefix: &str) -> Result<DataFrame> {
    let ohlc: Vec<Vec<f64>> = serde_json::from_str(json_str)?;
    let mut dates = Vec::with_capacity(ohlc.len());
    let mut opens = Vec::with_capacity(ohlc.len());
    let mut highs = Vec::with_capacity(ohlc.len());
    let mut lows = Vec::with_capacity(ohlc.len());
    let mut closes = Vec::with_capacity(ohlc.len());

    for item in ohlc {
        if item.len() >= 5 {
            let ts = item[0] as i64;
            let datetime = Utc
                .timestamp_millis_opt(ts)
                .single()
                .ok_or_else(|| anyhow!("Invalid timestamp: {}", ts))?;
            dates.push(datetime.format("%Y-%m-%d").to_string());
            opens.push(item[1]);
            highs.push(item[2]);
            lows.push(item[3]);
            closes.push(item[4]);
        }
    }

    let df = DataFrame::new(vec![
        Series::new("date", dates),
        Series::new(&format!("{}_open", prefix), opens),
        Series::new(&format!("{}_high", prefix), highs),
        Series::new(&format!("{}_low", prefix), lows),
        Series::new(&format!("{}_close", prefix), closes),
    ])?;

    // Group by date and aggregate: open -> mean, high -> max, low -> min, close -> mean
    let df = df
        .lazy()
        .group_by([col("date")])
        .agg([
            col(&format!("{}_open", prefix)).mean(),
            col(&format!("{}_high", prefix)).max(),
            col(&format!("{}_low", prefix)).min(),
            col(&format!("{}_close", prefix)).mean(),
        ])
        .sort("date", SortOptions::default())
        .collect()?;

    Ok(df)
}

#[derive(serde::Deserialize)]
struct TickersResponse {
    tickers: Vec<TickerItem>,
}

#[derive(serde::Deserialize)]
struct TickerItem {
    base: String,
    target: String,
    market: TickerMarket,
    last: Option<f64>,
    volume: Option<f64>,
    bid_ask_spread_percentage: Option<f64>,
}

#[derive(serde::Deserialize)]
struct TickerMarket {
    name: String,
}

pub fn parse_coingecko_tickers(json_str: &str) -> Result<DataFrame> {
    let resp: TickersResponse = serde_json::from_str(json_str)?;
    let mut exchanges = Vec::with_capacity(resp.tickers.len());
    let mut bases = Vec::with_capacity(resp.tickers.len());
    let mut targets = Vec::with_capacity(resp.tickers.len());
    let mut last_prices = Vec::with_capacity(resp.tickers.len());
    let mut volumes = Vec::with_capacity(resp.tickers.len());
    let mut spreads = Vec::with_capacity(resp.tickers.len());

    for ticker in resp.tickers {
        exchanges.push(ticker.market.name);
        bases.push(ticker.base);
        targets.push(ticker.target);
        last_prices.push(ticker.last.unwrap_or(0.0));
        volumes.push(ticker.volume.unwrap_or(0.0));
        spreads.push(ticker.bid_ask_spread_percentage.unwrap_or(0.0));
    }

    let df = DataFrame::new(vec![
        Series::new("exchange", exchanges),
        Series::new("base", bases),
        Series::new("target", targets),
        Series::new("last_price", last_prices),
        Series::new("volume", volumes),
        Series::new("bid_ask_spread_percentage", spreads),
    ])?;

    Ok(df)
}

pub fn calculate_orderbook_metrics(tickers_df: &DataFrame) -> Result<DataFrame> {
    let spread_col = tickers_df.column("bid_ask_spread_percentage")?;
    let volume_col = tickers_df.column("volume")?;
    let last_price_col = tickers_df.column("last_price")?;

    let spreads: Vec<f64> = spread_col
        .f64()?
        .into_iter()
        .map(|opt| opt.unwrap_or(0.0))
        .collect();
    let volumes: Vec<f64> = volume_col
        .f64()?
        .into_iter()
        .map(|opt| opt.unwrap_or(0.0))
        .collect();
    let last_prices: Vec<f64> = last_price_col
        .f64()?
        .into_iter()
        .map(|opt| opt.unwrap_or(0.0))
        .collect();

    let n = spreads.len();
    if n == 0 {
        return Err(anyhow!("Empty tickers dataframe for orderbook metrics"));
    }

    let avg_spread = spreads.iter().sum::<f64>() / n as f64;
    let total_vol = volumes.iter().sum::<f64>();

    let mean_price = last_prices.iter().sum::<f64>() / n as f64;
    let variance = if n > 1 {
        last_prices
            .iter()
            .map(|&x| (x - mean_price).powi(2))
            .sum::<f64>()
            / (n - 1) as f64
    } else {
        0.0
    };
    let std_dev = variance.sqrt();

    let df = DataFrame::new(vec![
        Series::new("average_spread", vec![avg_spread]),
        Series::new("total_volume", vec![total_vol]),
        Series::new("price_variance", vec![variance]),
        Series::new("price_std_dev", vec![std_dev]),
    ])?;

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
            let filled = if name.ends_with("_volume") {
                df.column(name)?
                    .fill_null(FillNullStrategy::Zero)?
            } else {
                df.column(name)?
                    .fill_null(FillNullStrategy::Forward(None))?
                    .fill_null(FillNullStrategy::Backward(None))?
            };
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

fn calculate_atr(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = close.len();
    let mut atr = vec![None; n];
    if n < period {
        return atr;
    }

    let mut tr = vec![0.0; n];
    tr[0] = high[0] - low[0];
    for i in 1..n {
        let h_l = high[i] - low[i];
        let h_pc = (high[i] - close[i - 1]).abs();
        let l_pc = (low[i] - close[i - 1]).abs();
        tr[i] = h_l.max(h_pc).max(l_pc);
    }

    // First ATR is the SMA of TR over the first period
    let sum_tr: f64 = tr[0..period].iter().sum();
    let mut current_atr = sum_tr / period as f64;
    atr[period - 1] = Some(current_atr);

    for i in period..n {
        current_atr = (current_atr * (period - 1) as f64 + tr[i]) / period as f64;
        atr[i] = Some(current_atr);
    }

    atr
}

#[allow(clippy::needless_range_loop)]
fn calculate_stochastic(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    let n = close.len();
    let mut percent_k = vec![None; n];
    let mut percent_d = vec![None; n];
    if n < period {
        return (percent_k, percent_d);
    }

    for i in (period - 1)..n {
        let start = i + 1 - period;
        let mut highest_high = high[start];
        let mut lowest_low = low[start];
        for j in start..=i {
            if high[j] > highest_high {
                highest_high = high[j];
            }
            if low[j] < lowest_low {
                lowest_low = low[j];
            }
        }
        let denominator = highest_high - lowest_low;
        if denominator != 0.0 {
            percent_k[i] = Some((close[i] - lowest_low) / denominator * 100.0);
        } else {
            percent_k[i] = Some(100.0);
        }
    }

    // 3-period SMA of %K
    for i in (period - 1 + 2)..n {
        let mut sum = 0.0;
        let mut count = 0;
        for j in (i - 2)..=i {
            if let Some(k_val) = percent_k[j] {
                sum += k_val;
                count += 1;
            }
        }
        if count == 3 {
            percent_d[i] = Some(sum / 3.0);
        }
    }

    (percent_k, percent_d)
}

#[allow(clippy::needless_range_loop)]
fn calculate_adx(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = close.len();
    let mut adx = vec![None; n];
    if n < 2 * period {
        return adx;
    }

    let mut tr = vec![0.0; n];
    let mut dm_plus = vec![0.0; n];
    let mut dm_minus = vec![0.0; n];

    tr[0] = high[0] - low[0];
    for i in 1..n {
        let h_l = high[i] - low[i];
        let h_pc = (high[i] - close[i - 1]).abs();
        let l_pc = (low[i] - close[i - 1]).abs();
        tr[i] = h_l.max(h_pc).max(l_pc);

        let up_move = high[i] - high[i - 1];
        let down_move = low[i - 1] - low[i];

        if up_move > down_move && up_move > 0.0 {
            dm_plus[i] = up_move;
        } else {
            dm_plus[i] = 0.0;
        }

        if down_move > up_move && down_move > 0.0 {
            dm_minus[i] = down_move;
        } else {
            dm_minus[i] = 0.0;
        }
    }

    let mut smoothed_tr = vec![0.0; n];
    let mut smoothed_dm_plus = vec![0.0; n];
    let mut smoothed_dm_minus = vec![0.0; n];

    let sum_tr: f64 = tr[0..period].iter().sum();
    let sum_dm_plus: f64 = dm_plus[0..period].iter().sum();
    let sum_dm_minus: f64 = dm_minus[0..period].iter().sum();

    smoothed_tr[period - 1] = sum_tr;
    smoothed_dm_plus[period - 1] = sum_dm_plus;
    smoothed_dm_minus[period - 1] = sum_dm_minus;

    for i in period..n {
        smoothed_tr[i] = smoothed_tr[i - 1] - (smoothed_tr[i - 1] / period as f64) + tr[i];
        smoothed_dm_plus[i] =
            smoothed_dm_plus[i - 1] - (smoothed_dm_plus[i - 1] / period as f64) + dm_plus[i];
        smoothed_dm_minus[i] =
            smoothed_dm_minus[i - 1] - (smoothed_dm_minus[i - 1] / period as f64) + dm_minus[i];
    }

    let mut dx = vec![None; n];
    for i in (period - 1)..n {
        let tr_val = smoothed_tr[i];
        if tr_val > 0.0 {
            let di_plus = (smoothed_dm_plus[i] / tr_val) * 100.0;
            let di_minus = (smoothed_dm_minus[i] / tr_val) * 100.0;
            let diff = (di_plus - di_minus).abs();
            let sum = di_plus + di_minus;
            if sum > 0.0 {
                dx[i] = Some((diff / sum) * 100.0);
            } else {
                dx[i] = Some(0.0);
            }
        } else {
            dx[i] = Some(0.0);
        }
    }

    let mut dx_valid = Vec::new();
    for i in (period - 1)..n {
        if let Some(val) = dx[i] {
            dx_valid.push((i, val));
        }
    }

    if dx_valid.len() >= period {
        let mut sum_dx = 0.0;
        for j in 0..period {
            sum_dx += dx_valid[j].1;
        }
        let mut current_adx = sum_dx / period as f64;
        let first_adx_idx = dx_valid[period - 1].0;
        adx[first_adx_idx] = Some(current_adx);

        for j in period..dx_valid.len() {
            let idx = dx_valid[j].0;
            let val = dx_valid[j].1;
            current_adx = (current_adx * (period - 1) as f64 + val) / period as f64;
            adx[idx] = Some(current_adx);
        }
    }

    adx
}

fn calculate_obv(close: &[f64], volume: &[f64]) -> Vec<Option<f64>> {
    let n = close.len();
    let mut obv = vec![None; n];
    if n == 0 {
        return obv;
    }
    let mut current_obv = volume[0];
    obv[0] = Some(current_obv);
    for i in 1..n {
        if close[i] > close[i - 1] {
            current_obv += volume[i];
        } else if close[i] < close[i - 1] {
            current_obv -= volume[i];
        }
        obv[i] = Some(current_obv);
    }
    obv
}

pub fn compute_returns_and_indicators(df: &DataFrame, target_column: &str) -> Result<DataFrame> {
    let prices_series = df.column(target_column)?;
    let prices_raw: Vec<Option<f64>> = prices_series.f64()?.into_iter().collect();
    let n = prices_raw.len();

    // Build index map and filtered price slice, skipping nulls
    let valid_indices: Vec<usize> = prices_raw
        .iter()
        .enumerate()
        .filter_map(|(i, opt)| if opt.is_some() { Some(i) } else { None })
        .collect();
    let prices: Vec<f64> = valid_indices
        .iter()
        .map(|&i| prices_raw[i].unwrap())
        .collect();

    // Returns (computed over the valid filtered slice)
    let mut smp_returns_filtered = vec![None; prices.len()];
    let mut log_returns_filtered = vec![None; prices.len()];
    for i in 1..prices.len() {
        let prev = prices[i - 1];
        if prev != 0.0 {
            smp_returns_filtered[i] = Some(((prices[i] / prev) - 1.0) * 100.0);
            log_returns_filtered[i] = Some((prices[i] / prev).ln() * 100.0);
        }
    }

    // Indicators (computed over the valid filtered slice)
    let sma20 = calculate_sma(&prices, 20);
    let ema20 = calculate_ema(&prices, 20);
    let rsi14 = calculate_rsi(&prices, 14);
    let (macd_line, macd_signal, macd_hist) = calculate_macd(&prices);
    let (_, bb_upper, bb_lower) = calculate_bollinger_bands(&prices, 20, 2.0);

    // Scatter filtered results back to full-length vectors
    let scatter = |filtered: &[Option<f64>]| -> Vec<Option<f64>> {
        let mut full = vec![None; n];
        for (pos, &idx) in valid_indices.iter().enumerate() {
            full[idx] = filtered[pos];
        }
        full
    };

    let mut out_df = df.clone();
    out_df.insert_column(
        out_df.width(),
        Series::new(
            &format!("{}_simple_return", target_column),
            scatter(&smp_returns_filtered),
        ),
    )?;
    out_df.insert_column(
        out_df.width(),
        Series::new(
            &format!("{}_log_return", target_column),
            scatter(&log_returns_filtered),
        ),
    )?;
    out_df.insert_column(
        out_df.width(),
        Series::new(&format!("{}_sma_20", target_column), scatter(&sma20)),
    )?;
    out_df.insert_column(
        out_df.width(),
        Series::new(&format!("{}_ema_20", target_column), scatter(&ema20)),
    )?;
    out_df.insert_column(
        out_df.width(),
        Series::new(&format!("{}_rsi_14", target_column), scatter(&rsi14)),
    )?;
    out_df.insert_column(
        out_df.width(),
        Series::new(&format!("{}_macd_line", target_column), scatter(&macd_line)),
    )?;
    out_df.insert_column(
        out_df.width(),
        Series::new(
            &format!("{}_macd_signal", target_column),
            scatter(&macd_signal),
        ),
    )?;
    out_df.insert_column(
        out_df.width(),
        Series::new(
            &format!("{}_macd_histogram", target_column),
            scatter(&macd_hist),
        ),
    )?;
    out_df.insert_column(
        out_df.width(),
        Series::new(
            &format!("{}_bollinger_upper", target_column),
            scatter(&bb_upper),
        ),
    )?;
    out_df.insert_column(
        out_df.width(),
        Series::new(
            &format!("{}_bollinger_lower", target_column),
            scatter(&bb_lower),
        ),
    )?;

    // Advanced technical indicators (ATR, Stochastic, ADX, OBV) if columns exist
    let high_col = format!("{}_high", target_column);
    let low_col = format!("{}_low", target_column);
    let vol_col = format!("{}_volume", target_column);

    if df.column(&high_col).is_ok() && df.column(&low_col).is_ok() {
        let highs_raw: Vec<Option<f64>> = df.column(&high_col)?.f64()?.into_iter().collect();
        let lows_raw: Vec<Option<f64>> = df.column(&low_col)?.f64()?.into_iter().collect();

        let highs: Vec<f64> = valid_indices
            .iter()
            .map(|&i| highs_raw[i].unwrap_or(0.0))
            .collect();
        let lows: Vec<f64> = valid_indices
            .iter()
            .map(|&i| lows_raw[i].unwrap_or(0.0))
            .collect();

        let atr = calculate_atr(&highs, &lows, &prices, 14);
        let (stoch_k, stoch_d) = calculate_stochastic(&highs, &lows, &prices, 14);
        let adx = calculate_adx(&highs, &lows, &prices, 14);

        out_df.insert_column(
            out_df.width(),
            Series::new(&format!("{}_atr_14", target_column), scatter(&atr)),
        )?;
        out_df.insert_column(
            out_df.width(),
            Series::new(&format!("{}_stoch_k_14", target_column), scatter(&stoch_k)),
        )?;
        out_df.insert_column(
            out_df.width(),
            Series::new(&format!("{}_stoch_d_3", target_column), scatter(&stoch_d)),
        )?;
        out_df.insert_column(
            out_df.width(),
            Series::new(&format!("{}_adx_14", target_column), scatter(&adx)),
        )?;
    }

    if df.column(&vol_col).is_ok() {
        let vols_raw: Vec<Option<f64>> = df.column(&vol_col)?.f64()?.into_iter().collect();
        let vols: Vec<f64> = valid_indices
            .iter()
            .map(|&i| vols_raw[i].unwrap_or(0.0))
            .collect();

        let obv = calculate_obv(&prices, &vols);
        out_df.insert_column(
            out_df.width(),
            Series::new(&format!("{}_obv", target_column), scatter(&obv)),
        )?;
    }

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

        // Calculate min, max, mean, std using sample variance (N-1) to match sklearn convention
        let valid_values: Vec<f64> = values.iter().filter_map(|&v| v).collect();

        if valid_values.is_empty() {
            continue;
        }

        let n = valid_values.len();
        let min = valid_values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = valid_values
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let mean: f64 = valid_values.iter().sum::<f64>() / n as f64;
        // Sample variance (N-1) matches sklearn StandardScaler default
        let variance: f64 = if n > 1 {
            valid_values
                .iter()
                .map(|&x| {
                    let diff = x - mean;
                    diff * diff
                })
                .sum::<f64>()
                / (n - 1) as f64
        } else {
            0.0
        };
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

    #[test]
    fn test_parse_coingecko_ohlc() {
        let json_data = r#"[[1700000000000, 50000.0, 51000.0, 49000.0, 50500.0]]"#;
        let df = parse_coingecko_ohlc(json_data, "bitcoin_usd").unwrap();
        assert_eq!(df.height(), 1);
        assert_eq!(
            df.column("bitcoin_usd_open").unwrap().f64().unwrap().get(0),
            Some(50000.0)
        );
        assert_eq!(
            df.column("bitcoin_usd_high").unwrap().f64().unwrap().get(0),
            Some(51000.0)
        );
        assert_eq!(
            df.column("bitcoin_usd_low").unwrap().f64().unwrap().get(0),
            Some(49000.0)
        );
        assert_eq!(
            df.column("bitcoin_usd_close")
                .unwrap()
                .f64()
                .unwrap()
                .get(0),
            Some(50500.0)
        );
    }

    #[test]
    fn test_parse_coingecko_tickers_and_orderbook() {
        let json_data = r#"{
            "name": "Bitcoin",
            "tickers": [
                {
                    "base": "BTC",
                    "target": "USD",
                    "market": { "name": "Binance" },
                    "last": 60000.0,
                    "volume": 100.0,
                    "bid_ask_spread_percentage": 0.02
                },
                {
                    "base": "BTC",
                    "target": "USD",
                    "market": { "name": "Coinbase" },
                    "last": 60200.0,
                    "volume": 200.0,
                    "bid_ask_spread_percentage": 0.04
                }
            ]
        }"#;
        let df = parse_coingecko_tickers(json_data).unwrap();
        assert_eq!(df.height(), 2);
        assert_eq!(
            df.column("exchange").unwrap().str().unwrap().get(0),
            Some("Binance")
        );
        assert_eq!(
            df.column("exchange").unwrap().str().unwrap().get(1),
            Some("Coinbase")
        );

        let metrics = calculate_orderbook_metrics(&df).unwrap();
        assert_eq!(metrics.height(), 1);
        assert!(
            (metrics
                .column("average_spread")
                .unwrap()
                .f64()
                .unwrap()
                .get(0)
                .unwrap()
                - 0.03)
                .abs()
                < 1e-9
        );
        assert_eq!(
            metrics
                .column("total_volume")
                .unwrap()
                .f64()
                .unwrap()
                .get(0),
            Some(300.0)
        );
        assert_eq!(
            metrics
                .column("price_variance")
                .unwrap()
                .f64()
                .unwrap()
                .get(0),
            Some(20000.0)
        );
    }

    #[test]
    fn test_advanced_indicators_computation() {
        let dates: Vec<String> = (0..35).map(|i| format!("2026-06-{:02}", i + 1)).collect();
        let highs: Vec<f64> = (0..35).map(|i| 102.0 + i as f64).collect();
        let lows: Vec<f64> = (0..35).map(|i| 98.0 + i as f64).collect();
        let closes: Vec<f64> = (0..35).map(|i| 100.0 + i as f64).collect();
        let volumes: Vec<f64> = (0..35).map(|i| 1000.0 + i as f64).collect();

        let df = DataFrame::new(vec![
            Series::new("date", dates),
            Series::new("bitcoin_usd_high", highs),
            Series::new("bitcoin_usd_low", lows),
            Series::new("bitcoin_usd", closes),
            Series::new("bitcoin_usd_volume", volumes),
        ])
        .unwrap();

        let res = compute_returns_and_indicators(&df, "bitcoin_usd").unwrap();
        assert!(res.column("bitcoin_usd_atr_14").is_ok());
        assert!(res.column("bitcoin_usd_stoch_k_14").is_ok());
        assert!(res.column("bitcoin_usd_stoch_d_3").is_ok());
        assert!(res.column("bitcoin_usd_adx_14").is_ok());
        assert!(res.column("bitcoin_usd_obv").is_ok());
    }

}
