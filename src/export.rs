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
