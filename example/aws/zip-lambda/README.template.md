{{ ossHeader }}

# Momento Rust SDK - AWS Lambda CDK Example

This directory contains an example project defining a Rust-based AWS Lambda function that interacts with the Momento Rust SDK.
The Lambda function is packaged as a zip file that you can upload via the AWS console.

## Example Requirements

- Follow the [installation guide](https://doc.rust-lang.org/cargo/getting-started/installation.html) to install Rust and Cargo.
- You will also need the [cargo-lambda cargo extension](https://www.cargo-lambda.info/)
- To get started with Momento you will need a Momento API key. You can get one from the [Momento Console](https://console.gomomento.com).

## Creating your Lambda Function through the AWS console

Before building and deploying the Rust code, you'll need to create your lambda function:

- In the AWS console, create a new function that uses the "Amazon Linux 2023" runtime. Other than selecting that runtime,
  the default values are fine for the remaining options.
- After creating the function you'll need to go to the "Configuration" tab and add an environment variable called
  `MOMENTO_API_KEY`. Set the value to the Momento API key that you created in the Momento console.

## Building the Lambda Function Rust code


To build the zip archive containing the Rust Lambda Function, run the following commands:

```bash
cd momento-simple-lambda
cargo lambda build --release --output-format zip
```

After the build completes, you should see your lambda zip archive at this path: `./target/lambda/momento-simple-lambda/bootstrap.zip`.

## Deploying the Lambda Function

Now, back to the AWS lambda console. On the "Code" tab, click the "Upload from" dropdown and select "Zip file". Browse your
filesystem to find the zip archive you just built, and upload it.

After the lambda is deployed, you can visit the AWS console and click the "Test" button to run it! The function will
do a simple Momento cache set followed by a cache get, and log the results.

## Interesting Files in this Example

- `momento-simple-lambda/Cargo.toml` - this is the Cargo.toml file for the Lambda function. It includes the `lambda_runtime`
  crate as a dependency; this makes it easy to write Lambda functions in Rust. It also includes the `lazy_static` crate,
  which allows us to re-use the Momento client object across multiple function invocations.
- `momento_simple_lambda/src/main.rs` - this is the Rust code for the Lambda function. It uses the AWS `lambda_runtime`
  crate to implement the `main` function in a way that is compatible with AWS's provided Amazon Linux runtimes.

{{ ossFooter }}
