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
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::task::{Context, Poll};

use super::utils::hrw::hrw_hash;

thread_local! {
    static RNG: RefCell<rand::rngs::SmallRng> =
        RefCell::new(rand::rngs::SmallRng::from_os_rng());
}

#[derive(Debug)]
pub(crate) struct ConnectionPool {
    connector: ProtosocketConnectionManager,
    address_connections: RwLock<BoundedAddressConnectionMap>,
    address_provider: Arc<AddressProvider>,
    last_synced_generation: AtomicU64,
    az_id: Option<String>,
}

impl ConnectionPool {
    pub async fn new(
        connector: ProtosocketConnectionManager,
        max_total_connections: usize,
        address_provider: Arc<AddressProvider>,
        az_id: Option<String>,
    ) -> MomentoResult<Self> {
        let address_connections = BoundedAddressConnectionMap::new(max_total_connections);

        // Force a sync by presetting the synced generation to MAX
        let last_synced_generation = AtomicU64::new(u64::MAX);

        Ok(Self {
            connector,
            address_connections: RwLock::new(address_connections),
            address_provider,
            last_synced_generation,
            az_id,
        })
    }

    #[allow(clippy::expect_used)]
    fn ensure_addresses_current(&self) {
        let current_generation = self.address_provider.get_generation();
        let last_synced = self.last_synced_generation.load(Ordering::Acquire);

        if current_generation == last_synced {
            return;
        }

        let current_addresses = self.address_provider.get_addresses(self.az_id.as_deref());

        // Order the addresses in a consistent way in case we're only able to add a subset of them
        let ordered_addresses: Vec<_> =
            place_targets(&[], 0, current_addresses.iter().copied()).collect();

        let mut address_connections = self
            .address_connections
            .write()
            .expect("address lock must not be poisoned");

        // Re-check after acquiring the lock in case another thread synced already
        if current_generation <= self.last_synced_generation.load(Ordering::Acquire) {
            return;
        }

        address_connections.sync_addresses(&ordered_addresses);

        // Update after successfully syncing
        self.last_synced_generation
            .store(current_generation, Ordering::Release);
    }

    /// Get a consistent connection for the given key using HRW hashing.
    #[allow(clippy::expect_used)]
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

    #[allow(clippy::expect_used)]
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

    #[allow(clippy::expect_used)]
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
}

#[allow(clippy::expect_used)]
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

#[derive(Debug, Default)]
enum ConnectionState<Request, Response>
where
    Request: Message,
    Response: Message,
{
    Connecting(futures::future::Shared<SpawnedConnect<Request, Response>>),
    Connected(RpcClient<Request, Response>),
    #[default]
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

type PoolConnection = Mutex<ConnectionState<CacheCommand, CacheResponse>>;
type ServerConnections = Arc<Vec<PoolConnection>>;

#[derive(Debug)]
struct BoundedAddressConnectionMap {
    map: HashMap<SocketAddr, ServerConnections>,
    max_total_connections: usize,
}

impl BoundedAddressConnectionMap {
    fn new(max_total_connections: usize) -> Self {
        Self {
            map: HashMap::new(),
            max_total_connections,
        }
    }

    fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    fn len(&self) -> usize {
        self.map.len()
    }

    fn iter(&self) -> impl Iterator<Item = (&SocketAddr, &ServerConnections)> {
        self.map.iter()
    }

    fn sync_addresses(&mut self, ordered_addresses: &[SocketAddr]) {
        if ordered_addresses.is_empty() {
            self.map.clear();
            return;
        }

        // Determine how many connections per address we want
        let target_per_server = (self.max_total_connections / ordered_addresses.len()).max(1);

        // Build a list of addresses that will be in the map and retain based on that.
        let max_addresses = self.max_total_connections / target_per_server;
        let addresses_to_use: &[SocketAddr] = ordered_addresses
            .get(..max_addresses)
            .unwrap_or(ordered_addresses);

        // Remove stale addresses
        self.map.retain(|addr, _| addresses_to_use.contains(addr));

        for address in addresses_to_use {
            if let Some(connections) = self.map.get_mut(address) {
                // Ensure we have the correct number of connections for this address
                if connections.len() != target_per_server {
                    let new_connections: Vec<PoolConnection> =
                        std::iter::repeat_with(Mutex::default)
                            .take(target_per_server)
                            .collect();

                    let copy_count = connections.len().min(target_per_server);
                    for i in 0..copy_count {
                        let old_state = std::mem::take(
                            &mut *connections[i]
                                .lock()
                                .expect("connections mutex must not be poisoned"),
                        );
                        *new_connections[i]
                            .lock()
                            .expect("connections mutex must not be poisoned") = old_state;
                    }

                    *connections = Arc::new(new_connections);
                }
            } else {
                // Add the new address if we have space
                self.map.insert(
                    *address,
                    Arc::new(
                        std::iter::repeat_with(Mutex::default)
                            .take(target_per_server)
                            .collect(),
                    ),
                );
            }
        }
    }
}
