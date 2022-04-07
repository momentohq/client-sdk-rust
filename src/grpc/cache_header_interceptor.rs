#[derive(Clone)]
pub struct CacheHeaderInterceptor {
    pub auth_key: String,
}

impl tonic::service::Interceptor for CacheHeaderInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        request.metadata_mut().insert(
            "authorization",
            tonic::metadata::AsciiMetadataValue::from_str(self.auth_key.as_str()).unwrap(),
        );
        // for reasons unknown, tonic seems to be stripping out the content-type. So we need to add this as
        // a workaround so that the requests are successful
        request.metadata_mut().insert(
            "content-type",
            tonic::metadata::AsciiMetadataValue::from_str("application/grpc").unwrap(),
        );
        Ok(request)
    }
}
