use anyhow::Result;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write, BufRead};
use std::path::PathBuf;

#[async_trait]
pub trait Storage<T: Serialize + DeserializeOwned + Send + Sync> {
    async fn append(&self, entry: &T) -> Result<()>;
    async fn load_all(&self) -> Result<Vec<T>>;
}

pub struct JsonlStorage {
    path: PathBuf,
}

impl JsonlStorage {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

#[async_trait]
impl<T: Serialize + DeserializeOwned + Send + Sync> Storage<T> for JsonlStorage {
    async fn append(&self, entry: &T) -> Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        let mut writer = BufWriter::new(file);
        let line = serde_json::to_string(entry)? + "\n";
        writer.write_all(line.as_bytes())?;
        writer.flush()?;
        Ok(())
    }

    async fn load_all(&self) -> Result<Vec<T>> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if !line.is_empty() {
                let entry: T = serde_json::from_str(&line)?;
                entries.push(entry);
            }
        }
        Ok(entries)
    }
}
