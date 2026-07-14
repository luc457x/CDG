use crate::optimization::Portfolio;
use anyhow::{anyhow, Result};
use plotters::prelude::*;
use polars::prelude::*;
use std::path::Path;

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
    let l = if i.is_multiple_of(2) { 0.45 } else { 0.60 };
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
        if vol < x_min {
            x_min = vol;
        }
        if vol > x_max {
            x_max = vol;
        }
        if ret < y_min {
            y_min = ret;
        }
        if ret > y_max {
            y_max = ret;
        }
    }

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
        .caption(
            "Efficient Frontier & Portfolio Optimization",
            ("sans-serif", 30).into_font(),
        )
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
            .map(|&(vol, ret, _)| Circle::new((vol, ret), 2, simulated_color.filled())),
    )?;

    // Draw Min Volatility Portfolio (Green)
    let min_vol_color = GREEN;
    chart
        .draw_series(std::iter::once(Circle::new(
            (min_vol.annualized_volatility, min_vol.annualized_return),
            8,
            min_vol_color.filled(),
        )))?
        .label("Minimum Volatility Portfolio")
        .legend(move |(x, y)| Circle::new((x + 10, y), 5, min_vol_color.filled()));

    // Draw Max Sharpe Portfolio (Red)
    let max_sharpe_color = RED;
    chart
        .draw_series(std::iter::once(Circle::new(
            (
                max_sharpe.annualized_volatility,
                max_sharpe.annualized_return,
            ),
            8,
            max_sharpe_color.filled(),
        )))?
        .label("Maximum Sharpe Ratio Portfolio")
        .legend(move |(x, y)| Circle::new((x + 10, y), 5, max_sharpe_color.filled()));

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    Ok(())
}

pub fn plot_backtest_equity(
    dates: &[String],
    strategy_equity: &[f64],
    bh_equity: &[f64],
    coin_name: &str,
    strategy_name: &str,
    output_path: &str,
) -> Result<()> {
    if let Some(parent) = Path::new(output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let n_rows = strategy_equity.len();
    if n_rows == 0 {
        return Err(anyhow!("Cannot plot empty equity data"));
    }

    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;

    for i in 0..n_rows {
        let val_s = strategy_equity[i];
        let val_bh = bh_equity[i];
        if val_s < y_min {
            y_min = val_s;
        }
        if val_s > y_max {
            y_max = val_s;
        }
        if val_bh < y_min {
            y_min = val_bh;
        }
        if val_bh > y_max {
            y_max = val_bh;
        }
    }

    let padding = if y_max != y_min {
        (y_max - y_min) * 0.1
    } else {
        1000.0
    };
    let y_min = (y_min - padding).max(0.0);
    let y_max = y_max + padding;

    let root = BitMapBackend::new(output_path, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let title = format!(
        "Backtest Equity Curve: {} ({})",
        coin_name.to_uppercase(),
        strategy_name.to_uppercase()
    );

    let mut chart = ChartBuilder::on(&root)
        .caption(title, ("sans-serif", 30).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0..n_rows, y_min..y_max)?;

    let label_step = (n_rows / 8).max(1);

    chart
        .configure_mesh()
        .x_label_formatter(&|&idx| {
            if idx < n_rows && idx % label_step == 0 {
                dates[idx].clone()
            } else {
                "".to_string()
            }
        })
        .draw()?;

    // Plot Strategy Equity (Red)
    let strat_color = RED;
    chart
        .draw_series(LineSeries::new(
            strategy_equity
                .iter()
                .enumerate()
                .map(|(idx, &val)| (idx, val)),
            strat_color,
        ))?
        .label("Strategy Equity")
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], strat_color));

    // Plot Buy & Hold Equity (Blue)
    let bh_color = BLUE;
    chart
        .draw_series(LineSeries::new(
            bh_equity.iter().enumerate().map(|(idx, &val)| (idx, val)),
            bh_color,
        ))?
        .label("Buy & Hold Equity")
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], bh_color));

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .draw()?;

    Ok(())
}

