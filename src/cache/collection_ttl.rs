use std::time::Duration;

/// Represents the desired behavior for managing the TTL on collection objects.
///
/// For cache operations that modify a collection (dictionaries, lists, or sets), there
/// are a few things to consider. The first time the collection is created, we need to
/// set a TTL on it. For subsequent operations that modify the collection you may choose
/// to update the TTL in order to prolong the life of the cached collection object, or
/// you may choose to leave the TTL unmodified in order to ensure that the collection
/// expires at the original TTL.
///
/// The default behaviour is to refresh the TTL (to prolong the life of the collection)
/// each time it is written using the client's default item TTL.
#[derive(Copy, Clone, Debug)]
pub struct CollectionTtl {
    ttl: Option<Duration>,
    refresh: bool,
}

impl CollectionTtl {
    /// Create a collection TTL with the provided `ttl` and `refresh` settings.
    pub const fn new(ttl: Option<Duration>, refresh: bool) -> Self {
        Self { ttl, refresh }
    }

    /// Create a collection TTL that updates the TTL for the collection any time it is
    /// modified.
    ///
    /// If `ttl` is `None` then the default item TTL for the client will be used.
    pub fn refresh_on_update(ttl: impl Into<Option<Duration>>) -> Self {
        Self::new(ttl.into(), true)
    }

    /// Create a collection TTL that will not refresh the TTL for the collection when
    /// it is updated.
    ///
    /// Use this if you want to be sure that the collection expires at the originally
    /// specified time, even if you make modifications to the value of the collection.
    ///
    /// The TTL will still be used when a new collection is created. If `ttl` is `None`
    /// then the default item TTL for the client will be used.
    pub fn initialize_only(ttl: impl Into<Option<Duration>>) -> Self {
        Self::new(ttl.into(), false)
    }

    /// Create a collection TTL that updates the TTL for the collection only if an
    /// explicit `ttl` is provided here.
    pub fn refresh_if_provided(ttl: impl Into<Option<Duration>>) -> Self {
        let ttl = ttl.into();
        Self::new(ttl, ttl.is_some())
    }

    /// Return a new collection TTL which uses the same TTL but refreshes on updates.
    pub fn with_refresh_on_update(self) -> Self {
        Self::new(self.ttl(), true)
    }

    /// Return a new collection TTL which uses the same TTL but does not refresh on
    /// updates.
    pub fn with_no_refresh_on_update(self) -> Self {
        Self::new(self.ttl(), false)
    }

    /// Return a new collecton TTL which has the same refresh behaviour but uses the
    /// provided TTL.
    pub fn with_ttl(self, ttl: impl Into<Option<Duration>>) -> Self {
        Self::new(ttl.into(), self.refresh())
    }

    /// The [`Duration`] after which the cached collection should be expired from the
    /// cache.
    ///
    /// If `None`, the default item TTL for the client will be used.
    pub fn ttl(&self) -> Option<Duration> {
        self.ttl
    }

    /// Whether the collection's TTL will be refreshed on every update.
    ///
    /// If true, this will extend the time at which the collection would expire when
    /// an update operation happens. Otherwise, the collection's TTL will only be set
    /// when it is initially created.
    pub fn refresh(&self) -> bool {
        self.refresh
    }
}

impl Default for CollectionTtl {
    fn default() -> Self {
        Self::new(None, true)
    }
}