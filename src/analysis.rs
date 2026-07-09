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

        let vol = total_volumes.get(i).map(|(_, v)| *v).unwrap_or(0.0);
        volumes.push(vol);
    }

    let df = DataFrame::new(vec![
        Series::new("date", dates),
        Series::new(price_col_name, values),
        Series::new(&format!("{}_volume", price_col_name), volumes),
    ])?;

    // Group by date and aggregate: price -> last close of the day, volume -> mean
    let df = df
        .lazy()
        .group_by([col("date")])
        .agg([
            col(price_col_name).last(),
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
        } else {
            eprintln!("Warning: Skipped malformed OHLC row: {:?}", item);
        }
    }

    let df = DataFrame::new(vec![
        Series::new("date", dates),
        Series::new(&format!("{}_open", prefix), opens),
        Series::new(&format!("{}_high", prefix), highs),
        Series::new(&format!("{}_low", prefix), lows),
        Series::new(&format!("{}_close", prefix), closes),
    ])?;

    // Group by date and aggregate: open -> first, high -> max, low -> min, close -> last
    let df = df
        .lazy()
        .group_by([col("date")])
        .agg([
            col(&format!("{}_open", prefix)).first(),
            col(&format!("{}_high", prefix)).max(),
            col(&format!("{}_low", prefix)).min(),
            col(&format!("{}_close", prefix)).last(),
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

        dates.push(date_str);
        prices.push(price_opt);
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
                df.column(name)?.fill_null(FillNullStrategy::Zero)?
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
    let mut has_nan = false;
    for i in 1..=period {
        let diff = prices[i] - prices[i - 1];
        if diff.is_nan() {
            has_nan = true;
        }
        if diff > 0.0 {
            gains += diff;
        } else if diff < 0.0 {
            losses -= diff;
        }
    }
    let mut avg_gain = gains / period as f64;
    let mut avg_loss = losses / period as f64;

    if has_nan {
        rsi[period] = Some(f64::NAN);
    } else if avg_loss == 0.0 {
        if avg_gain == 0.0 {
            rsi[period] = Some(50.0);
        } else {
            rsi[period] = Some(100.0);
        }
    } else {
        let rs = avg_gain / avg_loss;
        rsi[period] = Some(100.0 - (100.0 / (1.0 + rs)));
    }

    for i in (period + 1)..prices.len() {
        let diff = prices[i] - prices[i - 1];
        if diff.is_nan() || avg_gain.is_nan() || avg_loss.is_nan() {
            avg_gain = f64::NAN;
            avg_loss = f64::NAN;
            rsi[i] = Some(f64::NAN);
            continue;
        }
        let gain = if diff > 0.0 { diff } else { 0.0 };
        let loss = if diff < 0.0 { -diff } else { 0.0 };

        avg_gain = (avg_gain * (period - 1) as f64 + gain) / period as f64;
        avg_loss = (avg_loss * (period - 1) as f64 + loss) / period as f64;

        if avg_loss == 0.0 {
            if avg_gain == 0.0 {
                rsi[i] = Some(50.0);
            } else {
                rsi[i] = Some(100.0);
            }
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
            if e12.is_nan() || e26.is_nan() {
                macd_line[i] = Some(f64::NAN);
            } else {
                macd_line[i] = Some(e12 - e26);
            }
        }
    }

    let mut signal_line = vec![None; prices.len()];
    let first_valid_macd = macd_line.iter().position(|x| x.is_some());
    if let Some(start_idx) = first_valid_macd {
        let macd_slice: Vec<f64> = macd_line[start_idx..]
            .iter()
            .map(|x| x.unwrap_or(f64::NAN))
            .collect();
        let signal_slice = calculate_ema(&macd_slice, 9);
        for j in 0..signal_slice.len() {
            signal_line[start_idx + j] = signal_slice[j];
        }
    }

    let mut histogram = vec![None; prices.len()];
    for i in 0..prices.len() {
        if let (Some(m), Some(s)) = (macd_line[i], signal_line[i]) {
            if m.is_nan() || s.is_nan() {
                histogram[i] = Some(f64::NAN);
            } else {
                histogram[i] = Some(m - s);
            }
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
            if mean.is_nan() {
                upper[i] = Some(f64::NAN);
                lower[i] = Some(f64::NAN);
                continue;
            }
            let mut has_nan = false;
            let mut sum_diff_sq = 0.0;
            for p in &prices[(i + 1 - period)..=i] {
                if p.is_nan() {
                    has_nan = true;
                }
                let diff = p - mean;
                sum_diff_sq += diff * diff;
            }
            if has_nan {
                upper[i] = Some(f64::NAN);
                lower[i] = Some(f64::NAN);
            } else {
                let variance = sum_diff_sq / period as f64;
                let std_dev = variance.sqrt();
                upper[i] = Some(mean + k * std_dev);
                lower[i] = Some(mean - k * std_dev);
            }
        }
    }
    (sma, upper, lower)
}

fn calculate_atr(
    high: &[Option<f64>],
    low: &[Option<f64>],
    close: &[f64],
    period: usize,
) -> Vec<Option<f64>> {
    let n = close.len();
    let mut atr = vec![None; n];
    if n < period {
        return atr;
    }

    let mut tr = vec![None; n];
    if let (Some(h), Some(l)) = (high[0], low[0]) {
        if h.is_nan() || l.is_nan() {
            tr[0] = Some(f64::NAN);
        } else {
            tr[0] = Some(h - l);
        }
    }
    for i in 1..n {
        if let (Some(h), Some(l)) = (high[i], low[i]) {
            let prev_c = close[i - 1];
            if h.is_nan() || l.is_nan() || prev_c.is_nan() {
                tr[i] = Some(f64::NAN);
            } else {
                let h_l = h - l;
                let h_pc = (h - prev_c).abs();
                let l_pc = (l - prev_c).abs();
                tr[i] = Some(h_l.max(h_pc).max(l_pc));
            }
        }
    }

    // First ATR is the SMA of TR over the first period
    let mut sum_tr = 0.0;
    let mut has_none = false;
    let mut has_nan = false;
    for j in 0..period {
        if let Some(val) = tr[j] {
            if val.is_nan() {
                has_nan = true;
            }
            sum_tr += val;
        } else {
            has_none = true;
            break;
        }
    }

    let mut current_atr = if has_none {
        None
    } else if has_nan {
        let val = f64::NAN;
        atr[period - 1] = Some(val);
        Some(val)
    } else {
        let val = sum_tr / period as f64;
        atr[period - 1] = Some(val);
        Some(val)
    };

    for i in period..n {
        if let (Some(prev_atr), Some(tr_val)) = (current_atr, tr[i]) {
            let val = if prev_atr.is_nan() || tr_val.is_nan() {
                f64::NAN
            } else {
                (prev_atr * (period - 1) as f64 + tr_val) / period as f64
            };
            atr[i] = Some(val);
            current_atr = Some(val);
        } else {
            atr[i] = None;
            current_atr = None;
        }
    }

    atr
}

#[allow(clippy::needless_range_loop)]
fn calculate_stochastic(
    high: &[Option<f64>],
    low: &[Option<f64>],
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
        let mut highest_high = None;
        let mut lowest_low = None;
        let mut has_none = false;
        let mut has_nan = false;

        for j in start..=i {
            match (high[j], low[j]) {
                (Some(h), Some(l)) => {
                    if h.is_nan() || l.is_nan() {
                        has_nan = true;
                    }
                    highest_high = Some(highest_high.map_or(h, |v: f64| v.max(h)));
                    lowest_low = Some(lowest_low.map_or(l, |v: f64| v.min(l)));
                }
                _ => {
                    has_none = true;
                    break;
                }
            }
        }

        if has_none {
            percent_k[i] = None;
        } else if has_nan || close[i].is_nan() {
            percent_k[i] = Some(f64::NAN);
        } else if let (Some(hh), Some(ll)) = (highest_high, lowest_low) {
            let denominator = hh - ll;
            if denominator != 0.0 {
                percent_k[i] = Some((close[i] - ll) / denominator * 100.0);
            } else {
                percent_k[i] = Some(100.0);
            }
        }
    }

    // 3-period SMA of %K
    for i in (period - 1 + 2)..n {
        let mut sum = 0.0;
        let mut count = 0;
        let mut has_nan = false;
        for j in (i - 2)..=i {
            if let Some(k_val) = percent_k[j] {
                if k_val.is_nan() {
                    has_nan = true;
                }
                sum += k_val;
                count += 1;
            }
        }
        if has_nan {
            percent_d[i] = Some(f64::NAN);
        } else if count == 3 {
            percent_d[i] = Some(sum / 3.0);
        } else {
            percent_d[i] = None;
        }
    }

    (percent_k, percent_d)
}