pub fn plot_candlestick(
    df: &DataFrame,
    open_col: &str,
    high_col: &str,
    low_col: &str,
    close_col: &str,
    title: &str,
    output_path: &str,
) -> Result<()> {
    if let Some(parent) = Path::new(output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let n_rows = df.height();
    if n_rows == 0 {
        return Err(anyhow!("Cannot plot empty DataFrame"));
    }

    let date_column = df.column("date")?.str()?;
    
    let open_series = df.column(open_col)?;
    let high_series = df.column(high_col)?;
    let low_series = df.column(low_col)?;
    let close_series = df.column(close_col)?;

    let open_vals: Vec<Option<f64>> = open_series.f64()?.into_iter().collect();
    let high_vals: Vec<Option<f64>> = high_series.f64()?.into_iter().collect();
    let low_vals: Vec<Option<f64>> = low_series.f64()?.into_iter().collect();
    let close_vals: Vec<Option<f64>> = close_series.f64()?.into_iter().collect();

    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    for i in 0..n_rows {
        if let (Some(_o), Some(h), Some(l), Some(_c)) = (open_vals[i], high_vals[i], low_vals[i], close_vals[i]) {
            if l < y_min { y_min = l; }
            if h > y_max { y_max = h; }
        }
    }

    if y_min.is_infinite() || y_max.is_infinite() {
        return Err(anyhow!("No valid price data to plot candlestick"));
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

    let green_style = GREEN.filled();
    let red_style = RED.filled();

    chart.draw_series(
        (0..n_rows).filter_map(|idx| {
            if let (Some(o), Some(h), Some(l), Some(c)) = (open_vals[idx], high_vals[idx], low_vals[idx], close_vals[idx]) {
                Some(CandleStick::new(
                    idx,
                    o,
                    h,
                    l,
                    c,
                    green_style.clone(),
                    red_style.clone(),
                    6,
                ))
            } else {
                None
            }
        })
    )?;

    Ok(())
}

pub fn print_candlestick_stdout(df: &DataFrame, col: &str, max_width: usize) -> Result<()> {
    let n_rows = df.height();
    if n_rows == 0 {
        return Err(anyhow!("Cannot print empty DataFrame"));
    }

    let open_col_name = format!("{}_open", col);
    let high_col_name = format!("{}_high", col);
    let low_col_name = format!("{}_low", col);
    let close_col_name = format!("{}_close", col);

    let open_vals: Vec<Option<f64>> = df.column(&open_col_name)?.f64()?.into_iter().collect();
    let high_vals: Vec<Option<f64>> = df.column(&high_col_name)?.f64()?.into_iter().collect();
    let low_vals: Vec<Option<f64>> = df.column(&low_col_name)?.f64()?.into_iter().collect();
    let close_vals: Vec<Option<f64>> = df.column(&close_col_name)?.f64()?.into_iter().collect();

    let width = max_width.min(n_rows).max(1);
    let bucket_size = n_rows as f64 / width as f64;

    struct Candle {
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        is_rise: bool,
    }

    let mut candles = Vec::with_capacity(width);
    for b in 0..width {
        let start = (b as f64 * bucket_size).round() as usize;
        let end = (((b + 1) as f64 * bucket_size).round() as usize).min(n_rows);
        let range = start..end;
        if range.is_empty() {
            continue;
        }

        let mut o_val = None;
        for i in range.clone() {
            if let Some(v) = open_vals[i] {
                o_val = Some(v);
                break;
            }
        }

        let mut c_val = None;
        for i in range.clone().rev() {
            if let Some(v) = close_vals[i] {
                c_val = Some(v);
                break;
            }
        }

        let mut h_val = None;
        for i in range.clone() {
            if let Some(v) = high_vals[i] {
                h_val = Some(h_val.map_or(v, |m: f64| m.max(v)));
            }
        }

        let mut l_val = None;
        for i in range.clone() {
            if let Some(v) = low_vals[i] {
                l_val = Some(l_val.map_or(v, |m: f64| m.min(v)));
            }
        }

        if let (Some(open), Some(high), Some(low), Some(close)) = (o_val, h_val, l_val, c_val) {
            candles.push(Candle {
                open,
                high,
                low,
                close,
                is_rise: close >= open,
            });
        }
    }

    if candles.is_empty() {
        return Err(anyhow!("No valid candle data after downsampling"));
    }

    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for c in &candles {
        if c.low < min_y { min_y = c.low; }
        if c.high > max_y { max_y = c.high; }
    }

    if min_y == max_y {
        min_y -= 1.0;
        max_y += 1.0;
    }

    let height = 20;
    println!("\n--- Candlestick Chart for {} ---", col);
    for row in (0..height).rev() {
        let price_level_high = min_y + (row + 1) as f64 * (max_y - min_y) / height as f64;
        let price_level_low = min_y + row as f64 * (max_y - min_y) / height as f64;
        let price_mid = (price_level_high + price_level_low) / 2.0;

        print!("{:10.2} | ", price_mid);

        for candle in &candles {
            let body_min = candle.open.min(candle.close);
            let body_max = candle.open.max(candle.close);

            let in_body = price_mid >= body_min && price_mid <= body_max;
            let in_wick = price_mid >= candle.low && price_mid <= candle.high;

            let color_code = if candle.is_rise {
                "\x1b[32m"
            } else {
                "\x1b[31m"
            };
            let reset_code = "\x1b[0m";

            if in_body {
                print!("{}{}{}", color_code, "█", reset_code);
            } else if in_wick {
                print!("{}{}{}", color_code, "│", reset_code);
            } else {
                print!(" ");
            }
        }
        println!();
    }
    println!("--------------------------------------------------");
    Ok(())
}

