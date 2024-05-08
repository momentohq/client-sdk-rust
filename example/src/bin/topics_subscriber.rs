use momento::{CredentialProvider, MomentoError, TopicClient};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), MomentoError> {
    let topic_client = TopicClient::connect(
        CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string())?,
        None,
    )
    .expect("could not connect");

    // Make a subscription
    let mut subscription = topic_client
        .subscribe("cache", "topic_name", None)
        .await
        .expect("subscribe rpc failed");

    // Consume the subscription
    while let Some(item) = subscription.next().await {
        println!("Received subscription item: {item:?}")
    }

    Ok(())
}
