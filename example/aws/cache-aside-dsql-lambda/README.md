<img src="https://docs.momentohq.com/img/momento-logo-forest.svg" alt="logo" width="400"/>

[![project status](https://momentohq.github.io/standards-and-practices/badges/project-status-official.svg)](https://github.com/momentohq/standards-and-practices/blob/main/docs/momento-on-github.md)
[![project stability](https://momentohq.github.io/standards-and-practices/badges/project-stability-beta.svg)](https://github.com/momentohq/standards-and-practices/blob/main/docs/momento-on-github.md)


# Momento Rust SDK - AWS Lambda CDK Example

This directory contains an example project defining a Rust-based AWS Lambda function that implements a cache-aside strategy when working with AWS DSQL and uses the Momento Rust SDK to interact with the Cache.

The Lambda function is deployed using the AWS Cloud Development Kit (CDK).

## Example Requirements

- Follow the [installation guide](https://doc.rust-lang.org/cargo/getting-started/installation.html) to install Rust and Cargo.
- You will also need the [cargo-lambda cargo extension](https://www.cargo-lambda.info/)
- The CDK code in this repo is written in TypeScript, so you will need `Node.js` version 16 or later, and a compatible
  version of `npm` installed. If you don't have these, we recommend [nodenv](https://github.com/nodenv/nodenv).
- To get started with Momento you will need a Momento API key. You can get one from the [Momento Console](https://console.gomomento.com).

## Building and Deploying the Lambda Function

This solution uses SQLx which requires connecting to the Database to build the `.sqlx` prepared query directory.  As of this writing, working with it and DSQL is a challenge so the easiest way to get this up and running is to create a local Postgres instance in Docker, build the table there, point the DATABASE_URL at that local container, and then run the DDL queries against DSQL when done.

To build and deploy the Lambda function, first make sure that your AWS credentials are set up properly (via env vars or ~/.aws/credentials file). Then all you need to do is run the following commands:

```bash
cd infra
export DATABASE_URL=<Local Postgre Connection String> 
export CLUSTER_ENDPOINT=<DSQL Cluster Endpoint>
export MOMENTO_API_KEY=<Momento API Key>
cdk deploy
```

After the lambda is deployed, you can use the defined FunctionURL on the Lambda Function and supply and `?id=` to the URL for the Item in your database.  Make sure to build a table in DSQL that matches the Rust `CacheableItem` struct definition.

## Interesting Files in this Example

- `infra/bin/lambda-cache-aside.ts` - this is the CDK stack that defines the Lambda function its associated resources. It uses the `RustFunction` construct from the `cargo-lambda-cdk` package.
- `rust/get-lambda/Cargo.toml` - this is the Cargo.toml file for the Lambda function. It includes the `lambda_runtime`crate as a dependency; this makes it easy to write Lambda functions in Rust
- `rust/get-lambda/src/main.rs` - this is the Rust code for the Lambda function. It uses the AWS `lambda_runtime` crate to implement the `main` function in a way that is compatible with AWS's provided Amazon Linux runtimes.

----------------------------------------------------------------------------------------
For more info, visit our website at [https://gomomento.com](https://gomomento.com)!
