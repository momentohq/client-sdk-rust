use std::time::Duration;

use crate::{
    functions::{FunctionClient, MomentoRequest, Wasm},
    utils::prep_request_with_timeout,
    MomentoError, MomentoResult,
};

/// Create or update a wasm.
/// This doesn't create a Function, but rather a wasm archive that can be used in a Function.
///
/// # Example
///
/// ```rust
/// # fn main() -> anyhow::Result<()> {
/// # tokio_test::block_on(async {
/// use momento::{CredentialProvider, FunctionClient};
/// use momento::functions::PutWasmRequest;
/// # use momento_test_util::echo_wasm;
/// # let (function_client, cache_name) = momento_test_util::create_doctest_function_client();
/// // load your wasm from a .wasm file compiled with wasm32-wasip2
/// let function_body = echo_wasm();
///
/// let request = PutWasmRequest::new(cache_name, function_body);
/// let wasm = function_client.send(request).await?;
/// println!("Created a wasm: {wasm:?}");
/// # Ok(())
/// # })
/// # }
/// ```
pub struct PutWasmRequest {
    wasm_name: String,
    description: String,
    wasm_source: Vec<u8>,
}

impl PutWasmRequest {
    /// Create a new PublishRequest.
    pub fn new(wasm_name: impl Into<String>, wasm_source: impl Into<Vec<u8>>) -> Self {
        Self {
            wasm_name: wasm_name.into(),
            description: String::new(),
            wasm_source: wasm_source.into(),
        }
    }

    /// Set the Function's description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }
}

impl MomentoRequest for PutWasmRequest {
    type Response = Wasm;

    async fn send(self, client: &FunctionClient) -> MomentoResult<Self::Response> {
        let request = prep_request_with_timeout(
            "inapplicable",
            Duration::from_secs(15),
            momento_protos::function::PutWasmRequest {
                name: self.wasm_name,
                description: self.description,
                wasm_put_kind: Some(
                    momento_protos::function::put_wasm_request::WasmPutKind::Inline(
                        self.wasm_source,
                    ),
                ),
            },
        )?;

        let response = client.client().clone().put_wasm(request).await?;
        let wasm: Wasm = response
            .into_inner()
            .wasm
            .ok_or_else(|| {
                MomentoError::unknown_error(
                    "put_wasm",
                    Some("service did not return a wasm description".to_string()),
                )
            })?
            .into();
        Ok(wasm)
    }
}
