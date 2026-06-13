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
    let mut df = DataFrame::new(vec![
        Series::new(
            "date",
            vec!["2026-06-01", "2026-06-02", "2026-06-03", "2026-06-04"],
        ),
        Series::new("BTC", vec![60000.0, 61000.0, 59000.0, 62000.0]),
        Series::new("ETH", vec![3000.0, 3100.0, 2950.0, 3150.0]),
    ])
    .unwrap();

    let assets = vec!["BTC".to_string(), "ETH".to_string()];
    let res = run_monte_carlo(&mut df, &assets, 100).unwrap();

    assert_eq!(res.simulated_points.len(), 100);
    assert_eq!(res.max_sharpe.weights.len(), 2);
    assert_eq!(res.min_volatility.weights.len(), 2);
}
