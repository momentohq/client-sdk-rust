use std::convert::TryFrom;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone)]
pub struct HeaderInterceptor {
    auth_token: String,
    sdk_agent: String,
}

impl HeaderInterceptor {
    pub fn new(authorization: &str, sdk_agent: &str) -> HeaderInterceptor {
        HeaderInterceptor {
            auth_token: authorization.to_string(),
            sdk_agent: sdk_agent.to_string(),
        }
    }
}

impl tonic::service::Interceptor for HeaderInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        static ARE_ONLY_ONCE_HEADER_SENT: AtomicBool = AtomicBool::new(false);

        request.metadata_mut().insert(
            tonic::metadata::AsciiMetadataKey::from_static("authorization"),
            tonic::metadata::AsciiMetadataValue::try_from(&self.auth_token).unwrap(),
        );

        if !ARE_ONLY_ONCE_HEADER_SENT.load(Ordering::Relaxed) {
            request.metadata_mut().insert(
                tonic::metadata::AsciiMetadataKey::from_static("agent"),
                tonic::metadata::AsciiMetadataValue::try_from(&self.sdk_agent).unwrap(),
            );
            ARE_ONLY_ONCE_HEADER_SENT.store(true, Ordering::Relaxed);
        }

        Ok(request)
    }
}
