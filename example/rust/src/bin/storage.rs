use std::process;

use momento::{
    storage::{configurations, GetResponse, ListStoresResponse},
    CredentialProvider, MomentoError, PreviewStorageClient,
};

#[tokio::main]
async fn main() -> Result<(), MomentoError> {
    let storage_client = match PreviewStorageClient::builder()
        .configuration(configurations::Laptop::latest())
        .credential_provider(
            CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string())
                .expect("auth token should be valid"),
        )
        .build()
    {
        Ok(client) => client,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };

    let store_name = "my_momento_store";
    storage_client.create_store(store_name).await?;

    // List all stores and validate my_momento_store was created
    let list_stores_response = storage_client.list_stores().await?;
    if !store_exists(&list_stores_response, store_name) {
        eprintln!("{store_name} was not created");
        process::exit(1);
    }
    println!("{store_name} was created");

    let key = "foo";
    let value = "bar";

    // Put a key in store
    match storage_client.put(store_name, key, value).await {
        Ok(_) => {
            println!("Key {key} was successfully persisted in {store_name}");
        }
        Err(err) => {
            eprintln!("Failed to persist key {key} in {store_name}: {err}");
            process::exit(1);
        }
    }

    // Get the key from store and validate it got persisted
    match storage_client.get(store_name, key).await {
        Ok(res) => match res {
            GetResponse::Found { value } => {
                let found_value: String = value.try_into()?;
                println!("Key {key} was found with value {found_value}")
            }
            GetResponse::NotFound => println!("Key {key} not found in {store_name}"),
        },
        Err(err) => {
            eprint!("error while getting key: {err}");
            process::exit(1);
        }
    }

    // Delete the key from storage
    match storage_client.delete(store_name, key).await {
        Ok(_) => {
            println!("Key {key} was successfully deleted from {store_name}");
        }
        Err(err) => {
            eprint!("error while deleting key: {err}");
            process::exit(1);
        }
    }

    // Get the key from storage and validate it doesn't exist
    match storage_client.get(store_name, key).await {
        Ok(res) => match res {
            GetResponse::NotFound => println!("Key {key} not found in {store_name}"),
            GetResponse::Found { value } => {
                let found_value: String = value.try_into()?;
                eprint!("Key {key} should have been deleted; instead got value as {found_value}");
                process::exit(1);
            }
        },
        Err(err) => {
            eprint!("error while getting key: {err}");
            process::exit(1);
        }
    }

    // Delete store
    storage_client.delete_store(store_name).await?;

    // Validate store was deleted
    let list_stores_response = storage_client.list_stores().await?;
    if store_exists(&list_stores_response, store_name) {
        eprintln!("{store_name} was not deleted");
        process::exit(1);
    }

    println!("Store was deleted");

    Ok(())
}

fn store_exists(list_stores_response: &ListStoresResponse, store_name: &str) -> bool {
    list_stores_response
        .stores
        .iter()
        .any(|x| x.name == store_name)
}
