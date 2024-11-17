use anyhow::{Context, Result};
use csv::{Reader, Writer};
use serde::de::DeserializeOwned;
use std::fs::File;

/// Reads a CSV file and deserializes its content into a vector of type `T`.
///
/// # Arguments
///
/// * `file` - A string slice that holds the name of the file to be read.
///
/// # Returns
///
/// * `Result<Vec<T>>` - A result containing a vector of deserialized records of type `T` if successful, or an error if not.
///
/// # Errors
///
/// This function will return an error if the file cannot be opened, or if any record cannot be deserialized.
pub fn read_csv<T: DeserializeOwned>(file: &str) -> Result<Vec<T>> {
    // Open the file
    let file = File::open(file).with_context(|| format!("Failed to open file: {}", file))?;
    let mut rdr = Reader::from_reader(file);

    // Deserialize each record and collect them into a vector
    rdr.deserialize()
        .enumerate()
        .map(|(i, result)| {
            result.with_context(|| format!("Failed to deserialize record at line {}", i + 1))
        })
        .collect()
}

/// Writes a vector of serializable data to a CSV file.
///
/// # Arguments
///
/// * `file` - A string slice that holds the name of the file to be written.
/// * `data` - A vector of data to be serialized and written to the file.
///
/// # Returns
///
/// * `Result<()>` - An empty result if successful, or an error if not.
///
/// # Errors
///
/// This function will return an error if the file cannot be created, or if any record cannot be serialized.
pub fn write_csv<T: serde::Serialize>(file: &str, data: Vec<T>) -> Result<()> {
    // Create a CSV writer for the specified file
    let mut wtr = Writer::from_path(file)
        .with_context(|| format!("Failed to create CSV writer for file: {}", file))?;

    // Serialize each record and write it to the file
    data.into_iter().enumerate().try_for_each(|(i, record)| {
        wtr.serialize(record)
            .with_context(|| format!("Failed to serialize record at index {}", i))
    })?;

    // Flush the writer to ensure all data is written to the file
    wtr.flush()
        .with_context(|| format!("Failed to flush CSV writer for file: {}", file))?;
    Ok(())
}
