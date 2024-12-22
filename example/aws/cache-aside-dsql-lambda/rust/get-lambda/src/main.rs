use std::{env, str::FromStr, time::Duration};

use ::tracing::Instrument;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_dsql::auth_token::{AuthTokenGenerator, Config};
use lambda_http::{
    http::{Response, StatusCode},
    run, service_fn, Error, IntoResponse, Request, RequestExt,
};
use momento::{cache::configurations, CacheClient, CredentialProvider, MomentoError};
use serde_json::json;
use shared::models::model::CacheableItem;
use sqlx::PgPool;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    query_as,
};
use tracing::instrument;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Registry};
use uuid::Uuid;

/// Handles the writing/setting of the item to be cached into the Momento cache
#[instrument(name = "Write Cache")]
async fn write_to_cache(client: &CacheClient, cache_name: String, item: CacheableItem) {
    // query span to track latency of the Momento Set operation
    let query_span = tracing::info_span!("Momento SET");

    let value = serde_json::to_string(&item).unwrap();
    let result = client
        .set(cache_name, item.id.to_string(), value.clone())
        .instrument(query_span)
        .await;

    match result {
        Ok(_) => {
            tracing::info!("Cache item set");
            tracing::info!("(Item)={:?}", value);
        }
        Err(e) => {
            tracing::error!("(CacheWriteError)={}", e);
        }
    }
}

/// Run a GET operation on the Momento cache to query for the item
///     if the item is found, returns Some
///     None if not
#[instrument(name = "Query Cache")]
async fn query_cache(
    client: &CacheClient,
    cache_name: String,
    id: String,
) -> Option<CacheableItem> {
    // span for tracing the latency of the Momento GET
    let query_span = tracing::info_span!("Momento GET");
    let response = client.get(cache_name, id).instrument(query_span).await;

    match response {
        Ok(r) => {
            let item: Result<String, MomentoError> = r.try_into();

            match item {
                Ok(i) => {
                    let o: CacheableItem = serde_json::from_str(i.as_str()).unwrap();
                    tracing::info!("(CacheItem)={:?}", o);
                    Some(o)
                }
                Err(e) => {
                    tracing::info!("(Cache MISS)={}", e);
                    None
                }
            }
        }
        Err(e) => {
            tracing::error!("(GetResponseError)={}", e);
            None
        }
    }
}

/// Queries the DSQL table looking for the supplied item
#[instrument(name = "DSQL Query")]
async fn query_row(pool: &PgPool, u: Uuid) -> Option<CacheableItem> {
    /// span for tracking the latency of the query
    let query_span = tracing::info_span!("DSQL Read");
    let item = query_as!(
        CacheableItem,
        "select id, first_name, last_name, created_at, updated_at from CacheableTable where id = $1",
        u
    )
    .fetch_optional(pool)
        .instrument(query_span)
    .await;

    /// return the unwrapped item OR return the default implementation
    item.unwrap_or_default()
}

/// Main function handle that is executed on every supplied payload to the
/// Lambda function.  function_handler coordinates the operations with
/// the Momento cache_client, the pool (Postgres) and processes the
/// request: Request
#[instrument(name = "Function Handler")]
async fn function_handler(
    pool: &PgPool,
    cache_client: &CacheClient,
    cache_name: &str,
    request: Request,
) -> Result<impl IntoResponse, Error> {
    let id = request
        .query_string_parameters_ref()
        .and_then(|params| params.first("id"))
        .unwrap();

    let mut body = json!("").to_string();
    let mut status_code = StatusCode::OK;
    let u = Uuid::from_str(id).unwrap();
    /// queries the cache first for the item
    let cache_item = query_cache(cache_client, cache_name.to_owned(), id.to_string()).await;

    match cache_item {
        // returns the queried item if Momento has it
        Some(i) => {
            tracing::info!("Cache HIT!");
            body = serde_json::to_string(&i).unwrap();
        }
        // if Momento doesn't have it, go to DSQL
        None => {
            tracing::info!("Cache MISS!");
            let item = query_row(pool, u).await;
            match item {
                Some(i) => {
                    write_to_cache(cache_client, cache_name.to_owned(), i.clone()).await;
                    body = serde_json::to_string(&i).unwrap();
                }
                None => {
                    status_code = StatusCode::NOT_FOUND;
                }
            }
        }
    }

    let response = Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(Box::new)?;
    Ok(response)
}

/// Main is the initialization function of the Lambda.  This runs only
/// once and sets up the reusable dependencies that only require setup once.
///
/// Also establishes the event loop for passing payloads back into the function_handler
#[tokio::main]
async fn main() -> Result<(), Error> {
    let logger = tracing_subscriber::fmt::layer().json().flatten_event(true);
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .without_time();

    Registry::default()
        .with(fmt_layer)
        .with(logger)
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let region = "us-east-1";
    let cluster_endpoint = env::var("CLUSTER_ENDPOINT").expect("CLUSTER_ENDPOINT required");
    let momento_key = env::var("MOMENTO_API_KEY").expect("MOMENTO_API_KEY required");
    let cache_name = env::var("CACHE_NAME").expect("CACHE_NAME required");

    // Generate auth token
    let sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let signer = AuthTokenGenerator::new(
        Config::builder()
            .hostname(&cluster_endpoint)
            .region(Region::new(region))
            .build()
            .unwrap(),
    );
    let password_token = signer
        .db_connect_admin_auth_token(&sdk_config)
        .await
        .unwrap();

    // Setup connections
    let connection_options = PgConnectOptions::new()
        .host(cluster_endpoint.as_str())
        .port(5432)
        .database("postgres")
        .username("admin")
        .password(password_token.as_str())
        .ssl_mode(sqlx::postgres::PgSslMode::VerifyFull);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect_with(connection_options.clone())
        .await?;
    let shared_pool = &pool;

    let cache_client = CacheClient::builder()
        .default_ttl(Duration::from_secs(5))
        .configuration(configurations::Lambda::latest())
        .credential_provider(CredentialProvider::from_string(momento_key).unwrap())
        .build()?;

    let shared_cache_client = &cache_client;
    let shared_cache_name = &cache_name;

    run(service_fn(move |event: Request| async move {
        function_handler(shared_pool, shared_cache_client, shared_cache_name, event).await
    }))
    .await
}
