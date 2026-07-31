use rand::Rng;
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

/// How long the local availability zone is skipped after its first connect failure.
const BASE_WIDEN_MS: u64 = 500;

/// The ceiling on that window.
const MAX_WIDEN_MS: u64 = 5_000;

/// Tracks whether the local availability zone is currently reachable, so that
/// [`AzAffinity::Preferred`](super::config::configuration::AzAffinity) can fall
/// back to other zones instead of staying pinned to a zone it cannot reach.
///
/// This deliberately does *not* track health per address. The `/endpoints` API
/// publishes healthy hosts, so per-host health is the control plane's job and
/// duplicating it client-side would be both redundant and less informed. What
/// the control plane cannot see is a fault visible only from this client's
/// vantage point -- a degraded cross-zone link, or egress broken for one zone --
/// where the published hosts really are healthy and we still cannot reach them.
/// That is the one thing this covers.
#[derive(Debug)]
pub(crate) struct AzCircuit {
    state: Mutex<State>,
    base: Duration,
    cap: Duration,
}

#[derive(Debug, Default)]
struct State {
    /// When set and still in the future, skip the local zone.
    widen_until: Option<Instant>,
    consecutive_failures: u32,
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
            state: Mutex::new(State::default()),
            base,
            cap,
        }
    }

    /// Whether the local availability zone should be skipped right now.
    #[allow(clippy::expect_used)]
    pub fn should_widen(&self) -> bool {
        let state = self.state.lock().expect("local mutex must not be poisoned");
        state
            .widen_until
            .is_some_and(|deadline| deadline > Instant::now())
    }

    /// Record a failed connection attempt against an address in the local zone,
    /// opening (or extending) the window during which the zone is skipped.
    #[allow(clippy::expect_used)]
    pub fn record_local_failure(&self) {
        let mut state = self.state.lock().expect("local mutex must not be poisoned");
        state.consecutive_failures = state.consecutive_failures.saturating_add(1);
        let window = self.widen_window(state.consecutive_failures);
        state.widen_until = Some(Instant::now() + window);
        log::warn!(
            "local availability zone connect failure #{}, preferring other zones for {:?}",
            state.consecutive_failures,
            window,
        );
    }

    /// Record a successful connection against an address in the local zone,
    /// closing the circuit and resetting the backoff.
    #[allow(clippy::expect_used)]
    pub fn record_local_success(&self) {
        let mut state = self.state.lock().expect("local mutex must not be poisoned");
        if state.widen_until.is_some() {
            log::info!("local availability zone is reachable again");
        }
        state.widen_until = None;
        state.consecutive_failures = 0;
    }

    /// Exponential growth with jitter over the lower half of the window, so
    /// that connections which failed together do not probe the local zone in
    /// lockstep.
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
    fn a_local_failure_widens_then_expires() {
        let circuit = test_circuit();

        circuit.record_local_failure();
        assert!(circuit.should_widen());

        // The first window is 20-40ms.
        std::thread::sleep(Duration::from_millis(100));
        assert!(!circuit.should_widen());
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
    fn success_resets_the_backoff_ladder() {
        let circuit = test_circuit();

        for _ in 0..10 {
            circuit.record_local_failure();
        }
        circuit.record_local_success();
        circuit.record_local_failure();

        let state = circuit.state.lock().expect("not poisoned");
        assert_eq!(state.consecutive_failures, 1);
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
