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

/// A connection pool that supports consistent connections per key using HRW hashing
///
/// The pool gets a list of addresses it connects to at attempts to maintain max_connections / num_addresses
/// connections per address, with a minimum of one. It uses addresses as keys so that it can use HRW hashing
/// to select a consistent address for a given key. If max_connections is smaller than num_addresses, it will
/// only connect to enough addresses to allow for one connection per address up to max_connections.
///
/// The address provider is checked when attempting to get a connection by comparing the last synced generation
/// number with the generation counter the address provider maintains. If those are different, the pool gets the
/// latest addresses from the provider and sorts them using HRW to make sure a consistent subset of them will be
/// used. It then gets a write lock on the address_connections map and updates it with the new addresses,
/// preserving any connections it has to addresses that haven't changed with the new generation.
///
/// Connections that have been dispensed by the pool remain valid even if their address has been removed from
/// the pool to allow for in-flight requests during an address update.
#[derive(Debug)]
pub(crate) struct ConnectionPool {
    connector: ProtosocketConnectionManager,
    address_connections:
        RwLock<BoundedAddressConnectionMap<ConnectionState<CacheCommand, CacheResponse>>>,
    address_provider: Arc<AddressProvider>,
    last_synced_generation: AtomicU64,
    az_id: Option<String>,
}

