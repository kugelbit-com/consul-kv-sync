use base64::{Engine as _, engine::general_purpose};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct ConsulClient {
    address: String,
    token: String,
    client: Client,
}

impl ConsulClient {

    fn create_client() -> Client {

        let connector = reqwest::ClientBuilder::new()
            .use_native_tls()
            .tls_built_in_root_certs(true)
            .tls_built_in_webpki_certs(true)
            .build();


        return connector.unwrap()
    }

    pub fn new(address: &str, token: &str) -> Self {
        ConsulClient {
            address: address.to_string(),
            token: token.to_string(),
            client: Self::create_client(),
        }
    }

    pub async fn get_key(&self, key: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let response = self.client.get(format!("{}/v1/kv/{}", self.address, key))
            .bearer_auth(&self.token)
            .send()
            .await?;

        if response.status().is_success() {
            let json: Vec<KeyValue> = response.json().await?;
            if let Some(kv) = json.first() {
                let value = base64::decode(&kv.value)?;
                return Ok(Some(String::from_utf8(value)?));
            }
        }

        Ok(None)
    }

    pub async fn get_key_metadata(&self, key: &str) -> Result<Option<KeyMetadata>, Box<dyn std::error::Error>> {
        let response = self.client.get(format!("{}/v1/kv/{}", self.address, key))
            .bearer_auth(&self.token)
            .send()
            .await?;

        if response.status().is_success() {
            let json: Vec<KeyValue> = response.json().await?;
            if let Some(kv) = json.first() {
                let value = base64::decode(&kv.value)?;
                let value_string = String::from_utf8(value)?;
                let value: KeyMetadata = serde_json::from_str(&value_string)?;
                return Ok(Some(value));
            }
        }

        Ok(None)
    }

    pub async fn set_key(&self, key: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        let response = self.client.put(format!("{}/v1/kv/{}", self.address, key))
            .bearer_auth(&self.token)
            .body(value.to_string())
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Box::new(TransactionError))
        }
    }

    pub async fn upload_files(&self, files: Vec<(String, String)>) -> Result<(), Box<dyn std::error::Error>> {
        let mut transactions = Vec::new();
        for (key, value) in files {
            let key_clone = &key.clone();
            let encoded_value = general_purpose::STANDARD.encode(value);
            transactions.push(Transaction {
                kv: KV {
                    verb: "set".to_string(),
                    key,
                    value: encoded_value,
                }
            });
            println!("Uploading key: {}", key_clone)
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

#[derive(Serialize, Deserialize, Debug)]
struct KeyValue {
    #[serde(rename = "Key")]
    key: String,

    #[serde(rename = "Value")]
    value: String,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyMetadata {
    #[serde(rename = "hash")]
    pub(crate) hash: String,

    #[serde(rename = "permitOverride", default = "default_true")]
    pub(crate) permit_override: bool,
}

#[derive(Debug)]
struct TransactionError;

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Transaction failed")
    }
}

fn default_true() -> bool {
    true
}

impl std::error::Error for TransactionError {}
