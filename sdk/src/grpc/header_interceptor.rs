use std::convert::TryFrom;
use std::str::FromStr;
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

        let (auth_header_name, auth_header_value) =
            create_header_from_string("authorization", &self.auth_token)?;
        request
            .metadata_mut()
            .insert(auth_header_name, auth_header_value);

        if !ARE_ONLY_ONCE_HEADER_SENT.load(Ordering::Relaxed) {
            let (agent_header_name, agent_header_value) =
                create_header_from_string("agent", &self.sdk_agent)?;
            request
                .metadata_mut()
                .insert(agent_header_name, agent_header_value);

            // Because the `runtime-version` header makes more sense for interpreted languages,
            // we send this sentinel value to ensure we report *some* value for this sdk.
            let (runtime_version_header_name, runtime_version_header_value) =
                create_header_from_string("runtime-version", "rust")?;
            request
                .metadata_mut()
                .insert(runtime_version_header_name, runtime_version_header_value);
            ARE_ONLY_ONCE_HEADER_SENT.store(true, Ordering::Relaxed);
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
            format!("Couldn't parse header name {}: {}", name, e),
        )
    })?;
    let header_value = tonic::metadata::AsciiMetadataValue::try_from(value).map_err(|e| {
        tonic::Status::new(
            tonic::Code::InvalidArgument,
            format!("Couldn't parse header value for {}: {}", name, e),
        )
    })?;
    Ok((header_name, header_value))
}
