use crate::config::grpc_configuration::GrpcConfiguration;

/// Low-level settings for communicating with Momento.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct TransportStrategy {
    /// Low-level gRPC settings for communicating with Momento.
    pub(crate) grpc_configuration: GrpcConfiguration,
}

impl TransportStrategy {
    /// Constructs a new TransportStrategyBuilder.
    pub fn builder() -> TransportStrategyBuilder<NeedsGrpcConfiguration> {
        TransportStrategyBuilder(NeedsGrpcConfiguration(()))
    }
}

/// The initial state of the TransportStrategyBuilder.
pub struct TransportStrategyBuilder<State>(State);

/// The state of the TransportStrategyBuilder when it is waiting for a GRPC configuration.
pub struct NeedsGrpcConfiguration(());

/// The state of the TransportStrategyBuilder when it is ready to build a TransportStrategy.
pub struct ReadyToBuild {
    grpc_configuration: GrpcConfiguration,
}

impl TransportStrategyBuilder<NeedsGrpcConfiguration> {
    /// Sets the GRPC configuration for the TransportStrategy and returns
    /// the TransportStrategyBuilder in the ReadyToBuild state.
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
    /// Constructs the TransportStrategy with the given GRPC configuration.
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
