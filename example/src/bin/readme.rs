use momento::config::configurations::laptop;
use momento::cache::{Get, SortOrder, SortedSetFetch};
use momento::{CacheClient, CredentialProvider, MomentoError};
use std::time::Duration;

const CACHE_NAME: &str = "cache";

#[tokio::main]
pub async fn main() -> Result<(), MomentoError> {
    let cache_client = CacheClient::builder()
        .default_ttl(Duration::from_secs(60))
        .configuration(laptop::latest())
        .credential_provider(CredentialProvider::from_env_var(
            "MOMENTO_API_KEY".to_string(),
        )?)
        .build()?;

    cache_client.create_cache(CACHE_NAME.to_string()).await?;

    cache_client.flush_cache(CACHE_NAME).await?;

    match cache_client.list_caches().await {
        Ok(response) => println!("Caches: {:#?}", response.caches),
        Err(e) => println!("Error listing caches: {}", e),
    }

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

    match cache_client.set_add_elements(
        CACHE_NAME,
        "set_name",
        vec!["value1", "value2"]
    ).await {
        Ok(_) => println!("Elements added to set"),
        Err(e) => println!("Error adding elements to set: {}", e),
    }

    match cache_client.sorted_set_put_element(
        CACHE_NAME,
        "sorted_set_name",
        "value1",
        1.0
    ).await {
        Ok(_) => println!("Element added to sorted set"),
        Err(e) => println!("Error adding element to sorted set: {}", e),
    }

    match cache_client.sorted_set_put_elements(
        CACHE_NAME,
        "sorted_set_name",
        vec![("value2", 2.0), ("value3", 3.0)]
    ).await {
        Ok(_) => println!("Elements added to sorted set"),
        Err(e) => println!("Error adding elements to sorted set: {}", e),
    }

    match cache_client.sorted_set_fetch_by_rank(
        CACHE_NAME,
        "sorted_set_name",
        SortOrder::Ascending,
        Some(1),
        Some(2)
    ).await? {
        SortedSetFetch::Hit { elements } => println!("Elements fetched by rank from sorted set: {:?}", elements.into_strings()),
        SortedSetFetch::Miss => println!("Cache not found"),
    }

    match cache_client.sorted_set_fetch_by_score(
        CACHE_NAME,
        "sorted_set_name",
        SortOrder::Ascending,
    ).await? {
        SortedSetFetch::Hit { elements } => println!("Elements fetched by score from sorted set: {:?}", elements.into_strings()),
        SortedSetFetch::Miss => println!("Cache not found"),
    }

    cache_client.delete_cache(CACHE_NAME).await?;

    Ok(())
}
