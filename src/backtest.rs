use anyhow::{anyhow, Result};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BacktestMetrics {
    pub coin: String,
    pub currency: String,
    pub strategy: String,
    pub strategy_return: f64,
    pub buy_and_hold_return: f64,
    pub strategy_sharpe: f64,
    pub buy_and_hold_sharpe: f64,
    pub strategy_max_drawdown: f64,
    pub buy_and_hold_max_drawdown: f64,
    pub prediction_accuracy: f64,
    pub prediction_r2: f64,
    pub active_win_rate: f64,
    pub prediction_rating: String,
    pub strategy_rating: String,
    pub true_positives: usize,
    pub false_positives: usize,
    pub true_negatives: usize,
    pub false_negatives: usize,
    pub total_trades: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BacktestReport {
    pub timestamp: String,
    pub metrics: HashMap<String, BacktestMetrics>,
}

pub fn calculate_mda(actuals: &[f64], predicted: &[f64]) -> f64 {
    if actuals.is_empty() || predicted.is_empty() || actuals.len() != predicted.len() {
        return 0.0;
    }
    let mut correct = 0;
    for i in 0..actuals.len() {
        let a_sign = if actuals[i] > 0.0 {
            1
        } else if actuals[i] < 0.0 {
            -1
        } else {
            0
        };
        let p_sign = if predicted[i] > 0.0 {
            1
        } else if predicted[i] < 0.0 {
            -1
        } else {
            0
        };
        if a_sign == p_sign {
            correct += 1;
        }
    }
    correct as f64 / actuals.len() as f64
}

pub fn calculate_r2(actuals: &[f64], predicted: &[f64]) -> f64 {
    if actuals.is_empty() || predicted.is_empty() || actuals.len() != predicted.len() {
        return 0.0;
    }
    let n = actuals.len() as f64;
    let sum_actual: f64 = actuals.iter().sum();
    let mean_actual = sum_actual / n;

    let mut ss_res = 0.0;
    let mut ss_tot = 0.0;
    for i in 0..actuals.len() {
        ss_res += (actuals[i] - predicted[i]).powi(2);
        ss_tot += (actuals[i] - mean_actual).powi(2);
    }
    if ss_tot == 0.0 {
        return 0.0;
    }
    1.0 - (ss_res / ss_tot)
}

pub fn calculate_sharpe(returns: &[f64], annualization_factor: f64) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }
    let n = returns.len() as f64;
    let mean: f64 = returns.iter().sum::<f64>() / n;

    let var = if n > 1.0 {
        returns.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0)
    } else {
        0.0
    };
    let std_dev = var.sqrt();
    if std_dev == 0.0 {
        return 0.0;
    }
    (mean / std_dev) * annualization_factor.sqrt()
}

pub fn calculate_max_drawdown(equity: &[f64]) -> f64 {
    if equity.is_empty() {
        return 0.0;
    }
    let mut peak = equity[0];
    let mut max_dd = 0.0;
    for &eq in equity {
        if eq > peak {
            peak = eq;
        }
        if peak > 0.0 {
            let dd = (peak - eq) / peak;
            if dd > max_dd {
                max_dd = dd;
            }
        }
    }
    max_dd
}

pub fn classify_prediction_rating(mda: f64, active_win_rate: f64) -> String {
    if mda >= 0.58 || active_win_rate >= 0.62 {
        "excellent".to_string()
    } else if mda >= 0.53 || active_win_rate >= 0.55 {
        "good".to_string()
    } else {
        "bad".to_string()
    }
}

pub fn classify_strategy_rating(
    sharpe: f64,
    strategy_return: f64,
    buy_and_hold_return: f64,
) -> String {
    if sharpe >= 1.5 && strategy_return > buy_and_hold_return {
        "excellent".to_string()
    } else if sharpe >= 0.5 && strategy_return >= 0.00 {
        "good".to_string()
    } else {
        "bad".to_string()
    }
}

pub fn calculate_confusion_matrix(
    signals: &[i64],
    actuals: &[f64],
) -> (usize, usize, usize, usize) {
    let mut tp = 0;
    let mut fp = 0;
    let mut tn = 0;
    let mut fn_count = 0;

    for i in 0..signals.len() {
        let signal = signals[i];
        let actual = actuals[i];

        if signal == 2 {
            if actual > 0.0 {
                tp += 1;
            } else {
                fp += 1;
            }
        } else if signal == 0 {
            if actual < 0.0 {
                tn += 1;
            } else {
                fn_count += 1;
            }
        }
    }

    (tp, fp, tn, fn_count)
}

