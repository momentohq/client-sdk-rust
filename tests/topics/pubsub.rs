use futures::StreamExt;
use momento::topics::TopicPublishResponse;
use momento::{MomentoErrorCode, MomentoResult};
use momento_test_util::CACHE_TEST_STATE;
use momento_test_util::{unique_cache_name, unique_topic_name};

mod publish_and_subscribe {
    use std::convert::TryInto;

    use super::*;

    #[tokio::test]
    async fn nonexistentent_cache_returns_not_found() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.topic_client;
        let cache_name = unique_cache_name();

        let result = client
            .publish(&cache_name, "topic", "value")
            .await
            .unwrap_err();
        assert_eq!(result.error_code, MomentoErrorCode::CacheNotFoundError);

        // We know that subscribing to a nonexistent cache is going to produce a NotFoundError,
        // but because Subscription can't implement the Debug macro, we can't use the safe version,
        // `unwrap_err()`, to get the error code, so we must use the unsafe version in an unsafe block.
        let result = unsafe {
            client
                .subscribe(&cache_name, "topic")
                .await
                .unwrap_err_unchecked()
        };
        assert_eq!(result.error_code, MomentoErrorCode::CacheNotFoundError);

        Ok(())
    }

    #[tokio::test]
    async fn publish_and_subscribe() -> MomentoResult<()> {
        let client = &CACHE_TEST_STATE.topic_client;
        let cache_name = &CACHE_TEST_STATE.cache_name;
        let topic_name = unique_topic_name();

        let mut subscription = client.subscribe(cache_name, &topic_name).await?;
        let subscription_handle = tokio::spawn(async move {
            while let Some(message) = subscription.next().await {
                let message_text: String =
                    message.try_into().expect("Expected to receive a string");
                assert_eq!(message_text, "value");
            }
        });

        let result = client.publish(cache_name, &topic_name, "value").await?;
        assert_eq!(result, TopicPublishResponse {});

        subscription_handle.abort();
        Ok(())
    }
}
