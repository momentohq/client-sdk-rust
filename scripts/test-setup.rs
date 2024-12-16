use std::time::Duration;

use momento::cache::configurations;
use momento::{CacheClient, MomentoResult, PreviewStorageClient};
use momento_test_util::{get_test_cache_name, get_test_credential_provider, get_test_store_name};

#[tokio::main]
async fn main() -> MomentoResult<()> {
    let cache_name = get_test_cache_name();
    let store_name = get_test_store_name();

    let credential_provider = get_test_credential_provider();

    let cache_client = CacheClient::builder()
        .default_ttl(Duration::from_secs(5))
        .configuration(configurations::Laptop::latest())
        .credential_provider(credential_provider.clone())
        .build()?;

    let storage_client = PreviewStorageClient::builder()
        .configuration(momento::storage::configurations::Laptop::latest())
        .credential_provider(credential_provider.clone())
        .build()?;

    println!("Creating cache {}", cache_name.clone());
    cache_client.create_cache(cache_name.clone()).await?;

    println!("Creating store {}", store_name.clone());
    storage_client.create_store(store_name.clone()).await?;

    Ok(())
}