fn calculate_rolling_std(values: &[f64], index: usize, period: usize) -> f64 {
    let start = index.saturating_sub(period);
    let slice = &values[start..=index];
    let n = slice.len() as f64;
    if n < 2.0 {
        return 1e-6;
    }
    let mean = slice.iter().sum::<f64>() / n;
    let variance = slice.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std = variance.sqrt();
    if std <= 0.0 {
        1e-6
    } else {
        std
    }
}

pub fn run_backtest_for_asset(
    df: &DataFrame,
    coin: &str,
    strategy_name: &str,
    fee: f64,
    slippage: f64,
    annualization_factor: f64,
) -> Result<(BacktestMetrics, Vec<f64>, Vec<f64>)> {
    let close_col = if df.column(&format!("{}_close", coin)).is_ok() {
        format!("{}_close", coin)
    } else {
        coin.to_string()
    };

    let prices: Vec<Option<f64>> = df.column(&close_col)?.f64()?.into_iter().collect();
    let n = prices.len();
    if n < 2 {
        return Err(anyhow!("DataFrame too short to run backtest"));
    }

    let rsi: Option<Vec<Option<f64>>> = df
        .column(&format!("{}_rsi_14", coin))
        .ok()
        .map(|c| c.f64().unwrap().into_iter().collect());

    let macd_line: Option<Vec<Option<f64>>> = df
        .column(&format!("{}_macd_line", coin))
        .ok()
        .map(|c| c.f64().unwrap().into_iter().collect());
    let macd_signal: Option<Vec<Option<f64>>> = df
        .column(&format!("{}_macd_signal", coin))
        .ok()
        .map(|c| c.f64().unwrap().into_iter().collect());
    let macd_hist: Option<Vec<Option<f64>>> = df
        .column(&format!("{}_macd_histogram", coin))
        .ok()
        .map(|c| c.f64().unwrap().into_iter().collect());

    let bb_upper: Option<Vec<Option<f64>>> = df
        .column(&format!("{}_bollinger_upper", coin))
        .ok()
        .map(|c| c.f64().unwrap().into_iter().collect());
    let bb_lower: Option<Vec<Option<f64>>> = df
        .column(&format!("{}_bollinger_lower", coin))
        .ok()
        .map(|c| c.f64().unwrap().into_iter().collect());
    let bb_mid: Option<Vec<Option<f64>>> = df
        .column(&format!("{}_sma_20", coin))
        .ok()
        .map(|c| c.f64().unwrap().into_iter().collect());

    let mut start_idx = 0;
    for i in 0..n {
        let price_ok = prices[i].is_some();
        let strat_ok = match strategy_name.to_lowercase().as_str() {
            "rsi" => rsi.as_ref().and_then(|v| v[i]).is_some(),
            "macd" => {
                macd_line.as_ref().and_then(|v| v[i]).is_some()
                    && macd_signal.as_ref().and_then(|v| v[i]).is_some()
                    && macd_hist.as_ref().and_then(|v| v[i]).is_some()
            }
            "bollinger" => {
                bb_upper.as_ref().and_then(|v| v[i]).is_some()
                    && bb_lower.as_ref().and_then(|v| v[i]).is_some()
                    && bb_mid.as_ref().and_then(|v| v[i]).is_some()
            }
            _ => return Err(anyhow!("Unknown strategy: {}", strategy_name)),
        };
        if price_ok && strat_ok {
            start_idx = i;
            break;
        }
    }

    if start_idx >= n - 1 {
        return Err(anyhow!("No sufficient valid data points to backtest {}", coin));
    }

    let mut current_position = 1; 
    let mut equity = vec![10000.0; n];
    let mut bh_equity = vec![10000.0; n];

    let start_price = prices[start_idx].unwrap();

    let mut signals = vec![1; n];
    let mut actual_returns = vec![0.0; n];
    let mut strategy_returns = vec![0.0; n];
    let mut total_trades = 0;

    let macd_hist_raw: Vec<f64> = macd_hist
        .as_ref()
        .map(|v| v.iter().map(|opt| opt.unwrap_or(0.0)).collect())
        .unwrap_or_default();

    for t in (start_idx + 1)..n {
        let prev_t = t - 1;
        let price_t = match prices[t] {
            Some(p) => p,
            None => {
                equity[t] = equity[prev_t];
                bh_equity[t] = bh_equity[prev_t];
                continue;
            }
        };
        let price_prev = prices[prev_t].unwrap_or(start_price);

        let r_t = (price_t - price_prev) / price_prev;
        actual_returns[t] = r_t;

        bh_equity[t] = 10000.0 * (price_t / start_price);

        let mut sig_t = current_position;
        let mut conf_t = 1.0;

        match strategy_name.to_lowercase().as_str() {
            "rsi" => {
                if let Some(ref rsi_vec) = rsi {
                    if let Some(rsi_val) = rsi_vec[prev_t] {
                        if rsi_val < 30.0 {
                            sig_t = 2; 
                            conf_t = ((30.0 - rsi_val) / 15.0).clamp(0.1, 1.0);
                        } else if rsi_val > 70.0 {
                            sig_t = 0; 
                            conf_t = ((rsi_val - 70.0) / 15.0).clamp(0.1, 1.0);
                        }
                    }
                }
            }
            "bollinger" => {
                if let (Some(ref upper_vec), Some(ref lower_vec), Some(ref mid_vec)) =
                    (&bb_upper, &bb_lower, &bb_mid)
                {
                    if let (Some(up), Some(lo), Some(mid)) =
                        (upper_vec[prev_t], lower_vec[prev_t], mid_vec[prev_t])
                    {
                        let prev_price = price_prev;
                        let std = ((up - mid) / 2.0).max(1e-6);
                        if prev_price < lo {
                            sig_t = 2; 
                            conf_t = ((lo - prev_price) / std).clamp(0.1, 1.0);
                        } else if prev_price > up {
                            sig_t = 0; 
                            conf_t = ((prev_price - up) / std).clamp(0.1, 1.0);
                        }
                    }
                }
            }
            "macd" => {
                if let (Some(ref line_vec), Some(ref sig_vec)) = (&macd_line, &macd_signal) {
                    if let (Some(line), Some(sig)) = (line_vec[prev_t], sig_vec[prev_t]) {
                        if line > sig {
                            sig_t = 2; 
                        } else if line < sig {
                            sig_t = 0; 
                        }
                        let std_hist = calculate_rolling_std(&macd_hist_raw, prev_t, 20);
                        let hist_val = macd_hist_raw[prev_t];
                        conf_t = (hist_val.abs() / std_hist).clamp(0.1, 1.0);
                    }
                }
            }
            _ => {}
        }

        signals[t] = sig_t;

        let mut eq_base = equity[prev_t];
        if sig_t != current_position {
            eq_base *= 1.0 - (fee + slippage);
            total_trades += 1;
        }

        let r_strat = if sig_t == 2 {
            r_t * conf_t
        } else if sig_t == 0 {
            -r_t * conf_t
        } else {
            0.0
        };

        strategy_returns[t] = r_strat;
        equity[t] = eq_base * (1.0 + r_strat);
        current_position = sig_t;
    }

    let final_strat_return = ((equity[n - 1] - 10000.0) / 10000.0) * 100.0;
    let final_bh_return = ((bh_equity[n - 1] - 10000.0) / 10000.0) * 100.0;

    let actual_returns_slice = &actual_returns[(start_idx + 1)..n];
    let strategy_returns_slice = &strategy_returns[(start_idx + 1)..n];
    let signals_slice = &signals[(start_idx + 1)..n];

    let strat_sharpe = calculate_sharpe(strategy_returns_slice, annualization_factor);
    let bh_sharpe = calculate_sharpe(actual_returns_slice, annualization_factor);

    let strat_drawdown = calculate_max_drawdown(&equity[start_idx..n]) * 100.0;
    let bh_drawdown = calculate_max_drawdown(&bh_equity[start_idx..n]) * 100.0;

    let (tp, fp, tn, fn_count) = calculate_confusion_matrix(signals_slice, actual_returns_slice);
    let total_predictions = tp + fp + tn + fn_count;
    let prediction_accuracy = if total_predictions > 0 {
        (tp + tn) as f64 / total_predictions as f64
    } else {
        0.0
    };

    let positive_strat_days = strategy_returns_slice.iter().filter(|&&r| r > 0.0).count();
    let total_active_days = strategy_returns_slice.iter().filter(|&&r| r != 0.0).count();
    let active_win_rate = if total_active_days > 0 {
        positive_strat_days as f64 / total_active_days as f64
    } else {
        0.0
    };

    let prediction_rating = classify_prediction_rating(prediction_accuracy, active_win_rate);
    let strategy_rating = classify_strategy_rating(strat_sharpe, final_strat_return, final_bh_return);

    let parts: Vec<&str> = coin.split('_').collect();
    let coin_base = if parts.len() >= 2 { parts[0].to_string() } else { coin.to_string() };
    let currency_base = if parts.len() >= 2 { parts[1].to_string() } else { "usd".to_string() };

    let metrics = BacktestMetrics {
        coin: coin_base,
        currency: currency_base,
        strategy: strategy_name.to_string(),
        strategy_return: final_strat_return,
        buy_and_hold_return: final_bh_return,
        strategy_sharpe: strat_sharpe,
        buy_and_hold_sharpe: bh_sharpe,
        strategy_max_drawdown: strat_drawdown,
        buy_and_hold_max_drawdown: bh_drawdown,
        prediction_accuracy,
        prediction_r2: 0.0,
        active_win_rate,
        prediction_rating,
        strategy_rating,
        true_positives: tp,
        false_positives: fp,
        true_negatives: tn,
        false_negatives: fn_count,
        total_trades,
    };

    Ok((metrics, equity, bh_equity))
}

