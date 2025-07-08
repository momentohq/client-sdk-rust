use std::time::Duration;

use futures::StreamExt;

use crate::{
    functions::{Function, FunctionClient, MomentoRequest},
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
/// use momento::functions::ListFunctionsRequest;
/// use futures::StreamExt;
/// # let (function_client, cache_name) = momento_test_util::create_doctest_function_client();
///
/// let request = ListFunctionsRequest::new(cache_name);
/// let functions = function_client.send(request).await?.collect::<Vec<_>>();
/// println!("Functions: {functions:?}");
/// # Ok(())
/// # })
/// # }
/// ```
pub struct ListFunctionsRequest {
    cache_name: String,
}

impl ListFunctionsRequest {
    /// Create a new ListFunctionsRequest.
    pub fn new(cache_name: impl Into<String>) -> Self {
        Self {
            cache_name: cache_name.into(),
        }
    }
}

impl MomentoRequest for ListFunctionsRequest {
    type Response = ListFunctionsStream;

    async fn send(self, client: &FunctionClient) -> MomentoResult<Self::Response> {
        let request = prep_request_with_timeout(
            &self.cache_name.to_string(),
            Duration::from_secs(15),
            momento_protos::function::ListFunctionsRequest {
                cache_name: self.cache_name,
            },
        )?;

        let response = client.client().clone().list_functions(request).await?;
        Ok(ListFunctionsStream::new(response.into_inner()))
    }
}

/// A stream of responses from a ListFunctionsRequest.
/// You can iterate the stream or collect it into a Vec using `futures::StreamExt`.
#[derive(Debug)]
pub struct ListFunctionsStream {
    stream: tonic::Streaming<momento_protos::function_types::Function>,
}
impl ListFunctionsStream {
    /// Create a new Stream from a tonic Streaming object.
    pub(crate) fn new(stream: tonic::Streaming<momento_protos::function_types::Function>) -> Self {
        Self { stream }
    }
}

impl futures::Stream for ListFunctionsStream {
    type Item = MomentoResult<Function>;

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
