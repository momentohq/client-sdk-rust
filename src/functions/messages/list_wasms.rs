use std::time::Duration;

use futures::StreamExt;

use crate::{
    functions::{FunctionClient, MomentoRequest, Wasm},
    utils::prep_request_with_timeout,
    MomentoError, MomentoResult,
};

/// List the wasm archives you have.
///
/// # Example
///
/// ```rust
/// # fn main() -> anyhow::Result<()> {
/// # tokio_test::block_on(async {
/// use momento::{CredentialProvider, FunctionClient};
/// use momento::functions::ListWasmsRequest;
/// # let (function_client, cache_name) = momento_test_util::create_doctest_function_client();
///
/// let request = ListWasmsRequest::new();
/// let wasms = function_client.send(request).await?.into_vec().await;
/// println!("Wasms: {wasms:?}");
/// # Ok(())
/// # })
/// # }
/// ```
pub struct ListWasmsRequest {}

impl ListWasmsRequest {
    /// Create a new ListWasmsRequest.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }
}

impl MomentoRequest for ListWasmsRequest {
    type Response = ListWasmsStream;

    async fn send(self, client: &FunctionClient) -> MomentoResult<Self::Response> {
        let request = prep_request_with_timeout(
            "inapplicable",
            Duration::from_secs(15),
            momento_protos::function::ListWasmsRequest {},
        )?;

        let response = client.client().clone().list_wasms(request).await?;
        Ok(ListWasmsStream::new(response.into_inner()))
    }
}

/// A stream of responses from a ListWasmsRequest.
/// You can iterate the stream or collect it into a Vec using `futures::StreamExt`.
#[derive(Debug)]
pub struct ListWasmsStream {
    stream: tonic::Streaming<momento_protos::function_types::Wasm>,
}
impl ListWasmsStream {
    /// Create a new Stream from a tonic Streaming object.
    pub(crate) fn new(stream: tonic::Streaming<momento_protos::function_types::Wasm>) -> Self {
        Self { stream }
    }

    /// Collect the response stream into a vector.
    pub async fn into_vec(self) -> MomentoResult<Vec<Wasm>> {
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

impl futures::Stream for ListWasmsStream {
    type Item = MomentoResult<Wasm>;

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
