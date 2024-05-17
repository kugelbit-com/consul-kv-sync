mod consul;

use consul::ConsulClient;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use sha2::{Sha256, Digest};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <directory> <consul_address> <consul_token>", args[0]);
        std::process::exit(1);
    }

    let directory = &args[1];
    let consul_address = &args[2];
    let consul_token = &args[3];

    let consul_client = ConsulClient::new(consul_address, consul_token);

    let mut file_hashes = HashMap::new();
    let mut changed_files = Vec::new();

    for entry in WalkDir::new(directory).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            match process_file(path, &mut file_hashes).await {
                Ok(Some(key_value)) => changed_files.push(key_value),
                Ok(None) => {},
                Err(e) => eprintln!("Error processing file {}: {}", path.display(), e),
            }
        }
    }

    if !changed_files.is_empty() {
        match consul_client.upload_files(changed_files).await {
            Ok(_) => println!("Files successfully uploaded."),
            Err(e) => eprintln!("Failed to upload files: {}", e),
        }
    } else {
        println!("No files changed.");
    }
}

async fn process_file(path: &Path, file_hashes: &mut HashMap<String, String>) -> Result<Option<(String, String)>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let hash = calculate_hash(&content);

    let relative_path = path.to_string_lossy().to_string();
    let key = relative_path.replace("\\", "/");

    if let Some(existing_hash) = file_hashes.get(&key) {
        if existing_hash == &hash {
            return Ok(None);
        }
    }

    file_hashes.insert(key.clone(), hash);

    Ok(Some((key, content)))
}

fn calculate_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}
