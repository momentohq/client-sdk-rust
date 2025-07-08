use std::time::Duration;

use futures::StreamExt;

use crate::{
    functions::{function::FunctionVersion, FunctionClient, MomentoRequest},
    utils::prep_request_with_timeout,
    MomentoError, MomentoResult,
};

/// List the versions of a Function.
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
/// let versions = function_client.send(request).await?.into_vec().await;
/// println!("Function versions: {versions:?}");
/// # Ok(())
/// # })
/// # }
/// ```
pub struct ListFunctionVersionsRequest {
    function_id: String,
}

impl ListFunctionVersionsRequest {
    /// Create a new ListFunctionVersionsRequest.
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

/// A stream of responses from a ListFunctionVersionsRequest.
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

    /// Collect the response stream into a vector.
    pub async fn into_vec(self) -> MomentoResult<Vec<FunctionVersion>> {
        let mut versions = Vec::new();
        let mut stream = self.stream;
        while let Some(item) = stream.next().await {
            match item {
                Ok(version) => versions.push(version.into()),
                Err(e) => return Err(MomentoError::from(e)),
            }
        }
        Ok(versions)
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
