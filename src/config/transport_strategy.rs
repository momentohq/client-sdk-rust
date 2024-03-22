use crate::config::grpc_configuration::GrpcConfiguration;

/// Low-level settings for communicating with Momento.
#[derive(Clone, Debug)]
pub struct TransportStrategy {
    /// Low-level gRPC settings for communicating with Momento.
    pub(crate) grpc_configuration: GrpcConfiguration,
}

impl TransportStrategy {
    pub fn builder() -> TransportStrategyBuilder<NeedsGrpcConfiguration> {
        TransportStrategyBuilder(NeedsGrpcConfiguration(()))
    }
}

pub struct TransportStrategyBuilder<State>(State);

pub struct NeedsGrpcConfiguration(());

pub struct ReadyToBuild {
    grpc_configuration: GrpcConfiguration,
}

impl TransportStrategyBuilder<NeedsGrpcConfiguration> {
    pub fn grpc_configuration(
        self,
        grpc_configuration: impl Into<GrpcConfiguration>,
    ) -> TransportStrategyBuilder<ReadyToBuild> {
        TransportStrategyBuilder(ReadyToBuild {
            grpc_configuration: grpc_configuration.into(),
        })
    }
}

impl TransportStrategyBuilder<ReadyToBuild> {
    pub fn build(self) -> TransportStrategy {
        TransportStrategy {
            grpc_configuration: self.0.grpc_configuration,
        }
    }
}

impl From<TransportStrategyBuilder<ReadyToBuild>> for TransportStrategy {
    fn from(builder: TransportStrategyBuilder<ReadyToBuild>) -> TransportStrategy {
        builder.build()
    }
}
