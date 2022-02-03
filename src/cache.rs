use std::{
    convert::TryFrom,
    time::{Duration, SystemTime},
};

use tokio::time::sleep;

use tonic::{
    codegen::InterceptedService,
    transport::{Channel, ClientTlsConfig, Uri},
    Request,
};

use crate::{
    generated::cache_client::{scs_data_client::ScsDataClient, ECacheResult, GetRequest, SetRequest},
    response::cache_get_response::MomentoGetResponse,
};
use crate::{
    grpc::cache_header_interceptor::CacheHeaderInterceptor,
    response::{
        cache_get_response::MomentoGetStatus,
        cache_set_response::{MomentoSetResponse, MomentoSetStatus},
        error::MomentoError,
    },
};
pub trait MomentoRequest {
    fn into_bytes(self) -> Vec<u8>;
}

impl MomentoRequest for String {
    fn into_bytes(self) -> Vec<u8> {
        self.into_bytes()
    }
}

impl MomentoRequest for Vec<u8> {
    fn into_bytes(self) -> Vec<u8> {
        self
    }
}

impl MomentoRequest for &str {
    fn into_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

pub struct CacheClient {
    client: Option<ScsDataClient<InterceptedService<Channel, CacheHeaderInterceptor>>>,
    channel: Option<tonic::transport::Channel>,
    endpoint: String,
    auth_key: String,
    cache_name: String,
    default_ttl_seconds: u32,
}

impl CacheClient {
    pub fn new(
        cache_name: String,
        endpoint: String,
        auth_key: String,
        default_ttl_seconds: u32,
    ) -> Self {
        return Self {
            client: Option::None,
            endpoint: endpoint,
            channel: Option::None,
            auth_key: auth_key,
            cache_name: cache_name,
            default_ttl_seconds: default_ttl_seconds,
        };
    }

    pub async fn connect(&mut self,  cache_name: String) -> Result<(), MomentoError> {
        let uri = Uri::try_from(self.endpoint.as_str())?;
        let channel = Channel::builder(uri)
            .tls_config(ClientTlsConfig::default())
            .unwrap()
            .connect()
            .await?;

        self.channel = Some(channel);
        let interceptor = InterceptedService::new(
            self.channel.clone().unwrap(),
            CacheHeaderInterceptor {
                auth_key: self.auth_key.to_string(),
                cache_name: cache_name,
            },
        );
        let client = ScsDataClient::new(interceptor);
        self.client = Some(client);
        Ok(())
    }





    pub async fn set<I: MomentoRequest>(
        &self,
        cache_name: String,
        key: I,
        body: I,
        ttl_seconds: Option<u32>,
    ) -> Result<MomentoSetResponse, MomentoError> {
        let request = tonic::Request::new(SetRequest {
            cache_key: key.into_bytes(),
            cache_body: body.into_bytes(),
            ttl_milliseconds: ttl_seconds.unwrap_or(self.default_ttl_seconds) * 1000,
        });
        let _ = self.client.as_ref().unwrap().clone().set(request).await?;
        Ok(MomentoSetResponse {
            result: MomentoSetStatus::OK,
        })
    }
}
