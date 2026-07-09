use anyhow::{Context, Result};
use polars::prelude::*;
use std::fs::File;
use std::path::Path;

pub fn export_csv(df: &DataFrame, path: &str) -> Result<()> {
    crate::utils::validate_safe_path(path)?;
    if let Some(parent) = Path::new(path).parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating parent dir for {}", path))?;
    }
    let file = File::create(path)?;
    CsvWriter::new(file).finish(&mut df.clone())?;
    Ok(())
}

pub fn export_parquet(df: &DataFrame, path: &str) -> Result<()> {
    crate::utils::validate_safe_path(path)?;
    if let Some(parent) = Path::new(path).parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating parent dir for {}", path))?;
    }
    let file = File::create(path)?;
    ParquetWriter::new(file).finish(&mut df.clone())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exports() {
        let df = DataFrame::new(vec![
            Series::new("a", vec![1.0, 2.0]),
            Series::new("b", vec![3.0, 4.0]),
        ])
        .unwrap();

        let temp_dir = tempfile::tempdir().unwrap();
        let temp_csv = temp_dir.path().join("temp_test.csv");
        let temp_pq = temp_dir.path().join("temp_test.parquet");

        let temp_csv_str = temp_csv.to_str().unwrap();
        let temp_pq_str = temp_pq.to_str().unwrap();

        assert!(export_csv(&df, temp_csv_str).is_ok());
        assert!(export_parquet(&df, temp_pq_str).is_ok());
    }
}
