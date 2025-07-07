use std::{collections::HashMap, iter::FromIterator, time::Duration};

use crate::{
    functions::{EnvironmentValue, Function, FunctionClient, MomentoRequest, WasmSource},
    utils::prep_request_with_timeout,
    MomentoError, MomentoResult,
};

/// Create or update a Function.
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
/// use momento::functions::PutFunctionRequest;
/// # use momento_test_util::echo_wasm;
/// # let (function_client, cache_name) = momento_test_util::create_doctest_function_client();
/// // load your wasm from a .wasm file compiled with wasm32-wasip2
/// let function_body = echo_wasm();
///
/// let request = PutFunctionRequest::new(cache_name, "hello functions", function_body);
/// let function = function_client.send(request).await?;
/// println!("Created a function: {function:?}");
/// # Ok(())
/// # })
/// # }
/// ```
pub struct PutFunctionRequest {
    cache_name: String,
    function_name: String,
    description: String,
    environment: HashMap<String, EnvironmentValue>,
    wasm_source: WasmSource,
}

impl PutFunctionRequest {
    /// Create a new PublishRequest.
    pub fn new(
        cache_name: impl Into<String>,
        function_name: impl Into<String>,
        wasm_source: impl Into<WasmSource>,
    ) -> Self {
        Self {
            cache_name: cache_name.into(),
            function_name: function_name.into(),
            description: String::new(),
            environment: HashMap::new(),
            wasm_source: wasm_source.into(),
        }
    }

    /// Set the Function's description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set the Function's environment variables
    ///
    /// This overrides any previously set environment variables.
    /// ```rust
    /// # use momento::functions::PutFunctionRequest;
    /// # fn example(request: PutFunctionRequest) {
    /// let request = request.with_environment([("key", "value")]);
    /// # }
    /// ```
    pub fn environment(
        mut self,
        environment: impl IntoIterator<Item = (impl Into<String>, impl Into<EnvironmentValue>)>,
    ) -> Self {
        self.environment =
            HashMap::from_iter(environment.into_iter().map(|(k, v)| (k.into(), v.into())));
        self
    }

    /// Set an environment variable for the Function
    ///
    /// This is additive to any previously set environment variables, but it replaces the value for the key if present.
    pub fn environment_variable<S: Into<String> + Default>(
        mut self,
        key: impl Into<Option<S>>,
        value: impl Into<EnvironmentValue>,
    ) -> Self {
        self.environment
            .insert(key.into().unwrap_or_default().into(), value.into());
        self
    }
}

impl MomentoRequest for PutFunctionRequest {
    type Response = Function;

    async fn send(self, client: &FunctionClient) -> MomentoResult<Function> {
        let request = prep_request_with_timeout(
            &self.cache_name.to_string(),
            Duration::from_secs(15),
            momento_protos::function::PutFunctionRequest {
                cache_name: self.cache_name,
                name: self.function_name,
                description: self.description,
                environment: self
                    .environment
                    .into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect(),
                wasm_location: Some(self.wasm_source.into()),
            },
        )?;

        let response = client.client().clone().put_function(request).await?;
        let function: Function = response
            .into_inner()
            .function
            .ok_or_else(|| {
                MomentoError::unknown_error(
                    "put_function",
                    Some("service did not return a Function description".to_string()),
                )
            })?
            .into();
        Ok(function)
    }
}
