use aws_lambda_events::{
    apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse},
    http::HeaderMap,
};
use base64::prelude::*;
use lambda_http::lambda_runtime::{run, service_fn, Error, LambdaEvent};

async fn handler(
    event: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, Error> {
    println!("event data payload: {:?}", event);

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/grpc+proto".parse().unwrap());
    headers.insert("grpc-status", "0".parse().unwrap());
    let resp = ApiGatewayProxyResponse {
        status_code: 200,
        multi_value_headers: headers.clone(),
        is_base64_encoded: Some(true),
        body: Some(aws_lambda_events::encodings::Body::Text(BASE64_STANDARD.encode("Hello AWS Lambda HTTP request"))),
        headers,
    };
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
