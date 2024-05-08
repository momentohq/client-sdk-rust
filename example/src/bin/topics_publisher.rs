use momento::{CredentialProvider, MomentoError, TopicClient};

#[tokio::main]
async fn main() -> Result<(), MomentoError> {
    let topic_client = TopicClient::connect(
        CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string())?,
        None,
    )
    .expect("could not connect");

    // publish 10 messages 1 second apart
    for i in 0..10 {
        let message = format!("Hello, Momento! {}", i);
        topic_client
            .publish("cache", "topic_name", &*message)
            .await?;
        println!("Published message: {}", message);
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    Ok(())
}
