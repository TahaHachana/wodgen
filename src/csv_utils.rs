use anyhow::{Context, Result};
use csv::{Reader, Writer};
use serde::de::DeserializeOwned;
use std::fs::File;

pub fn read_csv<T: DeserializeOwned>(file: &str) -> Result<Vec<T>> {
    let file = File::open(file).with_context(|| format!("Failed to open file: {}", file))?;
    let mut rdr = Reader::from_reader(file);
    rdr.deserialize()
        .enumerate()
        .map(|(i, result)| {
            result.with_context(|| format!("Failed to deserialize record at line {}", i + 1))
        })
        .collect()
}

pub fn write_csv<T: serde::Serialize>(file: &str, data: Vec<T>) -> Result<()> {
    let mut wtr = Writer::from_path(file)
        .with_context(|| format!("Failed to create CSV writer for file: {}", file))?;
    data.into_iter().enumerate().try_for_each(|(i, record)| {
        wtr.serialize(record)
            .with_context(|| format!("Failed to serialize record at index {}", i))
    })?;
    wtr.flush()
        .with_context(|| format!("Failed to flush CSV writer for file: {}", file))?;
    Ok(())
}