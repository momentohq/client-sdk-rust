use std::sync::Arc;
use std::time::Duration;

use once_cell::sync::Lazy;
use tokio::sync::watch::channel;

use crate::{get_test_cache_name, get_test_credential_provider};
use momento::config::configurations;
use momento::{CacheClient, MomentoError};

pub static CACHE_TEST_STATE: Lazy<Arc<CacheTestState>> =
    Lazy::new(|| Arc::new(CacheTestState::new()));

pub struct CacheTestState {
    pub client: Arc<CacheClient>,
    pub cache_name: String,
    #[allow(dead_code)]
    runtime: tokio::runtime::Runtime,
}

impl CacheTestState {
    fn new() -> Self {
        let cache_name = get_test_cache_name();
        println!("Using cache name: {}", cache_name);
        let thread_cache_name = cache_name.clone();

        let credential_provider = get_test_credential_provider();

        // The cache client must be created using a separate tokio runtime because each test
        // creates it own runtime, and the client will stop running if its runtime is destroyed.
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("test state tokio runtime failure");
        let (sender, client_receiver) = channel(None);
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let thread_barrier = barrier.clone();
        runtime.spawn(async move {
            let cache_client = CacheClient::builder()
                .default_ttl(Duration::from_secs(5))
                .configuration(configurations::laptop::latest())
                .credential_provider(credential_provider)
                .build()
                .expect("Failed to create cache client");

            match cache_client.clone().create_cache(thread_cache_name).await {
                Ok(_) => {}
                Err(e) => match e {
                    MomentoError::AlreadyExists { .. } => {
                        println!("Cache already exists.");
                    }
                    _ => {
                        panic!("Failed to create cache: {:?}", e);
                    }
                },
            }

            sender
                .send(Some(cache_client))
                .expect("client should be sent to test state thread");
            thread_barrier.wait();
        });
        barrier.wait();

        // Retrieve the client from the runtime that created it.
        let client = client_receiver
            .borrow()
            .as_ref()
            .expect("Client should already exist")
            .clone();

        CacheTestState {
            client: Arc::new(client),
            cache_name,
            runtime,
        }
    }
}
