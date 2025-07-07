use std::convert::TryFrom;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Clone)]
pub struct HeaderInterceptor {
    auth_token: String,
    sdk_agent: String,
    are_only_once_header_sent: Arc<AtomicBool>,
}

impl HeaderInterceptor {
    pub fn new(authorization: &str, sdk_agent: &str) -> HeaderInterceptor {
        HeaderInterceptor {
            auth_token: authorization.to_string(),
            sdk_agent: sdk_agent.to_string(),
            are_only_once_header_sent: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Insert a header into the request.
    fn insert_header(
        &self,
        request: &mut tonic::Request<()>,
        name: &str,
        value: &str,
    ) -> Result<(), tonic::Status> {
        let (header_name, header_value) = create_header_from_string(name, value)?;
        request.metadata_mut().insert(header_name, header_value);
        Ok(())
    }
}

impl tonic::service::Interceptor for HeaderInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> Result<tonic::Request<()>, tonic::Status> {
        self.insert_header(&mut request, "authorization", &self.auth_token)?;

        if !self.are_only_once_header_sent.load(Ordering::Relaxed) {
            self.insert_header(&mut request, "agent", &self.sdk_agent)?;

            // Because the `runtime-version` header makes more sense for interpreted languages,
            // we send this sentinel value to ensure we report *some* value for this sdk.
            self.insert_header(&mut request, "runtime-version", "rust")?;
            self.are_only_once_header_sent
                .store(true, Ordering::Relaxed);
        }

        Ok(request)
    }
}

fn create_header_from_string(
    name: &str,
    value: &str,
) -> Result<
    (
        tonic::metadata::AsciiMetadataKey,
        tonic::metadata::AsciiMetadataValue,
    ),
    tonic::Status,
> {
    let header_name = tonic::metadata::AsciiMetadataKey::from_str(name).map_err(|e| {
        tonic::Status::new(
            tonic::Code::InvalidArgument,
            format!("Couldn't parse header name {name}: {e}"),
        )
    })?;
    let header_value = tonic::metadata::AsciiMetadataValue::try_from(value).map_err(|e| {
        tonic::Status::new(
            tonic::Code::InvalidArgument,
            format!("Couldn't parse header value for {name}: {e}"),
        )
    })?;
    Ok((header_name, header_value))
}
