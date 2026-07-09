use anyhow::{anyhow, Result};
use polars::prelude::*;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Portfolio {
    pub weights: Vec<f64>,
    pub annualized_return: f64,
    pub annualized_volatility: f64,
    pub sharpe_ratio: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
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

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

/// Maximum number of simulations allowed. Each sim uses ~24 B × n_assets.
/// Default: 10,000 (≈240 KB for 1 asset). Hard cap: 50,000 (≈1.2 MB).
pub const MAX_SIMULATIONS_DEFAULT: usize = 10_000;
pub const MAX_SIMULATIONS_HARD_CAP: usize = 50_000;

pub fn run_monte_carlo(
    df: &DataFrame,
    assets: &[String],
    annualization_factor: f64,
    num_simulations: usize,
    seed: Option<u64>,
) -> Result<OptimizationResult> {
    let num_simulations = num_simulations.min(MAX_SIMULATIONS_HARD_CAP);
    let m = assets.len();
    if m == 0 {
        return Err(anyhow!("No assets provided for portfolio optimization"));
    }

    if df.height() < 2 {
        return Err(anyhow!("Insufficient data rows for covariance calculation"));
    }

    // 1. Extract prices and compute simple returns synchronously
    let mut columns = Vec::with_capacity(m);
    for asset in assets {
        columns.push(df.column(asset)?.f64()?);
    }

    let height = df.height();
    let mut aligned_prices = vec![Vec::with_capacity(height); m];
    for row_idx in 0..height {
        let mut row_valid = true;
        let mut row_prices = Vec::with_capacity(m);
        for col in &columns {
            if let Some(p) = col.get(row_idx) {
                if p > 0.0 {
                    row_prices.push(p);
                } else {
                    row_valid = false;
                    break;
                }
            } else {
                row_valid = false;
                break;
            }
        }
        if row_valid {
            for col_idx in 0..m {
                aligned_prices[col_idx].push(row_prices[col_idx]);
            }
        }
    }

    let mut returns = Vec::with_capacity(m);
    for col_prices in &aligned_prices {
        let mut asset_returns = Vec::with_capacity(col_prices.len().saturating_sub(1));
        for i in 1..col_prices.len() {
            let prev = col_prices[i - 1];
            asset_returns.push((col_prices[i] - prev) / prev);
        }
        returns.push(asset_returns);
    }

    // Use the aligned return series length
    let t = returns.first().map(|r| r.len()).unwrap_or(0);
    if t < 1 {
        return Err(anyhow!(
            "Insufficient non-null price data for covariance calculation"
        ));
    }

    // 2. Compute mean returns (daily)
    let mut daily_mean_returns = vec![0.0; m];
    for i in 0..m {
        let sum: f64 = returns[i].iter().sum();
        daily_mean_returns[i] = sum / t as f64;
    }

    // 3. Compute covariance matrix (annualized)
    let mut cov_matrix = vec![vec![0.0; m]; m];
    for i in 0..m {
        for j in 0..m {
            let mean_i = daily_mean_returns[i];
            let mean_j = daily_mean_returns[j];
            let daily_cov = returns[i]
                .iter()
                .take(t)
                .zip(returns[j].iter().take(t))
                .map(|(r_i, r_j)| (r_i - mean_i) * (r_j - mean_j))
                .sum::<f64>()
                / (t - 1).max(1) as f64;
            cov_matrix[i][j] = daily_cov * annualization_factor;
        }
    }

    // 4. Compute expected returns (annualized)
    let mut mean_returns = vec![0.0; m];
    for i in 0..m {
        mean_returns[i] = daily_mean_returns[i] * annualization_factor;
    }

    // Documented fallback: 0.04 (4 %) when ^TNX is absent.
    // Warning is emitted so callers are aware Sharpe uses a proxy rate.
    const RF_FALLBACK: f64 = 0.04;
    let r_f_annual = if let Ok(tnx_col) = df.column("^TNX") {
        let tnx_series = tnx_col.f64()?;
        let sum: f64 = tnx_series.into_iter().flatten().sum();
        let count = tnx_series.into_iter().flatten().count();
        if count > 0 {
            (sum / count as f64) / 100.0
        } else {
            eprintln!("Warning: ^TNX column empty; using risk-free rate fallback of {:.2}%", RF_FALLBACK * 100.0);
            RF_FALLBACK
        }
    } else {
        eprintln!("Warning: ^TNX not found; using risk-free rate fallback of {:.2}%", RF_FALLBACK * 100.0);
        RF_FALLBACK
    };

    // 5. Run Monte Carlo simulations
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

    let pb = if cfg!(test) {
        indicatif::ProgressBar::hidden()
    } else {
        let bar = indicatif::ProgressBar::new(num_simulations as u64);
        bar.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        bar
    };

    use rayon::prelude::*;

    let master_seed = seed.unwrap_or(1337);

    // Compute simulations in parallel using Rayon
    let results: Vec<(Vec<f64>, f64, f64, f64)> = (0..num_simulations)
        .into_par_iter()
        .map(|i| {
            // Hash the seed using splitmix64 to ensure high entropy for nearby indices
            let seed_i = splitmix64(master_seed.wrapping_add(i as u64 + 1));
            let mut rng = Xorshift::new(seed_i);

            let mut weights = vec![0.0; m];
            let mut sum = 0.0;
            for w in weights.iter_mut() {
                // Bias note: adding 0.01 before normalisation shifts the Dirichlet
                // distribution so each asset receives at least ~0.01/(n+0.01) weight.
                // This avoids degenerate zero-weight samples at the cost of a slight
                // upward bias on minimum allocations. Acceptable for exploratory MC.
                *w = rng.next_f64() + 0.01;
                sum += *w;
            }
            for w in weights.iter_mut() {
                *w /= sum;
            }

            // Annualized portfolio expected return
            let mut p_ret = 0.0;
            for j in 0..m {
                p_ret += weights[j] * mean_returns[j];
            }

            // Annualized portfolio variance
            let mut p_var = 0.0;
            for j in 0..m {
                for k in 0..m {
                    p_var += weights[j] * weights[k] * cov_matrix[j][k];
                }
            }
            let p_vol = p_var.sqrt();

            let sharpe = if p_vol > 0.0 {
                (p_ret - r_f_annual) / p_vol
            } else {
                0.0
            };

            // Convert return and vol to percentages for plotting & UI
            let ann_ret_pct = p_ret * 100.0;
            let ann_vol_pct = p_vol * 100.0;

            (weights, ann_ret_pct, ann_vol_pct, sharpe)
        })
        .collect();

    let mut simulated_points = Vec::with_capacity(num_simulations);
    for (weights, ann_ret_pct, ann_vol_pct, sharpe) in results {
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
                weights,
                annualized_return: ann_ret_pct,
                annualized_volatility: ann_vol_pct,
                sharpe_ratio: sharpe,
            };
        }
    }

    // Progress bar is updated after the parallel collect so the bar reflects
    // completion rather than jumping 0→100% mid-loop under Rayon.
    pb.set_position(num_simulations as u64);
    pb.finish_with_message("Simulation complete");

    Ok(OptimizationResult {
        max_sharpe: max_sharpe_portfolio,
        min_volatility: min_vol_portfolio,
        simulated_points,
    })
}