#[allow(clippy::needless_range_loop)]
fn calculate_adx(
    high: &[Option<f64>],
    low: &[Option<f64>],
    close: &[f64],
    period: usize,
) -> Vec<Option<f64>> {
    let n = close.len();
    let mut adx = vec![None; n];
    if n < 2 * period {
        return adx;
    }

    let mut tr = vec![None; n];
    let mut dm_plus = vec![None; n];
    let mut dm_minus = vec![None; n];

    if let (Some(h0), Some(l0)) = (high[0], low[0]) {
        if h0.is_nan() || l0.is_nan() {
            tr[0] = Some(f64::NAN);
            dm_plus[0] = Some(f64::NAN);
            dm_minus[0] = Some(f64::NAN);
        } else {
            tr[0] = Some(h0 - l0);
            dm_plus[0] = Some(0.0);
            dm_minus[0] = Some(0.0);
        }
    }

    for i in 1..n {
        match (high[i], low[i], high[i - 1], low[i - 1]) {
            (Some(hi), Some(li), Some(hi_prev), Some(li_prev)) => {
                let prev_c = close[i - 1];
                if hi.is_nan()
                    || li.is_nan()
                    || hi_prev.is_nan()
                    || li_prev.is_nan()
                    || prev_c.is_nan()
                {
                    tr[i] = Some(f64::NAN);
                    dm_plus[i] = Some(f64::NAN);
                    dm_minus[i] = Some(f64::NAN);
                } else {
                    let h_l = hi - li;
                    let h_pc = (hi - prev_c).abs();
                    let l_pc = (li - prev_c).abs();
                    tr[i] = Some(h_l.max(h_pc).max(l_pc));

                    let up_move = hi - hi_prev;
                    let down_move = li_prev - li;

                    if up_move > down_move && up_move > 0.0 {
                        dm_plus[i] = Some(up_move);
                    } else {
                        dm_plus[i] = Some(0.0);
                    }

                    if down_move > up_move && down_move > 0.0 {
                        dm_minus[i] = Some(down_move);
                    } else {
                        dm_minus[i] = Some(0.0);
                    }
                }
            }
            _ => {
                tr[i] = None;
                dm_plus[i] = None;
                dm_minus[i] = None;
            }
        }
    }

    let mut smoothed_tr = vec![None; n];
    let mut smoothed_dm_plus = vec![None; n];
    let mut smoothed_dm_minus = vec![None; n];

    let mut sum_tr = 0.0;
    let mut sum_dm_plus = 0.0;
    let mut sum_dm_minus = 0.0;
    let mut initial_has_none = false;
    let mut initial_has_nan = false;

    for j in 0..period {
        match (tr[j], dm_plus[j], dm_minus[j]) {
            (Some(t_val), Some(dp_val), Some(dm_val)) => {
                if t_val.is_nan() || dp_val.is_nan() || dm_val.is_nan() {
                    initial_has_nan = true;
                }
                sum_tr += t_val;
                sum_dm_plus += dp_val;
                sum_dm_minus += dm_val;
            }
            _ => {
                initial_has_none = true;
                break;
            }
        }
    }

    if initial_has_none {
        // Leave as None
    } else if initial_has_nan {
        smoothed_tr[period - 1] = Some(f64::NAN);
        smoothed_dm_plus[period - 1] = Some(f64::NAN);
        smoothed_dm_minus[period - 1] = Some(f64::NAN);
    } else {
        smoothed_tr[period - 1] = Some(sum_tr);
        smoothed_dm_plus[period - 1] = Some(sum_dm_plus);
        smoothed_dm_minus[period - 1] = Some(sum_dm_minus);
    }

    for i in period..n {
        match (
            smoothed_tr[i - 1],
            smoothed_dm_plus[i - 1],
            smoothed_dm_minus[i - 1],
            tr[i],
            dm_plus[i],
            dm_minus[i],
        ) {
            (
                Some(str_prev),
                Some(sdmp_prev),
                Some(sdmm_prev),
                Some(tr_curr),
                Some(dmp_curr),
                Some(dmm_curr),
            ) => {
                if str_prev.is_nan()
                    || sdmp_prev.is_nan()
                    || sdmm_prev.is_nan()
                    || tr_curr.is_nan()
                    || dmp_curr.is_nan()
                    || dmm_curr.is_nan()
                {
                    smoothed_tr[i] = Some(f64::NAN);
                    smoothed_dm_plus[i] = Some(f64::NAN);
                    smoothed_dm_minus[i] = Some(f64::NAN);
                } else {
                    smoothed_tr[i] = Some(str_prev - (str_prev / period as f64) + tr_curr);
                    smoothed_dm_plus[i] = Some(sdmp_prev - (sdmp_prev / period as f64) + dmp_curr);
                    smoothed_dm_minus[i] = Some(sdmm_prev - (sdmm_prev / period as f64) + dmm_curr);
                }
            }
            _ => {
                smoothed_tr[i] = None;
                smoothed_dm_plus[i] = None;
                smoothed_dm_minus[i] = None;
            }
        }
    }

    let mut dx = vec![None; n];
    for i in (period - 1)..n {
        match (smoothed_tr[i], smoothed_dm_plus[i], smoothed_dm_minus[i]) {
            (Some(tr_val), Some(dmp_val), Some(dmm_val)) => {
                if tr_val.is_nan() || dmp_val.is_nan() || dmm_val.is_nan() {
                    dx[i] = Some(f64::NAN);
                } else if tr_val > 0.0 {
                    let di_plus = (dmp_val / tr_val) * 100.0;
                    let di_minus = (dmm_val / tr_val) * 100.0;
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
            _ => {
                dx[i] = None;
            }
        }
    }

    let mut dx_valid = Vec::new();
    for i in (period - 1)..n {
        if let Some(val) = dx[i] {
            dx_valid.push((i, val));
        }
    }

    if dx_valid.len() >= period {
        let mut is_contiguous = true;
        let mut sum_dx = 0.0;
        let mut has_nan = false;
        for j in 0..period {
            if j > 0 && dx_valid[j].0 != dx_valid[j - 1].0 + 1 {
                is_contiguous = false;
            }
            let val = dx_valid[j].1;
            if val.is_nan() {
                has_nan = true;
            }
            sum_dx += val;
        }

        let mut current_adx = if is_contiguous {
            if has_nan {
                Some(f64::NAN)
            } else {
                Some(sum_dx / period as f64)
            }
        } else {
            None
        };

        let first_adx_idx = dx_valid[period - 1].0;
        adx[first_adx_idx] = current_adx;

        for j in period..dx_valid.len() {
            let idx = dx_valid[j].0;
            let val = dx_valid[j].1;

            if dx_valid[j].0 != dx_valid[j - 1].0 + 1 {
                current_adx = None;
            }

            if let Some(prev_adx) = current_adx {
                if prev_adx.is_nan() || val.is_nan() {
                    current_adx = Some(f64::NAN);
                } else {
                    current_adx = Some((prev_adx * (period - 1) as f64 + val) / period as f64);
                }
            }
            adx[idx] = current_adx;
        }
    }

    adx
}

fn calculate_obv(close: &[f64], volume: &[Option<f64>]) -> Vec<Option<f64>> {
    let n = close.len();
    let mut obv = vec![None; n];
    if n == 0 {
        return obv;
    }
    let mut current_obv = if let Some(v) = volume[0] {
        if v.is_nan() || close[0].is_nan() {
            Some(f64::NAN)
        } else {
            Some(v)
        }
    } else {
        None
    };
    obv[0] = current_obv;
    for i in 1..n {
        match (current_obv, volume[i]) {
            (Some(curr), Some(v)) => {
                if curr.is_nan() || v.is_nan() || close[i].is_nan() || close[i - 1].is_nan() {
                    current_obv = Some(f64::NAN);
                } else if close[i] > close[i - 1] {
                    current_obv = Some(curr + v);
                } else if close[i] < close[i - 1] {
                    current_obv = Some(curr - v);
                } else {
                    current_obv = Some(curr);
                }
            }
            _ => {
                current_obv = None;
            }
        }
        obv[i] = current_obv;
    }
    obv
}

pub fn compute_returns_and_indicators(df: &DataFrame, target_column: &str) -> Result<DataFrame> {
    let prices_series = df.column(target_column)?;
    let prices_raw: Vec<Option<f64>> = prices_series.f64()?.into_iter().collect();
    let n = prices_raw.len();

    let dates_series = df.column("date")?;
    let dates_raw: Vec<String> = dates_series
        .str()?
        .into_iter()
        .map(|opt| opt.unwrap_or("unknown-date").to_string())
        .collect();

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
            if prices[i].is_nan() || prev.is_nan() {
                smp_returns_filtered[i] = Some(f64::NAN);
                log_returns_filtered[i] = Some(f64::NAN);
            } else {
                smp_returns_filtered[i] = Some(((prices[i] / prev) - 1.0) * 100.0);
                log_returns_filtered[i] = Some((prices[i] / prev).ln() * 100.0);
            }
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

    let mut new_cols = Vec::new();
    new_cols.push(Series::new(
        &format!("{}_simple_return", target_column),
        scatter(&smp_returns_filtered),
    ));
    new_cols.push(Series::new(
        &format!("{}_log_return", target_column),
        scatter(&log_returns_filtered),
    ));
    new_cols.push(Series::new(
        &format!("{}_sma_20", target_column),
        scatter(&sma20),
    ));
    new_cols.push(Series::new(
        &format!("{}_ema_20", target_column),
        scatter(&ema20),
    ));
    new_cols.push(Series::new(
        &format!("{}_rsi_14", target_column),
        scatter(&rsi14),
    ));
    new_cols.push(Series::new(
        &format!("{}_macd_line", target_column),
        scatter(&macd_line),
    ));
    new_cols.push(Series::new(
        &format!("{}_macd_signal", target_column),
        scatter(&macd_signal),
    ));
    new_cols.push(Series::new(
        &format!("{}_macd_histogram", target_column),
        scatter(&macd_hist),
    ));
    new_cols.push(Series::new(
        &format!("{}_bollinger_upper", target_column),
        scatter(&bb_upper),
    ));
    new_cols.push(Series::new(
        &format!("{}_bollinger_lower", target_column),
        scatter(&bb_lower),
    ));

    // Advanced technical indicators (ATR, Stochastic, ADX, OBV) if columns exist
    let high_col = format!("{}_high", target_column);
    let low_col = format!("{}_low", target_column);
    let vol_col = format!("{}_volume", target_column);

    if df.column(&high_col).is_ok() && df.column(&low_col).is_ok() {
        let highs_raw: Vec<Option<f64>> = df.column(&high_col)?.f64()?.into_iter().collect();
        let lows_raw: Vec<Option<f64>> = df.column(&low_col)?.f64()?.into_iter().collect();

        // Gating warnings for null values
        for &i in &valid_indices {
            if highs_raw[i].is_none() {
                eprintln!(
                    "{}: {} high was null — ATR/ADX/Stoch set to None",
                    target_column, dates_raw[i]
                );
            }
            if lows_raw[i].is_none() {
                eprintln!(
                    "{}: {} low was null — ATR/ADX/Stoch set to None",
                    target_column, dates_raw[i]
                );
            }
        }

        let highs: Vec<Option<f64>> = valid_indices.iter().map(|&i| highs_raw[i]).collect();
        let lows: Vec<Option<f64>> = valid_indices.iter().map(|&i| lows_raw[i]).collect();

        let atr = calculate_atr(&highs, &lows, &prices, 14);
        let (stoch_k, stoch_d) = calculate_stochastic(&highs, &lows, &prices, 14);
        let adx = calculate_adx(&highs, &lows, &prices, 14);

        new_cols.push(Series::new(
            &format!("{}_atr_14", target_column),
            scatter(&atr),
        ));
        new_cols.push(Series::new(
            &format!("{}_stoch_k_14", target_column),
            scatter(&stoch_k),
        ));
        new_cols.push(Series::new(
            &format!("{}_stoch_d_3", target_column),
            scatter(&stoch_d),
        ));
        new_cols.push(Series::new(
            &format!("{}_adx_14", target_column),
            scatter(&adx),
        ));
    }

    if df.column(&vol_col).is_ok() {
        let vols_raw: Vec<Option<f64>> = df.column(&vol_col)?.f64()?.into_iter().collect();

        for &i in &valid_indices {
            if vols_raw[i].is_none() {
                eprintln!(
                    "{}: {} volume was null — OBV set to None",
                    target_column, dates_raw[i]
                );
            }
        }

        let vols: Vec<Option<f64>> = valid_indices.iter().map(|&i| vols_raw[i]).collect();

        let obv = calculate_obv(&prices, &vols);
        new_cols.push(Series::new(
            &format!("{}_obv", target_column),
            scatter(&obv),
        ));
    }

    let out_df = df.hstack(&new_cols)?;
    Ok(out_df)
}

pub fn prep_ml(df: &DataFrame) -> Result<DataFrame> {
    let col_names: Vec<String> = df
        .get_column_names()
        .iter()
        .map(|&s| s.to_string())
        .collect();

    let mut new_cols = Vec::new();

    for name in col_names {
        if name == "date" {
            continue;
        }

        let series = df.column(&name)?;
        let values: Vec<Option<f64>> = series.f64()?.into_iter().collect();

        // Calculate min, max, mean, std using sample variance (N-1) to match sklearn convention
        let valid_values: Vec<f64> = values
            .iter()
            .filter_map(|&v| v)
            .filter(|&v| v.is_finite())
            .collect();

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
        // Constant-variance fallback: std = 1.0 is intentional (avoids division by zero if all values are identical)
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

        new_cols.push(Series::new(&format!("{}_minmax", name), minmax));
        new_cols.push(Series::new(&format!("{}_standard", name), standard));
    }

    let out_df = df.hstack(&new_cols)?;
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

    #[test]
    fn test_obv_monotonic_flat_price() {
        let dates: Vec<String> = (0..10).map(|i| format!("2026-06-{:02}", i + 1)).collect();
        let closes = vec![100.0; 10];
        let volumes = vec![10.0, 20.0, 15.0, 30.0, 5.0, 40.0, 50.0, 2.0, 8.0, 12.0];

        let df = DataFrame::new(vec![
            Series::new("date", dates),
            Series::new("bitcoin_usd", closes),
            Series::new("bitcoin_usd_volume", volumes),
        ])
        .unwrap();

        let res = compute_returns_and_indicators(&df, "bitcoin_usd").unwrap();
        let obv = res.column("bitcoin_usd_obv").unwrap().f64().unwrap();
        for i in 0..obv.len() {
            assert_eq!(obv.get(i), Some(10.0));
        }
    }

    // --- Parser bug fixes unit tests (Phase 01) ---

    #[test]
    fn test_parse_coingecko_market_chart_prices_greater_than_volumes() {
        let json_data = r#"{
            "prices": [
                [1700000000000, 50000.0],
                [1700086400000, 51000.0]
            ],
            "total_volumes": [
                [1700000000000, 100.0]
            ]
        }"#;
        // Should not panic, volume for second day should be 0.0
        let df = parse_coingecko_market_chart(json_data, "bitcoin").unwrap();
        assert_eq!(df.height(), 2);
        assert_eq!(
            df.column("bitcoin_volume").unwrap().f64().unwrap().get(1),
            Some(0.0)
        );
    }

    #[test]
    fn test_parse_coingecko_market_chart_multi_price_day_last_close() {
        // Multiple entries on same day (2023-11-14 UT timestamps)
        let json_data = r#"{
            "prices": [
                [1700000000000, 50000.0],
                [1700001000000, 50500.0],
                [1700002000000, 51000.0]
            ],
            "total_volumes": [
                [1700000000000, 100.0],
                [1700001000000, 200.0],
                [1700002000000, 300.0]
            ]
        }"#;
        let df = parse_coingecko_market_chart(json_data, "bitcoin").unwrap();
        assert_eq!(df.height(), 1);
        // Price should be the last close of the day (51000.0)
        assert_eq!(
            df.column("bitcoin").unwrap().f64().unwrap().get(0),
            Some(51000.0)
        );
        // Volume should be the mean: (100 + 200 + 300) / 3 = 200.0
        assert_eq!(
            df.column("bitcoin_volume").unwrap().f64().unwrap().get(0),
            Some(200.0)
        );
    }

    #[test]
    fn test_parse_coingecko_ohlc_aggregation_first_last() {
        let json_data = r#"[
            [1700000000000, 50000.0, 51000.0, 49000.0, 50500.0],
            [1700001000000, 50500.0, 52000.0, 48000.0, 51500.0]
        ]"#;
        let df = parse_coingecko_ohlc(json_data, "bitcoin").unwrap();
        assert_eq!(df.height(), 1);
        // Open should be the first open (50000.0)
        assert_eq!(
            df.column("bitcoin_open").unwrap().f64().unwrap().get(0),
            Some(50000.0)
        );
        // High should be the max high (52000.0)
        assert_eq!(
            df.column("bitcoin_high").unwrap().f64().unwrap().get(0),
            Some(52000.0)
        );
        // Low should be the min low (48000.0)
        assert_eq!(
            df.column("bitcoin_low").unwrap().f64().unwrap().get(0),
            Some(48000.0)
        );
        // Close should be the last close (51500.0)
        assert_eq!(
            df.column("bitcoin_close").unwrap().f64().unwrap().get(0),
            Some(51500.0)
        );
    }

    #[test]
    fn test_parse_coingecko_ohlc_malformed_row() {
        let json_data = r#"[
            [1700000000000, 50000.0, 51000.0, 49000.0, 50500.0],
            [1700001000000],
            [1700002000000, 50500.0, 52000.0, 48000.0, 51500.0]
        ]"#;
        let df = parse_coingecko_ohlc(json_data, "bitcoin").unwrap();
        assert_eq!(df.height(), 1); // Grouped to 1 day
    }

    #[test]
    fn test_parse_yahoo_json_null_continuity() {
        let json_data = r#"{
            "chart": {
                "result": [
                    {
                        "timestamp": [1700000000, 1700086400, 1700172800],
                        "indicators": {
                            "quote": [
                                {
                                    "close": [151.0, null, 153.0]
                                }
                            ],
                            "adjclose": [
                                {
                                    "adjclose": [151.0, null, 153.0]
                                }
                            ]
                        }
                    }
                ],
                "error": null
            }
        }"#;
        let df = parse_yahoo_json(json_data, "^GSPC").unwrap();
        assert_eq!(df.height(), 3);
        assert_eq!(
            df.column("^GSPC").unwrap().f64().unwrap().get(0),
            Some(151.0)
        );
        assert_eq!(df.column("^GSPC").unwrap().f64().unwrap().get(1), None);
        assert_eq!(
            df.column("^GSPC").unwrap().f64().unwrap().get(2),
            Some(153.0)
        );
    }

    // --- Phase 02 null-fill checks ---

    #[test]
    fn test_ohlc_null_propagation() {
        let dates: Vec<String> = (0..20).map(|i| format!("2026-06-{:02}", i + 1)).collect();
        let closes: Vec<f64> = (0..20).map(|i| 100.0 + i as f64).collect();
        let mut highs: Vec<Option<f64>> = (0..20).map(|i| Some(102.0 + i as f64)).collect();
        let mut lows: Vec<Option<f64>> = (0..20).map(|i| Some(98.0 + i as f64)).collect();
        let mut volumes: Vec<Option<f64>> = (0..20).map(|i| Some(1000.0 + i as f64)).collect();

        // Inject missing values
        highs[5] = None;
        lows[10] = None;
        volumes[15] = None;

        let df = DataFrame::new(vec![
            Series::new("date", dates),
            Series::new("bitcoin_usd_high", highs),
            Series::new("bitcoin_usd_low", lows),
            Series::new("bitcoin_usd", closes),
            Series::new("bitcoin_usd_volume", volumes),
        ])
        .unwrap();

        let res = compute_returns_and_indicators(&df, "bitcoin_usd").unwrap();

        let atr = res.column("bitcoin_usd_atr_14").unwrap().f64().unwrap();
        let stoch_k = res.column("bitcoin_usd_stoch_k_14").unwrap().f64().unwrap();
        let adx = res.column("bitcoin_usd_adx_14").unwrap().f64().unwrap();
        let obv = res.column("bitcoin_usd_obv").unwrap().f64().unwrap();

        // ATR at index 5 should be None due to missing high
        assert_eq!(atr.get(5), None);
        // Stochastic at index 5 should be None due to missing high
        assert_eq!(stoch_k.get(5), None);
        // ADX at index 5 should be None
        assert_eq!(adx.get(5), None);

        // ATR at index 10 should be None due to missing low
        assert_eq!(atr.get(10), None);
        // Stochastic at index 10 should be None
        assert_eq!(stoch_k.get(10), None);
        // ADX at index 10 should be None
        assert_eq!(adx.get(10), None);

        // OBV at index 15 should be None due to missing volume
        assert_eq!(obv.get(15), None);
    }

    // --- Phase 03 golden tests for technical indicators ---

    #[test]
    fn test_sma_golden() {
        let prices = vec![10.0, 12.0, 14.0, 13.0, 15.0];
        let sma = calculate_sma(&prices, 3);
        assert_eq!(sma[0], None);
        assert_eq!(sma[1], None);
        assert!((sma[2].unwrap() - 12.0).abs() < 1e-6);
        assert!((sma[3].unwrap() - 13.0).abs() < 1e-6);
        assert!((sma[4].unwrap() - 14.0).abs() < 1e-6);

        // period > data.len()
        let sma_edge = calculate_sma(&prices, 10);
        for val in sma_edge {
            assert_eq!(val, None);
        }
    }

    #[test]
    fn test_ema_golden() {
        let prices = vec![10.0, 12.0, 14.0, 16.0];
        let ema = calculate_ema(&prices, 3);
        assert_eq!(ema[0], None);
        assert_eq!(ema[1], None);
        assert!((ema[2].unwrap() - 12.0).abs() < 1e-6);
        assert!((ema[3].unwrap() - 14.0).abs() < 1e-6);
    }

    #[test]
    fn test_rsi_golden() {
        // Uptrend
        let mut up_prices = Vec::new();
        for i in 0..20 {
            up_prices.push(100.0 + i as f64 * 5.0);
        }
        let rsi_up = calculate_rsi(&up_prices, 14);
        // Uptrend RSI should be close to 100.0 or rising
        assert!(rsi_up[19].unwrap() > 90.0);

        // Downtrend
        let mut down_prices = Vec::new();
        for i in 0..20 {
            down_prices.push(200.0 - i as f64 * 5.0);
        }
        let rsi_down = calculate_rsi(&down_prices, 14);
        assert!(rsi_down[19].unwrap() < 10.0);

        // Sideways / Flat
        let sideways = vec![150.0; 20];
        let rsi_flat = calculate_rsi(&sideways, 14);
        assert_eq!(rsi_flat[14], Some(50.0));
        assert_eq!(rsi_flat[19], Some(50.0));
    }

    #[test]
    fn test_bollinger_bands_golden() {
        let prices = vec![10.0, 12.0, 14.0];
        let (sma, upper, lower) = calculate_bollinger_bands(&prices, 3, 2.0);
        assert_eq!(sma[0], None);
        assert_eq!(upper[0], None);
        assert_eq!(lower[0], None);

        assert!((sma[2].unwrap() - 12.0).abs() < 1e-6);
        assert!((upper[2].unwrap() - 15.265986).abs() < 1e-6);
        assert!((lower[2].unwrap() - 8.734014).abs() < 1e-6);
    }

    #[test]
    fn test_atr_golden() {
        let high = vec![Some(12.0), Some(15.0), Some(16.0), Some(14.0)];
        let low = vec![Some(8.0), Some(10.0), Some(12.0), Some(9.0)];
        let close = vec![10.0, 13.0, 14.0, 11.0];

        let atr = calculate_atr(&high, &low, &close, 3);
        assert_eq!(atr[0], None);
        assert_eq!(atr[1], None);
        assert!((atr[2].unwrap() - 4.333333).abs() < 1e-6);
        assert!((atr[3].unwrap() - 4.555556).abs() < 1e-6);
    }

    #[test]
    fn test_stochastic_golden() {
        let high = vec![Some(12.0), Some(15.0), Some(16.0), Some(14.0)];
        let low = vec![Some(8.0), Some(10.0), Some(12.0), Some(9.0)];
        let close = vec![10.0, 13.0, 14.0, 11.0];

        let (percent_k, percent_d) = calculate_stochastic(&high, &low, &close, 3);
        assert_eq!(percent_k[0], None);
        assert_eq!(percent_k[1], None);
        assert!((percent_k[2].unwrap() - 75.0).abs() < 1e-6);
        assert!((percent_k[3].unwrap() - 28.571428).abs() < 1e-6);
        assert_eq!(percent_d[3], None); // Not enough for 3-period SMA of %K yet
    }

    #[test]
    fn test_obv_golden() {
        let close = vec![10.0, 11.0, 9.0, 9.0];
        let volume = vec![Some(100.0), Some(150.0), Some(120.0), Some(80.0)];
        let obv = calculate_obv(&close, &volume);
        assert_eq!(obv[0], Some(100.0));
        assert_eq!(obv[1], Some(250.0));
        assert_eq!(obv[2], Some(130.0));
        assert_eq!(obv[3], Some(130.0));
    }

    #[test]
    fn test_edge_cases_single_bar() {
        let prices = vec![10.0];
        let sma = calculate_sma(&prices, 3);
        assert_eq!(sma[0], None);

        let ema = calculate_ema(&prices, 3);
        assert_eq!(ema[0], None);

        let rsi = calculate_rsi(&prices, 3);
        assert_eq!(rsi[0], None);
    }

    #[test]
    fn test_nan_propagation_indicators() {
        let prices = vec![10.0, f64::NAN, 12.0, 14.0];
        let sma = calculate_sma(&prices, 3);
        // Since there is a NaN, index 2 and 3 should have NaN
        assert!(sma[2].unwrap().is_nan());
        assert!(sma[3].unwrap().is_nan());

        let ema = calculate_ema(&prices, 3);
        assert!(ema[2].unwrap().is_nan());
        assert!(ema[3].unwrap().is_nan());

        let rsi = calculate_rsi(&prices, 3);
        assert!(rsi[3].unwrap().is_nan());
    }
}
