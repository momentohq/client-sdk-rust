use base64::prelude::*;
use lambda_http::{
    http::{Response, StatusCode},
    run, service_fn, Error, IntoResponse, Request,
};
use momento::{CacheClient, CredentialProvider};
use std::time::Duration;

async fn function_handler(event: Request) -> Result<impl IntoResponse, Error> {
    println!("event data payload: {:?}", event);

    let api_key = event.headers().get("authorization").unwrap().to_str()?;
    println!("got api key: {:?}", api_key);

    let creds = CredentialProvider::from_string(api_key);
    match creds {
        Ok(creds) => {
            println!("creds: {:?}", creds);
            let momento = CacheClient::builder()
                .default_ttl(Duration::from_secs(60))
                .configuration(momento::cache::configurations::Lambda::latest())
                .credential_provider(creds)
                .build()?;

            let list_caches_resp = momento.list_caches().await?;
            print!("list_caches_resp: {:?}", list_caches_resp);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/grpc")
        .header("grpc-status", 0)
        .header("grpc-message", "Ok")
        .body(BASE64_STANDARD.encode("Hello AWS Lambda HTTP request"))
        .map_err(Box::new)?;

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(function_handler)).await
}
