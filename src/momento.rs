pub mod control_client {
    tonic::include_proto!("control_client");
}

use std::convert::TryFrom;

use control_client::{CreateCacheRequest,DeleteCacheRequest,CreateCacheResponse,DeleteCacheResponse,scs_control_client::ScsControlClient};
use tonic::{Request, codegen::InterceptedService, transport::{Channel, ClientTlsConfig, Uri}};

use crate::{cache::CacheClient, grpc::auth_header_interceptor::AuthHeaderInterceptor, jwt::decode_jwt, response::momento_error::MomentoError};

pub struct Momento {
    client: ScsControlClient<InterceptedService<Channel,  AuthHeaderInterceptor>>,
    cache_endpoint: String,
    auth_key: String,
}

impl Momento {
    pub async fn new(auth_key: String) -> Result<Self, MomentoError> {
        let claims = decode_jwt(&auth_key)?;
        let formatted_cp_endpoint = format!("https://{}:443", claims.cp);
        let uri = Uri::try_from(formatted_cp_endpoint.as_str())?;
        let channel = Channel::builder(uri).tls_config(ClientTlsConfig::default()).unwrap().connect().await?;
        let interceptor = InterceptedService::new(channel.clone(), AuthHeaderInterceptor { auth_key: auth_key.clone() });
        let client = ScsControlClient::new(interceptor);
        return Ok(Self {
            auth_key: auth_key,
            cache_endpoint: format!("https://{}:443", claims.c),
            client: client,
        })
    }

    pub async fn get_cache(&mut self, name: &str) -> Result<CacheClient, MomentoError> {
        let mut client = CacheClient::new(name.to_string(), self.cache_endpoint.clone(), self.auth_key.clone());
        client.connect().await?;
        return Ok(client);
    }

    pub async fn create_cache(&mut self, name: &str) -> Result<(), MomentoError> {
        let request = Request::new(CreateCacheRequest {
            cache_name: name.to_string()
        });

        self.client.create_cache(request).await?;
        Ok(())
    }

    pub async fn delete_cache(&mut self, name: &str) -> Result<(), MomentoError> {
        let request = Request::new(DeleteCacheRequest {
            cache_name: name.to_string(),
        });
        self.client.delete_cache(request).await?;
        Ok(())
    }
}
