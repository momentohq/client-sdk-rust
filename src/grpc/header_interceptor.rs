use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

const AUTHORIZATION: &str = "authorization";
const AGENT: &str = "agent";

#[derive(Clone)]
pub struct HeaderInterceptor {
    pub header: HashMap<String, String>,
}

impl tonic::service::Interceptor for HeaderInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        static ARE_ONLY_ONCE_HEADER_SENT: AtomicBool = AtomicBool::new(false);
        for (key, value) in self.header.iter() {
            if *key == *AUTHORIZATION {
                request.metadata_mut().insert(
                    tonic::metadata::AsciiMetadataKey::from_static(AUTHORIZATION),
                    tonic::metadata::AsciiMetadataValue::from_str(value).unwrap(),
                );
            }
            if !ARE_ONLY_ONCE_HEADER_SENT.load(Ordering::Relaxed) && *key != *AUTHORIZATION {
                request.metadata_mut().insert(
                    tonic::metadata::AsciiMetadataKey::from_static(AGENT),
                    tonic::metadata::AsciiMetadataValue::from_str(value).unwrap(),
                );
                ARE_ONLY_ONCE_HEADER_SENT.store(true, Ordering::Relaxed);
            }
        }
        Ok(request)
    }
}
