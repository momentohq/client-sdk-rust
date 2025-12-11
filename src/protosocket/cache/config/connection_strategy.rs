/// Strategy for selecting which connection to use for a request.
#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub enum ConnectionStrategy {
    /// Select a connection randomly for each request.
    #[default]
    Random,
    /// Select a connection based on a hash of the cache key.
    /// This provides consistent routing for the same keys.
    KeyHash,
}
