use momento::config::configurations::laptop;
use momento::requests::cache::basic::get::Get;
use momento::{CacheClient, CredentialProvider, MomentoError};
use std::time::Duration;

const CACHE_NAME: &str = "cache";

#[tokio::main]
pub async fn main() -> Result<(), MomentoError> {
    let cache_client = CacheClient::new(
        CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string())?,
        laptop::latest(),
        Duration::from_secs(60),
    )?;

    cache_client.create_cache(CACHE_NAME.to_string()).await?;

    match cache_client.set(CACHE_NAME, "mykey", "myvalue").await {
        Ok(_) => println!("Successfully stored key 'mykey' with value 'myvalue'"),
        Err(e) => println!("Error: {}", e),
    }

    let value: String = match cache_client.get(CACHE_NAME, "mykey").await? {
        Get::Hit { value } => value.try_into().expect("I stored a string!"),
        Get::Miss => {
            println!("Cache miss!");
            return Ok(()); // probably you'll do something else here
        }
    };

    println!("Successfully retrieved value: {}", value);

    Ok(())
}
