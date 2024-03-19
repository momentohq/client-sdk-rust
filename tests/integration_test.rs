#[cfg(test)]
mod tests {
    use std::{env, time::Duration};

    use momento::response::{Get, GetValue};
    use momento::{CredentialProvider, SimpleCacheClient};
    use momento::{MomentoError, SimpleCacheClientBuilder};
    use serde_json::Value;
    use tokio::time::sleep;
    use uuid::Uuid;

    fn hit(value: impl Into<Vec<u8>>) -> Get {
        Get::Hit {
            value: GetValue::new(value.into()),
        }
    }

    fn get_momento_instance_with_token(
        auth_token: String,
    ) -> Result<SimpleCacheClientBuilder, MomentoError> {
        SimpleCacheClientBuilder::new_with_explicit_agent_name(
            CredentialProvider::from_string(auth_token)?,
            Duration::from_secs(5),
            "integration_test",
        )
    }

    fn get_momento_instance() -> SimpleCacheClient {
        let auth_token = env::var("MOMENTO_API_KEY").expect("env var MOMENTO_API_KEY must be set");
        get_momento_instance_with_token(auth_token)
            .expect("failed to build an integration test client builder")
            .build()
    }

    fn create_random_cache_name() -> String {
        "rust-sdk-".to_string() + &Uuid::new_v4().to_string()
    }

    #[tokio::test]
    async fn cache_miss() {
        let cache_name = create_random_cache_name();
        let cache_key = Uuid::new_v4().to_string();
        let mut mm = get_momento_instance();
        mm.create_cache(&cache_name)
            .await
            .expect("failed to create cache");
        let result = mm
            .get(&cache_name, cache_key)
            .await
            .expect("failure when trying get");
        assert_eq!(result, Get::Miss);
        mm.delete_cache(&cache_name)
            .await
            .expect("failed to delete cache");
    }

    #[tokio::test]
    async fn cache_validation() {
        let cache_name = "";
        let mut mm = get_momento_instance();
        let result = mm.create_cache(cache_name).await.unwrap_err();
        let _err_msg = "Cache name cannot be empty".to_string();
        assert!(matches!(result.to_string(), _err_message))
    }

    #[tokio::test]
    async fn key_id_validation() {
        let key_id = "";
        let mut mm = get_momento_instance();
        let result = mm.revoke_signing_key(key_id).await.unwrap_err();
        let _expected = "".to_string();
        assert!(matches!(result.to_string(), _expected))
    }

    #[tokio::test]
    async fn ttl_validation() {
        let cache_name = create_random_cache_name();
        let cache_key = Uuid::new_v4().to_string();
        let cache_body = Uuid::new_v4().to_string();
        let mut mm = get_momento_instance();
        mm.create_cache(&cache_name)
            .await
            .expect("failed to create cache");
        let ttl: u64 = 18446744073709551615;
        let max_ttl = u64::MAX / 1000_u64;
        let result = mm
            .set(&cache_name, cache_key, cache_body, Duration::from_secs(ttl)) // 18446744073709551615 > 2^64/1000
            .await
            .unwrap_err();
        let _err_message =
            format!("TTL provided, {ttl}, needs to be less than the maximum TTL {max_ttl}");
        assert!(matches!(result.to_string(), _err_message));
        mm.delete_cache(&cache_name)
            .await
            .expect("failed to delete cache");
    }

    #[tokio::test]
    async fn cache_hit() {
        let cache_name = create_random_cache_name();
        let cache_key = Uuid::new_v4().to_string();
        let cache_body = Uuid::new_v4().to_string();
        let mut mm = get_momento_instance();
        mm.create_cache(&cache_name)
            .await
            .expect("failed to create cache");
        mm.set(&cache_name, cache_key.clone(), cache_body.clone(), None)
            .await
            .expect("failed to perform set");
        let result = mm
            .get(&cache_name, cache_key.clone())
            .await
            .expect("failed to perform get");
        assert_eq!(result, hit(cache_body));
        mm.delete_cache(&cache_name)
            .await
            .expect("failed to delete cache");
    }

    #[tokio::test]
    async fn cache_respects_default_ttl() {
        let cache_name = create_random_cache_name();
        let cache_key = Uuid::new_v4().to_string();
        let cache_body = Uuid::new_v4().to_string();
        let mut mm = get_momento_instance();
        mm.create_cache(&cache_name)
            .await
            .expect("failed to create cache");
        mm.set(&cache_name, cache_key.clone(), cache_body.clone(), None)
            .await
            .expect("failed to perform set");
        sleep(Duration::new(1, 0)).await;
        let result = mm
            .get(&cache_name, cache_key.clone())
            .await
            .expect("failed to perform get");
        assert_eq!(result, hit(cache_body));
        mm.delete_cache(&cache_name)
            .await
            .expect("failed to delete cache");
    }

