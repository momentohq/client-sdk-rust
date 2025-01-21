mod models;

use aws_config::BehaviorVersion;
use lambda_http::{run, service_fn, Error, IntoResponse, Request, Response};
use models::{MomentoSecretString, TokenRequest, VendedToken};
use momento::{
    auth::{ExpiresIn, PermissionScopes},
    AuthClient, CredentialProvider,
};
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Registry};

// generate_token
//
// uses the Momento auth client to create a disposable
// token that is returned back to the call.  Also includes the value for when the
// token expires so that the client can request a new one
async fn generate_token(
    client: &AuthClient,
    expires_in_minutes: u64,
    cache_name: String,
    topic_name: String,
) -> Result<VendedToken, Error> {
    let expires_in = ExpiresIn::minutes(expires_in_minutes);
    let scopes = PermissionScopes::topic_publish_subscribe(cache_name, topic_name).into();
    let token = client.generate_disposable_token(scopes, expires_in).await?;
    let expires_at = token.clone().expires_at();
    let vended_token = VendedToken {
        auth_token: token.auth_token(),
        expires_at: expires_at.epoch(),
    };

    Ok(vended_token)
}

// handler
//
// function handler which responds to each event (http request) that is supplied into the function
// this will coordinate with the Momento Auth Client to generate the disposable token and return
// success which will have the token and expiration or the failure
async fn handler(
    client: &AuthClient,
    token_expires_in_minutes: u64,
    request: Request,
) -> Result<impl IntoResponse, Error> {
    let body = request.body();
    let body_string = std::str::from_utf8(body)?;
    let parsed_body = serde_json::from_str::<TokenRequest>(body_string);

    match parsed_body {
        Ok(token_request) => {
            let token = generate_token(
                client,
                token_expires_in_minutes,
                token_request.cache_name,
                token_request.topic_name,
            )
            .await?;
            let token_body = serde_json::to_string(&token)?;
            let response = Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .body(token_body)
                .map_err(Box::new)?;
            Ok(response)
        }
        Err(e) => {
            println!("(Error)={}", e);
            let response = Response::builder()
                .status(400)
                .header("Content-Type", "application/json")
                .body("Bad request".to_string())
                .map_err(Box::new)?;
            Ok(response)
        }
    }
}

// main
//
// entry point for the lambda function.  Initializes the Momento Auth Client
// and setups the function handler to be run when new HTTP Requests (events) are
// supplied to Lambda runtime
#[tokio::main]
async fn main() -> Result<(), Error> {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_target(false)
        .without_time();

    Registry::default()
        .with(fmt_layer)
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // aws client and secrets sdk
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let secrets_client = aws_sdk_secretsmanager::Client::new(&config);

    // this is how long the token has before expiring.  If no value is supplied, then the
    // default of 60 seconds is used
    let expires_duration_minutes = u64::try_from(
        env::var("KEY_EXPIRES_DURATION")
            .as_deref()
            .unwrap_or("")
            .parse()
            .unwrap_or(60),
    )?;
    let resp = secrets_client
        .get_secret_value()
        .secret_id("MomentoApiKeySecret")
        .send()
        .await?;
    let string_field = resp
        .secret_string()
        .expect("Secret string must have a value");
    let secret_string: MomentoSecretString = serde_json::from_str(&string_field)
        .expect("Secret string must serde into the correct type");
    let cache_client = AuthClient::builder()
        .credential_provider(CredentialProvider::from_string(
            secret_string.momento_secret,
        )?)
        .build()?;
    let shared_cache_client = &cache_client;

    run(service_fn(move |event: Request| async move {
        handler(shared_cache_client, expires_duration_minutes, event).await
    }))
    .await
}
