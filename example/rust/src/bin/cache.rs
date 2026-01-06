use momento::cache::{configurations, GetResponse};
use momento::{CacheClient, CredentialProvider, MomentoError};
use std::process;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), MomentoError> {
    // Initializing Momento
    let cache_client = match CacheClient::builder()
        .default_ttl(Duration::from_secs(60))
        .configuration(configurations::Laptop::latest())
        .credential_provider(
            CredentialProvider::from_default_env_var_v2().expect("auth token should be valid"),
        )
        .build()
    {
        Ok(client) => client,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };

    // Creating a cache named "cache"
    let cache_name = String::from("cache");
    match cache_client.create_cache(&cache_name).await {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{err}");
        }
    }

    // List the caches
    println!("Listing caches:");
    match cache_client.list_caches().await {
        Ok(list_cache_result) => {
            for listed_cache in list_cache_result.caches {
                println!("{}", listed_cache.name);
            }
        }
        Err(err) => {
            eprintln!("{err}");
        }
    };
    println!();

    // Sets key with default TTL and get value with that key
    let key = String::from("my_key");
    let value = String::from("my_value");
    println!("Setting key: {key}, value: {value}");
    match cache_client
        .set(&cache_name, key.clone(), value.clone())
        .await
    {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{err}");
        }
    };
    match cache_client.get(&cache_name, key.clone()).await {
        Ok(r) => match r {
            GetResponse::Hit { value } => {
                let v: String = value.try_into().expect("I stored a string!");
                println!("Got value: {v}");
            }
            GetResponse::Miss => {
                println!("Cache miss!");
            }
        },
        Err(err) => {
            eprintln!("{err}");
        }
    };
    Ok(())
}