pub fn format_backtest_table(metrics: &[BacktestMetrics]) -> String {
    use cli_table::{format::Justify, Cell, Style, Table};

    let mut rows = Vec::new();
    for m in metrics {
        rows.push(vec![
            format!("{}_{}", m.coin.to_uppercase(), m.currency.to_uppercase()).cell(),
            m.strategy.to_uppercase().cell(),
            format!("{:.2}%", m.strategy_return).cell().justify(Justify::Right),
            format!("{:.2}%", m.buy_and_hold_return).cell().justify(Justify::Right),
            format!("{:.2}", m.strategy_sharpe).cell().justify(Justify::Right),
            format!("{:.2}", m.buy_and_hold_sharpe).cell().justify(Justify::Right),
            format!("{:.2}%", m.strategy_max_drawdown).cell().justify(Justify::Right),
            format!("{:.2}%", m.buy_and_hold_max_drawdown).cell().justify(Justify::Right),
            format!("{:.1}%", m.active_win_rate * 100.0).cell().justify(Justify::Right),
            m.total_trades.cell().justify(Justify::Right),
            m.strategy_rating.clone().cell(),
        ]);
    }

    let table = rows.table().title(vec![
        "Asset".cell().bold(true),
        "Strategy".cell().bold(true),
        "Strat Ret".cell().bold(true).justify(Justify::Right),
        "B&H Ret".cell().bold(true).justify(Justify::Right),
        "Strat Sharpe".cell().bold(true).justify(Justify::Right),
        "B&H Sharpe".cell().bold(true).justify(Justify::Right),
        "Strat MaxDD".cell().bold(true).justify(Justify::Right),
        "B&H MaxDD".cell().bold(true).justify(Justify::Right),
        "Win Rate".cell().bold(true).justify(Justify::Right),
        "Trades".cell().bold(true).justify(Justify::Right),
        "Rating".cell().bold(true),
    ]);

    match table.display() {
        Ok(d) => d.to_string(),
        Err(_) => "Error generating table".to_string(),
    }
}

