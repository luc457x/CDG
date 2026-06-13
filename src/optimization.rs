use anyhow::{anyhow, Result};
use polars::prelude::*;

#[derive(Debug, Clone)]
pub struct Portfolio {
    pub weights: Vec<f64>,
    pub annualized_return: f64,
    pub annualized_volatility: f64,
    pub sharpe_ratio: f64,
}

#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub max_sharpe: Portfolio,
    pub min_volatility: Portfolio,
    pub simulated_points: Vec<(f64, f64, f64)>, // (volatility, return, sharpe)
}

struct Xorshift {
    state: u64,
}

impl Xorshift {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() as f64) / (u64::MAX as f64)
    }
}

pub fn run_monte_carlo(
    df: &DataFrame,
    assets: &[String],
    num_simulations: usize,
) -> Result<OptimizationResult> {
    let m = assets.len();
    if m == 0 {
        return Err(anyhow!("No assets provided for portfolio optimization"));
    }

    let n_rows = df.height();
    if n_rows < 2 {
        return Err(anyhow!("Insufficient data rows for covariance calculation"));
    }

    // 1. Extract prices and compute simple returns
    let mut returns = Vec::with_capacity(m);
    for asset in assets {
        let series = df.column(asset)?;
        let prices: Vec<f64> = series
            .f64()?
            .into_iter()
            .map(|opt| opt.unwrap_or(0.0))
            .collect();

        let mut asset_returns = Vec::with_capacity(n_rows - 1);
        for i in 1..prices.len() {
            let prev = prices[i - 1];
            if prev > 0.0 {
                asset_returns.push((prices[i] - prev) / prev);
            } else {
                asset_returns.push(0.0);
            }
        }
        returns.push(asset_returns);
    }

    let t = n_rows - 1;

    // 2. Compute mean returns (daily)
    let mut mean_returns = vec![0.0; m];
    for i in 0..m {
        let sum: f64 = returns[i].iter().sum();
        mean_returns[i] = sum / t as f64;
    }

    // 3. Compute covariance matrix (daily)
    let mut cov_matrix = vec![vec![0.0; m]; m];
    for i in 0..m {
        for j in 0..m {
            let mut sum = 0.0;
            for k in 0..t {
                sum += (returns[i][k] - mean_returns[i]) * (returns[j][k] - mean_returns[j]);
            }
            cov_matrix[i][j] = sum / (t - 1).max(1) as f64;
        }
    }

    // 4. Run Monte Carlo simulations
    let mut max_sharpe_portfolio = Portfolio {
        weights: vec![0.0; m],
        annualized_return: f64::NEG_INFINITY,
        annualized_volatility: f64::INFINITY,
        sharpe_ratio: f64::NEG_INFINITY,
    };

    let mut min_vol_portfolio = Portfolio {
        weights: vec![0.0; m],
        annualized_return: 0.0,
        annualized_volatility: f64::INFINITY,
        sharpe_ratio: f64::NEG_INFINITY,
    };

    let mut simulated_points = Vec::with_capacity(num_simulations);
    let mut rng = Xorshift::new(1337); // Fixed seed for reproducibility

    for _ in 0..num_simulations {
        let mut weights = vec![0.0; m];
        let mut sum = 0.0;
        for j in 0..m {
            weights[j] = rng.next_f64();
            sum += weights[j];
        }
        for j in 0..m {
            weights[j] /= sum;
        }

        // Daily portfolio expected return
        let mut p_ret = 0.0;
        for j in 0..m {
            p_ret += weights[j] * mean_returns[j];
        }

        // Daily portfolio variance
        let mut p_var = 0.0;
        for j in 0..m {
            for k in 0..m {
                p_var += weights[j] * weights[k] * cov_matrix[j][k];
            }
        }
        let p_vol = p_var.sqrt();

        // Annualize (using 365 days)
        let ann_ret = p_ret * 365.0;
        let ann_vol = p_vol * 365.0f64.sqrt();
        let sharpe = if ann_vol > 0.0 { ann_ret / ann_vol } else { 0.0 };

        // Convert return and vol to percentages for plotting & UI
        let ann_ret_pct = ann_ret * 100.0;
        let ann_vol_pct = ann_vol * 100.0;

        simulated_points.push((ann_vol_pct, ann_ret_pct, sharpe));

        if sharpe > max_sharpe_portfolio.sharpe_ratio {
            max_sharpe_portfolio = Portfolio {
                weights: weights.clone(),
                annualized_return: ann_ret_pct,
                annualized_volatility: ann_vol_pct,
                sharpe_ratio: sharpe,
            };
        }

        if ann_vol_pct < min_vol_portfolio.annualized_volatility {
            min_vol_portfolio = Portfolio {
                weights: weights.clone(),
                annualized_return: ann_ret_pct,
                annualized_volatility: ann_vol_pct,
                sharpe_ratio: sharpe,
            };
        }
    }

    Ok(OptimizationResult {
        max_sharpe: max_sharpe_portfolio,
        min_volatility: min_vol_portfolio,
        simulated_points,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xorshift() {
        let mut rng = Xorshift::new(42);
        let val1 = rng.next_f64();
        let val2 = rng.next_f64();
        assert!(val1 >= 0.0 && val1 <= 1.0);
        assert!(val2 >= 0.0 && val2 <= 1.0);
        assert_ne!(val1, val2);
    }

    #[test]
    fn test_run_monte_carlo() {
        let df = DataFrame::new(vec![
            Series::new("date", vec!["2026-06-01", "2026-06-02", "2026-06-03"]),
            Series::new("asset_a", vec![100.0, 101.0, 102.0]),
            Series::new("asset_b", vec![10.0, 9.8, 10.1]),
        ])
        .unwrap();

        let assets = vec!["asset_a".to_string(), "asset_b".to_string()];
        let result = run_monte_carlo(&df, &assets, 500).unwrap();

        // Verify simulated points count
        assert_eq!(result.simulated_points.len(), 500);

        // Verify weights sum to 1.0
        let sum_max_sharpe: f64 = result.max_sharpe.weights.iter().sum();
        let sum_min_vol: f64 = result.min_volatility.weights.iter().sum();
        assert!((sum_max_sharpe - 1.0).abs() < 1e-9);
        assert!((sum_min_vol - 1.0).abs() < 1e-9);

        // Volatility of minimum volatility portfolio must be <= max sharpe portfolio's volatility
        assert!(result.min_volatility.annualized_volatility <= result.max_sharpe.annualized_volatility);
    }
}
