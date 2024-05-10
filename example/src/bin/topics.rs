use futures::StreamExt;
use momento::topics::configurations;
use momento::{CredentialProvider, MomentoResult, TopicClient};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> MomentoResult<()> {
    let topic_client = TopicClient::builder()
        .configuration(configurations::Laptop::latest())
        .credential_provider(CredentialProvider::from_env_var("MOMENTO_API_KEY")?)
        .build()?;

    /*******************************************************************************/

    // Example 1: spawn a task that consumes messages from a subscription and
    // call `abort()` on the task handle after messages are published.
    let mut subscription1 = topic_client.subscribe("cache", "my-topic").await?;
    let subscriber_handle1 = tokio::spawn(async move {
        println!("Subscriber should keep receiving until task is aborted");
        while let Some(message) = subscription1.next().await {
            println!("[1] Received message: {:?}", message);
        }
    });

    for i in 0..10 {
        topic_client
            .publish("cache", "my-topic", format!("Hello, World! {}", i))
            .await?;
        sleep(std::time::Duration::from_millis(400)).await;
    }

    // Abort the spawned task after messages are published
    subscriber_handle1.abort();

    /*******************************************************************************/

    // Example 2: spawn a task that consumes messages from a subscription and
    // let the task end after receiving 10 messages.
    let mut subscription2 = topic_client.subscribe("cache", "my-topic").await?;
    tokio::spawn(async move {
        println!("Subscriber should receive 10 messages then exit");
        for _ in 0..10 {
            let message = subscription2.next().await;
            println!("[2] Received message: {:?}", message);
        }
    });

    for i in 0..10 {
        topic_client
            .publish("cache", "my-topic", format!("Hello, World! {}", i))
            .await?;
        sleep(std::time::Duration::from_millis(400)).await;
    }

    Ok(())
}
