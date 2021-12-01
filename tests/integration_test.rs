#[cfg(test)]
mod tests {
    use std::{env, time::Duration};

    use momento::{response::cache_get_response::MomentoGetStatus, sdk::Momento};
    use tokio::time::sleep;
    use uuid::Uuid;

    async fn get_momento_instance() -> Momento {
        let auth_token = env::var("TEST_AUTH_TOKEN").expect("env var TEST_AUTH_TOKEN must be set");
        return Momento::new(auth_token).await.unwrap();
    }

    fn get_shared_cache_name() -> String {
        return env::var("TEST_CACHE_NAME").expect("env var TEST_CACHE_NAME must be set");
    }

    #[tokio::test]
    async fn cache_miss() {
        let cache_name = get_shared_cache_name();
        let cache_key = Uuid::new_v4().to_string();
        let mut mm = get_momento_instance().await;
        let cache = mm.get_cache(cache_name.as_str(), 10).await.unwrap();
        let result = cache.get(cache_key).await.unwrap();
        assert!(matches!(result.result, MomentoGetStatus::MISS));
    }

    #[tokio::test]
    async fn cache_hit() {
        let cache_name = get_shared_cache_name();
        let cache_key = Uuid::new_v4().to_string();
        let cache_body = Uuid::new_v4().to_string();
        let mut mm = get_momento_instance().await;
        let cache = mm.get_cache(cache_name.as_str(), 10).await.unwrap();
        cache
            .set(cache_key.clone(), cache_body.clone(), None)
            .await
            .unwrap();
        let result = cache.get(cache_key.clone()).await.unwrap();
        assert!(matches!(result.result, MomentoGetStatus::HIT));
        assert_eq!(result.value, cache_body.as_bytes());
    }

    #[tokio::test]
    async fn cache_respects_default_ttl() {
        let cache_name = get_shared_cache_name();
        let cache_key = Uuid::new_v4().to_string();
        let cache_body = Uuid::new_v4().to_string();
        let mut mm = get_momento_instance().await;
        let cache = mm.get_cache(cache_name.as_str(), 1).await.unwrap();
        cache
            .set(cache_key.clone(), cache_body.clone(), None)
            .await
            .unwrap();
        sleep(Duration::new(1, 0)).await;
        let result = cache.get(cache_key.clone()).await.unwrap();
        assert!(matches!(result.result, MomentoGetStatus::MISS));
    }

    #[tokio::test]
    async fn create_cache_then_set() {
        let cache_name = Uuid::new_v4().to_string();
        let cache_key = Uuid::new_v4().to_string();
        let cache_body = Uuid::new_v4().to_string();
        let mut mm = get_momento_instance().await;
        mm.create_cache(&cache_name).await.unwrap();
        let cache = mm.get_cache(&cache_name, 10).await.unwrap();
        cache
            .set(cache_key.clone(), cache_body.clone(), None)
            .await
            .unwrap();
        let result = cache.get(cache_key.clone()).await.unwrap();
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
