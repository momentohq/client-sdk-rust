#[derive(Clone)]
pub struct CacheHeaderInterceptor {
    pub cache_name: String,
    pub auth_key: String,
}

impl tonic::service::Interceptor for CacheHeaderInterceptor {
    fn call(&mut self, request: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        let mut result = tonic::Request::new(request.into_inner());
        result.metadata_mut().insert(
            "cache",
            tonic::metadata::AsciiMetadataValue::from_str(self.cache_name.as_str()).unwrap(),
        );
        result.metadata_mut().insert(
            "authorization",
            tonic::metadata::AsciiMetadataValue::from_str(self.auth_key.as_str()).unwrap(),
        );
        result.metadata_mut().insert(
            "content-type",
            tonic::metadata::AsciiMetadataValue::from_str("application/grpc").unwrap(),
        );
        Ok(result)
    }
}
