use std::time::Duration;

use momento::config::configurations;
use momento::{CacheClient, MomentoResult};
use momento_test_util::{get_test_cache_name, get_test_credential_provider};

#[tokio::main]
async fn main() -> MomentoResult<()> {
    let cache_name = get_test_cache_name();

    let credential_provider = get_test_credential_provider();

    let cache_client = CacheClient::builder()
        .default_ttl(Duration::from_secs(5))
        .configuration(configurations::laptop::latest())
        .credential_provider(credential_provider)
        .build()?;

    println!("Creating cache {}", cache_name.clone());
    cache_client.create_cache(cache_name.clone()).await?;

    Ok(())
}
