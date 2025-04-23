use std::sync::Arc;
use std::time::Duration;

use momento::cache::CreateCacheResponse;
use once_cell::sync::Lazy;
use tokio::sync::watch::channel;

use crate::test_utils::get_test_auth_cache_name;
use crate::{get_test_cache_name, get_test_credential_provider};
use momento::cache::configurations;
use momento::{AuthClient, CacheClient, LeaderboardClient, TopicClient};

pub static CACHE_TEST_STATE: Lazy<Arc<CacheTestState>> =
    Lazy::new(|| Arc::new(CacheTestState::new()));

pub struct CacheTestState {
    pub client: Arc<CacheClient>,
    pub cache_name: String,
    pub leaderboard_client: Arc<LeaderboardClient>,
    pub topic_client: Arc<TopicClient>,
    pub auth_client: Arc<AuthClient>,
    pub auth_cache_name: String,
    #[allow(dead_code)]
    runtime: tokio::runtime::Runtime,
}

#[allow(clippy::expect_used)] // we want to panic if clients can't be built
impl CacheTestState {
    fn new() -> Self {
        let cache_name = get_test_cache_name();
        println!("Using cache name: {}", cache_name);
        let thread_cache_name = cache_name.clone();

        let auth_cache_name = get_test_auth_cache_name();
        println!("Using auth cache name: {}", auth_cache_name);
        let thread_auth_cache_name = auth_cache_name.clone();

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
                .configuration(configurations::Laptop::latest())
                .credential_provider(credential_provider.clone())
                .build()
                .expect("Failed to create cache client");

            match cache_client.clone().create_cache(thread_cache_name).await {
                Ok(ok) => match ok {
                    CreateCacheResponse::Created => println!("Cache created."),
                    CreateCacheResponse::AlreadyExists => println!("Cache already exists."),
                },
                Err(e) => panic!("Failed to create cache: {:?}", e),
            }

            let leaderboard_client = momento::LeaderboardClient::builder()
                .configuration(momento::leaderboard::configurations::Laptop::latest())
                .credential_provider(credential_provider.clone())
                .build()
                .expect("Failed to create leaderboard client");

            let topic_client = TopicClient::builder()
                .configuration(momento::topics::configurations::Laptop::latest())
                .credential_provider(credential_provider.clone())
                .build()
                .expect("Failed to create topic client");

            let auth_client = AuthClient::builder()
                .credential_provider(credential_provider)
                .build()
                .expect("Failed to create auth client");

            match cache_client
                .clone()
                .create_cache(thread_auth_cache_name)
                .await
            {
                Ok(ok) => match ok {
                    CreateCacheResponse::Created => println!("Auth cache created."),
                    CreateCacheResponse::AlreadyExists => println!("Auth cache already exists."),
                },
                Err(e) => panic!("Failed to create cache: {:?}", e),
            }

            sender
                .send(Some((
                    cache_client,
                    leaderboard_client,
                    topic_client,
                    auth_client,
                )))
                .expect("client should be sent to test state thread");
            thread_barrier.wait();
        });
        barrier.wait();

        // Retrieve the client from the runtime that created it.
        let (client, leaderboard_client, topic_client, auth_client) = client_receiver
            .borrow()
            .as_ref()
            .expect("Clients should already exist")
            .clone();

        CacheTestState {
            client: Arc::new(client.clone()),
            leaderboard_client: Arc::new(leaderboard_client.clone()),
            topic_client: Arc::new(topic_client.clone()),
            auth_client: Arc::new(auth_client.clone()),
            cache_name,
            runtime,
            auth_cache_name,
        }
    }
}
