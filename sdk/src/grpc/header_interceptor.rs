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

        let auth_token =
            tonic::metadata::AsciiMetadataValue::try_from(&self.auth_token).map_err(|e| {
                tonic::Status::new(
                    tonic::Code::InvalidArgument,
                    format!("Couldn't parse auth token for auth header: {}", e),
                )
            })?;

        request.metadata_mut().insert(
            tonic::metadata::AsciiMetadataKey::from_static("authorization"),
            auth_token,
        );

        let sdk_agent =
            tonic::metadata::AsciiMetadataValue::try_from(&self.sdk_agent).map_err(|e| {
                tonic::Status::new(
                    tonic::Code::InvalidArgument,
                    format!("Couldn't parse sdk agent for agent header: {}", e),
                )
            })?;

        if !ARE_ONLY_ONCE_HEADER_SENT.load(Ordering::Relaxed) {
            request.metadata_mut().insert(
                tonic::metadata::AsciiMetadataKey::from_static("agent"),
                sdk_agent,
            );
            // Because the `runtime-version` header makes more sense for interpreted languages,
            // we send this sentinel value to ensure we report *some* value for this sdk.
            request.metadata_mut().insert(
                tonic::metadata::AsciiMetadataKey::from_static("runtime-version"),
                "rust".parse().unwrap(),
            );
            ARE_ONLY_ONCE_HEADER_SENT.store(true, Ordering::Relaxed);
        }

        Ok(request)
    }
}
