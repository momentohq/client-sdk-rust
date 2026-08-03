use rand::Rng;
use std::{
    sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    time::{Duration, Instant},
};

/// How long the local zone is skipped after its first connect failure.
const BASE_WIDEN_MS: u64 = 500;

/// Ceiling on that window.
const MAX_WIDEN_MS: u64 = 5_000;

/// Sentinel for "not currently widening", stored in `widen_until_millis`.
const NOT_WIDENING: u64 = u64::MAX;

/// Tracks whether the local availability zone is currently reachable, so
/// `az_id` preference can fall back to other zones instead of staying pinned
/// to one it can't reach.
///
/// Deliberately doesn't track per-address health -- the `/endpoints` API
/// already publishes healthy hosts, so that's the control plane's job. What it
/// can't see is a fault visible only from this client (a degraded cross-zone
/// link, broken egress for one zone), which is what this covers.
#[derive(Debug)]
pub(crate) struct AzCircuit {
    /// Fixed reference point; `widen_until_millis` is stored as an offset from
    /// this, since `Instant` itself can't live in an atomic.
    epoch: Instant,
    widen_until_millis: AtomicU64,
    consecutive_failures: AtomicU32,
    /// Set by a failure, cleared only by a confirmed full recovery (not by an
    /// ordinary connect landing back on the local zone) -- see
    /// [`Self::acknowledge_recovery`].
    recovery_needed: AtomicBool,
    base: Duration,
    cap: Duration,
}

impl Default for AzCircuit {
    fn default() -> Self {
        Self::new(
            Duration::from_millis(BASE_WIDEN_MS),
            Duration::from_millis(MAX_WIDEN_MS),
        )
    }
}

impl AzCircuit {
    fn new(base: Duration, cap: Duration) -> Self {
        Self {
            epoch: Instant::now(),
            widen_until_millis: AtomicU64::new(NOT_WIDENING),
            consecutive_failures: AtomicU32::new(0),
            recovery_needed: AtomicBool::new(false),
            base,
            cap,
        }
    }

    /// Whether the local availability zone should be skipped right now.
    pub fn should_widen(&self) -> bool {
        let widen_until = self.widen_until_millis.load(Ordering::Relaxed);
        widen_until != NOT_WIDENING && widen_until > self.millis_since_epoch()
    }

    /// Whether a recovery sweep is outstanding -- there has been a local
    /// failure since the last confirmed full recovery. Unlike
    /// [`Self::should_widen`], this does not decay on its own; only
    /// [`Self::acknowledge_recovery`] clears it.
    pub fn recovery_needed(&self) -> bool {
        self.recovery_needed.load(Ordering::Relaxed)
    }

    /// Record a local-zone connect failure, opening or extending the skip window.
    pub fn record_local_failure(&self) {
        let failures = self.consecutive_failures.fetch_add(1, Ordering::Relaxed) + 1;
        let window = self.widen_window(failures);
        let deadline = self
            .millis_since_epoch()
            .saturating_add(window.as_millis() as u64);
        self.widen_until_millis.store(deadline, Ordering::Relaxed);
        self.recovery_needed.store(true, Ordering::Relaxed);
        log::warn!(
            "local availability zone connect failure #{failures}, preferring other zones for {window:?}",
        );
    }

    /// Record a local-zone connect success, closing the circuit. Does not by
    /// itself clear [`Self::recovery_needed`] -- an ordinary connect landing
    /// on the local zone only proves one pool slot is fine, not that every
    /// slot is. Only a prober's full-pool rebuild can claim that; see
    /// [`Self::acknowledge_recovery`].
    pub fn record_local_success(&self) {
        let was_widening = self
            .widen_until_millis
            .swap(NOT_WIDENING, Ordering::Relaxed)
            != NOT_WIDENING;
        self.consecutive_failures.store(0, Ordering::Relaxed);
        if was_widening {
            log::info!("local availability zone is reachable again");
        }
    }

    /// Record that a recovery sweep has confirmed the local zone is healthy
    /// and rebuilt every pool slot's chance to use it. Closes the circuit and
    /// clears [`Self::recovery_needed`].
    pub fn acknowledge_recovery(&self) {
        self.record_local_success();
        self.recovery_needed.store(false, Ordering::Relaxed);
    }

