use momento::response::Get;
use momento::{CredentialProvider, MomentoError, SimpleCacheClientBuilder};
use std::process;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), MomentoError> {
    // Initializing Momento
    let mut cache_client = match SimpleCacheClientBuilder::new(
        CredentialProvider::from_env_var("MOMENTO_AUTH_TOKEN".to_string())?,
        Duration::from_secs(60),
    ) {
        Ok(client) => client,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    }
    .build();

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
                println!("{}", listed_cache.cache_name);
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
        .set(&cache_name, key.clone(), value.clone(), None)
        .await
    {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{err}");
        }
    };
    match cache_client.get(&cache_name, key.clone()).await {
        Ok(r) => match r {
            Get::Hit { value } => {
                let v: String = value.try_into().expect("I stored a string!");
                println!("Got value: {v}");
            }
            Get::Miss => {
                println!("Cache miss!");
            }
        },
        Err(err) => {
            eprintln!("{err}");
        }
    };
    // Permanently deletes cache
    match cache_client.delete_cache(&cache_name).await {
        Ok(_) => {
            println!("Permanently deleted cache named, {cache_name}");
        }
        Err(err) => {
            eprintln!("{err}");
        }
    };
    Ok(())
}
