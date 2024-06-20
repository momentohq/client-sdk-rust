use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::primitives::Blob;
use aws_sdk_dynamodb::types::AttributeValue;
use momento::{CredentialProvider, MomentoError, storage};

#[tokio::main]
async fn main() -> Result<(), MomentoError> {
    println!("Hello, storage!");
    let ddb_client = aws_sdk_dynamodb::Client::new(
        &aws_config::defaults(BehaviorVersion::latest())
            .region("us-west-2")
            .load()
            .await,
    );

    let table_name = "DELETEME-ddbtest";

    let put_resp = ddb_client
        .put_item()
        .table_name(table_name)
        .item("id", AttributeValue::S("1".into()))
        .item("number", AttributeValue::N("42".into()))
        .item("bytes", AttributeValue::B(Blob::new(vec![1, 2, 3])))
        .send()
        .await
        .expect("derp");

    println!("Put response: {:?}", put_resp);

    let get_resp = ddb_client
        .get_item()
        .table_name(table_name)
        .key("id", AttributeValue::S("1".into()))
        .send()
        .await
        .expect("derp");

    println!("Get response: {:?}", get_resp);

    let number = get_resp
        .item()
        .expect("item not found")
        .get("number")
        .expect("number not found")
        .as_n()
        .expect("number not a number");

    println!("Number: {:?}", number);

    let storage_client = momento::PreviewStorageClient::builder()
        .configuration(storage::configurations::Laptop::latest())
        .credential_provider(
            CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string())
                .expect("auth token should be valid")
        ).build()?;

    let store_name = "store";

    let put_result = storage_client.put(store_name, "number", 42).await?;
    println!("Momento put result: {:?}", put_result);

    let get_result = storage_client.get(store_name, "number").await?;
    println!("Momento get result: {:?}", get_result);

    let put_result2 = storage_client.put(store_name, "double", 42.0).await?;
    println!("Momento put result2: {:?}", put_result2);

    let get_result2 = storage_client.get(store_name, "double").await?;
    println!("Momento get result2: {:?}", get_result2);

    match get_result2.clone().value {
        None => {
            println!("get result 2 not found!");
        }
        Some(v) => {
            println!("get result 2 found: {:?}", v);
            let get_result2_f64: f64 = v.try_into().expect("double not a double");
            println!("get result 2 f64: {:?}", get_result2_f64);
        }
    }


    let double_option: Option<f64> = get_result2.clone().try_into().expect("double not a double");
    println!("DoubleOption: {:?}", double_option);
    let double_f64: f64 = get_result2.clone().try_into().expect("double not a double");
    println!("Double as f64: {:?}", double_f64);
    
    // these lines compile/run but fail at runtime as expected, so commented out
    // let double_string: String = get_result2.try_into().expect("double not a string");
    // println!("Double string: {:?}", double_string);

    let not_found_result = storage_client.get(store_name, "not_found").await?;
    println!("Momento not found result: {:?}", not_found_result);
    println!("Is not_found_result None? {:?}", not_found_result.value.is_none());
    let not_found_as_option_f64: Option<f64> = not_found_result.clone().try_into().expect("not found not a double");
    println!("not_found as option f64: {:?}", not_found_as_option_f64);

    // these lines compile/run but fail at runtime as expected, so commented out
    // let not_found_as_f64: f64 = not_found_result.try_into().expect("not found not a double");
    // println!("not_found as f64: {:?}", not_found_as_f64);
    
    Ok(())
}