    fn millis_since_epoch(&self) -> u64 {
        self.epoch.elapsed().as_millis() as u64
    }

    /// Exponential growth, jittered so connections that failed together don't
    /// re-probe the local zone in lockstep.
    fn widen_window(&self, consecutive_failures: u32) -> Duration {
        let shift = consecutive_failures.saturating_sub(1).min(16);
        let ceiling = ((self.base.as_millis() as u64) << shift).min(self.cap.as_millis() as u64);
        Duration::from_millis(rand::rng().random_range(ceiling / 2..=ceiling))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Windows short enough to wait out in a test.
    fn test_circuit() -> AzCircuit {
        AzCircuit::new(Duration::from_millis(40), Duration::from_millis(120))
    }

    #[test]
    fn a_fresh_circuit_prefers_the_local_zone() {
        assert!(!test_circuit().should_widen());
    }

    #[test]
    fn a_fresh_circuit_needs_no_recovery() {
        assert!(!test_circuit().recovery_needed());
    }

    #[test]
    fn a_local_failure_widens_then_expires() {
        let circuit = test_circuit();

        circuit.record_local_failure();
        assert!(circuit.should_widen());

        // The first window is 20-40ms.
        std::thread::sleep(Duration::from_millis(100));
        assert!(!circuit.should_widen());
    }

    #[test]
    fn a_local_failure_marks_recovery_needed_and_it_does_not_decay() {
        let circuit = test_circuit();

        circuit.record_local_failure();
        assert!(circuit.recovery_needed());

        // Unlike should_widen, this stays true even after the window expires.
        std::thread::sleep(Duration::from_millis(100));
        assert!(!circuit.should_widen());
        assert!(circuit.recovery_needed());
    }

    #[test]
    fn success_closes_the_circuit_immediately() {
        let circuit = test_circuit();

        circuit.record_local_failure();
        assert!(circuit.should_widen());

        circuit.record_local_success();
        assert!(!circuit.should_widen());
    }

    #[test]
    fn an_ordinary_success_does_not_clear_recovery_needed() {
        let circuit = test_circuit();

        circuit.record_local_failure();
        circuit.record_local_success();

        // One slot reconnecting on its own doesn't prove every slot has.
        assert!(circuit.recovery_needed());
    }

    #[test]
    fn only_acknowledge_recovery_clears_recovery_needed() {
        let circuit = test_circuit();

        circuit.record_local_failure();
        circuit.acknowledge_recovery();

        assert!(!circuit.recovery_needed());
        assert!(!circuit.should_widen());
    }

    #[test]
    fn success_resets_the_backoff_ladder() {
        let circuit = test_circuit();

        for _ in 0..10 {
            circuit.record_local_failure();
        }
        circuit.record_local_success();
        circuit.record_local_failure();

        assert_eq!(circuit.consecutive_failures.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn the_window_is_capped_and_never_zero() {
        let circuit = test_circuit();
        // Including values that would overflow an unclamped shift.
        for failures in [1, 2, 5, 10, 32, 64, u32::MAX] {
            let window = circuit.widen_window(failures);
            assert!(
                window <= Duration::from_millis(120),
                "{} failures produced {:?}, above the cap",
                failures,
                window
            );
            assert!(
                window >= Duration::from_millis(20),
                "{} failures produced {:?}, below the floor",
                failures,
                window
            );
        }
    }

    #[test]
    fn the_window_grows_with_consecutive_failures() {
        // Jitter covers the lower half of each window, so compare the ceilings:
        // one failure tops out at 40ms, four at 120ms (the cap).
        let circuit = test_circuit();
        assert!(circuit.widen_window(1) <= Duration::from_millis(40));
        assert!(circuit.widen_window(4) > Duration::from_millis(40));
    }

    #[test]
    fn production_defaults_are_sane() {
        let circuit = AzCircuit::default();
        assert!(circuit.widen_window(1) <= Duration::from_millis(BASE_WIDEN_MS));
        assert!(circuit.widen_window(u32::MAX) <= Duration::from_millis(MAX_WIDEN_MS));
    }
}
