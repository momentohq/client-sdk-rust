use std::time::Duration;

use futures::StreamExt;

use crate::{
    functions::{function::FunctionVersion, FunctionClient, MomentoRequest},
    utils::prep_request_with_timeout,
    MomentoError, MomentoResult,
};

/// List the Functions within a cache namespace.
///
/// # Example
///
/// ```rust
/// # fn main() -> anyhow::Result<()> {
/// # tokio_test::block_on(async {
/// use momento::{CredentialProvider, FunctionClient};
/// use momento::functions::ListFunctionVersionsRequest;
/// use futures::StreamExt;
/// # let (function_client, cache_name) = momento_test_util::create_doctest_function_client();
///
/// let request = ListFunctionVersionsRequest::new("function-id");
/// let versions = function_client.send(request).await?.collect::<Vec<_>>();
/// println!("Function versions: {versions:?}");
/// # Ok(())
/// # })
/// # }
/// ```
pub struct ListFunctionVersionsRequest {
    function_id: String,
}

impl ListFunctionVersionsRequest {
    /// Create a new ListFunctionsRequest.
    pub fn new(cache_name: impl Into<String>) -> Self {
        Self {
            function_id: cache_name.into(),
        }
    }
}

impl MomentoRequest for ListFunctionVersionsRequest {
    type Response = ListFunctionVersionsStream;

    async fn send(self, client: &FunctionClient) -> MomentoResult<Self::Response> {
        let request = prep_request_with_timeout(
            &self.function_id.to_string(),
            Duration::from_secs(15),
            momento_protos::function::ListFunctionVersionsRequest {
                function_id: self.function_id,
            },
        )?;

        let response = client
            .client()
            .clone()
            .list_function_versions(request)
            .await?;
        Ok(ListFunctionVersionsStream::new(response.into_inner()))
    }
}

/// A stream of responses from a ListFunctionsRequest.
/// You can iterate the stream or collect it into a Vec using `futures::StreamExt`.
#[derive(Debug)]
pub struct ListFunctionVersionsStream {
    stream: tonic::Streaming<momento_protos::function_types::FunctionVersion>,
}
impl ListFunctionVersionsStream {
    /// Create a new Stream from a tonic Streaming object.
    pub(crate) fn new(
        stream: tonic::Streaming<momento_protos::function_types::FunctionVersion>,
    ) -> Self {
        Self { stream }
    }
}

impl futures::Stream for ListFunctionVersionsStream {
    type Item = MomentoResult<FunctionVersion>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match futures::ready!(self.stream.poll_next_unpin(context)) {
            Some(Ok(item)) => std::task::Poll::Ready(Some(Ok(item.into()))),
            Some(Err(e)) => std::task::Poll::Ready(Some(Err(MomentoError::from(e)))),
            None => std::task::Poll::Ready(None),
        }
    }
}
