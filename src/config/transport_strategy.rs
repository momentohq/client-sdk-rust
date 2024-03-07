use crate::config::grpc_configuration::GrpcConfiguration;

/// Low-level settings for communicating with Momento.
#[derive(Clone)]
pub struct TransportStrategy {
    /// Low-level gRPC settings for communicating with Momento.
    pub(crate) grpc_configuration: GrpcConfiguration,
}

impl TransportStrategy {
    pub fn builder(grpc_configuration: GrpcConfiguration) -> TransportStrategyBuilder {
        TransportStrategyBuilder { grpc_configuration }
    }
}

pub struct TransportStrategyBuilder {
    grpc_configuration: GrpcConfiguration,
}

impl TransportStrategyBuilder {
    pub fn with_grpc_configuration(mut self, grpc_configuration: GrpcConfiguration) -> Self {
        self.grpc_configuration = grpc_configuration;
        self
    }

    pub fn build(self) -> TransportStrategy {
        TransportStrategy {
            grpc_configuration: self.grpc_configuration,
        }
    }
}