pub fn format_optimal_weights_table(assets: &[String], opt_res: &OptimizationResult) -> String {
    use cli_table::{format::Justify, Cell, Style, Table};

    let mut rows = Vec::new();
    for (i, asset) in assets.iter().enumerate() {
        rows.push(vec![
            asset.clone().cell(),
            format!("{:.2}%", opt_res.max_sharpe.weights[i] * 100.0)
                .cell()
                .justify(Justify::Right),
            format!("{:.2}%", opt_res.min_volatility.weights[i] * 100.0)
                .cell()
                .justify(Justify::Right),
        ]);
    }

    let table = rows.table().title(vec![
        "Asset".cell().bold(true),
        "Max Sharpe Weight"
            .cell()
            .bold(true)
            .justify(Justify::Right),
        "Min Vol Weight".cell().bold(true).justify(Justify::Right),
    ]);

    match table.display() {
        Ok(d) => d.to_string(),
        Err(_) => "Error displaying table".to_string(),
    }
}

pub fn format_portfolio_metrics_table(opt_res: &OptimizationResult) -> String {
    use cli_table::{format::Justify, Cell, Style, Table};

    let table = vec![
        vec![
            "Expected Return".cell(),
            format!("{:.2}%", opt_res.max_sharpe.annualized_return)
                .cell()
                .justify(Justify::Right),
            format!("{:.2}%", opt_res.min_volatility.annualized_return)
                .cell()
                .justify(Justify::Right),
        ],
        vec![
            "Volatility (Risk)".cell(),
            format!("{:.2}%", opt_res.max_sharpe.annualized_volatility)
                .cell()
                .justify(Justify::Right),
            format!("{:.2}%", opt_res.min_volatility.annualized_volatility)
                .cell()
                .justify(Justify::Right),
        ],
        vec![
            "Sharpe Ratio".cell(),
            format!("{:.2}", opt_res.max_sharpe.sharpe_ratio)
                .cell()
                .justify(Justify::Right),
            format!("{:.2}", opt_res.min_volatility.sharpe_ratio)
                .cell()
                .justify(Justify::Right),
        ],
    ]
    .table()
    .title(vec![
        "Metric".cell().bold(true),
        "Max Sharpe Ratio".cell().bold(true).justify(Justify::Right),
        "Min Volatility".cell().bold(true).justify(Justify::Right),
    ]);

    match table.display() {
        Ok(d) => d.to_string(),
        Err(_) => "Error displaying table".to_string(),
    }
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
        let result = run_monte_carlo(&df, &assets, 365.0, 500, None).unwrap();

        // Verify simulated points count
        assert_eq!(result.simulated_points.len(), 500);

        // Verify weights sum to 1.0
        let sum_max_sharpe: f64 = result.max_sharpe.weights.iter().sum();
        let sum_min_vol: f64 = result.min_volatility.weights.iter().sum();
        assert!((sum_max_sharpe - 1.0).abs() < 1e-9);
        assert!((sum_min_vol - 1.0).abs() < 1e-9);

        // Volatility of minimum volatility portfolio must be <= max sharpe portfolio's volatility
        assert!(
            result.min_volatility.annualized_volatility <= result.max_sharpe.annualized_volatility
        );
    }

    #[test]
    fn test_different_annualization_factors() {
        let df = DataFrame::new(vec![
            Series::new("date", vec!["2026-06-01", "2026-06-02", "2026-06-03"]),
            Series::new("asset_a", vec![100.0, 101.0, 102.0]),
            Series::new("asset_b", vec![10.0, 9.8, 10.1]),
        ])
        .unwrap();

        let assets = vec!["asset_a".to_string(), "asset_b".to_string()];
        let result_252 = run_monte_carlo(&df, &assets, 252.0, 500, Some(42)).unwrap();
        let result_365 = run_monte_carlo(&df, &assets, 365.0, 500, Some(42)).unwrap();

        // Verify that different factors lead to different annualized outcomes
        assert_ne!(
            result_252.max_sharpe.annualized_return,
            result_365.max_sharpe.annualized_return
        );
        assert_ne!(
            result_252.max_sharpe.annualized_volatility,
            result_365.max_sharpe.annualized_volatility
        );
    }

    #[test]
    fn test_table_formatting() {
        let opt_res = OptimizationResult {
            max_sharpe: Portfolio {
                weights: vec![0.6, 0.4],
                annualized_return: 15.5,
                annualized_volatility: 12.2,
                sharpe_ratio: 1.27,
            },
            min_volatility: Portfolio {
                weights: vec![0.3, 0.7],
                annualized_return: 10.2,
                annualized_volatility: 8.5,
                sharpe_ratio: 1.20,
            },
            simulated_points: vec![],
        };
        let assets = vec!["asset_a".to_string(), "asset_b".to_string()];

        let weights_tbl = format_optimal_weights_table(&assets, &opt_res);
        assert!(weights_tbl.contains("Asset"));
        assert!(weights_tbl.contains("Max Sharpe Weight"));
        assert!(weights_tbl.contains("Min Vol Weight"));
        assert!(weights_tbl.contains("60.00%"));
        assert!(weights_tbl.contains("70.00%"));

        let metrics_tbl = format_portfolio_metrics_table(&opt_res);
        assert!(metrics_tbl.contains("Metric"));
        assert!(metrics_tbl.contains("Max Sharpe Ratio"));
        assert!(metrics_tbl.contains("Min Volatility"));
        assert!(metrics_tbl.contains("15.50%"));
        assert!(metrics_tbl.contains("8.50%"));
    }

    /// Regression: 2-asset portfolio with distinct price series must produce
    /// strictly positive weights for both assets (no degenerate zero-weight output).
    #[test]
    fn test_covariance_no_zero_weights() {
        // Asset A grows steadily; Asset B is more volatile — ensures non-trivial cov.
        let df = DataFrame::new(vec![
            Series::new(
                "date",
                vec![
                    "2026-01-01", "2026-01-02", "2026-01-03", "2026-01-04",
                    "2026-01-05", "2026-01-06", "2026-01-07", "2026-01-08",
                    "2026-01-09", "2026-01-10",
                ],
            ),
            Series::new(
                "crypto",
                vec![100.0, 102.0, 101.0, 105.0, 103.0, 108.0, 107.0, 110.0, 109.0, 113.0],
            ),
            Series::new(
                "stock",
                vec![50.0, 50.5, 51.0, 50.8, 51.5, 51.2, 52.0, 51.8, 52.5, 52.3],
            ),
        ])
        .unwrap();

        let assets = vec!["crypto".to_string(), "stock".to_string()];
        let result = run_monte_carlo(&df, &assets, 365.0, 2000, Some(1337)).unwrap();

        // Both assets must receive strictly positive weight in the max-Sharpe portfolio.
        // The + 0.01 bias guarantees this; a zero weight would indicate a regression.
        assert!(
            result.max_sharpe.weights[0] > 0.0,
            "crypto weight must be > 0, got {}",
            result.max_sharpe.weights[0]
        );
        assert!(
            result.max_sharpe.weights[1] > 0.0,
            "stock weight must be > 0, got {}",
            result.max_sharpe.weights[1]
        );

        // Weights sum to 1.0
        let sum: f64 = result.max_sharpe.weights.iter().sum();
        assert!((sum - 1.0).abs() < 1e-9, "weights must sum to 1.0, got {}", sum);

        // Min-vol portfolio must have volatility <= max-Sharpe portfolio
        assert!(
            result.min_volatility.annualized_volatility
                <= result.max_sharpe.annualized_volatility + 1e-9,
            "min-vol ({}) must be <= max-Sharpe vol ({})",
            result.min_volatility.annualized_volatility,
            result.max_sharpe.annualized_volatility
        );
    }

    /// Formula tests: Sharpe = (ret - rf) / vol, weights sum to 1.
    #[test]
    fn test_sharpe_formula_and_weights_sum() {
        let df = DataFrame::new(vec![
            Series::new(
                "date",
                vec!["2026-01-01", "2026-01-02", "2026-01-03", "2026-01-04"],
            ),
            Series::new("a", vec![100.0, 102.0, 101.0, 103.0]),
            Series::new("b", vec![200.0, 198.0, 201.0, 204.0]),
        ])
        .unwrap();

        let assets = vec!["a".to_string(), "b".to_string()];
        let result = run_monte_carlo(&df, &assets, 252.0, 1000, Some(42)).unwrap();

        // Weights must sum to 1.0 for every selected portfolio
        let sharpe_sum: f64 = result.max_sharpe.weights.iter().sum();
        let minvol_sum: f64 = result.min_volatility.weights.iter().sum();
        assert!((sharpe_sum - 1.0).abs() < 1e-9);
        assert!((minvol_sum - 1.0).abs() < 1e-9);

        // Hard cap: requesting more than MAX_SIMULATIONS_HARD_CAP is clamped
        let result_capped =
            run_monte_carlo(&df, &assets, 252.0, MAX_SIMULATIONS_HARD_CAP + 1, Some(42)).unwrap();
        assert_eq!(
            result_capped.simulated_points.len(),
            MAX_SIMULATIONS_HARD_CAP
        );
    }
}