impl ConnectionPool {
    pub async fn new(
        connector: ProtosocketConnectionManager,
        max_connections: usize,
        address_provider: Arc<AddressProvider>,
        az_id: Option<String>,
    ) -> MomentoResult<Self> {
        let address_connections = BoundedAddressConnectionMap::new(max_connections);

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

    /// Check if the list of addresses used by the pool is current and update it if not.
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
        if current_generation == self.last_synced_generation.load(Ordering::Acquire) {
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

    /// Get a random connection from the pool.
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

type PoolConnection<S> = Mutex<S>;
type ServerConnections<S> = Arc<Vec<PoolConnection<S>>>;

/// Map of SocketAddrs to Vecs of connections.
///
/// The length of the Vecs for each key is max_connections / keys().len(), with a minimum of 1.
/// If the map is updated with more keys than max_connections, only the first max_connections keys
// will be used in the map to let each key have a connection and avoid going over max_connections.
#[derive(Debug)]
struct BoundedAddressConnectionMap<S: Default> {
    map: HashMap<SocketAddr, ServerConnections<S>>,
    max_total_connections: usize,
}

impl<S: Default> BoundedAddressConnectionMap<S> {
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

    #[cfg(test)]
    fn get_connections(&self, addr: &SocketAddr) -> Option<&ServerConnections<S>> {
        self.map.get(addr)
    }

    fn iter(&self) -> impl Iterator<Item = (&SocketAddr, &ServerConnections<S>)> {
        self.map.iter()
    }

    /// Replace the addresses in the map with the given ordered list of addresses.
    ///
    /// The sum of the lengths of the Vecs in the map will always be less than max_total_connections,
    /// even if only a subset of addreses can be used. If the map contains addresses in common with
    /// those in the given list, it will preserve those connections.
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
                    let new_connections: Vec<PoolConnection<S>> =
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddrV4};

    mod bounded_address_connection_map {
        use super::*;

        fn make_addr(port: u16) -> SocketAddr {
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port))
        }

        #[derive(Debug, Default, PartialEq)]
        enum TestState {
            #[default]
            Empty,
            HasValue(u32),
        }

        type TestMap = BoundedAddressConnectionMap<TestState>;

        #[test]
        fn new_creates_empty_map() {
            let map: TestMap = BoundedAddressConnectionMap::new(10);
            assert!(map.is_empty());
            assert_eq!(map.len(), 0);
        }

        #[test]
        fn sync_addresses_adds_new_addresses() {
            let mut map: TestMap = BoundedAddressConnectionMap::new(10);
            let addresses = vec![make_addr(8080), make_addr(8081)];

            map.sync_addresses(&addresses);

            assert_eq!(map.len(), 2);
        }

        #[test]
        fn sync_addresses_removes_stale_addresses() {
            let mut map: TestMap = BoundedAddressConnectionMap::new(10);
            let initial = vec![make_addr(8080), make_addr(8081), make_addr(8082)];
            map.sync_addresses(&initial);

            let updated = vec![make_addr(8080), make_addr(8082)];
            map.sync_addresses(&updated);

            assert_eq!(map.len(), 2);
            assert!(map.get_connections(&make_addr(8080)).is_some());
            assert!(map.get_connections(&make_addr(8081)).is_none());
            assert!(map.get_connections(&make_addr(8082)).is_some());
        }

        #[test]
        fn sync_addresses_clears_on_empty_input() {
            let mut map: TestMap = BoundedAddressConnectionMap::new(10);
            map.sync_addresses(&[make_addr(8080), make_addr(8081)]);

            map.sync_addresses(&[]);

            assert!(map.is_empty());
        }

        #[test]
        fn sync_addresses_creates_correct_connections_per_server() {
            let mut map: TestMap = BoundedAddressConnectionMap::new(7);
            let addresses = vec![make_addr(8080), make_addr(8081)];

            map.sync_addresses(&addresses);

            // 7 total / 2 addresses = 3 connections per server
            let connections = map.get_connections(&addresses[0]).unwrap();
            assert_eq!(connections.len(), 3);
        }

        #[test]
        fn sync_addresses_ensures_minimum_of_one_connection_per_server() {
            let mut map: TestMap = BoundedAddressConnectionMap::new(4);
            let addresses = vec![make_addr(8080), make_addr(8081), make_addr(8082)];

            map.sync_addresses(&addresses);

            for addr in map.iter() {
                assert!(addr.1.len() == 1);
            }
        }

        #[test]
        fn sync_addresses_removes_addresses_if_order_changes() {
            let mut map: TestMap = BoundedAddressConnectionMap::new(3);
            let addresses = vec![
                make_addr(8080),
                make_addr(8081),
                make_addr(8082),
                make_addr(8083),
            ];
            map.sync_addresses(&addresses);

            assert!(map.get_connections(&make_addr(8080)).is_some());
            assert!(map.get_connections(&make_addr(8081)).is_some());
            assert!(map.get_connections(&make_addr(8082)).is_some());
            assert!(map.get_connections(&make_addr(8083)).is_none());

            // Same addresses, different order
            let addresses = vec![
                make_addr(8083),
                make_addr(8081),
                make_addr(8082),
                make_addr(8080),
            ];
            map.sync_addresses(&addresses);

            assert!(map.get_connections(&make_addr(8080)).is_none());
            assert!(map.get_connections(&make_addr(8081)).is_some());
            assert!(map.get_connections(&make_addr(8082)).is_some());
            assert!(map.get_connections(&make_addr(8083)).is_some());
        }

        #[test]
        fn sync_addresses_preserves_existing_connections_when_resizing() {
            let mut map: TestMap = BoundedAddressConnectionMap::new(10);
            let address = make_addr(8080);
            map.sync_addresses(&[address]);

            // Change the state of a connection
            {
                let connections = map.get_connections(&address).unwrap();
                let mut state = connections[0].lock().unwrap();
                assert!(matches!(*state, TestState::Empty));

                *state = TestState::HasValue(1);
            }

            // Existing connections should be preserved during a sync
            map.sync_addresses(&[address]);

            let connections = map.get_connections(&address).unwrap();
            let state = connections[0].lock().unwrap();
            assert!(matches!(*state, TestState::HasValue(1)));
        }

        #[test]
        fn sync_preserves_state_when_shrinking_connections_per_server() {
            let mut map: TestMap = BoundedAddressConnectionMap::new(20);
            let address = make_addr(8080);
            map.sync_addresses(&[address]); // 20 connections per server

            // Set states in first 6 slots
            {
                let connections = map.get_connections(&address).unwrap();
                for i in 0..6 {
                    *connections[i].lock().unwrap() = TestState::HasValue(i as u32);
                }
            }

            // 5 connections per server
            map.sync_addresses(&[address, make_addr(8081), make_addr(8082), make_addr(8083)]);

            // First 5 states should be preserved, 6th was dropped
            let connections = map.get_connections(&address).unwrap();
            assert_eq!(connections.len(), 5);
            for i in 0..5 {
                assert_eq!(
                    *connections[i].lock().unwrap(),
                    TestState::HasValue(i as u32)
                );
            }
        }

        #[test]
        fn sync_preserves_state_when_growing_connections_per_server() {
            let mut map: TestMap = BoundedAddressConnectionMap::new(20);
            let addresses = vec![
                make_addr(8080),
                make_addr(8081),
                make_addr(8082),
                make_addr(8083),
            ];
            map.sync_addresses(&addresses); // 5 connections per server

            // Set state on first server
            {
                let connections = map.get_connections(&addresses[0]).unwrap();
                *connections[0].lock().unwrap() = TestState::HasValue(1);
            }

            map.sync_addresses(&[addresses[0]]); // 20 connections per server

            // State should be preserved
            let connections = map.get_connections(&addresses[0]).unwrap();
            assert_eq!(connections.len(), 20);
            assert_eq!(*connections[0].lock().unwrap(), TestState::HasValue(1));
        }
    }
}
