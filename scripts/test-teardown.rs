use std::time::Duration;

use momento::cache::configurations;
use momento::CacheClient;
use momento_test_util::{get_test_cache_name, get_test_credential_provider};

#[tokio::main]
async fn main() {
    let cache_name = get_test_cache_name();
    let credential_provider = get_test_credential_provider();

    let cache_client = CacheClient::builder()
        .default_ttl(Duration::from_secs(5))
        .configuration(configurations::Laptop::latest())
        .credential_provider(credential_provider)
        .build()
        .expect("cache client cannot be created");

    println!("Deleting cache {}", cache_name.clone());
    cache_client
        .delete_cache(cache_name.clone())
        .await
        .expect("cache cannot be deleted");
}
