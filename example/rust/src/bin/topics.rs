use futures::StreamExt;
use momento::auth::{
    DisposableTokenScope, ExpiresIn, GenerateDisposableTokenRequest, Permission, Permissions,
    TopicPermission, TopicRole,
};
use momento::topics::configurations;
use momento::{AuthClient, CredentialProvider, MomentoResult, TopicClient};
use tokio::sync::oneshot;

#[tokio::main]
async fn main() -> MomentoResult<()> {
    let auth_token = get_topic_client_auth_token().await?;
    let topic_client = TopicClient::builder()
        .configuration(configurations::Laptop::latest())
        .credential_provider(CredentialProvider::from_string(auth_token)?)
        .build()?;

    /*******************************************************************************/

    // Example 1: spawn a task that consumes messages from a subscription and
    // call `abort()` on the task handle after messages are published.
    let mut subscription1 = topic_client.subscribe("cache", "my-topic").await?;
    let subscriber_handle1 = tokio::spawn(async move {
        println!("\n Subscriber [1] should keep receiving until task is aborted");
        while let Some(message) = subscription1.next().await {
            println!(
                "[1] Received message: \n\tKind: {:?} \n\tSequence number: {:?} \n\tSequence page: {:?} \n\tPublisher ID: {:?}", 
                message.kind,
                message.topic_sequence_number,
                message.topic_sequence_page,
                message.publisher_id
            );
        }
    });

    for i in 0..10 {
        topic_client
            .publish("cache", "my-topic", format!("Hello, World! {}", i))
            .await?;
    }

    // Abort the spawned task after messages are published
    subscriber_handle1.abort();

    /*******************************************************************************/

    // Example 2: spawn a task that consumes messages from a subscription and use a
    // single-message channel to indicate the subscriber is done.
    let (sender, receiver) = oneshot::channel();

    let mut subscription2 = topic_client.subscribe("cache", "my-topic").await?;
    let subscriber_handle2 = tokio::spawn(async move {
        println!("\nSubscriber [2] should receive 10 messages then send done message");
        for _ in 0..10 {
            let message = subscription2.next().await;
            match message {
                Some(message) => {
                    println!(
                        "[2] Received message: \n\tKind: {:?} \n\tSequence number: {:?} \n\tSequence page: {:?} \n\tPublisher ID: {:?}", 
                        message.kind,
                        message.topic_sequence_number,
                        message.topic_sequence_page,
                        message.publisher_id
                    );
                }
                None => {
                    println!("[2] Received None item from subscription");
                }
            }
        }
        match sender.send("done") {
            Ok(_) => println!("Subscriber [2] sent done message"),
            Err(err) => println!("Subscriber [2] failed to send done message: {}", err),
        }
    });

    for i in 0..10 {
        topic_client
            .publish("cache", "my-topic", format!("Hello, World! {}", i))
            .await?;
    }

    // After subscriber receives 10 messages, we should receive a "done" message
    // on the oneshot channel and end the subscriber task.
    match receiver.await {
        Ok(_) => {
            println!("Received done message, ending subscriber [2] task");
            subscriber_handle2.abort();
        }
        Err(err) => println!("Failed to receive done message: {}", err),
    }

    Ok(())
}

// This function generates a disposable token with a token ID for the topic client.
// The token ID shows up as the publisher ID on the messages received by the subscriber.
async fn get_topic_client_auth_token() -> MomentoResult<String> {
    let auth_client = AuthClient::builder()
        .credential_provider(CredentialProvider::from_env_var("MOMENTO_API_KEY")?)
        .build()?;
    let expiry = ExpiresIn::minutes(1);
    let scope = DisposableTokenScope::Permissions::<String>(Permissions {
        permissions: vec![Permission::TopicPermission(TopicPermission {
            cache: "cache".into(),
            topic: "my-topic".into(),
            role: TopicRole::PublishSubscribe,
        })],
    });
    let request =
        GenerateDisposableTokenRequest::new(scope, expiry).token_id("my-token-id".to_string());
    let response = auth_client.send_request(request).await?;
    Ok(response.clone().auth_token())
}
