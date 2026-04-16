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
///
/// # // put the function first
/// # use momento::functions::PutFunctionRequest;
/// # use momento_test_util::echo_wasm;
/// # let (function_client, cache_name) = momento_test_util::create_doctest_function_client();
/// # let function_body = echo_wasm();
/// # let request = PutFunctionRequest::new(cache_name.clone(), "hello functions", function_body);
/// # let function = function_client.send(request).await?;
/// # println!("Created a function: {function:?}");
///
/// let request = PutFunctionConfigRequest::from_function_name(cache_name, "hello functions").current_version(0);
/// let function = function_client.send(request).await?;
/// println!("Updated a function's config: {function:?}");
/// # Ok(())
/// # })
/// # }
/// ```
pub struct PutFunctionConfigRequest {
    cache_name: String,
    function_specifier: FunctionSpecifier,
    new_version: Option<CurrentFunctionVersion>,
}

impl PutFunctionConfigRequest {
    /// Create a new PutFunctionConfigRequest, specified by function name
    pub fn from_function_name(
        cache_name: impl Into<String>,
        function_name: impl Into<String>,
    ) -> Self {
        let cache_name = cache_name.into();
        Self {
            cache_name: cache_name.clone(),
            function_specifier: FunctionSpecifier::FunctionKey(FunctionKey {
                cache_name,
                name: function_name.into(),
            }),
            new_version: None,
        }
    }

    /// Create a new PutFunctionConfigRequest, specified by function ID
    pub fn from_function_id(cache_name: impl Into<String>, function_id: impl Into<String>) -> Self {
        let cache_name = cache_name.into();
        Self {
            cache_name: cache_name.clone(),
            function_specifier: FunctionSpecifier::FunctionId(function_id.into()),
            new_version: None,
        }
    }

    /// Choose the version to use upon invocation
    pub fn current_version(mut self, current_version: impl Into<CurrentFunctionVersion>) -> Self {
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
                function_specifier: Some(self.function_specifier),
                new_version: self
                    .new_version
                    .map(momento_protos::function_types::CurrentFunctionVersion::from),
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
