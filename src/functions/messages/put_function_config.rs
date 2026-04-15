use std::time::Duration;

use crate::{
    functions::{CurrentFunctionVersion, Function, FunctionClient, MomentoRequest},
    utils::prep_request_with_timeout,
    MomentoError, MomentoResult,
};

use momento_protos::function::put_function_config_request::FunctionSpecifier;
use momento_protos::function_types::FunctionKey;

/// Update a Function's configuration.
/// The cache is used as a namespace for your Functions.
///
/// # Arguments
///
/// * `cache_name` - The name of the cache to use as a namespace for the Function.
/// * `function_name` - The name of the Function.
///
/// # Example
///
/// ```rust
/// # fn main() -> anyhow::Result<()> {
/// # tokio_test::block_on(async {
/// use momento::{CredentialProvider, FunctionClient};
/// use momento::functions::{CurrentFunctionVersion, PutFunctionConfigRequest};
/// # use momento_test_util::echo_wasm;
/// # let (function_client, cache_name) = momento_test_util::create_doctest_function_client();
/// // load your wasm from a .wasm file compiled with wasm32-wasip2
/// let function_body = echo_wasm();
///
/// let request = PutFunctionConfigRequest::new(cache_name).function_name("hello functions").current_version(CurrentFunctionVersion::Pinned(0));
/// let function = function_client.send(request).await?;
/// println!("Updated a function's config: {function:?}");
/// # Ok(())
/// # })
/// # }
/// ```
pub struct PutFunctionConfigRequest {
    cache_name: String,
    function_specifier: Option<FunctionSpecifier>,
    new_version: Option<momento_protos::function_types::CurrentFunctionVersion>,
}

impl PutFunctionConfigRequest {
    /// Create a new PutFunctionConfigRequest.
    pub fn new(cache_name: impl Into<String>) -> Self {
        Self {
            cache_name: cache_name.into(),
            function_specifier: None,
            new_version: None,
        }
    }

    /// Specify the function by name.
    pub fn function_name(mut self, function_name: impl Into<String>) -> Self {
        self.function_specifier = Some(FunctionSpecifier::FunctionKey(FunctionKey {
            cache_name: self.cache_name.clone(),
            name: function_name.into(),
        }));
        self
    }

    /// Specify the function by ID.
    pub fn function_id(mut self, function_id: impl Into<String>) -> Self {
        self.function_specifier = Some(FunctionSpecifier::FunctionId(function_id.into()));
        self
    }

    /// Choose the version to use upon invocation
    pub fn current_version(mut self, current_version: CurrentFunctionVersion) -> Self {
        self.new_version = Some(current_version.into());
        self
    }
}

impl MomentoRequest for PutFunctionConfigRequest {
    type Response = Function;

    async fn send(self, client: &FunctionClient) -> MomentoResult<Function> {
        let request = prep_request_with_timeout(
            &self.cache_name.to_string(),
            Duration::from_secs(15),
            momento_protos::function::PutFunctionConfigRequest {
                function_specifier: self.function_specifier,
                new_version: self.new_version,
            },
        )?;

        let response = client.client().clone().put_function_config(request).await?;
        let function: Function = response
            .into_inner()
            .function
            .ok_or_else(|| {
                MomentoError::unknown_error(
                    "put_function_config",
                    Some("service did not return a Function description".to_string()),
                )
            })?
            .into();
        Ok(function)
    }
}
