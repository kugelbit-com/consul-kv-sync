use base64::{Engine as _, engine::general_purpose};
use reqwest::Client;
use serde::Serialize;

pub struct ConsulClient {
    address: String,
    token: String,
    client: Client,
}

impl ConsulClient {
    pub fn new(address: &str, token: &str) -> Self {
        ConsulClient {
            address: address.to_string(),
            token: token.to_string(),
            client: Client::new(),
        }
    }

    pub async fn upload_files(&self, files: Vec<(String, String)>) -> Result<(), Box<dyn std::error::Error>> {
        let mut transactions = Vec::new();
        for (key, value) in files {
            let encoded_value = general_purpose::STANDARD.encode(value);
            transactions.push(Transaction {
                kv: KV {
                    verb: "set".to_string(),
                    key,
                    value: encoded_value,
                }
            });
        }

        let response = self.client.put(format!("{}/v1/txn", self.address))
            .bearer_auth(&self.token)
            .json(&transactions)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Box::new(TransactionError))
        }
    }
}

#[derive(Serialize)]
struct Transaction {
    kv: KV,
}

#[derive(Serialize)]
struct KV {
    verb: String,
    key: String,
    value: String,
}

#[derive(Debug)]
struct TransactionError;

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Transaction failed")
    }
}

impl std::error::Error for TransactionError {}
