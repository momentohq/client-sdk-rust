use std::env;
use std::time::Duration;

use momento::config::configurations;
use momento::{CacheClient, CredentialProviderBuilder};
use momento_test_util::{get_test_cache_name, get_test_credential_provider};

#[tokio::main]
async fn main() {
    let cache_name = get_test_cache_name();
    let credential_provider = get_test_credential_provider();

    let cache_client = CacheClient::new(
        credential_provider,
        configurations::laptop::latest(),
        Duration::from_secs(5),
    )
    .expect("cache client cannot be created");

    println!("Deleting cache {}", cache_name.clone());
    cache_client
        .delete_cache(cache_name.clone())
        .await
        .expect("cache cannot be deleted");
}
