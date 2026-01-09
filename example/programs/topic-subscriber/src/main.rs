mod models;

use futures::StreamExt;
use momento::{topics::Subscription, CredentialProvider, MomentoError, TopicClient};
use std::{env, error::Error};
use tracing::{error, info};

use crate::models::MomentoModel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .json()
        .init();

    // create a new Momento client
    let topic_client = TopicClient::builder()
        .configuration(momento::topics::configurations::Laptop::latest())
        .credential_provider(CredentialProvider::from_default_env_var_v2()?)
        .build()?;

    // variables for topic and cache are required
    let cache = env::var("CACHE").expect("CACHE Variable is required");
    let topic = env::var("TOPIC").expect("TOPIC Variable is required");

    let mut subscription: Subscription = topic_client.subscribe(cache, topic).await?;
    // Consume the subscription
    while let Some(item) = subscription.next().await {
        info!("Received subscription item: {item:?}");
        let value: Result<String, MomentoError> = item.try_into();
        match value {
            Ok(v) => {
                let o: MomentoModel = serde_json::from_str(v.as_str())?;
                info!(
                    "(Value)={}|(MoModel)={o:?}|(MillisecondsBetween)={}",
                    v,
                    o.milliseconds_between_publish_and_received()
                );
            }
            Err(e) => {
                error!("(Error Momento)={}", e);
            }
        }
    }

    Ok(())
}
