use momento_protos::function::function_registry_client::FunctionRegistryClient;
use tonic::{service::interceptor::InterceptedService, transport::Channel};

use crate::{
    functions::{function_client_builder::NeedsCredentialProvider, MomentoRequest},
    grpc::header_interceptor::HeaderInterceptor,
    MomentoResult,
};

/// A client for interacting with Momento Functions.
pub struct FunctionClient {
    client: FunctionRegistryClient<InterceptedService<Channel, HeaderInterceptor>>,
}

impl FunctionClient {
    /// Create a new FunctionClient
    pub fn builder() -> crate::functions::FunctionClientBuilder<NeedsCredentialProvider> {
        crate::functions::FunctionClientBuilder(NeedsCredentialProvider)
    }

    pub(crate) fn new(
        client: FunctionRegistryClient<InterceptedService<Channel, HeaderInterceptor>>,
    ) -> Self {
        Self { client }
    }

    /// Access the grpc client for a message
    pub(in crate::functions) fn client(
        &self,
    ) -> &FunctionRegistryClient<InterceptedService<Channel, HeaderInterceptor>> {
        &self.client
    }

    /// Send a Functions request
    pub async fn send<R: MomentoRequest>(&self, request: R) -> MomentoResult<R::Response> {
        request.send(self).await
    }
}
