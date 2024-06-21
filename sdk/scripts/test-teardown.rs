use std::time::Duration;

use momento::cache::configurations;
use momento::{CacheClient, PreviewStorageClient};
use momento_test_util::{get_test_cache_name, get_test_credential_provider, get_test_store_name};

#[tokio::main]
#[allow(clippy::expect_used)] // we want to panic if teardown cannot complete
async fn main() {
    let cache_name = get_test_cache_name();
    let store_name = get_test_store_name();

    let credential_provider = get_test_credential_provider();

    let cache_client = CacheClient::builder()
        .default_ttl(Duration::from_secs(5))
        .configuration(configurations::Laptop::latest())
        .credential_provider(credential_provider.clone())
        .build()
        .expect("cache client cannot be created");

    let storage_client = PreviewStorageClient::builder()
        .configuration(momento::storage::configurations::Laptop::latest())
        .credential_provider(credential_provider.clone())
        .build()
        .expect("storage client cannot be created");

    println!("Deleting cache {}", cache_name.clone());
    cache_client
        .delete_cache(cache_name.clone())
        .await
        .expect("cache cannot be deleted");

    println!("Deleting store {}", store_name.clone());
    storage_client
        .delete_store(store_name.clone())
        .await
        .expect("store cannot be deleted");
}
