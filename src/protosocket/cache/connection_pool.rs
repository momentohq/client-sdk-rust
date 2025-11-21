use crate::protosocket::cache::address_provider::AddressProvider;
use crate::protosocket::cache::connection_manager::ProtosocketConnectionManager;
use crate::protosocket::cache::utils::hrw::{place_targets, PlacementTarget};
use crate::{MomentoError, MomentoResult};
use futures::FutureExt;
use momento_protos::protosocket::cache::{CacheCommand, CacheResponse};
use protosocket_rpc::client::RpcClient;
use protosocket_rpc::Message;
use rand::{Rng, SeedableRng};
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::{pin, Pin};
use std::sync::{Arc, Mutex, RwLock};
use std::task::{Context, Poll};

use super::utils::hrw::hrw_hash;

type PoolConnection = Mutex<ConnectionState<CacheCommand, CacheResponse>>;
type ServerConnections = Arc<Vec<PoolConnection>>;
type AddressConnectionMap = HashMap<SocketAddr, ServerConnections>;

thread_local! {
    static RNG: RefCell<rand::rngs::SmallRng> =
        RefCell::new(rand::rngs::SmallRng::from_os_rng());
}

#[derive(Debug)]
pub(crate) struct ConnectionPool {
    connector: ProtosocketConnectionManager,
    address_connections: RwLock<AddressConnectionMap>,
    connections_per_server: usize,
    address_provider: Arc<AddressProvider>,
    az_id: Option<String>,
}

impl ConnectionPool {
    pub async fn new(
        connector: ProtosocketConnectionManager,
        connections_per_server: usize,
        address_provider: Arc<AddressProvider>,
        az_id: Option<String>,
    ) -> MomentoResult<Self> {
        let addresses = address_provider.get_addresses(az_id.as_deref());

        let mut address_connections = HashMap::new();
        for &addr in addresses.iter() {
            let connections: Vec<Mutex<ConnectionState<CacheCommand, CacheResponse>>> = (0
                ..connections_per_server)
                .map(|_| Mutex::new(ConnectionState::Disconnected))
                .collect();
            address_connections.insert(addr, Arc::new(connections));
        }

        Ok(Self {
            connector,
            address_connections: RwLock::new(address_connections),
            connections_per_server,
            address_provider,
            az_id,
        })
    }

    /// Get a consistent connection for the given key using HRW hashing.
    pub async fn get_connection_for_key(
        &self,
        key: &[u8],
    ) -> MomentoResult<RpcClient<CacheCommand, CacheResponse>> {
        self.ensure_addresses_current();

        let (addr, connections) = {
            let address_connections = self
                .address_connections
                .read()
                .expect("address lock must not be poisoned");

            place_targets(
                key,
                0,
                address_connections
                    .iter()
                    .map(|(&addr, conns)| (addr, Arc::clone(conns))),
            )
            .next()
            .ok_or(MomentoError::unknown_error(
                "protosocket_connection",
                Some("No addresses available".to_string()),
            ))?
        };

        let slot = RNG.with_borrow_mut(|rng| rng.random_range(0..connections.len()));
        let connection_state = &connections[slot];

        self.get_or_create_connection(connection_state, addr).await
    }

    pub async fn get_connection(&self) -> MomentoResult<RpcClient<CacheCommand, CacheResponse>> {
        self.ensure_addresses_current();

        let (addr, connections) = {
            let address_connections = self
                .address_connections
                .read()
                .expect("address lock must not be poisoned");

            if address_connections.is_empty() {
                return Err(MomentoError::unknown_error(
                    "protosocket_connection",
                    Some("No addresses available".to_string()),
                ));
            }

            let addr_index =
                RNG.with_borrow_mut(|rng| rng.random_range(0..address_connections.len()));
            let (&addr, connections) = address_connections
                .iter()
                .nth(addr_index)
                .expect("random address index should be within the bounds of the hashmap");
            (addr, Arc::clone(connections))
        };

        let slot = RNG.with_borrow_mut(|rng| rng.random_range(0..connections.len()));
        let connection_state = &connections[slot];

        self.get_or_create_connection(connection_state, addr).await
    }

