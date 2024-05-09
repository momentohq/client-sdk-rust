use futures::StreamExt;
use momento::topics::{configurations, TopicPublish};
use momento::{CredentialProvider, MomentoError, MomentoErrorCode, MomentoResult, TopicClient};
use tokio::task::JoinHandle;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> MomentoResult<()> {
    let topic_client = TopicClient::builder()
        .configuration(configurations::laptop::latest())
        .credential_provider(CredentialProvider::from_env_var("MOMENTO_API_KEY")?)
        .build()?;

    let mut subscription = topic_client.subscribe("cache", "my-topic").await?;
    let mut subscriber_handle: JoinHandle<MomentoResult<()>> = tokio::spawn(async move {
        println!("Subscriber should do some work with the subscription!");
        while let Some(message) = subscription.next().await {
            println!("Received message: {:?}", message);
        }
        Ok(())
    });

    let publish_result = topic_client
        .publish("cache", "my-topic", "Hello, World!")
        .await?;
    match publish_result {
        TopicPublish {} => {
            println!("Publish result is a TopicPublish!");
        }
    }
    sleep(std::time::Duration::from_secs(5)).await;

    // subscription.close().await?;
    println!("Joining subscriber handle");
    subscriber_handle.await.or(Err(MomentoError {
        message: "Subscriber handle failed".to_string(),
        error_code: MomentoErrorCode::UnknownError,
        details: None,
        inner_error: None,
    }))??;
    println!("Joined subscriber handle");

    Ok(())
}
