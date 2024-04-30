//! A transaction stream that fetches transactions from a YAML or JSON file.

use std::{fs::File, path::Path};

use async_trait::async_trait;
use serde::Deserialize;
use tokio::sync::mpsc::Receiver;

use crate::{models::Transaction, stream::TransactionStream};

#[derive(Debug, Deserialize, Clone)]
pub struct FileTransaction {
    pub intent_hash: String,
    pub state_version: u64,
    pub unix_timestamp_nanos: i64,
    pub events: Vec<radix_client::gateway::models::Event>,
}

impl From<FileTransaction> for Transaction {
    fn from(transaction: FileTransaction) -> Self {
        Self {
            intent_hash: transaction.intent_hash,
            state_version: transaction.state_version,
            confirmed_at: chrono::DateTime::from_timestamp_nanos(
                transaction.unix_timestamp_nanos,
            ),
            events: transaction
                .events
                .into_iter()
                .map(|event| event.into())
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct FileTransactionStream {
    transactions: Vec<FileTransaction>,
}

impl FileTransactionStream {
    pub fn new(file_path: String) -> Self {
        let file = File::open(&file_path).expect("Unable to open file");

        // Determine file extension
        let extension = Path::new(&file_path)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("");

        let transactions: Vec<FileTransaction> = match extension {
            "json" => serde_json::from_reader(&file)
                .expect("Error deserializing JSON"),
            "yaml" | "yml" => serde_yaml::from_reader(&file)
                .expect("Error deserializing YAML"),
            _ => panic!("Unsupported file type"),
        };

        Self { transactions }
    }
}

#[async_trait]
impl TransactionStream for FileTransactionStream {
    async fn start(&mut self) -> Result<Receiver<Transaction>, anyhow::Error> {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let transactions = self.transactions.clone();
        tokio::spawn(async move {
            for transaction in transactions.into_iter() {
                if tx.send(transaction.into()).await.is_err() {
                    break;
                }
            }
        });
        Ok(rx)
    }
    // no task is spawned, so no need to do anything on stop
    async fn stop(&mut self) {}
}