    #[tokio::test]
    async fn create_cache_then_set() {
        let cache_name = create_random_cache_name();
        let cache_key = Uuid::new_v4().to_string();
        let cache_body = Uuid::new_v4().to_string();
        let mut mm = get_momento_instance();
        mm.create_cache(&cache_name)
            .await
            .expect("failed to create cache");
        mm.set(&cache_name, cache_key.clone(), cache_body.clone(), None)
            .await
            .expect("failed to perform set");
        let result = mm
            .get(&cache_name, cache_key.clone())
            .await
            .expect("failed to perform get");
        assert_eq!(result, hit(cache_body));
        mm.delete_cache(&cache_name)
            .await
            .expect("failed to delete cache");
    }

    #[tokio::test]
    async fn list_caches() {
        let cache_name = create_random_cache_name();
        let mut mm = get_momento_instance();
        mm.create_cache(&cache_name)
            .await
            .expect("failed to create cache");

        let list_cache_result = mm.list_caches().await.expect("failed to list caches");

        assert!(!list_cache_result.caches.is_empty());
        mm.delete_cache(&cache_name)
            .await
            .expect("failed to delete cache");
    }

    #[tokio::test]
    async fn flush_cache() {
        let cache_name = create_random_cache_name();
        let mut client = get_momento_instance();
        client
            .create_cache(&cache_name)
            .await
            .expect("failed to create cache");
        client
            .set(
                &cache_name,
                "firstKey",
                "firstValue",
                Duration::from_secs(60),
            )
            .await
            .expect("set firstKey failed");
        client
            .set(
                &cache_name,
                "secondKey",
                "secondValue",
                Duration::from_secs(60),
            )
            .await
            .expect("set secondKey failed");

        let first_key_get1 = client
            .get(&cache_name, "firstKey")
            .await
            .expect("failed to get first key");
        assert_eq!(first_key_get1, hit("firstValue"));

        let second_key_get1 = client
            .get(&cache_name, "secondKey")
            .await
            .expect("failed to get second key");
        assert_eq!(second_key_get1, hit("secondValue"));

        client
            .flush_cache(&cache_name)
            .await
            .expect("failed to flush cache");

        let first_key_get2 = client
            .get(&cache_name, "firstKey")
            .await
            .expect("failed to get first key");
        assert_eq!(first_key_get2, Get::Miss);

        let second_key_get2 = client
            .get(&cache_name, "secondKey")
            .await
            .expect("failed to get second key");
        assert_eq!(second_key_get2, Get::Miss);
        client
            .delete_cache(&cache_name)
            .await
            .expect("failed to delete cache");
    }

    #[tokio::test]
    async fn create_list_revoke_signing_key() {
        let mut mm = get_momento_instance();
        let response = mm
            .create_signing_key(10)
            .await
            .expect("failed to create signing key");

        let key: Value =
            serde_json::from_str(&response.key).expect("failed to parse key from json");
        let obj = key.as_object().expect("failed to map key to object");
        let kid = obj.get("kid").expect("'kid' was not present");
        assert_eq!(
            kid.as_str().expect("failed to cast kid to str"),
            response.key_id
        );

        let list_response = mm
            .list_signing_keys()
            .await
            .expect("couldn't list signing keys");
        assert!(
            list_response
                .signing_keys
                .iter()
                .map(|k| k.key_id.as_str())
                .any(|x| x == kid.as_str().expect("couldn't cast kid as str")),
            "newly created signing key was not found in list response"
        );

        mm.revoke_signing_key(&response.key_id)
            .await
            .expect("couldn't revoke signing key");
    }

