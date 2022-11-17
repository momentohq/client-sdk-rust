#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;
    use std::{env, time::Duration};

    use momento::response::error::MomentoError;
    use momento::simple_cache_client::SimpleCacheClientBuilder;
    use momento::{
        response::cache_get_response::MomentoGetStatus, simple_cache_client::SimpleCacheClient,
    };
    use serde_json::Value;
    use tokio::time::sleep;
    use uuid::Uuid;

    fn get_momento_instance_with_token(
        auth_token: String,
    ) -> Result<SimpleCacheClientBuilder, MomentoError> {
        SimpleCacheClientBuilder::new_with_explicit_agent_name(
            auth_token,
            NonZeroU64::new(5).expect("expected a non-zero number"),
            "integration_test",
            None,
        )
    }

    fn get_momento_instance() -> SimpleCacheClient {
        let auth_token = env::var("TEST_AUTH_TOKEN").expect("env var TEST_AUTH_TOKEN must be set");
        get_momento_instance_with_token(auth_token)
            .expect("failed to build an integration test client builder")
            .build()
    }

    #[tokio::test]
    async fn cache_miss() {
        let cache_name = Uuid::new_v4().to_string();
        let cache_key = Uuid::new_v4().to_string();
        let mut mm = get_momento_instance();
        mm.create_cache(&cache_name).await.expect("failed to create cache");
        let result = mm.get(&cache_name, cache_key).await.expect("failure when trying get");
        assert!(matches!(result.result, MomentoGetStatus::MISS));
        mm.delete_cache(&cache_name).await.expect("failed to delete cache");
    }

    #[tokio::test]
    async fn cache_validation() {
        let cache_name = "";
        let mut mm = get_momento_instance();
        let result = mm.create_cache(cache_name).await.unwrap_err();
        let _err_msg = "Cache name cannot be empty".to_string();
        assert!(matches!(
            result,
            MomentoError::InvalidArgument(_err_message)
        ))
    }

    #[tokio::test]
    async fn key_id_validation() {
        let key_id = "";
        let mut mm = get_momento_instance();
        let result = mm.revoke_signing_key(key_id).await.unwrap_err();
        assert!(matches!(result, MomentoError::InvalidArgument(_)))
    }

    #[tokio::test]
    async fn ttl_validation() {
        let cache_name = Uuid::new_v4().to_string();
        let cache_key = Uuid::new_v4().to_string();
        let cache_body = Uuid::new_v4().to_string();
        let mut mm = get_momento_instance();
        mm.create_cache(&cache_name).await.expect("failed to create cache");
        let ttl: u64 = 18446744073709551615;
        let max_ttl = u64::MAX / 1000_u64;
        let result = mm
            .set(
                &cache_name,
                cache_key,
                cache_body,
                Some(NonZeroU64::new(ttl).expect("failed to get non zero u64")),
            ) // 18446744073709551615 > 2^64/1000
            .await
            .unwrap_err();
        let _err_message = format!(
            "TTL provided, {}, needs to be less than the maximum TTL {}",
            ttl, max_ttl
        );
        assert!(matches!(
            result,
            MomentoError::InvalidArgument(_err_message)
        ));
        mm.delete_cache(&cache_name).await.expect("failed to delete cache");
    }

    #[tokio::test]
    async fn cache_hit() {
        let cache_name = Uuid::new_v4().to_string();
        let cache_key = Uuid::new_v4().to_string();
        let cache_body = Uuid::new_v4().to_string();
        let mut mm = get_momento_instance();
        mm.create_cache(&cache_name).await.expect("failed to create cache");
        mm.set(&cache_name, cache_key.clone(), cache_body.clone(), None)
            .await
            .expect("failed to perform set");
        let result = mm.get(&cache_name, cache_key.clone()).await.expect("failed to perform get");
        assert!(matches!(result.result, MomentoGetStatus::HIT));
        assert_eq!(result.value, cache_body.as_bytes());
        mm.delete_cache(&cache_name).await.expect("failed to delete cache");
    }

    #[tokio::test]
    async fn cache_respects_default_ttl() {
        let cache_name = Uuid::new_v4().to_string();
        let cache_key = Uuid::new_v4().to_string();
        let cache_body = Uuid::new_v4().to_string();
        let mut mm = get_momento_instance();
        mm.create_cache(&cache_name).await.expect("failed to create cache");
        mm.set(&cache_name, cache_key.clone(), cache_body.clone(), None)
            .await
            .expect("failed to perform set");
        sleep(Duration::new(1, 0)).await;
        let result = mm.get(&cache_name, cache_key.clone()).await.expect("failed to perform get");
        assert!(matches!(result.result, MomentoGetStatus::HIT));
        mm.delete_cache(&cache_name).await.expect("failed to delete cache");
    }

    #[tokio::test]
    async fn create_cache_then_set() {
        let cache_name = Uuid::new_v4().to_string();
        let cache_key = Uuid::new_v4().to_string();
        let cache_body = Uuid::new_v4().to_string();
        let mut mm = get_momento_instance();
        mm.create_cache(&cache_name).await.expect("failed to create cache");
        mm.set(&cache_name, cache_key.clone(), cache_body.clone(), None)
            .await
            .expect("failed to perform set");
        let result = mm.get(&cache_name, cache_key.clone()).await.expect("failed to perform get");
        assert!(matches!(result.result, MomentoGetStatus::HIT));
        assert_eq!(result.value, cache_body.as_bytes());
        mm.delete_cache(&cache_name).await.expect("failed to delete cache");
    }

    #[tokio::test]
    async fn list_caches() {
        let cache_name = Uuid::new_v4().to_string();
        let mut mm = get_momento_instance();
        mm.create_cache(&cache_name).await.expect("failed to create cache");

        let mut next_token: Option<String> = None;
        let mut num_pages = 0;
        loop {
            let list_cache_result = mm.list_caches(next_token).await.expect("failed to list caches");
            num_pages += 1;
            next_token = list_cache_result.next_token;
            if next_token == None {
                break;
            }
        }

        assert_eq!(1, num_pages, "we expect a single page of caches to list");
        mm.delete_cache(&cache_name).await.expect("failed to delete cache");
    }

    #[tokio::test]
    async fn create_list_revoke_signing_key() {
        let mut mm = get_momento_instance();
        let response = mm.create_signing_key(10).await.expect("failed to create signing key");

        let key: Value = serde_json::from_str(&response.key).expect("failed to parse key from json");
        let obj = key.as_object().expect("failed to map key to object");
        let kid = obj.get("kid").expect("'kid' was not present");
        assert_eq!(kid.as_str().expect("failed to cast kid to str"), response.key_id);

        let auth_token = env::var("TEST_AUTH_TOKEN").expect("env var TEST_AUTH_TOKEN must be set");
        let parts: Vec<&str> = auth_token.split('.').collect();
        assert!(
            std::str::from_utf8(base64_url::decode(parts[1]).expect("expected base64 part not present in position 1").as_slice())
                .expect("couldn't parse base64")
                .contains(&response.endpoint)
        );

        let list_response = mm.list_signing_keys(None).await.expect("couldn't list signing keys");
        assert!(
            list_response
                .signing_keys
                .iter()
                .map(|k| k.key_id.as_str())
                .any(|x| x == kid.as_str().expect("couldn't cast kid as str")),
            "newly created signing key was not found in list response"
        );

        mm.revoke_signing_key(&response.key_id).await.expect("couldn't revoke signing key");
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
        assert!(matches!(
            create_cache_result,
            MomentoError::InternalServerError(_err_msg_internal)
        ));
        // Can reach data plane
        let set_result = client
            .set("cache", "hello", "world", None)
            .await
            .unwrap_err();
        let _err_msg_unauthenticated = "Invalid signature".to_string();
        assert!(matches!(
            set_result,
            MomentoError::Unauthenticated(_err_msg)
        ));
        let get_result = client.get("cache", "hello").await.unwrap_err();
        assert!(matches!(
            get_result,
            MomentoError::Unauthenticated(_err_msg_unauthenticated)
        ));
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
            create_cache_result,
            MomentoError::Unauthenticated(_err_msg_unauthenticated)
        ));
        // Unable to reach data plane
        let set_result = client
            .set("cache", "hello", "world", None)
            .await
            .unwrap_err();
        let _err_msg_internal = "error trying to connect: dns error: failed to lookup address information: nodename nor servname provided, or not known".to_string();
        assert!(matches!(
            set_result,
            MomentoError::InternalServerError(_err_msg_internal)
        ));
        let get_result = client.get("cache", "hello").await.unwrap_err();
        assert!(matches!(
            get_result,
            MomentoError::InternalServerError(_err_msg_internal)
        ));
    }

    #[tokio::test]
    async fn delete_item() {
        let cache_name = Uuid::new_v4().to_string();
        let cache_key = Uuid::new_v4().to_string();
        let cache_body = Uuid::new_v4().to_string();
        let mut mm = get_momento_instance();
        mm.create_cache(&cache_name).await.expect("failed to create cache");
        mm.set(
            &cache_name,
            cache_key.clone(),
            cache_body.clone(),
            NonZeroU64::new(10000),
        )
        .await
        .expect("failed to perform set");
        let result = mm.get(&cache_name, cache_key.clone()).await.expect("failed to exercise get");
        assert!(matches!(result.result, MomentoGetStatus::HIT));
        assert_eq!(result.value, cache_body.as_bytes());
        mm.delete(&cache_name, cache_key.clone()).await.expect("failed to delete cache");
        let result = mm.get(&cache_name, cache_key.clone()).await.expect("failed to exercise get");
        assert!(matches!(result.result, MomentoGetStatus::MISS));
        mm.delete_cache(&cache_name).await.expect("failed to delete cache");
    }
}
