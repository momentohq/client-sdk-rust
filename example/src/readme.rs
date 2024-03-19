use std::time::Duration;
use momento::{CacheClient, CredentialProvider};
use momento::config::configurations::laptop;

const CACHE_NAME: String = "cache";

pub async fn main() {
    let cache_client = CacheClient::new(
        CredentialProvider::from_env_var("MOMENTO_API_KEY")?,
        laptop::latest(),
        Duration::from_secs(60)
    )?;

    cache_client.create_cache(CACHE_NAME).await?;
    
    match(cache_client.set(CACHE_NAME, "mykey", "myvalue").await) {
        Ok(_) => println!("Successfully stored key 'mykey' with value 'myvalue'"),
        Err(e) => println!("Error: {}", e)
    }
    
    let value = match(cache_client.get(CACHE_NAME, "mykey").await?) {
        
    };


}