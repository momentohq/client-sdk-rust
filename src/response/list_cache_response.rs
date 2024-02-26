/// Response cache object for list of caches.
#[derive(Debug)]
pub struct MomentoCache {
    /// Name of the cache associated with a specific client.
    pub cache_name: String,
}

/// The result of a cache list operation.
#[derive(Debug)]
pub struct MomentoListCacheResponse {
    /// Vector of cache information defined in MomentoCache.
    pub caches: Vec<MomentoCache>,
}
