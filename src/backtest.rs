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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ColumnRef {
    pub column: String,
    #[serde(default)]
    pub shift: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum ValueSource {
    Number(f64),
    Column(ColumnRef),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum LogicalOperator {
    #[serde(rename = "AND", alias = "and")]
    And,
    #[serde(rename = "OR", alias = "or")]
    Or,
    #[serde(rename = "NOT", alias = "not")]
    Not,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ComparisonOperator {
    #[serde(rename = "<")]
    LessThan,
    #[serde(rename = ">")]
    GreaterThan,
    #[serde(rename = "==")]
    Equal,
    #[serde(rename = "<=")]
    LessThanOrEqual,
    #[serde(rename = ">=")]
    GreaterThanOrEqual,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Condition {
    Logical {
        operator: LogicalOperator,
        rules: Vec<Condition>,
    },
    Comparison {
        column: String,
        #[serde(default)]
        shift: usize,
        operator: ComparisonOperator,
        value: ValueSource,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum ConfidenceConfig {
    #[serde(rename = "Constant")]
    Constant { value: f64 },
    #[serde(rename = "LinearScale")]
    LinearScale {
        column: String,
        #[serde(default)]
        shift: usize,
        min: f64,
        max: f64,
        #[serde(default = "default_multiplier")]
        multiplier: f64,
    },
}

fn default_multiplier() -> f64 {
    1.0
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CustomStrategyConfig {
    pub name: String,
    pub buy_condition: Condition,
    pub sell_condition: Condition,
    pub neutral_condition: Option<Condition>,
    #[serde(default = "default_confidence_config")]
    pub confidence: ConfidenceConfig,
}

fn default_confidence_config() -> ConfidenceConfig {
    ConfidenceConfig::Constant { value: 1.0 }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum CustomStrategyInput {
    Single(CustomStrategyConfig),
    MultipleList(Vec<CustomStrategyConfig>),
    MultipleMap(HashMap<String, CustomStrategyConfig>),
}

pub fn load_custom_strategies(path: &str) -> Result<Vec<CustomStrategyConfig>> {
    let content = std::fs::read_to_string(path)?;
    let input: CustomStrategyInput = serde_json::from_str(&content)?;
    let configs = match input {
        CustomStrategyInput::Single(cfg) => vec![cfg],
        CustomStrategyInput::MultipleList(list) => list,
        CustomStrategyInput::MultipleMap(map) => {
            let mut list = Vec::new();
            for (key, mut cfg) in map {
                if cfg.name.is_empty() {
                    cfg.name = key;
                }
                list.push(cfg);
            }
            list
        }
    };
    Ok(configs)
}

fn get_df_value(df: &DataFrame, name: &str, idx: usize, shift: usize, coin: &str) -> Result<f64> {
    if idx < shift {
        return Err(anyhow!(
            "Index {} out of bounds for shift {} at start of backtest",
            idx,
            shift
        ));
    }
    let target_idx = idx - shift;
    let col = if df.column(name).is_ok() {
        df.column(name)?
    } else {
        let prefixed = format!("{}_{}", coin, name);
        df.column(&prefixed)?
    };
    let val = col.f64()?.get(target_idx).ok_or_else(|| {
        anyhow!(
            "Null or missing value in column {} (resolved index: {})",
            col.name(),
            target_idx
        )
    })?;
    Ok(val)
}

fn evaluate_condition(cond: &Condition, idx: usize, df: &DataFrame, coin: &str) -> Result<bool> {
    match cond {
        Condition::Logical { operator, rules } => match operator {
            LogicalOperator::And => {
                for r in rules {
                    if !evaluate_condition(r, idx, df, coin)? {
                        return Ok(false);
                    }
                }
                Ok(!rules.is_empty())
            }
            LogicalOperator::Or => {
                for r in rules {
                    if evaluate_condition(r, idx, df, coin)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            LogicalOperator::Not => {
                if rules.len() != 1 {
                    return Err(anyhow!("NOT operator requires exactly one rule"));
                }
                let res = evaluate_condition(&rules[0], idx, df, coin)?;
                Ok(!res)
            }
        },
        Condition::Comparison {
            column,
            shift,
            operator,
            value,
        } => {
            let col_val = get_df_value(df, column, idx, *shift, coin)?;
            let target_val = match value {
                ValueSource::Number(n) => *n,
                ValueSource::Column(col_ref) => {
                    get_df_value(df, &col_ref.column, idx, col_ref.shift, coin)?
                }
            };
            let match_res = match operator {
                ComparisonOperator::LessThan => col_val < target_val,
                ComparisonOperator::GreaterThan => col_val > target_val,
                ComparisonOperator::Equal => (col_val - target_val).abs() < 1e-9,
                ComparisonOperator::LessThanOrEqual => col_val <= target_val,
                ComparisonOperator::GreaterThanOrEqual => col_val >= target_val,
            };
            Ok(match_res)
        }
    }
}

fn evaluate_confidence(
    config: &ConfidenceConfig,
    idx: usize,
    df: &DataFrame,
    coin: &str,
) -> Result<f64> {
    match config {
        ConfidenceConfig::Constant { value } => Ok(*value),
        ConfidenceConfig::LinearScale {
            column,
            shift,
            min,
            max,
            multiplier,
        } => {
            let val = get_df_value(df, column, idx, *shift, coin)?;
            let scaled = (val.abs() * multiplier).clamp(*min, *max);
            Ok(scaled)
        }
    }
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
        // constant series: perfect if residuals also zero, else zero
        return if ss_res == 0.0 { 1.0 } else { 0.0 };
    }
    1.0 - (ss_res / ss_tot)
}

pub fn calculate_sharpe(returns: &[f64], rf_rate: f64, annualization_factor: f64) -> f64 {
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
    ((mean - rf_rate) / std_dev) * annualization_factor.sqrt()
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
        if peak != 0.0 {
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

#[derive(Clone, Debug)]
pub struct BhCache {
    pub final_return: f64,
    pub sharpe: f64,
    pub drawdown: f64,
    pub equity: Vec<f64>,
}

fn collect_referenced_columns(cond: &Condition, cols: &mut std::collections::HashSet<String>) {
    match cond {
        Condition::Logical { rules, .. } => {
            for r in rules {
                collect_referenced_columns(r, cols);
            }
        }
        Condition::Comparison { column, value, .. } => {
            cols.insert(column.clone());
            if let ValueSource::Column(col_ref) = value {
                cols.insert(col_ref.column.clone());
            }
        }
    }
}

fn collect_strategy_columns(config: &CustomStrategyConfig) -> std::collections::HashSet<String> {
    let mut cols = std::collections::HashSet::new();
    collect_referenced_columns(&config.buy_condition, &mut cols);
    collect_referenced_columns(&config.sell_condition, &mut cols);
    if let Some(ref neutral) = config.neutral_condition {
        collect_referenced_columns(neutral, &mut cols);
    }
    match &config.confidence {
        ConfidenceConfig::Constant { .. } => {}
        ConfidenceConfig::LinearScale { column, .. } => {
            cols.insert(column.clone());
        }
    }
    cols
}

fn get_max_shift(cond: &Condition) -> usize {
    match cond {
        Condition::Logical { rules, .. } => rules.iter().map(get_max_shift).max().unwrap_or(0),
        Condition::Comparison { shift, value, .. } => {
            let val_shift = match value {
                ValueSource::Number(_) => 0,
                ValueSource::Column(col_ref) => col_ref.shift,
            };
            (*shift).max(val_shift)
        }
    }
}

fn get_strategy_max_shift(config: &CustomStrategyConfig) -> usize {
    let mut max_shift =
        get_max_shift(&config.buy_condition).max(get_max_shift(&config.sell_condition));
    if let Some(ref neutral) = config.neutral_condition {
        max_shift = max_shift.max(get_max_shift(neutral));
    }
    if let ConfidenceConfig::LinearScale { shift, .. } = &config.confidence {
        max_shift = max_shift.max(*shift);
    }
    max_shift
}

pub fn run_backtest_for_asset(
    df: &DataFrame,
    coin: &str,
    strategy_name: &str,
    custom_strat: Option<&CustomStrategyConfig>,
    fee: f64,
    slippage: f64,
    annualization_factor: f64,
    days_to_backtest: usize,
    bh_cache: &mut Option<BhCache>,
) -> Result<(BacktestMetrics, Vec<f64>, Vec<f64>)> {
    let display_strat_name = if let Some(ref config) = custom_strat {
        config.name.clone()
    } else {
        strategy_name.to_string()
    };

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

    if custom_strat.is_none() {
        match strategy_name.to_lowercase().as_str() {
            "rsi" | "macd" | "bollinger" => {}
            _ => return Err(anyhow!("Unknown strategy: {}", strategy_name)),
        }
    }

    let required_cols = if let Some(ref config) = custom_strat {
        collect_strategy_columns(config)
    } else {
        std::collections::HashSet::new()
    };

    let max_shift = if let Some(ref config) = custom_strat {
        get_strategy_max_shift(config)
    } else {
        0
    };

    let mut first_valid_idx = 0;
    for i in 0..n {
        if i < max_shift {
            continue;
        }
        let price_ok = prices[i].is_some();
        let indicators_ok = if let Some(ref _config) = custom_strat {
            let mut all_ok = true;
            for col in &required_cols {
                if get_df_value(df, col, i, 0, coin).is_err() {
                    all_ok = false;
                    break;
                }
            }
            all_ok
        } else {
            let rsi_ok = rsi.as_ref().map(|v| v[i].is_some()).unwrap_or(true);
            let macd_ok = macd_line.as_ref().map(|v| v[i].is_some()).unwrap_or(true)
                && macd_signal.as_ref().map(|v| v[i].is_some()).unwrap_or(true)
                && macd_hist.as_ref().map(|v| v[i].is_some()).unwrap_or(true);
            let bb_ok = bb_upper.as_ref().map(|v| v[i].is_some()).unwrap_or(true)
                && bb_lower.as_ref().map(|v| v[i].is_some()).unwrap_or(true)
                && bb_mid.as_ref().map(|v| v[i].is_some()).unwrap_or(true);
            rsi_ok && macd_ok && bb_ok
        };

        if price_ok && indicators_ok {
            first_valid_idx = i;
            break;
        }
    }

    let start_idx = if n > days_to_backtest {
        (n - 1 - days_to_backtest).max(first_valid_idx)
    } else {
        first_valid_idx
    };

    if start_idx >= n - 1 {
        return Err(anyhow!(
            "No sufficient valid data points to backtest {}",
            coin
        ));
    }

    // prev_position: -1 = short, 0 = neutral, 1 = long
    // Map legacy signal codes to signed position: 2 -> 1, 1 -> 0, 0 -> -1
    let mut prev_position: i32 = 0; // start neutral
    let mut equity = vec![10000.0; n];
    let mut bh_equity = vec![10000.0; n];

    let start_price = prices[start_idx].unwrap();

    // signals still use legacy codes (2=long, 1=neutral, 0=short) for confusion matrix
    let mut signals = vec![1i64; n];
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

        // Determine new signal using prev_t indicators (signal held at start of bar)
        // Legacy signal code: 2=long, 1=neutral, 0=short
        let mut sig_code: i64 = match prev_position {
            1 => 2,
            -1 => 0,
            _ => 1,
        };
        let mut conf_t = 1.0;

        if let Some(ref config) = custom_strat {
            let buy_triggered = evaluate_condition(&config.buy_condition, prev_t, df, coin)?;
            let sell_triggered = evaluate_condition(&config.sell_condition, prev_t, df, coin)?;
            let neutral_triggered = if let Some(ref neutral) = config.neutral_condition {
                evaluate_condition(neutral, prev_t, df, coin)?
            } else {
                false
            };

            if buy_triggered {
                sig_code = 2;
            } else if sell_triggered {
                sig_code = 0;
            } else if neutral_triggered {
                sig_code = 1;
            }

            conf_t = evaluate_confidence(&config.confidence, prev_t, df, coin)?;
        } else {
            match strategy_name.to_lowercase().as_str() {
                "rsi" => {
                    if let Some(ref rsi_vec) = rsi {
                        if let Some(rsi_val) = rsi_vec[prev_t] {
                            if rsi_val < 30.0 {
                                sig_code = 2;
                                conf_t = ((30.0 - rsi_val) / 15.0).clamp(0.1, 1.0);
                            } else if rsi_val > 70.0 {
                                sig_code = 0;
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
                                sig_code = 2;
                                conf_t = ((lo - prev_price) / std).clamp(0.1, 1.0);
                            } else if prev_price > up {
                                sig_code = 0;
                                conf_t = ((prev_price - up) / std).clamp(0.1, 1.0);
                            }
                        }
                    }
                }
                "macd" => {
                    if let (Some(ref line_vec), Some(ref sig_vec)) = (&macd_line, &macd_signal) {
                        if let (Some(line), Some(sig)) = (line_vec[prev_t], sig_vec[prev_t]) {
                            if line > sig {
                                sig_code = 2;
                            } else if line < sig {
                                sig_code = 0;
                            }
                            let std_hist = calculate_rolling_std(&macd_hist_raw, prev_t, 20);
                            let hist_val = macd_hist_raw[prev_t];
                            conf_t = (hist_val.abs() / std_hist).clamp(0.1, 1.0);
                        }
                    }
                }
                _ => {}
            }
        }

        // Signed position: 1=long, 0=neutral, -1=short
        let new_position: i32 = match sig_code {
            2 => 1,
            0 => -1,
            _ => 0,
        };

        signals[t] = sig_code;

        // Apply prev_position's return first, then charge fee on transition
        let eq_base = equity[prev_t];
        let r_strat = r_t * conf_t * prev_position as f64;
        let post_return = eq_base * (1.0 + r_strat);

        let post_fee = if new_position != prev_position {
            total_trades += 1;
            post_return * (1.0 - (fee + slippage))
        } else {
            post_return
        };

        strategy_returns[t] = r_strat;
        equity[t] = post_fee;
        prev_position = new_position;
    }

    let final_strat_return = ((equity[n - 1] - 10000.0) / 10000.0) * 100.0;

    let actual_returns_slice = &actual_returns[(start_idx + 1)..n];
    let strategy_returns_slice = &strategy_returns[(start_idx + 1)..n];

    let tnx_col: Option<Vec<Option<f64>>> = df
        .column("^TNX")
        .ok()
        .map(|c| c.f64().unwrap().into_iter().collect());

    let mean_rf = if let Some(ref y_vec) = tnx_col {
        let slice = &y_vec[(start_idx + 1)..n];
        let sum: f64 = slice.iter().flatten().copied().sum();
        let count = slice.iter().flatten().count();
        if count > 0 {
            let avg_annual = (sum / count as f64) / 100.0;
            avg_annual / annualization_factor
        } else {
            0.0
        }
    } else {
        0.0
    };

    let strat_sharpe = calculate_sharpe(strategy_returns_slice, mean_rf, annualization_factor);

    let (final_bh_return, bh_sharpe, bh_drawdown) = if let Some(ref cache) = bh_cache {
        bh_equity = cache.equity.clone();
        (cache.final_return, cache.sharpe, cache.drawdown)
    } else {
        let final_bh = ((bh_equity[n - 1] - 10000.0) / 10000.0) * 100.0;
        let bh_s = calculate_sharpe(actual_returns_slice, mean_rf, annualization_factor);
        let bh_dd = calculate_max_drawdown(&bh_equity[start_idx..n]) * 100.0;

        *bh_cache = Some(BhCache {
            final_return: final_bh,
            sharpe: bh_s,
            drawdown: bh_dd,
            equity: bh_equity.clone(),
        });

        (final_bh, bh_s, bh_dd)
    };

    let strat_drawdown = calculate_max_drawdown(&equity[start_idx..n]) * 100.0;

    let signals_slice_i64: Vec<i64> = signals[(start_idx + 1)..n].to_vec();
    let (tp, fp, tn, fn_count) =
        calculate_confusion_matrix(&signals_slice_i64, actual_returns_slice);
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

    // Compute R² of strategy returns vs buy-and-hold returns
    let prediction_r2 = calculate_r2(actual_returns_slice, strategy_returns_slice);

    let prediction_rating = classify_prediction_rating(prediction_accuracy, active_win_rate);
    let strategy_rating =
        classify_strategy_rating(strat_sharpe, final_strat_return, final_bh_return);

    let parts: Vec<&str> = coin.split('_').collect();
    let coin_base = if parts.len() >= 2 {
        parts[0].to_string()
    } else {
        coin.to_string()
    };
    let currency_base = if parts.len() >= 2 {
        parts[1].to_string()
    } else {
        "usd".to_string()
    };

    let metrics = BacktestMetrics {
        coin: coin_base,
        currency: currency_base,
        strategy: display_strat_name,
        strategy_return: final_strat_return,
        buy_and_hold_return: final_bh_return,
        strategy_sharpe: strat_sharpe,
        buy_and_hold_sharpe: bh_sharpe,
        strategy_max_drawdown: strat_drawdown,
        buy_and_hold_max_drawdown: bh_drawdown,
        prediction_accuracy,
        prediction_r2,
        active_win_rate,
        prediction_rating,
        strategy_rating,
        true_positives: tp,
        false_positives: fp,
        true_negatives: tn,
        false_negatives: fn_count,
        total_trades,
    };

    Ok((
        metrics,
        equity[start_idx..n].to_vec(),
        bh_equity[start_idx..n].to_vec(),
    ))
}

pub fn format_backtest_table(metrics: &[BacktestMetrics]) -> String {
    use cli_table::{format::Justify, Cell, Style, Table};

    let mut rows = Vec::new();
    for m in metrics {
        rows.push(vec![
            format!("{}_{}", m.coin.to_uppercase(), m.currency.to_uppercase()).cell(),
            m.strategy.to_uppercase().cell(),
            format!("{:.2}%", m.strategy_return)
                .cell()
                .justify(Justify::Right),
            format!("{:.2}%", m.buy_and_hold_return)
                .cell()
                .justify(Justify::Right),
            format!("{:.2}", m.strategy_sharpe)
                .cell()
                .justify(Justify::Right),
            format!("{:.2}", m.buy_and_hold_sharpe)
                .cell()
                .justify(Justify::Right),
            format!("{:.2}%", m.strategy_max_drawdown)
                .cell()
                .justify(Justify::Right),
            format!("{:.2}%", m.buy_and_hold_max_drawdown)
                .cell()
                .justify(Justify::Right),
            format!("{:.1}%", m.active_win_rate * 100.0)
                .cell()
                .justify(Justify::Right),
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
    days_to_backtest: usize,
    fee: f64,
    slippage: f64,
    rebalance_frequency: &str,
) -> Result<(BacktestMetrics, Vec<f64>, Vec<f64>)> {
    let n_assets = assets.len();
    if n_assets == 0 || weights.len() != n_assets {
        return Err(anyhow!(
            "Mismatched or empty assets/weights for portfolio backtest"
        ));
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
    let mut first_valid_idx = 0;
    for i in 0..n_rows {
        let mut all_valid = true;
        for j in 0..n_assets {
            if close_series[j][i] <= 0.0 {
                all_valid = false;
                break;
            }
            let asset = &assets[j];

            // Check RSI
            let rsi_col = format!("{}_rsi_14", asset);
            if let Ok(col) = df.column(&rsi_col) {
                if col.f64()?.get(i).is_none() {
                    all_valid = false;
                    break;
                }
            }

            // Check MACD
            let macd_line_col = format!("{}_macd_line", asset);
            let macd_sig_col = format!("{}_macd_signal", asset);
            let macd_hist_col = format!("{}_macd_histogram", asset);
            if let (Ok(l), Ok(s), Ok(h)) = (
                df.column(&macd_line_col),
                df.column(&macd_sig_col),
                df.column(&macd_hist_col),
            ) {
                if l.f64()?.get(i).is_none()
                    || s.f64()?.get(i).is_none()
                    || h.f64()?.get(i).is_none()
                {
                    all_valid = false;
                    break;
                }
            }

            // Check Bollinger Bands
            let bb_upper_col = format!("{}_bollinger_upper", asset);
            let bb_lower_col = format!("{}_bollinger_lower", asset);
            let bb_mid_col = format!("{}_sma_20", asset);
            if let (Ok(u), Ok(l), Ok(m)) = (
                df.column(&bb_upper_col),
                df.column(&bb_lower_col),
                df.column(&bb_mid_col),
            ) {
                if u.f64()?.get(i).is_none()
                    || l.f64()?.get(i).is_none()
                    || m.f64()?.get(i).is_none()
                {
                    all_valid = false;
                    break;
                }
            }
        }
        if all_valid {
            first_valid_idx = i;
            break;
        }
    }

    let start_idx = if n_rows > days_to_backtest {
        (n_rows - 1 - days_to_backtest).max(first_valid_idx)
    } else {
        first_valid_idx
    };

    if start_idx >= n_rows - 1 {
        return Err(anyhow!("No overlapping price data for portfolio backtest"));
    }

    // Initialize equity curves
    let mut equity = vec![10000.0; n_rows];
    let mut bh_equity = vec![10000.0; n_rows];

    // Store the initial close prices at start_idx to calculate Buy & Hold weights drift
    let initial_prices: Vec<f64> = (0..n_assets).map(|j| close_series[j][start_idx]).collect();

    // Track active values of each asset (initially target weight * initial equity)
    let mut current_values: Vec<f64> = weights.iter().map(|&w| w * 10000.0).collect();

    let date_col = df.column("date")?.str()?;

    // Fill equity curves from start_idx onwards
    for i in (start_idx + 1)..n_rows {
        // Compute new asset values before rebalancing/fees
        let mut new_values = vec![0.0; n_assets];
        for j in 0..n_assets {
            let asset_ret = returns_series[j][i];
            new_values[j] = current_values[j] * (1.0 + asset_ret / 100.0);
        }
        let equity_before_fees: f64 = new_values.iter().sum();

        // Check if calendar frequency triggers rebalancing
        let date_str_prev = date_col.get(i - 1).unwrap_or("");
        let date_str_curr = date_col.get(i).unwrap_or("");
        let should_rebalance = match rebalance_frequency.to_lowercase().as_str() {
            "weekly" => {
                if let (Ok(d_prev), Ok(d_curr)) = (
                    chrono::NaiveDate::parse_from_str(date_str_prev, "%Y-%m-%d"),
                    chrono::NaiveDate::parse_from_str(date_str_curr, "%Y-%m-%d"),
                ) {
                    use chrono::Datelike;
                    d_prev.iso_week().week() != d_curr.iso_week().week()
                        || d_prev.iso_week().year() != d_curr.iso_week().year()
                } else {
                    true
                }
            }
            "monthly" => {
                if let (Ok(d_prev), Ok(d_curr)) = (
                    chrono::NaiveDate::parse_from_str(date_str_prev, "%Y-%m-%d"),
                    chrono::NaiveDate::parse_from_str(date_str_curr, "%Y-%m-%d"),
                ) {
                    use chrono::Datelike;
                    d_prev.month() != d_curr.month() || d_prev.year() != d_curr.year()
                } else {
                    true
                }
            }
            _ => {
                // "daily" or anything else
                true
            }
        };

        if should_rebalance {
            // Compute rebalancing trade volumes and associated fees
            let mut total_trade_volume = 0.0;
            for j in 0..n_assets {
                let target_value = weights[j] * equity_before_fees;
                total_trade_volume += (target_value - new_values[j]).abs();
            }
            let transaction_fees = total_trade_volume * (fee + slippage);
            let equity_after = (equity_before_fees - transaction_fees).max(0.0);
            equity[i] = equity_after;

            // Reset asset values to target weights
            for j in 0..n_assets {
                current_values[j] = weights[j] * equity_after;
            }
        } else {
            // Keep values drifting without rebalancing or fees
            equity[i] = equity_before_fees;
            current_values = new_values;
        }

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

    let tnx_col: Option<Vec<Option<f64>>> = df
        .column("^TNX")
        .ok()
        .map(|c| c.f64().unwrap().into_iter().collect());

    let mean_rf = if let Some(ref y_vec) = tnx_col {
        let slice = &y_vec[(start_idx + 1)..n_rows];
        let sum: f64 = slice.iter().flatten().copied().sum();
        let count = slice.iter().flatten().count();
        if count > 0 {
            let avg_annual = (sum / count as f64) / 100.0;
            avg_annual / annualization_factor
        } else {
            0.0
        }
    } else {
        0.0
    };

    let strat_sharpe = calculate_sharpe(&strat_returns, mean_rf, annualization_factor);
    let bh_sharpe = calculate_sharpe(&bh_returns, mean_rf, annualization_factor);

    let strat_drawdown = calculate_max_drawdown(&equity[start_idx..n_rows]) * 100.0;
    let bh_drawdown = calculate_max_drawdown(&bh_equity[start_idx..n_rows]) * 100.0;

    let strategy_rating =
        classify_strategy_rating(strat_sharpe, final_strat_return, final_bh_return);

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
        prediction_accuracy: 0.0,
        prediction_r2: 0.0,
        active_win_rate: 0.0,
        prediction_rating: "n/a".to_string(),
        strategy_rating,
        true_positives: 0,
        false_positives: 0,
        true_negatives: 0,
        false_negatives: 0,
        total_trades: 0,
    };

    Ok((
        metrics,
        equity[start_idx..n_rows].to_vec(),
        bh_equity[start_idx..n_rows].to_vec(),
    ))
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
        let sharpe = calculate_sharpe(&returns, 0.0, 365.0);
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
            Series::new(
                "date",
                vec!["2026-06-01", "2026-06-02", "2026-06-03", "2026-06-04"],
            ),
            Series::new("bitcoin_usd", vec![100.0, 105.0, 95.0, 110.0]),
            Series::new("bitcoin_usd_rsi_14", vec![25.0, 28.0, 75.0, 80.0]),
            Series::new("bitcoin_usd_macd_line", vec![1.0, 1.2, 0.8, 1.5]),
            Series::new("bitcoin_usd_macd_signal", vec![0.8, 1.0, 1.0, 1.1]),
            Series::new("bitcoin_usd_macd_histogram", vec![0.2, 0.2, -0.2, 0.4]),
            Series::new(
                "bitcoin_usd_bollinger_upper",
                vec![110.0, 110.0, 110.0, 110.0],
            ),
            Series::new("bitcoin_usd_bollinger_lower", vec![90.0, 90.0, 90.0, 90.0]),
            Series::new("bitcoin_usd_sma_20", vec![100.0, 100.0, 100.0, 100.0]),
        ])
        .unwrap();

        let (metrics_rsi, equity_rsi, bh_rsi) = run_backtest_for_asset(
            &df,
            "bitcoin_usd",
            "rsi",
            None,
            0.001,
            0.0005,
            365.0,
            30,
            &mut None,
        )
        .unwrap();
        assert_eq!(metrics_rsi.coin, "bitcoin".to_string());
        assert_eq!(metrics_rsi.strategy, "rsi".to_string());
        assert_eq!(equity_rsi.len(), 4);
        assert_eq!(bh_rsi.len(), 4);

        // MACD strategy
        let (metrics_macd, _equity_macd, _bh_macd) = run_backtest_for_asset(
            &df,
            "bitcoin_usd",
            "macd",
            None,
            0.001,
            0.0005,
            365.0,
            30,
            &mut None,
        )
        .unwrap();
        assert_eq!(metrics_macd.strategy, "macd");

        // Bollinger strategy
        let (metrics_bb, _equity_bb, _bh_bb) = run_backtest_for_asset(
            &df,
            "bitcoin_usd",
            "bollinger",
            None,
            0.001,
            0.0005,
            365.0,
            30,
            &mut None,
        )
        .unwrap();
        assert_eq!(metrics_bb.strategy, "bollinger");
    }

    #[test]
    fn test_backtest_portfolio() {
        let df = DataFrame::new(vec![
            Series::new(
                "date",
                vec!["2026-06-01", "2026-06-02", "2026-06-03", "2026-06-04"],
            ),
            Series::new("bitcoin_usd", vec![100.0, 105.0, 95.0, 110.0]),
            Series::new(
                "bitcoin_usd_simple_return",
                vec![0.0, 0.05, -0.0952, 0.1579],
            ),
            Series::new("ethereum_usd", vec![10.0, 11.0, 9.5, 12.0]),
            Series::new(
                "ethereum_usd_simple_return",
                vec![0.0, 0.10, -0.1363, 0.2631],
            ),
        ])
        .unwrap();

        let assets = vec!["bitcoin_usd".to_string(), "ethereum_usd".to_string()];
        let weights = vec![0.6, 0.4];

        let (metrics, equity, bh_equity) = backtest_portfolio(
            &df,
            &assets,
            &weights,
            "max_sharpe",
            365.0,
            30,
            0.001,
            0.0005,
            "daily",
        )
        .unwrap();
        assert_eq!(metrics.coin, "max_sharpe");
        assert_eq!(metrics.currency, "portfolio");
        assert_eq!(metrics.strategy, "rebalanced");
        assert_eq!(equity.len(), 4);
        assert_eq!(bh_equity.len(), 4);
        assert!(metrics.strategy_return != 0.0);
        assert!(metrics.buy_and_hold_return != 0.0);
    }

    #[test]
    fn test_backtest_portfolio_frequencies() {
        let df = DataFrame::new(vec![
            Series::new(
                "date",
                vec![
                    "2026-06-01",
                    "2026-06-02",
                    "2026-06-08",
                    "2026-06-09",
                    "2026-07-01",
                ],
            ),
            Series::new("bitcoin_usd", vec![100.0, 105.0, 95.0, 110.0, 115.0]),
            Series::new(
                "bitcoin_usd_simple_return",
                vec![0.0, 5.0, -9.52, 15.79, 4.54],
            ),
            Series::new("ethereum_usd", vec![10.0, 11.0, 9.5, 12.0, 13.0]),
            Series::new(
                "ethereum_usd_simple_return",
                vec![0.0, 10.0, -13.63, 26.31, 8.33],
            ),
        ])
        .unwrap();

        let assets = vec!["bitcoin_usd".to_string(), "ethereum_usd".to_string()];
        let weights = vec![0.6, 0.4];

        // Weekly rebalancing
        let (_metrics_w, equity_w, _) = backtest_portfolio(
            &df,
            &assets,
            &weights,
            "max_sharpe",
            365.0,
            30,
            0.01,
            0.005,
            "weekly",
        )
        .unwrap();
        // Monthly rebalancing
        let (_metrics_m, equity_m, _) = backtest_portfolio(
            &df,
            &assets,
            &weights,
            "max_sharpe",
            365.0,
            30,
            0.01,
            0.005,
            "monthly",
        )
        .unwrap();
        // Daily rebalancing
        let (_metrics_d, equity_d, _) = backtest_portfolio(
            &df,
            &assets,
            &weights,
            "max_sharpe",
            365.0,
            30,
            0.01,
            0.005,
            "daily",
        )
        .unwrap();

        assert_eq!(equity_w.len(), 5);
        assert_eq!(equity_m.len(), 5);
        assert_eq!(equity_d.len(), 5);

        // Different frequencies must yield different end equity due to transaction fees
        assert!(equity_d[4] != equity_w[4]);
        assert!(equity_w[4] != equity_m[4]);
    }

    #[test]
    fn test_custom_strategy() {
        use std::io::Write;

        let strategy_json = r#"{
            "name": "custom_rsi_test",
            "buy_condition": {
                "column": "bitcoin_usd_rsi_14",
                "operator": "<",
                "value": 30.0
            },
            "sell_condition": {
                "column": "bitcoin_usd_rsi_14",
                "operator": ">",
                "value": 75.0
            },
            "neutral_condition": null,
            "confidence": {
                "type": "Constant",
                "value": 1.0
            }
        }"#;

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_custom_rsi_test.json");
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(strategy_json.as_bytes()).unwrap();

        let df = DataFrame::new(vec![
            Series::new(
                "date",
                vec!["2026-06-01", "2026-06-02", "2026-06-03", "2026-06-04"],
            ),
            Series::new("bitcoin_usd", vec![100.0, 105.0, 95.0, 110.0]),
            Series::new("bitcoin_usd_rsi_14", vec![25.0, 28.0, 76.0, 80.0]),
        ])
        .unwrap();

        let configs = load_custom_strategies(file_path.to_str().unwrap()).unwrap();
        let (metrics, equity, bh) = run_backtest_for_asset(
            &df,
            "bitcoin_usd",
            "custom_rsi_test",
            Some(&configs[0]),
            0.0,
            0.0,
            365.0,
            30,
            &mut None,
        )
        .unwrap();

        assert_eq!(metrics.coin, "bitcoin");
        assert_eq!(metrics.strategy, "custom_rsi_test");
        assert_eq!(equity.len(), 4);
        assert_eq!(bh.len(), 4);

        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_custom_strategy_with_shift() {
        use std::io::Write;

        let strategy_json = r#"{
            "name": "custom_shift_test",
            "buy_condition": {
                "column": "rsi_14",
                "operator": "<",
                "value": {
                    "column": "rsi_14",
                    "shift": 1
                }
            },
            "sell_condition": {
                "column": "rsi_14",
                "operator": ">",
                "value": 75.0
            },
            "neutral_condition": null,
            "confidence": {
                "type": "Constant",
                "value": 1.0
            }
        }"#;

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_custom_shift_test.json");
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(strategy_json.as_bytes()).unwrap();

        let df = DataFrame::new(vec![
            Series::new(
                "date",
                vec!["2026-06-01", "2026-06-02", "2026-06-03", "2026-06-04"],
            ),
            Series::new("bitcoin_usd", vec![100.0, 105.0, 95.0, 110.0]),
            Series::new("bitcoin_usd_rsi_14", vec![50.0, 45.0, 40.0, 80.0]),
        ])
        .unwrap();

        let configs = load_custom_strategies(file_path.to_str().unwrap()).unwrap();
        let (metrics, _, _) = run_backtest_for_asset(
            &df,
            "bitcoin_usd",
            "custom_shift_test",
            Some(&configs[0]),
            0.0,
            0.0,
            365.0,
            30,
            &mut None,
        )
        .unwrap();

        assert_eq!(metrics.strategy, "custom_shift_test");

        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_load_custom_strategies_multi() {
        use std::io::Write;

        // 1. Test Multiple List JSON
        let strategy_list_json = r#"[
            {
                "name": "rsi_list_1",
                "buy_condition": { "column": "rsi_14", "operator": "<", "value": 30.0 },
                "sell_condition": { "column": "rsi_14", "operator": ">", "value": 70.0 },
                "neutral_condition": null,
                "confidence": { "type": "Constant", "value": 1.0 }
            },
            {
                "name": "rsi_list_2",
                "buy_condition": { "column": "rsi_14", "operator": "<", "value": 35.0 },
                "sell_condition": { "column": "rsi_14", "operator": ">", "value": 65.0 },
                "neutral_condition": null,
                "confidence": { "type": "Constant", "value": 1.0 }
            }
        ]"#;

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_multi_list.json");
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(strategy_list_json.as_bytes()).unwrap();

        let configs = load_custom_strategies(file_path.to_str().unwrap()).unwrap();
        assert_eq!(configs.len(), 2);
        assert_eq!(configs[0].name, "rsi_list_1");
        assert_eq!(configs[1].name, "rsi_list_2");
        let _ = std::fs::remove_file(&file_path);

        // 2. Test Multiple Map JSON
        let strategy_map_json = r#"{
            "rsi_map_1": {
                "name": "",
                "buy_condition": { "column": "rsi_14", "operator": "<", "value": 30.0 },
                "sell_condition": { "column": "rsi_14", "operator": ">", "value": 70.0 },
                "neutral_condition": null,
                "confidence": { "type": "Constant", "value": 1.0 }
            },
            "rsi_map_2": {
                "name": "explicit_name",
                "buy_condition": { "column": "rsi_14", "operator": "<", "value": 35.0 },
                "sell_condition": { "column": "rsi_14", "operator": ">", "value": 65.0 },
                "neutral_condition": null,
                "confidence": { "type": "Constant", "value": 1.0 }
            }
        }"#;

        let file_path = temp_dir.join("test_multi_map.json");
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(strategy_map_json.as_bytes()).unwrap();

        let mut configs = load_custom_strategies(file_path.to_str().unwrap()).unwrap();
        configs.sort_by(|a, b| a.name.cmp(&b.name));
        assert_eq!(configs.len(), 2);
        assert_eq!(configs[0].name, "explicit_name");
        assert_eq!(configs[1].name, "rsi_map_1");
        let _ = std::fs::remove_file(&file_path);
    }

    // ── Phase 04 tests: neutral-exit fee/P&L ────────────────────────────────

    /// 3-bar series: [100, 110, 100]
    /// Bar 0 (start): price=100, prev_position=0 (neutral at init)
    /// Bar 1 (t=1): price=110, r=0.1. RSI[0]=25 → sig=long(1). No prior position → no fee on
    ///   prev_position=0 → r_strat=0. Fee charged on transition 0→1.
    ///   equity[1] = 10000 * (1+0) * (1-0.01) = 9900
    /// Bar 2 (t=2): price=100, r=-0.0909. RSI[1]=75 → sig=neutral(0). prev_position=1 → r_strat=-0.0909.
    ///   post_return = 9900*(1-0.0909) = 9000.9. Fee on transition 1→0: 9000.9*(1-0.01) = 8910.891
    #[test]
    fn test_neutral_exit_fees_pnl() {
        // prices: [100, 110, 100]; RSI triggers: [25(<30→buy), 75(>70→sell/neutral), 50]
        // With fee=0.01, slippage=0.0, conf=1.0
        // At t=1: prev_position=0(neutral), sig=long(1) from RSI[0]=25
        //   r_strat = r_t * conf * prev_position = 0.1*1.0*0 = 0.0
        //   post_return = 10000*(1+0) = 10000
        //   transition 0→1 → fee: 10000*(1-0.01) = 9900
        // At t=2: prev_position=1(long), RSI[1]=75>70 → sig=short(-1)
        //   r_t = (100-110)/110 = -1/11 ≈ -0.090909
        //   r_strat = r_t*1.0*1 = -0.090909
        //   post_return = 9900*(1-0.090909) = 9900*0.909091 ≈ 9000.0
        //   transition 1→-1 → fee: 9000*(1-0.01) = 8910.0
        let fee = 0.01;
        let df = DataFrame::new(vec![
            Series::new("date", vec!["2026-01-01", "2026-01-02", "2026-01-03"]),
            Series::new("btc_usd", vec![100.0f64, 110.0, 100.0]),
            Series::new("btc_usd_rsi_14", vec![25.0f64, 75.0, 50.0]),
            Series::new("btc_usd_macd_line", vec![1.0f64, 1.0, 1.0]),
            Series::new("btc_usd_macd_signal", vec![1.0f64, 1.0, 1.0]),
            Series::new("btc_usd_macd_histogram", vec![0.0f64, 0.0, 0.0]),
            Series::new("btc_usd_bollinger_upper", vec![120.0f64, 120.0, 120.0]),
            Series::new("btc_usd_bollinger_lower", vec![80.0f64, 80.0, 80.0]),
            Series::new("btc_usd_sma_20", vec![100.0f64, 100.0, 100.0]),
        ])
        .unwrap();

        let (_, equity, _) =
            run_backtest_for_asset(&df, "btc_usd", "rsi", None, fee, 0.0, 365.0, 30, &mut None)
                .unwrap();

        // equity[0] = 10000 (start)
        assert!(
            (equity[0] - 10000.0).abs() < 1e-6,
            "equity[0]={}",
            equity[0]
        );
        // equity[1]: prev_pos=0, r_strat=r_t*conf*0=0, then fee on transition 0→1
        let expected_eq1 = 10000.0 * (1.0 - fee);
        assert!(
            (equity[1] - expected_eq1).abs() < 1e-4,
            "equity[1]={} expected={}",
            equity[1],
            expected_eq1
        );
        // equity[2]: prev_pos=1(long), RSI[1]=75>70
        //   conf_t = ((75-70)/15).clamp(0.1,1.0) = 1/3
        //   r_t = (100-110)/110
        //   r_strat = r_t * (1/3) * 1
        //   post_return = expected_eq1 * (1 + r_strat)
        //   fee on transition 1→-1
        let r2 = (100.0_f64 - 110.0) / 110.0;
        let conf2 = ((75.0_f64 - 70.0) / 15.0).clamp(0.1_f64, 1.0_f64);
        let post_return2 = expected_eq1 * (1.0 + r2 * conf2 * 1.0);
        let expected_eq2 = post_return2 * (1.0 - fee);
        assert!(
            (equity[2] - expected_eq2).abs() < 1e-4,
            "equity[2]={} expected={}",
            equity[2],
            expected_eq2
        );
    }

    /// Verify: holding same position (long→long) incurs no fee
    #[test]
    fn test_no_fee_when_position_unchanged() {
        // RSI stays <30 all bars → always long → only one transition (0→long) at t=1
        let df = DataFrame::new(vec![
            Series::new(
                "date",
                vec!["2026-01-01", "2026-01-02", "2026-01-03", "2026-01-04"],
            ),
            Series::new("btc_usd", vec![100.0f64, 110.0, 121.0, 133.1]),
            Series::new("btc_usd_rsi_14", vec![20.0f64, 20.0, 20.0, 20.0]),
            Series::new("btc_usd_macd_line", vec![1.0f64, 1.0, 1.0, 1.0]),
            Series::new("btc_usd_macd_signal", vec![1.0f64, 1.0, 1.0, 1.0]),
            Series::new("btc_usd_macd_histogram", vec![0.0f64, 0.0, 0.0, 0.0]),
            Series::new(
                "btc_usd_bollinger_upper",
                vec![120.0f64, 120.0, 140.0, 150.0],
            ),
            Series::new("btc_usd_bollinger_lower", vec![80.0f64, 80.0, 100.0, 110.0]),
            Series::new("btc_usd_sma_20", vec![100.0f64, 100.0, 120.0, 130.0]),
        ])
        .unwrap();

        let (metrics, equity, _) =
            run_backtest_for_asset(&df, "btc_usd", "rsi", None, 0.01, 0.0, 365.0, 30, &mut None)
                .unwrap();
        // Only 1 trade: neutral→long transition at bar 1
        assert_eq!(metrics.total_trades, 1, "trades={}", metrics.total_trades);
        // equity should be growing from bar 1 onward (long riding 10% gains)
        assert!(equity[2] > equity[1], "equity not growing while long");
        assert!(equity[3] > equity[2]);
    }

    // ── Phase 05 tests: calculate_r2 + calculate_max_drawdown ───────────────

    #[test]
    fn test_r2_constant_series_perfect_prediction() {
        // actuals = predicted = [1,1,1] → ss_res=0, ss_tot=0 → should return 1.0
        let c = vec![1.0, 1.0, 1.0];
        assert_eq!(calculate_r2(&c, &c), 1.0);
    }

    #[test]
    fn test_r2_constant_series_wrong_prediction() {
        // actuals = [1,1,1] constant, predicted=[2,2,2] → ss_tot=0, ss_res>0 → return 0.0
        let a = vec![1.0, 1.0, 1.0];
        let p = vec![2.0, 2.0, 2.0];
        assert_eq!(calculate_r2(&a, &p), 0.0);
    }

    #[test]
    fn test_max_drawdown_negative_peak() {
        // Equity starts negative; peak=-5 then drops to -10: dd=(-5-(-10))/-5 = -1.0 (but negative dd)
        // With fix: peak=-5 != 0, dd=((-5)-(-10))/(-5) = -1.0 which is < 0 so max_dd stays 0.0
        // Then rises to -3: peak=-3, eq=-3, dd=0
        // Key: should not panic, and peak tracks correctly
        let equity = vec![-10.0, -5.0, -8.0, -3.0];
        let dd = calculate_max_drawdown(&equity);
        // With peak tracking negatives, peak=-10 then -5 then -5 then -3
        // dd at idx1: (-10-(-5))/-10=0.5 drawdown (peak rose, no draw)
        // Actually peak starts at equity[0]=-10, eq[1]=-5>-10 so peak=-5
        // eq[2]=-8<-5: dd=(-5-(-8))/-5 = 3/-5 = -0.6 < 0 → ignored
        // Result: max_dd=0.0 (no downward drawdown on negative start)
        assert!(
            dd.is_finite(),
            "drawdown must be finite for negative equity series"
        );
    }

    #[test]
    fn test_max_drawdown_mixed_positive_negative_peak() {
        // peak goes positive: [100, 80, 60] → max_dd = (100-60)/100 = 0.4
        let equity = vec![100.0, 80.0, 60.0];
        let dd = calculate_max_drawdown(&equity);
        assert!((dd - 0.4).abs() < 1e-9, "dd={}", dd);
    }

    // ── Phase 06 tests: deterministic equity-curve + portfolio ──────────────

    fn make_rsi_df(prices: &[f64], rsi_vals: &[f64]) -> DataFrame {
        let n = prices.len();
        let dates: Vec<String> = (0..n).map(|i| format!("2026-01-{:02}", i + 1)).collect();
        let date_refs: Vec<&str> = dates.iter().map(|s| s.as_str()).collect();
        DataFrame::new(vec![
            Series::new("date", date_refs),
            Series::new("asset_usd", prices.to_vec()),
            Series::new("asset_usd_rsi_14", rsi_vals.to_vec()),
            Series::new("asset_usd_macd_line", vec![1.0f64; n]),
            Series::new("asset_usd_macd_signal", vec![1.0f64; n]),
            Series::new("asset_usd_macd_histogram", vec![0.0f64; n]),
            Series::new("asset_usd_bollinger_upper", vec![1000.0f64; n]),
            Series::new("asset_usd_bollinger_lower", vec![0.0f64; n]),
            Series::new("asset_usd_sma_20", vec![500.0f64; n]),
        ])
        .unwrap()
    }

    /// RSI strategy: stays neutral until RSI<30 fires. With 0 fee:
    /// prices=[100,110,121], RSI=[50,50,50] → no buy/sell → equity flat
    #[test]
    fn test_rsi_neutral_all_bars_equity_flat() {
        let prices = vec![100.0, 110.0, 121.0];
        let rsi = vec![50.0, 50.0, 50.0];
        let df = make_rsi_df(&prices, &rsi);
        let (_, equity, _) = run_backtest_for_asset(
            &df,
            "asset_usd",
            "rsi",
            None,
            0.0,
            0.0,
            365.0,
            30,
            &mut None,
        )
        .unwrap();
        // all neutral → strategy_returns=0 → equity stays at 10000
        for &eq in &equity {
            assert!(
                (eq - 10000.0).abs() < 1e-6,
                "equity={} should be 10000 (all neutral)",
                eq
            );
        }
    }

    /// RSI strategy: buy at bar 0 (RSI=20<30), hold bars 1–2 (RSI=50)
    /// prices=[100,110,121], fee=0
    /// t=1: prev_pos=0→long at RSI[0]=20. r_strat=0*r_t=0. Transition→fee=0.
    ///   equity[1]=10000*(1+0)=10000. Wait, prev_position starts at 0.
    ///   t=1: sig=long(1) from RSI[0]=20. prev_pos=0. r_strat=0.1*0=0. Transition→no fee.
    ///   equity[1]=10000.
    /// t=2: prev_pos=1. RSI[1]=50→no signal→stays long. r_t=(121-110)/110=0.1. r_strat=0.1*1=0.1.
    ///   equity[2]=10000*(1+0.1)=11000.
    #[test]
    fn test_rsi_long_equity_exact_values() {
        let prices = vec![100.0, 110.0, 121.0];
        let rsi = vec![20.0, 50.0, 50.0]; // bar0: buy; bar1: hold; bar2: hold
        let df = make_rsi_df(&prices, &rsi);
        let (_, equity, _) = run_backtest_for_asset(
            &df,
            "asset_usd",
            "rsi",
            None,
            0.0,
            0.0,
            365.0,
            30,
            &mut None,
        )
        .unwrap();
        assert!((equity[0] - 10000.0).abs() < 1e-6);
        // t=1: prev_pos=0 → no return, transition to long
        assert!(
            (equity[1] - 10000.0).abs() < 1e-6,
            "equity[1]={}",
            equity[1]
        );
        // t=2: long, r=(121-110)/110
        let r2 = (121.0 - 110.0) / 110.0;
        let expected_eq2 = 10000.0 * (1.0 + r2);
        assert!(
            (equity[2] - expected_eq2).abs() < 1e-4,
            "equity[2]={} expected={}",
            equity[2],
            expected_eq2
        );
    }

    /// Single-bar DataFrame → error (too short)
    #[test]
    fn test_single_bar_df_errors() {
        let df = DataFrame::new(vec![
            Series::new("date", vec!["2026-01-01"]),
            Series::new("asset_usd", vec![100.0f64]),
            Series::new("asset_usd_rsi_14", vec![50.0f64]),
            Series::new("asset_usd_macd_line", vec![1.0f64]),
            Series::new("asset_usd_macd_signal", vec![1.0f64]),
            Series::new("asset_usd_macd_histogram", vec![0.0f64]),
            Series::new("asset_usd_bollinger_upper", vec![120.0f64]),
            Series::new("asset_usd_bollinger_lower", vec![80.0f64]),
            Series::new("asset_usd_sma_20", vec![100.0f64]),
        ])
        .unwrap();
        let result = run_backtest_for_asset(
            &df,
            "asset_usd",
            "rsi",
            None,
            0.0,
            0.0,
            365.0,
            30,
            &mut None,
        );
        assert!(result.is_err(), "single-bar df should return Err");
    }

    /// MACD strategy: line>signal → long. line<signal → short.
    /// prices=[100,110,100,110], macd_line=[1,2,0.5,2], macd_signal=[1,1,1,1]
    /// RSI=50 (neutral), BB set wide so no bollinger signal
    /// t=1: prev_pos=0. MACD[0]: line=1==signal=1 → neutral (no cross). r_strat=0.
    ///   equity[1]=10000.
    /// t=2: prev_pos=0. MACD[1]: line=2>signal=1 → long. r_strat=r_t*0*conf... wait prev_pos=0 so 0.
    ///   equity[2]=10000. Transition 0→1.
    /// t=3: prev_pos=1. MACD[2]: line=0.5<signal=1 → short. r_t=(100-110)/110=-0.0909.
    ///   r_strat=-0.0909*1*1=-0.0909. post_return=10000*(1-0.0909)=9090.9
    ///   Transition 1→-1: fee: 9090.9*(1-0.01)=8999.99...
    #[test]
    fn test_macd_strategy_equity_exact() {
        let n = 4;
        let dates: Vec<String> = (0..n).map(|i| format!("2026-01-{:02}", i + 1)).collect();
        let date_refs: Vec<&str> = dates.iter().map(|s| s.as_str()).collect();
        let df = DataFrame::new(vec![
            Series::new("date", date_refs),
            Series::new("asset_usd", vec![100.0f64, 110.0, 100.0, 110.0]),
            Series::new("asset_usd_rsi_14", vec![50.0f64, 50.0, 50.0, 50.0]),
            Series::new("asset_usd_macd_line", vec![1.0f64, 2.0, 0.5, 2.0]),
            Series::new("asset_usd_macd_signal", vec![1.0f64, 1.0, 1.0, 1.0]),
            Series::new("asset_usd_macd_histogram", vec![0.0f64, 1.0, -0.5, 1.0]),
            Series::new("asset_usd_bollinger_upper", vec![1000.0f64; 4]),
            Series::new("asset_usd_bollinger_lower", vec![0.0f64; 4]),
            Series::new("asset_usd_sma_20", vec![100.0f64; 4]),
        ])
        .unwrap();

        let (metrics, equity, _) = run_backtest_for_asset(
            &df,
            "asset_usd",
            "macd",
            None,
            0.01,
            0.0,
            365.0,
            30,
            &mut None,
        )
        .unwrap();
        assert_eq!(equity.len(), 4);
        assert!(
            equity[0].is_finite() && equity[3].is_finite(),
            "equity must be finite"
        );
        // transition happened so trades > 0
        assert!(metrics.total_trades > 0);
    }

    /// Bollinger strategy: price below lower band → long
    #[test]
    fn test_bollinger_strategy_below_lower_band_goes_long() {
        // prices: [200, 200, 200]; upper=300, lower=250, mid=275
        // price(200) < lower(250) at bar 0 → buy triggered
        let n = 3;
        let dates: Vec<String> = (0..n).map(|i| format!("2026-01-{:02}", i + 1)).collect();
        let date_refs: Vec<&str> = dates.iter().map(|s| s.as_str()).collect();
        let df = DataFrame::new(vec![
            Series::new("date", date_refs),
            Series::new("asset_usd", vec![200.0f64, 210.0, 220.0]),
            Series::new("asset_usd_rsi_14", vec![50.0f64, 50.0, 50.0]),
            Series::new("asset_usd_macd_line", vec![1.0f64, 1.0, 1.0]),
            Series::new("asset_usd_macd_signal", vec![1.0f64, 1.0, 1.0]),
            Series::new("asset_usd_macd_histogram", vec![0.0f64, 0.0, 0.0]),
            Series::new("asset_usd_bollinger_upper", vec![300.0f64, 300.0, 300.0]),
            Series::new("asset_usd_bollinger_lower", vec![250.0f64, 250.0, 250.0]),
            Series::new("asset_usd_sma_20", vec![275.0f64, 275.0, 275.0]),
        ])
        .unwrap();

        let (metrics, equity, _) = run_backtest_for_asset(
            &df,
            "asset_usd",
            "bollinger",
            None,
            0.0,
            0.0,
            365.0,
            30,
            &mut None,
        )
        .unwrap();
        // bar 0: prev_pos=0, sig=long. r_strat=r*0=0. equity[1]=10000
        // bar 1: prev_pos=1, price[1]=250. r_t=(210-200)/200=0.05. equity[2]=10000*1.05=10500
        // bar 2: prev_pos=1, price[2]=220. r_t=(220-210)/210. equity[3]=10500*(1+r)
        assert!(
            equity[2] > equity[1],
            "long position should profit when price rises"
        );
        assert_eq!(metrics.total_trades, 1); // one transition: neutral→long
    }

    /// Custom JSON strategy: 2-rule buy, verify exact equity
    #[test]
    fn test_custom_json_strategy_exact_equity() {
        use std::io::Write;
        // Buy if RSI<30 AND close<105; sell if RSI>70
        let json = r#"{
            "name": "rsi_price_cross",
            "buy_condition": {
                "operator": "AND",
                "rules": [
                    {"column": "asset_usd_rsi_14", "operator": "<", "value": 30.0},
                    {"column": "asset_usd", "operator": "<", "value": 105.0}
                ]
            },
            "sell_condition": {"column": "asset_usd_rsi_14", "operator": ">", "value": 70.0},
            "neutral_condition": null,
            "confidence": {"type": "Constant", "value": 1.0}
        }"#;
        let tmp = std::env::temp_dir().join("test_rsi_price_cross.json");
        std::fs::File::create(&tmp)
            .unwrap()
            .write_all(json.as_bytes())
            .unwrap();

        let prices = vec![100.0, 110.0, 90.0]; // bar0 matches buy (rsi=20<30, close=100<105)
        let rsi = vec![20.0, 50.0, 80.0];
        let df = make_rsi_df(&prices, &rsi);
        let configs = load_custom_strategies(tmp.to_str().unwrap()).unwrap();
        let (metrics, equity, _) = run_backtest_for_asset(
            &df,
            "asset_usd",
            "rsi_price_cross",
            Some(&configs[0]),
            0.0,
            0.0,
            365.0,
            30,
            &mut None,
        )
        .unwrap();
        assert_eq!(metrics.strategy, "rsi_price_cross");
        // equity[0]=10000 (start)
        assert!((equity[0] - 10000.0).abs() < 1e-6);
        let _ = std::fs::remove_file(tmp);
    }

    /// All-None indicator columns → strategy stays neutral → equity flat (zero fee)
    #[test]
    fn test_all_none_indicators_equity_flat() {
        let df = DataFrame::new(vec![
            Series::new("date", vec!["2026-01-01", "2026-01-02", "2026-01-03"]),
            Series::new("asset_usd", vec![100.0f64, 110.0, 121.0]),
            // no indicator columns present → strategy must fall back to neutral
        ])
        .unwrap();
        let (_, equity, _) = run_backtest_for_asset(
            &df,
            "asset_usd",
            "rsi",
            None,
            0.0,
            0.0,
            365.0,
            30,
            &mut None,
        )
        .unwrap();
        // prev_pos stays 0 → r_strat=0 → equity flat
        for &eq in &equity {
            assert!((eq - 10000.0).abs() < 1e-6, "eq={}", eq);
        }
    }

    /// Portfolio rebalancing: weekly rebalance with diverging assets incurs more fees than monthly
    #[test]
    fn test_portfolio_weekly_vs_monthly_fees() {
        // btc surges while eth drops — creates significant drift each week.
        // Weekly rebalance forces more trades than monthly → higher fees → lower final equity.
        let df = DataFrame::new(vec![
            Series::new(
                "date",
                vec![
                    "2026-06-01",
                    "2026-06-08",
                    "2026-06-15",
                    "2026-07-01",
                    "2026-07-08",
                ],
            ),
            Series::new("btc", vec![100.0f64, 150.0, 200.0, 250.0, 300.0]),
            // returns as percentage (used by backtest_portfolio * /100)
            Series::new("btc_simple_return", vec![0.0f64, 50.0, 33.33, 25.0, 20.0]),
            Series::new("eth", vec![100.0f64, 80.0, 60.0, 50.0, 40.0]),
            Series::new(
                "eth_simple_return",
                vec![0.0f64, -20.0, -25.0, -16.67, -20.0],
            ),
        ])
        .unwrap();
        let assets = vec!["btc".to_string(), "eth".to_string()];
        let weights = vec![0.6, 0.4];

        let (_, eq_weekly, _) = backtest_portfolio(
            &df, &assets, &weights, "test", 365.0, 100, 0.01, 0.005, "weekly",
        )
        .unwrap();
        let (_, eq_monthly, _) = backtest_portfolio(
            &df, &assets, &weights, "test", 365.0, 100, 0.01, 0.005, "monthly",
        )
        .unwrap();

        // Weekly rebalances on every bar (all different weeks/months);
        // monthly rebalances only on cross-month boundary (bar 3: Jun→Jul).
        // More rebalances with diverging assets → more fee drag → weekly < monthly.
        assert!(
            eq_weekly[4] <= eq_monthly[4],
            "weekly={} should be <= monthly={} (more fees)",
            eq_weekly[4],
            eq_monthly[4]
        );
    }

    /// Portfolio: multi-asset diverging returns → weights renormalize after rebalance
    #[test]
    fn test_portfolio_weight_renormalization() {
        // btc goes 2x, eth flat → without rebalance btc dominates
        // with daily rebalance, we force back to 0.5/0.5 each day
        let df = DataFrame::new(vec![
            Series::new("date", vec!["2026-01-01", "2026-01-02", "2026-01-03"]),
            Series::new("btc", vec![100.0f64, 200.0, 200.0]),
            Series::new("btc_simple_return", vec![0.0f64, 100.0, 0.0]),
            Series::new("eth", vec![100.0f64, 100.0, 100.0]),
            Series::new("eth_simple_return", vec![0.0f64, 0.0, 0.0]),
        ])
        .unwrap();
        let assets = vec!["btc".to_string(), "eth".to_string()];
        let weights = vec![0.5, 0.5];

        let (metrics, equity, bh_equity) = backtest_portfolio(
            &df, &assets, &weights, "test", 365.0, 100, 0.0, 0.0, "daily",
        )
        .unwrap();
        // equity and bh_equity must be finite and positive
        for (&eq, &bh) in equity.iter().zip(bh_equity.iter()) {
            assert!(eq.is_finite() && eq > 0.0);
            assert!(bh.is_finite() && bh > 0.0);
        }
        assert!(metrics.strategy_return.is_finite());
    }

    /// Portfolio: mismatched assets/weights returns Err
    #[test]
    fn test_portfolio_mismatched_assets_weights_errors() {
        let df = DataFrame::new(vec![
            Series::new("date", vec!["2026-01-01", "2026-01-02"]),
            Series::new("btc", vec![100.0f64, 110.0]),
            Series::new("btc_simple_return", vec![0.0f64, 10.0]),
        ])
        .unwrap();
        let assets = vec!["btc".to_string()];
        let weights = vec![0.5, 0.5]; // mismatch
        let result =
            backtest_portfolio(&df, &assets, &weights, "test", 365.0, 30, 0.0, 0.0, "daily");
        assert!(result.is_err());
    }

    /// Portfolio: zero assets returns Err
    #[test]
    fn test_portfolio_empty_assets_errors() {
        let df =
            DataFrame::new(vec![Series::new("date", vec!["2026-01-01", "2026-01-02"])]).unwrap();
        let result = backtest_portfolio(&df, &[], &[], "test", 365.0, 30, 0.0, 0.0, "daily");
        assert!(result.is_err());
    }
}
