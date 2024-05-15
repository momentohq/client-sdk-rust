use momento::cache::configurations;
use momento::{CacheClient, CredentialProvider, MomentoError};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), MomentoError> {
    let _cache_client = CacheClient::builder()
        .default_ttl(Duration::from_secs(60))
        .configuration(configurations::Laptop::latest())
        .credential_provider(CredentialProvider::from_env_var(
            "MOMENTO_API_KEY".to_string(),
        )?)
        .build()?;
    // ...
    Ok(())
}
