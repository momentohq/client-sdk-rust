use log::info;
use momento::{CredentialProvider, MomentoError, ProtosocketCacheClient};
use std::process;
use std::time::Duration;
use tokio_rustls::rustls::crypto::aws_lc_rs::default_provider;

#[tokio::main]
async fn main() -> Result<(), MomentoError> {
    tokio_rustls::rustls::crypto::CryptoProvider::install_default(default_provider())
        .expect("Error installing default crypto provider");

    env_logger::init();
    info!("Starting Momento ProtosocketCacheClient example");

    let credential_provider = CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string())
        .expect("auth token should be valid");

    let config = momento::protosocket::cache::Configuration::builder()
        .timeout(Duration::from_secs(60))
        .connection_count(1)
        .az_id(None)
        .build();

    // Initializing Momento protosocket cache client
    let cache_client = match ProtosocketCacheClient::builder()
        .default_ttl(Duration::from_secs(60))
        .configuration(config)
        .credential_provider(credential_provider)
        .runtime(tokio::runtime::Handle::current())
        .build()
        .await
    {
        Ok(client) => client,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };

    // Assumes this cache exists already -- you can make one in the Momento Console
    let cache_name = "cache";

    // First get should result in a miss
    match cache_client.get(cache_name, "key").await {
        Ok(resp) => {
            println!("Get response: {:?}", resp);
        }
        Err(err) => {
            eprintln!("{err}");
        }
    }

    // Set the value
    match cache_client.set(cache_name, "key", "value").await {
        Ok(_) => {
            println!("Successfully stored item in cache");
        }
        Err(err) => {
            eprintln!("{err}");
        }
    }

    // Second get should result in a hit
    match cache_client.get(cache_name, "key").await {
        Ok(resp) => {
            println!("Get response: {:?}", resp);
        }
        Err(err) => {
            eprintln!("{err}");
        }
    }

    Ok(())
}
