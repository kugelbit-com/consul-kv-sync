use std::fs;
use std::env;
use std::path::Path;

use clap::Parser;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use walkdir::{DirEntry, WalkDir};

use consul::ConsulClient;
use crate::consul::KeyMetadata;

mod consul;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {

    /// Files Directory
    #[arg(short, long, default_value = ".")]
    directory: String,

    /// Address for consul server agent. If not set or empty will use the default http://localhost:8500
    #[arg(long, default_value = "")]
    consul_address: String,

    /// Consul token for authentication
    #[arg(long, default_value = "")]
    consul_token: String,

    /// Ignore files with that name
    #[clap(short, long, value_parser, num_args = 1.., value_delimiter = ' ')]
    ignore: Vec<String>
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let mut ignored_files = vec![String::from(".git/")];

    for ignore in args.ignore {
        if(!ignored_files.contains(&ignore)) {
            ignored_files.push(ignore);
        }
    }

    let directory = &args.directory;
    let consul_address_env_variable = "CONSUL_ADDRESS";
    let consul_address_env_value =  env::var(consul_address_env_variable);
    let consul_token_env_variable = "CONSUL_TOKEN";
    let consul_token_env_value =  env::var(consul_token_env_variable);
    let mut consul_token = "".to_string();
    let mut consul_address = "".to_string();
    if(args.consul_token.trim().is_empty()) {
        consul_token = consul_token_env_value.unwrap_or("".to_string())
    } else {
        consul_token = args.consul_token
    }

    if(consul_token.trim().is_empty()) {
        panic!("Consul token needs to be set to a non empty string. Use --consul-token argument or the environment variable CONSUL_TOKEN")
    }

    if(args.consul_address.trim().is_empty()) {
        consul_address = consul_address_env_value.unwrap_or("".to_string())
    } else {
        consul_address = args.consul_address
    }

    if(consul_token.trim().is_empty()) {
        panic!("Consul token needs to be set to a non empty string. Use --consul-token argument or the environment variable CONSUL_TOKEN")
    }

    if(consul_address.trim().is_empty()) {
        consul_address = "http://localhost:8500".to_string()
    }

    println!("Using consul address {}", consul_address);

    let consul_client = ConsulClient::new(&consul_address, &consul_token);

    let mut changed_files = Vec::new();

    for entry in WalkDir::new(directory)
        .into_iter()
        .filter_entry(|e| filter_files(e, directory, &ignored_files))
        .filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            match process_file(path, &directory, &consul_client).await {
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

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn filter_files(entry: &DirEntry, root_directory: &String, ignored_files: &Vec<String>) -> bool {
    let path = entry.path();
    let relative_path = path.to_string_lossy().to_string();
    let relative_path_striped = relative_path.replace(root_directory, "");
    let key = relative_path_striped.replace("\\", "/");

    if is_hidden(entry) {
        return false
    }

    for ignored_file in ignored_files {
        if key.starts_with(ignored_file) {
            return false;
        }
    }
    return true
}

async fn process_file(path: &Path, root_directory: &String, consul_client: &ConsulClient) -> Result<Option<(String, String)>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let hash = calculate_hash(&content);

    let relative_path = path.to_string_lossy().to_string();
    let relative_path_striped = relative_path.replace(root_directory, "");
    let key = relative_path_striped.replace("\\", "/");
    let hash_key = format!("metadata/{}", key);
    if let Some(existing_meta) = consul_client.get_key_metadata(&hash_key).await? {
        if existing_meta.hash == hash {
            return Ok(None);
        }

        if !existing_meta.permit_override {
            return Ok(None)
        }
    }
    let metadata = KeyMetadata {
        hash: hash,
        permit_override: true
    };

    let metadata_json = serde_json::to_string(&metadata).unwrap();

    consul_client.set_key(&hash_key, &metadata_json).await?;

    Ok(Some((key, content)))
}

fn calculate_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}
