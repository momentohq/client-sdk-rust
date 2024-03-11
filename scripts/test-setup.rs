use std::env;
use std::time::Duration;

use tokio;

use momento::config::configurations;
use momento::{CacheClient, CredentialProviderBuilder};

#[tokio::main]
async fn main() {
    let cache_name =
        env::var("TEST_CACHE_NAME").expect("environment variable TEST_CACHE_NAME should be set");

    let credential_provider =
        CredentialProviderBuilder::from_environment_variable("TEST_API_KEY".to_string())
            .build()
            .expect("auth token should be valid");

    let cache_client = CacheClient::new(
        credential_provider,
        configurations::laptop::latest(),
        Duration::from_secs(5),
    )
    .expect("cache client cannot be created");

    println!("Creating cache {}", cache_name.clone());
    cache_client
        .create_cache(cache_name.clone())
        .await
        .expect("cache cannot be created");
}
