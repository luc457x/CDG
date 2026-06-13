use anyhow::{anyhow, Result};
use plotters::prelude::*;
use polars::prelude::*;
use std::path::Path;
use crate::optimization::Portfolio;

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = l - c / 2.0;
    let (r_p, g_p, b_p) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    (
        ((r_p + m) * 255.0).round() as u8,
        ((g_p + m) * 255.0).round() as u8,
        ((b_p + m) * 255.0).round() as u8,
    )
}

pub fn get_distinct_color(i: usize, total: usize) -> RGBColor {
    if total == 0 {
        return RGBColor(0, 0, 0);
    }
    let h = (i as f64) * (360.0 / total as f64);
    let s = 0.8;
    let l = if i % 2 == 0 { 0.45 } else { 0.60 };
    let (r, g, b) = hsl_to_rgb(h, s, l);
    RGBColor(r, g, b)
}

pub fn plot_line_chart(
    df: &DataFrame,
    columns: &[&str],
    title: &str,
    output_path: &str,
) -> Result<()> {
    if let Some(parent) = Path::new(output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let date_column = df.column("date")?.str()?;
    let n_rows = df.height();
    if n_rows == 0 {
        return Err(anyhow!("Cannot plot empty DataFrame"));
    }

    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;

    for &col_name in columns {
        let series = df.column(col_name)?;
        let f_array = series.f64()?;
        let col_min = f_array.min().unwrap_or(0.0);
        let col_max = f_array.max().unwrap_or(100.0);
        if col_min < y_min {
            y_min = col_min;
        }
        if col_max > y_max {
            y_max = col_max;
        }
    }

    let padding = if y_max != y_min {
        (y_max - y_min) * 0.1
    } else {
        1.0
    };
    let y_min = y_min - padding;
    let y_max = y_max + padding;

    let root = BitMapBackend::new(output_path, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(0..n_rows, y_min..y_max)?;

    // Calculate dynamic label step to avoid overlapping dates
    let label_step = (n_rows / 8).max(1);

    chart
        .configure_mesh()
        .x_label_formatter(&|&idx| {
            if idx < n_rows && idx % label_step == 0 {
                date_column.get(idx).unwrap_or("").to_string()
            } else {
                "".to_string()
            }
        })
        .draw()?;

    for (i, &col_name) in columns.iter().enumerate() {
        let series = df.column(col_name)?;
        let values: Vec<Option<f64>> = series.f64()?.into_iter().collect();
        let color = get_distinct_color(i, columns.len());

        chart
            .draw_series(LineSeries::new(
                values
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, &opt_val)| opt_val.map(|val| (idx, val))),
                color,
            ))?
            .label(col_name)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));
    }

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    Ok(())
}

