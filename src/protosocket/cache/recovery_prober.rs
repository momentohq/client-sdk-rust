use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use arc_swap::ArcSwap;
use protosocket_rpc::client::ConnectionPool;

use super::connection_manager::ProtosocketConnectionManager;

/// How often the prober checks whether a recovery sweep is due. Checking is
/// cheap (one atomic load) when nothing needs fixing, so this doesn't need to
/// be tight -- see [`AzCircuit::recovery_needed`](super::az_circuit::AzCircuit::recovery_needed).
const PROBE_INTERVAL: Duration = Duration::from_secs(30);

/// Spacing between prewarming successive slots of a rebuilt pool, so a
/// recovery sweep can never burst-connect the recovering zone even under
/// concurrent load.
const PREWARM_SPACING: Duration = Duration::from_millis(200);

/// Background task that watches for the preferred availability zone becoming
/// reachable again and, when it does, rebuilds the connection pool so every
/// slot gets a fresh chance to use it -- rather than waiting indefinitely for
/// an existing (healthy) fallback-zone connection to happen to die on its own.
#[derive(Debug)]
pub(crate) struct BackgroundRecoveryProber {
    alive: Arc<AtomicBool>,
    _join_handle: tokio::task::JoinHandle<()>,
}

impl Drop for BackgroundRecoveryProber {
    fn drop(&mut self) {
        if self.alive.swap(false, Ordering::Relaxed) {
            log::info!("shutting down az recovery prober");
        }
    }
}

impl BackgroundRecoveryProber {
    /// Spawn the prober, or return `None` if there's no availability zone
    /// preference to actively recover -- the loop would be a permanent no-op.
    pub(crate) fn spawn(
        runtime: &tokio::runtime::Handle,
        connector: ProtosocketConnectionManager,
        connection_count: usize,
        pool: Arc<ArcSwap<ConnectionPool<ProtosocketConnectionManager>>>,
    ) -> Option<Self> {
        if !connector.has_az_preference() {
            return None;
        }

        let alive = Arc::new(AtomicBool::new(true));
        let join_handle = runtime.spawn(recover_forever(
            alive.clone(),
            connector,
            connection_count,
            pool,
        ));
        Some(Self {
            alive,
            _join_handle: join_handle,
        })
    }
}

async fn recover_forever(
    alive: Arc<AtomicBool>,
    connector: ProtosocketConnectionManager,
    connection_count: usize,
    pool: Arc<ArcSwap<ConnectionPool<ProtosocketConnectionManager>>>,
) {
    let mut interval = tokio::time::interval(PROBE_INTERVAL);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let az_circuit = connector.az_circuit();

    loop {
        interval.tick().await;
        if !alive.load(Ordering::Relaxed) {
            break;
        }

        if !az_circuit.recovery_needed() {
            continue;
        }

        if !connector.probe_local_zone().await {
            log::debug!("az recovery probe: local zone still unreachable");
            continue;
        }

        log::info!("az recovery probe succeeded, rebuilding connection pool");
        let fresh_pool = ConnectionPool::new(connector.clone(), connection_count);
        for key in 0..connection_count {
            if let Err(e) = fresh_pool.get_connection_for_key(key).await {
                log::warn!("az recovery prewarm failed for slot {key}: {e:?}");
            }
            tokio::time::sleep(PREWARM_SPACING).await;
        }

        pool.store(Arc::new(fresh_pool));
        az_circuit.acknowledge_recovery();
        log::info!("az recovery complete: connection pool rebuilt against the local zone");
    }
}
