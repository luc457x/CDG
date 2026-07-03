use anyhow::Result;
use polars::prelude::*;
use std::fs::File;
use std::path::Path;

pub fn export_csv(df: &mut DataFrame, path: &str) -> Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = File::create(path)?;
    CsvWriter::new(file).finish(df)?;
    Ok(())
}

pub fn export_parquet(df: &mut DataFrame, path: &str) -> Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = File::create(path)?;
    ParquetWriter::new(file).finish(df)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    #[test]
    fn test_exports() {
        let mut df = DataFrame::new(vec![
            Series::new("a", vec![1.0, 2.0]),
            Series::new("b", vec![3.0, 4.0]),
        ])
        .unwrap();

        let temp_csv = "tests/temp_test.csv";
        let temp_pq = "tests/temp_test.parquet";

        assert!(export_csv(&mut df, temp_csv).is_ok());
        assert!(export_parquet(&mut df, temp_pq).is_ok());

        // Clean up
        let _ = std::fs::remove_file(temp_csv);
        let _ = std::fs::remove_file(temp_pq);
    }
}