pub fn plot_performance(df: &DataFrame, assets: &[String], output_path: &str) -> Result<()> {
    if let Some(parent) = Path::new(output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let date_column = df.column("date")?.str()?;
    let n_rows = df.height();
    if n_rows == 0 {
        return Err(anyhow!("Cannot plot empty DataFrame"));
    }

    let mut perf_df = DataFrame::new(vec![Series::new("date", date_column.clone())])?;
    let mut columns_to_plot = Vec::new();

    for asset in assets {
        let series = df.column(asset)?;
        let values: Vec<Option<f64>> = series.f64()?.into_iter().collect();

        // Find first valid non-zero price
        let mut first_val = None;
        for &val in values.iter().flatten() {
            if val != 0.0 {
                first_val = Some(val);
                break;
            }
        }

        if let Some(f_val) = first_val {
            let normalized: Vec<Option<f64>> = values
                .iter()
                .map(|&opt| opt.map(|v| (v / f_val) * 100.0))
                .collect();
            let col_name = format!("{}_perf", asset);
            perf_df.insert_column(perf_df.width(), Series::new(&col_name, normalized))?;
            columns_to_plot.push(col_name);
        }
    }

    let col_refs: Vec<&str> = columns_to_plot.iter().map(|s| s.as_str()).collect();
    plot_line_chart(
        &perf_df,
        &col_refs,
        "Asset Normalized Performance (Base 100%)",
        output_path,
    )?;

    Ok(())
}

fn get_log_return_stats(series: &Series) -> Option<(f64, f64)> {
    let prices: Vec<f64> = series.f64().ok()?.into_iter().flatten().collect();
    if prices.len() < 2 {
        return None;
    }
    let mut log_returns = Vec::new();
    for i in 1..prices.len() {
        let prev = prices[i - 1];
        if prev > 0.0 && prices[i] > 0.0 {
            log_returns.push((prices[i] / prev).ln() * 100.0);
        }
    }
    if log_returns.is_empty() {
        return None;
    }
    let mean = log_returns.iter().sum::<f64>() / log_returns.len() as f64;
    let variance: f64 = log_returns
        .iter()
        .map(|&x| {
            let diff = x - mean;
            diff * diff
        })
        .sum::<f64>()
        / log_returns.len() as f64;
    let std = variance.sqrt();
    Some((std, mean))
}

pub fn plot_risk_return(df: &DataFrame, assets: &[String], output_path: &str) -> Result<()> {
    if let Some(parent) = Path::new(output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut stats = Vec::new();
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;

    for asset in assets {
        let series = df.column(asset)?;
        if let Some((risk, ret)) = get_log_return_stats(series) {
            if risk < x_min {
                x_min = risk;
            }
            if risk > x_max {
                x_max = risk;
            }
            if ret < y_min {
                y_min = ret;
            }
            if ret > y_max {
                y_max = ret;
            }
            stats.push((asset.clone(), risk, ret));
        }
    }

    if stats.is_empty() {
        return Err(anyhow!("No valid returns data to plot risk/return scatter"));
    }

    // Add padding to axes
    let x_pad = if x_max != x_min {
        (x_max - x_min) * 0.1
    } else {
        1.0
    };
    let y_pad = if y_max != y_min {
        (y_max - y_min) * 0.1
    } else {
        1.0
    };

    let root = BitMapBackend::new(output_path, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Risk / Return Profile", ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(
            (x_min - x_pad)..(x_max + x_pad),
            (y_min - y_pad)..(y_max + y_pad),
        )?;

    chart
        .configure_mesh()
        .x_desc("Risk (Volatility %)")
        .y_desc("Return (Mean %)")
        .draw()?;

    for (i, (name, risk, ret)) in stats.iter().enumerate() {
        let color = get_distinct_color(i, stats.len());
        chart
            .draw_series(std::iter::once(Circle::new(
                (*risk, *ret),
                8,
                color.filled(),
            )))?
            .label(name)
            .legend(move |(x, y)| Circle::new((x + 10, y), 5, color.filled()));
    }

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    Ok(())
}

pub fn plot_efficient_frontier(
    simulated_points: &[(f64, f64, f64)],
    max_sharpe: &Portfolio,
    min_vol: &Portfolio,
    output_path: &str,
) -> Result<()> {
    if let Some(parent) = Path::new(output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    if simulated_points.is_empty() {
        return Err(anyhow!("No simulated points to plot"));
    }

    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;

    for &(vol, ret, _) in simulated_points {
        if vol < x_min { x_min = vol; }
        if vol > x_max { x_max = vol; }
        if ret < y_min { y_min = ret; }
        if ret > y_max { y_max = ret; }
    }

    let x_pad = if x_max != x_min { (x_max - x_min) * 0.1 } else { 1.0 };
    let y_pad = if y_max != y_min { (y_max - y_min) * 0.1 } else { 1.0 };

    let root = BitMapBackend::new(output_path, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Efficient Frontier & Portfolio Optimization", ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(
            (x_min - x_pad)..(x_max + x_pad),
            (y_min - y_pad)..(y_max + y_pad),
        )?;

    chart
        .configure_mesh()
        .x_desc("Annualized Volatility (%)")
        .y_desc("Annualized Expected Return (%)")
        .draw()?;

    // Draw simulated portfolios as small dots
    let simulated_color = RGBColor(180, 190, 200).mix(0.5);
    chart.draw_series(
        simulated_points
            .iter()
            .map(|&(vol, ret, _)| Circle::new((vol, ret), 2, simulated_color.filled()))
    )?;

    // Draw Min Volatility Portfolio (Green)
    let min_vol_color = GREEN;
    chart.draw_series(std::iter::once(
        Circle::new(
            (min_vol.annualized_volatility, min_vol.annualized_return),
            8,
            min_vol_color.filled(),
        )
    ))?
    .label("Minimum Volatility Portfolio")
    .legend(move |(x, y)| Circle::new((x + 10, y), 5, min_vol_color.filled()));

    // Draw Max Sharpe Portfolio (Red)
    let max_sharpe_color = RED;
    chart.draw_series(std::iter::once(
        Circle::new(
            (max_sharpe.annualized_volatility, max_sharpe.annualized_return),
            8,
            max_sharpe_color.filled(),
        )
    ))?
    .label("Maximum Sharpe Ratio Portfolio")
    .legend(move |(x, y)| Circle::new((x + 10, y), 5, max_sharpe_color.filled()));

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    Ok(())
}
