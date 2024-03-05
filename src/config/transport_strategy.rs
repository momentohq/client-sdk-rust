use crate::config::grpc_configuration::GrpcConfiguration;

/// Low-level settings for communicating with Momento.
#[derive(Clone)]
pub struct TransportStrategy {
    /// Low-level gRPC settings for communicating with Momento.
    pub grpc_configuration: GrpcConfiguration,
}

impl TransportStrategy {
    pub fn new(grpc_configuration: GrpcConfiguration) -> Self {
        TransportStrategy { grpc_configuration }
    }
}