pub fn backtest_portfolio(
    df: &DataFrame,
    assets: &[String],
    weights: &[f64],
    portfolio_name: &str,
    annualization_factor: f64,
) -> Result<(BacktestMetrics, Vec<f64>, Vec<f64>)> {
    let n_assets = assets.len();
    if n_assets == 0 || weights.len() != n_assets {
        return Err(anyhow!("Mismatched or empty assets/weights for portfolio backtest"));
    }

    let n_rows = df.height();
    if n_rows < 2 {
        return Err(anyhow!("DataFrame too short for portfolio backtest"));
    }

    // Extract close prices and daily simple returns for each asset
    let mut close_series = Vec::new();
    let mut returns_series = Vec::new();
    for asset in assets {
        let close: Vec<f64> = df
            .column(asset)?
            .f64()?
            .into_iter()
            .map(|opt| opt.unwrap_or(0.0))
            .collect();
        let ret: Vec<f64> = df
            .column(&format!("{}_simple_return", asset))?
            .f64()?
            .into_iter()
            .map(|opt| opt.unwrap_or(0.0))
            .collect();
        close_series.push(close);
        returns_series.push(ret);
    }

    // Find the first index where close prices > 0.0 and indicator columns are valid (non-null) if they exist
    let mut start_idx = 0;
    for i in 0..n_rows {
        let mut all_valid = true;
        for j in 0..n_assets {
            if close_series[j][i] <= 0.0 {
                all_valid = false;
                break;
            }
            let asset = &assets[j];
            let rsi_col = format!("{}_rsi_14", asset);
            if let Ok(col) = df.column(&rsi_col) {
                if col.f64()?.get(i).is_none() {
                    all_valid = false;
                    break;
                }
            }
        }
        if all_valid {
            start_idx = i;
            break;
        }
    }

    if start_idx >= n_rows - 1 {
        return Err(anyhow!("No overlapping price data for portfolio backtest"));
    }

    // Initialize equity curves
    let mut equity = vec![10000.0; n_rows];
    let mut bh_equity = vec![10000.0; n_rows];

    // Store the initial close prices at start_idx to calculate Buy & Hold weights drift
    let initial_prices: Vec<f64> = (0..n_assets)
        .map(|j| close_series[j][start_idx])
        .collect();

    // Fill equity curves from start_idx onwards
    for i in (start_idx + 1)..n_rows {
        // Daily rebalanced portfolio return (compounded as decimal, simple returns are in %)
        let mut daily_ret = 0.0;
        for j in 0..n_assets {
            let asset_ret = returns_series[j][i];
            daily_ret += weights[j] * asset_ret;
        }
        equity[i] = equity[i - 1] * (1.0 + daily_ret / 100.0);

        // Buy & Hold (no rebalancing) portfolio return
        let mut bh_val = 0.0;
        for j in 0..n_assets {
            let curr_price = close_series[j][i];
            let asset_ratio = curr_price / initial_prices[j];
            bh_val += weights[j] * asset_ratio;
        }
        bh_equity[i] = 10000.0 * bh_val;
    }

    // Calculate returns for Sharpe ratio calculation
    let mut strat_returns = Vec::new();
    let mut bh_returns = Vec::new();
    for i in (start_idx + 1)..n_rows {
        let prev_e = equity[i - 1];
        let curr_e = equity[i];
        strat_returns.push((curr_e - prev_e) / prev_e);

        let prev_bh = bh_equity[i - 1];
        let curr_bh = bh_equity[i];
        bh_returns.push((curr_bh - prev_bh) / prev_bh);
    }

    let final_strat_return = ((equity[n_rows - 1] - 10000.0) / 10000.0) * 100.0;
    let final_bh_return = ((bh_equity[n_rows - 1] - 10000.0) / 10000.0) * 100.0;

    let strat_sharpe = calculate_sharpe(&strat_returns, annualization_factor);
    let bh_sharpe = calculate_sharpe(&bh_returns, annualization_factor);

    let strat_drawdown = calculate_max_drawdown(&equity[start_idx..n_rows]) * 100.0;
    let bh_drawdown = calculate_max_drawdown(&bh_equity[start_idx..n_rows]) * 100.0;

    let strategy_rating = classify_strategy_rating(strat_sharpe, final_strat_return, final_bh_return);

    let metrics = BacktestMetrics {
        coin: portfolio_name.to_string(),
        currency: "portfolio".to_string(),
        strategy: "rebalanced".to_string(),
        strategy_return: final_strat_return,
        buy_and_hold_return: final_bh_return,
        strategy_sharpe: strat_sharpe,
        buy_and_hold_sharpe: bh_sharpe,
        strategy_max_drawdown: strat_drawdown,
        buy_and_hold_max_drawdown: bh_drawdown,
        prediction_accuracy: 1.0,
        prediction_r2: 0.0,
        active_win_rate: 1.0,
        prediction_rating: "N/A".to_string(),
        strategy_rating,
        true_positives: 0,
        false_positives: 0,
        true_negatives: 0,
        false_negatives: 0,
        total_trades: 0,
    };

    Ok((metrics, equity, bh_equity))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_mda() {
        let actuals = vec![0.1, -0.2, 0.3, 0.0];
        let predicted = vec![0.2, -0.1, -0.4, 0.0];
        assert_eq!(calculate_mda(&actuals, &predicted), 0.75);
    }

    #[test]
    fn test_calculate_r2() {
        let actuals = vec![1.0, 2.0, 3.0];
        let predicted = vec![1.1, 1.9, 3.0];
        let r2 = calculate_r2(&actuals, &predicted);
        assert!(r2 > 0.9);
    }

    #[test]
    fn test_calculate_sharpe() {
        let returns = vec![0.01, 0.02, -0.01, 0.015];
        let sharpe = calculate_sharpe(&returns, 365.0);
        assert!(sharpe > 0.0);
    }

    #[test]
    fn test_calculate_max_drawdown() {
        let equity = vec![100.0, 105.0, 95.0, 110.0, 88.0, 120.0];
        let dd = calculate_max_drawdown(&equity);
        assert!((dd - 0.20).abs() < 1e-5);
    }

    #[test]
    fn test_run_backtest_for_asset() {
        let df = DataFrame::new(vec![
            Series::new("date", vec!["2026-06-01", "2026-06-02", "2026-06-03", "2026-06-04"]),
            Series::new("bitcoin_usd", vec![100.0, 105.0, 95.0, 110.0]),
            Series::new("bitcoin_usd_rsi_14", vec![25.0, 28.0, 75.0, 80.0]),
            Series::new("bitcoin_usd_macd_line", vec![1.0, 1.2, 0.8, 1.5]),
            Series::new("bitcoin_usd_macd_signal", vec![0.8, 1.0, 1.0, 1.1]),
            Series::new("bitcoin_usd_macd_histogram", vec![0.2, 0.2, -0.2, 0.4]),
            Series::new("bitcoin_usd_bollinger_upper", vec![110.0, 110.0, 110.0, 110.0]),
            Series::new("bitcoin_usd_bollinger_lower", vec![90.0, 90.0, 90.0, 90.0]),
            Series::new("bitcoin_usd_sma_20", vec![100.0, 100.0, 100.0, 100.0]),
        ]).unwrap();

        let (metrics_rsi, equity_rsi, bh_rsi) = run_backtest_for_asset(&df, "bitcoin_usd", "rsi", 0.001, 0.0005, 365.0).unwrap();
        assert_eq!(metrics_rsi.coin, "bitcoin".to_string());
        assert_eq!(metrics_rsi.strategy, "rsi".to_string());
        assert_eq!(equity_rsi.len(), 4);
        assert_eq!(bh_rsi.len(), 4);

        // MACD strategy
        let (metrics_macd, equity_macd, bh_macd) = run_backtest_for_asset(&df, "bitcoin_usd", "macd", 0.001, 0.0005, 365.0).unwrap();
        assert_eq!(metrics_macd.strategy, "macd");

        // Bollinger strategy
        let (metrics_bb, equity_bb, bh_bb) = run_backtest_for_asset(&df, "bitcoin_usd", "bollinger", 0.001, 0.0005, 365.0).unwrap();
        assert_eq!(metrics_bb.strategy, "bollinger");
    }

    #[test]
    fn test_backtest_portfolio() {
        let df = DataFrame::new(vec![
            Series::new("date", vec!["2026-06-01", "2026-06-02", "2026-06-03", "2026-06-04"]),
            Series::new("bitcoin_usd", vec![100.0, 105.0, 95.0, 110.0]),
            Series::new("bitcoin_usd_simple_return", vec![0.0, 0.05, -0.0952, 0.1579]),
            Series::new("ethereum_usd", vec![10.0, 11.0, 9.5, 12.0]),
            Series::new("ethereum_usd_simple_return", vec![0.0, 0.10, -0.1363, 0.2631]),
        ]).unwrap();

        let assets = vec!["bitcoin_usd".to_string(), "ethereum_usd".to_string()];
        let weights = vec![0.6, 0.4];

        let (metrics, equity, bh_equity) = backtest_portfolio(&df, &assets, &weights, "max_sharpe", 365.0).unwrap();
        assert_eq!(metrics.coin, "max_sharpe");
        assert_eq!(metrics.currency, "portfolio");
        assert_eq!(metrics.strategy, "rebalanced");
        assert_eq!(equity.len(), 4);
        assert_eq!(bh_equity.len(), 4);
        assert!(metrics.strategy_return != 0.0);
        assert!(metrics.buy_and_hold_return != 0.0);
    }
}
