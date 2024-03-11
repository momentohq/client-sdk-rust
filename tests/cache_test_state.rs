use momento::config::configurations;
use momento::{CacheClient, CredentialProviderBuilder, MomentoError};
use once_cell::sync::Lazy;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch::channel;

pub static TEST_STATE: Lazy<Arc<TestState>> = Lazy::new(|| Arc::new(TestState::new()));

pub struct TestState {
    pub client: Arc<CacheClient>,
    pub cache_name: String,
    #[allow(dead_code)]
    runtime: tokio::runtime::Runtime,
}

impl TestState {
    fn new() -> Self {
        let cache_name =
            env::var("TEST_CACHE_NAME").unwrap_or_else(|_| "rust-test-cache".to_string());
        println!("Using cache name: {}", cache_name);
        let thread_cache_name = cache_name.clone();

        let credential_provider =
            CredentialProviderBuilder::from_environment_variable("TEST_API_KEY".to_string())
                .build()
                .expect("auth token should be valid");

        // The cache client must be created using a separate tokio runtime because each test
        // creates it own runtime, and the client will stop running if its runtime is destroyed.
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let (sender, client_receiver) = channel(None);
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let thread_barrier = barrier.clone();
        runtime.spawn(async move {
            let cache_client = CacheClient::new(
                credential_provider,
                configurations::laptop::latest(),
                Duration::from_secs(5),
            )
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

            sender.send(Some(cache_client)).unwrap();
            thread_barrier.wait();
        });
        barrier.wait();

        // Retrieve the client from the runtime that created it.
        let client = client_receiver
            .borrow()
            .as_ref()
            .expect("Client should already exist")
            .clone();

        TestState {
            client: Arc::new(client),
            cache_name,
            runtime,
        }
    }
}