    async fn get_or_create_connection(
        &self,
        connection_state: &Mutex<ConnectionState<CacheCommand, CacheResponse>>,
        addr: SocketAddr,
    ) -> MomentoResult<RpcClient<CacheCommand, CacheResponse>> {
        let connecting_handle = loop {
            let mut state = connection_state.lock().expect("internal mutex must work");
            break match &mut *state {
                ConnectionState::Connected(shared_connection) => {
                    if shared_connection.is_alive() {
                        return Ok(shared_connection.clone());
                    } else {
                        *state = ConnectionState::Disconnected;
                        continue;
                    }
                }
                ConnectionState::Connecting(join_handle) => join_handle.clone(),
                ConnectionState::Disconnected => {
                    let connector = self.connector.clone();
                    let load = SpawnedConnect {
                        inner: tokio::task::spawn(connector.connect(addr)),
                    }
                    .shared();
                    *state = ConnectionState::Connecting(load.clone());
                    continue;
                }
            };
        };

        match connecting_handle.await {
            Ok(client) => Ok(reconcile_client_slot(connection_state, client)),
            Err(connect_error) => {
                let mut state = connection_state.lock().expect("internal mutex must work");
                *state = ConnectionState::Disconnected;
                Err(MomentoError::unknown_error(
                    "protosocket_connection",
                    Some(format!("{connect_error:?}")),
                ))
            }
        }
    }

    fn ensure_addresses_current(&self) {
        let current_addresses = self.address_provider.get_addresses(self.az_id.as_deref());

        let needs_update = {
            let address_connections = self
                .address_connections
                .read()
                .expect("address lock must not be poisoned");

            if address_connections.len() != current_addresses.len() {
                true
            } else {
                current_addresses
                    .iter()
                    .any(|addr| !address_connections.contains_key(addr))
            }
        };

        if !needs_update {
            return;
        }

        let mut address_connections = self
            .address_connections
            .write()
            .expect("address lock must not be poisoned");

        for &addr in current_addresses.iter() {
            address_connections.entry(addr).or_insert_with(|| {
                Arc::new(
                    (0..self.connections_per_server)
                        .map(|_| Mutex::new(ConnectionState::Disconnected))
                        .collect(),
                )
            });
        }

        let current_set: std::collections::HashSet<_> = current_addresses.iter().copied().collect();
        address_connections.retain(|addr, _| current_set.contains(addr));
    }
}

fn reconcile_client_slot<Request, Response>(
    connection_state: &Mutex<ConnectionState<Request, Response>>,
    client: RpcClient<Request, Response>,
) -> RpcClient<Request, Response>
where
    Request: Message,
    Response: Message,
{
    let mut state = connection_state.lock().expect("internal mutex must work");
    match &mut *state {
        ConnectionState::Connecting(_shared) => {
            // Here we drop the shared handle. If there is another task still waiting on it, they will get notified when
            // the spawned connection task completes. When they come to reconcile with the connection slot, they will
            // favor this connection and drop their own.
            *state = ConnectionState::Connected(client.clone());
            client
        }
        ConnectionState::Connected(rpc_client) => {
            if rpc_client.is_alive() {
                // someone else beat us to it
                rpc_client.clone()
            } else {
                // well this one is broken too, so we should just replace it with our new one
                *state = ConnectionState::Connected(client.clone());
                client
            }
        }
        ConnectionState::Disconnected => {
            // we raced with a disconnect, but we have a new client, so use it
            *state = ConnectionState::Connected(client.clone());
            client
        }
    }
}

#[derive(Debug)]
enum ConnectionState<Request, Response>
where
    Request: Message,
    Response: Message,
{
    Connecting(futures::future::Shared<SpawnedConnect<Request, Response>>),
    Connected(RpcClient<Request, Response>),
    Disconnected,
}

struct SpawnedConnect<Request, Response>
where
    Request: Message,
    Response: Message,
{
    inner: tokio::task::JoinHandle<protosocket_rpc::Result<RpcClient<Request, Response>>>,
}
impl<Request, Response> Future for SpawnedConnect<Request, Response>
where
    Request: Message,
    Response: Message,
{
    type Output = protosocket_rpc::Result<RpcClient<Request, Response>>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        match pin!(&mut self.inner).poll(context) {
            Poll::Ready(Ok(client_result)) => Poll::Ready(client_result),
            Poll::Ready(Err(_join_err)) => {
                Poll::Ready(Err(protosocket_rpc::Error::ConnectionIsClosed))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T> PlacementTarget for (SocketAddr, T) {
    fn placement_seed(&self) -> i32 {
        self.0.placement_seed()
    }
}

impl PlacementTarget for SocketAddr {
    fn placement_seed(&self) -> i32 {
        match self {
            SocketAddr::V4(addr) => {
                let mut b = [0u8; 6];
                b[0..4].copy_from_slice(&addr.ip().octets());
                b[4..6].copy_from_slice(&addr.port().to_be_bytes());
                hrw_hash(&b)
            }
            SocketAddr::V6(addr) => {
                let mut b = [0u8; 18];
                b[0..16].copy_from_slice(&addr.ip().octets());
                b[16..18].copy_from_slice(&addr.port().to_be_bytes());
                hrw_hash(&b)
            }
        }
    }
}
