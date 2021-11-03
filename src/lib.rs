pub mod cache;
pub mod momento;
mod grpc;
mod jwt;
pub mod response;

#[cfg(test)]
mod tests {
    use crate::{cache::{CacheClient}, momento::Momento};

    #[tokio::test]
    async fn it_works() {
        let auth_key = "your_jwt".to_string();
        let mut mm = Momento::new(auth_key).await.unwrap();
        let cache = mm.get_cache("cache2").await.unwrap();
        let result = cache.get("matt").await.unwrap();
        println!("dfds, {:?}", result);
    }
}
