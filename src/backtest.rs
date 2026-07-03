use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BacktestMetrics {
    pub coin: String,
    pub currency: String,
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
    (mean / std_dev) * (annualization_factor).sqrt()
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

pub fn calculate_strategy_return(
    signal: i64,
    prev_signal: i64,
    actual_return: f64,
    predicted_return: f64,
    transaction_fee: f64,
    slippage: f64,
) -> f64 {
    let raw_strat_ret = if signal == 2 {
        actual_return
    } else if signal == 0 {
        -actual_return
    } else {
        0.0
    };

    let confidence_multiplier = (predicted_return.abs() / 0.5).clamp(0.0, 1.0);
    let scaled_return = raw_strat_ret * confidence_multiplier;

    let cost = if signal != prev_signal {
        transaction_fee + slippage
    } else {
        0.0
    };

    scaled_return - cost
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
            // Buy signal
            if actual > 0.0 {
                tp += 1;
            } else {
                fp += 1;
            }
        } else if signal == 0 {
            // Sell signal
            if actual < 0.0 {
                tn += 1;
            } else {
                fn_count += 1;
            }
        }
    }

    (tp, fp, tn, fn_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_mda() {
        let actuals = vec![0.1, -0.2, 0.3, 0.0];
        let predicted = vec![0.2, -0.1, -0.4, 0.0];
        assert_eq!(calculate_mda(&actuals, &predicted), 0.75); // 3 out of 4 correct signs
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
        // Peak 105 -> 95 is (105-95)/105 = 0.0952
        // Peak 110 -> 88 is (110-88)/110 = 0.20
        let dd = calculate_max_drawdown(&equity);
        assert!((dd - 0.20).abs() < 1e-5);
    }
}
