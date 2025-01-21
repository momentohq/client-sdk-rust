<img src="https://docs.momentohq.com/img/momento-logo-forest.svg" alt="logo" width="400"/>

[![project status](https://momentohq.github.io/standards-and-practices/badges/project-status-official.svg)](https://github.com/momentohq/standards-and-practices/blob/main/docs/momento-on-github.md)
[![project stability](https://momentohq.github.io/standards-and-practices/badges/project-stability-beta.svg)](https://github.com/momentohq/standards-and-practices/blob/main/docs/momento-on-github.md)


# Momento Rust SDK - Token Vending Machine

This directory contains a working implementation of a Token Vending Machine that generates and returns temporary and scoped credentials with access to a Momento Topic.  This example is deployed with AWS CDK built in TypeScript that will create the following 2 resources:

1. An AWS Secret which sets a long-lived Momento Token that is capable of creating the disposable tokens needed by a client
2. A Lambda Function that is built in Rust which uses the Momento Auth API to generate the disposable token

*This repository does not decide whether to use an API Gateway endpoint or a FunctionUrl for exposing the Lambda Function.  That distinction is left up to you.*

## Example Requirements

- Follow the [installation guide](https://doc.rust-lang.org/cargo/getting-started/installation.html) to install Rust and Cargo.
- You will also need the [cargo-lambda cargo extension](https://www.cargo-lambda.info/)
- The CDK code in this repo is written in TypeScript, so you will need `Node.js` version 18 or later, and a compatible
  version of `npm` installed. If you don't have these, we recommend [nodenv](https://github.com/nodenv/nodenv).
- To get started with Momento you will need a Momento API key. You can get one from the [Momento Console](https://console.gomomento.com).

## Building, Testing, and Deploying the Lambda Function

To build and test the Lambda function

```bash
npm install -g aws-cdk
cd infra
npm i
export MOMENTO_API_KEY=<YOUR_MOMENTO_API_KEY>
cdk deploy
```

After the lambda is deployed, you can visit the AWS console and click the "Test" button to run it!  You can use the test event in the file `lambdas/test-events/test-event-1.json`. The body of that event will match the `struct` in `lambdas/src/models.rs` documented as the input payload.  The content will be base64 encoded because that's what API Gateway will do to your input.  But when decoded, it will reveal this payload.

```json
echo ewogICAgImNhY2hlTmFtZSI6ICJTYW1wbGVDYWNoZSIsCiAgICAidG9waWNOYW1lIjogInNhbXBsZS10b3BpYyIKfQ== | base64 --decode
{
    "cacheName": "SampleCache",
    "topicName": "SampleTopic"
}
```



## Interesting Files in this Example

- `infra/lib/constructs/function-construct.ts` - this file contains the CDK Construct for building the AWS Secret and the Lambda Function.  When deployed, executing the Lambda Function will generate the disposable token and return that in a response payload. 
- `lambdas/Cargo.toml` - this is the Cargo.toml file for the Lambda function. It includes the `lambda_runtime`
  crate as a dependency; this makes it easy to write Lambda functions in Rust. invocations.
- `lambdas/src/main.rs` - this is the Rust code for the Lambda function. It uses the AWS `lambda_runtime`
  crate to implement the `main` function in a way that is compatible with AWS's provided Amazon Linux runtimes.
- `lambdas/src/models.rs` - this file holds 3 structs which act as models for request/response payloads and for working with the AWS SecretsManager service

## Structs/Payloads

Executing the Lambda Function requires supplying a body over a POST. 

### Requesting a Token 

When requesting a token, the Function expects a payload to look like what's in the cURL request below.

```bash
curl --location --request GET 'https://<api-url>' \
--header 'Content-Type: application/json' \
--data '{
    "cacheName": "SampleCache",
    "topicName": "sample-topic"
}'
```

This request will be deserialized into the following struct.

```rust
#[derive(Deserialize, Debug)]
pub struct TokenRequest {
    #[serde(rename = "cacheName")]
    pub cache_name: String,
    #[serde(rename = "topicName")]
    pub topic_name: String,
}
```

### Function Response

The return response from executing the function will have this shape.

```json
{
    "auth_token": "<JWT_TOKEN>",
    "expires_at": 1736622970
}
```

That response is created by serializing the `VendedToken` struct in the `src/models.rs` file.

```rust
#[derive(Serialize, Debug)]
#[serde(rename = "camelCase")]
pub struct VendedToken {
    pub auth_token: String,
    pub expires_at: u64,
}
```

----------------------------------------------------------------------------------------
For more info, visit our website at [https://gomomento.com](https://gomomento.com)!