    #[tokio::test]
    async fn invalid_control_token_can_still_initialize_sdk() {
        env_logger::init();
        let jwt_header_base64: String = String::from("eyJhbGciOiJIUzUxMiJ9");
        let jwt_invalid_signature_base_64: String =
            String::from("gdghdjjfjyehhdkkkskskmmls76573jnajhjjjhjdhnndy");
        // {"sub":"squirrel","cp":"invalidcontrol.cell-alpha-dev.preprod.a.momentohq.com","c":"cache.cell-alpha-dev.preprod.a.momentohq.com"}
        let jwt_payload_bad_control_plane_base64: String = String::from("eyJzdWIiOiJzcXVpcnJlbCIsImNwIjoiaW52YWxpZGNvbnRyb2wuY2VsbC1hbHBoYS1kZXYucHJlcHJvZC5hLm1vbWVudG9ocS5jb20iLCJjIjoiY2FjaGUuY2VsbC1hbHBoYS1kZXYucHJlcHJvZC5hLm1vbWVudG9ocS5jb20ifQ");
        // This JWT will result in UNAUTHENTICATED from the reachable backend since they have made up signatures
        let bad_control_plane_jwt = jwt_header_base64.clone()
            + "."
            + &jwt_payload_bad_control_plane_base64.clone()
            + "."
            + &jwt_invalid_signature_base_64.clone();
        let mut client = get_momento_instance_with_token(bad_control_plane_jwt)
            .expect("even with a bad control endpoint we should get a client")
            .build();

        // Unable to reach control plane
        let create_cache_result = client.create_cache("cache").await.unwrap_err();
        let _err_msg_internal = "error trying to connect: dns error: failed to lookup address information: nodename nor servname provided, or not known".to_string();
        assert!(matches!(create_cache_result.to_string(), _err_msg_internal));
        // Can reach data plane
        let set_result = client
            .set("cache", "hello", "world", None)
            .await
            .unwrap_err();
        let _err_msg_unauthenticated = "Invalid signature".to_string();
        assert!(matches!(set_result.to_string(), _err_msg));
        let get_result = client.get("cache", "hello").await.unwrap_err();
        assert!(matches!(get_result.to_string(), _err_msg_unauthenticated));
    }

    #[tokio::test]
    async fn invalid_data_token_can_still_initialize_sdk() {
        let jwt_header_base64: String = String::from("eyJhbGciOiJIUzUxMiJ9");
        let jwt_invalid_signature_base_64: String =
            String::from("gdghdjjfjyehhdkkkskskmmls76573jnajhjjjhjdhnndy");
        // {"sub":"squirrel","cp":"control.cell-alpha-dev.preprod.a.momentohq.com","c":"invalidcache.cell-alpha-dev.preprod.a.momentohq.com"}
        let jwt_payload_bad_data_plane_base64: String = String::from("eyJzdWIiOiJzcXVpcnJlbCIsImNwIjoiY29udHJvbC5jZWxsLWFscGhhLWRldi5wcmVwcm9kLmEubW9tZW50b2hxLmNvbSIsImMiOiJpbnZhbGlkY2FjaGUuY2VsbC1hbHBoYS1kZXYucHJlcHJvZC5hLm1vbWVudG9ocS5jb20ifQ");
        // This JWT will result in UNAUTHENTICATED from the reachable backend since they have made up signatures
        let bad_data_plane_jwt = jwt_header_base64.clone()
            + "."
            + &jwt_payload_bad_data_plane_base64.clone()
            + "."
            + &jwt_invalid_signature_base_64.clone();
        let mut client = get_momento_instance_with_token(bad_data_plane_jwt)
            .expect("even with a bad data endpoint we should get a client")
            .build();

        // Can reach control plane
        let create_cache_result = client.create_cache("cache").await.unwrap_err();
        let _err_msg_unauthenticated = "Invalid signature".to_string();
        assert!(matches!(
            create_cache_result.to_string(),
            _err_msg_unauthenticated
        ));
        // Unable to reach data plane
        let set_result = client
            .set("cache", "hello", "world", None)
            .await
            .unwrap_err();
        let _err_msg_internal = "error trying to connect: dns error: failed to lookup address information: nodename nor servname provided, or not known".to_string();
        assert!(matches!(set_result.to_string(), _err_msg_internal));
        let get_result = client.get("cache", "hello").await.unwrap_err();
        assert!(matches!(get_result.to_string(), _err_msg_internal));
    }

    #[tokio::test]
    async fn delete_item() {
        let cache_name = create_random_cache_name();
        let cache_key = Uuid::new_v4().to_string();
        let cache_body = Uuid::new_v4().to_string();
        let mut mm = get_momento_instance();
        mm.create_cache(&cache_name)
            .await
            .expect("failed to create cache");
        mm.set(
            &cache_name,
            cache_key.clone(),
            cache_body.clone(),
            Duration::from_millis(10000),
        )
        .await
        .expect("failed to perform set");
        let result = mm
            .get(&cache_name, cache_key.clone())
            .await
            .expect("failed to exercise get");
        assert_eq!(result, hit(cache_body));
        mm.delete(&cache_name, cache_key.clone())
            .await
            .expect("failed to delete cache");
        let result = mm
            .get(&cache_name, cache_key.clone())
            .await
            .expect("failed to exercise get");
        assert_eq!(result, Get::Miss);
        mm.delete_cache(&cache_name)
            .await
            .expect("failed to delete cache");
    }
}
