#[cfg(test)]
mod tests {
    use std::{env, time::Duration};

    use momento::{
        response::cache_get_response::MomentoGetStatus, simple_cache_client::SimpleCacheClient,
    };
    use tokio::time::sleep;
    use uuid::Uuid;

    async fn get_momento_instance() -> SimpleCacheClient {
        let auth_token = env::var("TEST_AUTH_TOKEN").expect("env var TEST_AUTH_TOKEN must be set");
        return SimpleCacheClient::new(auth_token, 5).await.unwrap();
    }

    fn get_shared_cache_name() -> String {
        return env::var("TEST_CACHE_NAME").expect("env var TEST_CACHE_NAME must be set");
    }

    #[tokio::test]
    async fn cache_miss() {
        let cache_name = get_shared_cache_name();
        let cache_key = Uuid::new_v4().to_string();
        let mut mm = get_momento_instance().await;
        let result = mm.get(cache_name, cache_key).await.unwrap();
        assert!(matches!(result.result, MomentoGetStatus::MISS));
    }

    #[tokio::test]
    async fn cache_hit() {
        let cache_name = get_shared_cache_name();
        let cache_key = Uuid::new_v4().to_string();
        let cache_body = Uuid::new_v4().to_string();
        let mut mm = get_momento_instance().await;
        mm.set(
            cache_name.clone(),
            cache_key.clone(),
            cache_body.clone(),
            None,
        )
        .await
        .unwrap();
        let result = mm.get(cache_name.clone(), cache_key.clone()).await.unwrap();
        assert!(matches!(result.result, MomentoGetStatus::HIT));
        assert_eq!(result.value, cache_body.as_bytes());
    }

    #[tokio::test]
    async fn cache_respects_default_ttl() {
        let cache_name = get_shared_cache_name();
        let cache_key = Uuid::new_v4().to_string();
        let cache_body = Uuid::new_v4().to_string();
        let mut mm = get_momento_instance().await;
        mm.set(
            cache_name.clone(),
            cache_key.clone(),
            cache_body.clone(),
            None,
        )
        .await
        .unwrap();
        sleep(Duration::new(1, 0)).await;
        let result = mm.get(cache_name.clone(), cache_key.clone()).await.unwrap();
        assert!(matches!(result.result, MomentoGetStatus::MISS));
    }

    #[tokio::test]
    async fn create_cache_then_set() {
        let cache_name = Uuid::new_v4().to_string();
        let cache_key = Uuid::new_v4().to_string();
        let cache_body = Uuid::new_v4().to_string();
        let mut mm = get_momento_instance().await;
        mm.create_cache(&cache_name).await.unwrap();
        mm.set(
            cache_name.clone(),
            cache_key.clone(),
            cache_body.clone(),
            None,
        )
        .await
        .unwrap();
        let result = mm.get(cache_name.clone(), cache_key.clone()).await.unwrap();
        assert!(matches!(result.result, MomentoGetStatus::HIT));
        assert_eq!(result.value, cache_body.as_bytes());
        mm.delete_cache(&cache_name).await.unwrap();
    }

    #[tokio::test]
    async fn list_caches() {
        let mut mm = get_momento_instance().await;
        mm.list_caches(None).await.unwrap();
    }
}
