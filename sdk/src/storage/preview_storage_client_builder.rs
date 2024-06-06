use momento_protos::control_client::scs_control_client::ScsControlClient;
use momento_protos::store::store_client::StoreClient;
use tonic::codegen::InterceptedService;

use crate::grpc::header_interceptor::HeaderInterceptor;
use crate::storage::{Configuration, PreviewStorageClient};
use crate::{utils, CredentialProvider, MomentoResult};

pub struct PreviewStorageClientBuilder<State>(pub State);

pub struct NeedsConfiguration;

pub struct NeedsCredentialProvider {
    configuration: Configuration,
}

pub struct ReadyToBuild {
    configuration: Configuration,
    credential_provider: CredentialProvider,
}

impl PreviewStorageClientBuilder<NeedsConfiguration> {
    pub fn configuration(
        self,
        configuration: impl Into<Configuration>,
    ) -> PreviewStorageClientBuilder<NeedsCredentialProvider> {
        PreviewStorageClientBuilder(NeedsCredentialProvider {
            configuration: configuration.into(),
        })
    }
}

impl PreviewStorageClientBuilder<NeedsCredentialProvider> {
    pub fn credential_provider(
        self,
        credential_provider: CredentialProvider,
    ) -> PreviewStorageClientBuilder<ReadyToBuild> {
        PreviewStorageClientBuilder(ReadyToBuild {
            configuration: self.0.configuration,
            credential_provider,
        })
    }
}

impl PreviewStorageClientBuilder<ReadyToBuild> {
    pub fn build(self) -> MomentoResult<PreviewStorageClient> {
        let agent_value = &utils::user_agent("sdk");

        let data_channel = utils::connect_channel_lazily_configurable(
            &self.0.credential_provider.storage_endpoint,
            self.0
                .configuration
                .transport_strategy
                .grpc_configuration
                .clone(),
        )?;
        let control_channel = utils::connect_channel_lazily_configurable(
            &self.0.credential_provider.control_endpoint,
            self.0
                .configuration
                .transport_strategy
                .grpc_configuration
                .clone(),
        )?;

        let data_interceptor = InterceptedService::new(
            data_channel,
            HeaderInterceptor::new(&self.0.credential_provider.auth_token, agent_value),
        );
        let control_interceptor = InterceptedService::new(
            control_channel,
            HeaderInterceptor::new(&self.0.credential_provider.auth_token, agent_value),
        );

        let storage_client = StoreClient::new(data_interceptor);
        let control_client = ScsControlClient::new(control_interceptor);

        Ok(PreviewStorageClient {
            storage_client,
            control_client,
            configuration: self.0.configuration,
        })
    }
}
