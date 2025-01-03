use crate::IntoBytes;

use super::permission_scope::{CacheRole, CacheSelector, Permissions};

/// A key for a specific item in a cache.
pub struct CacheItemKey<K: IntoBytes> {
    /// The cache item key
    pub key: K,
}

// An [IntoBytes] type can be passed in as a CacheItemKey.
impl<K: IntoBytes> From<K> for CacheItemKey<K> {
    fn from(key: K) -> Self {
        CacheItemKey { key }
    }
}

/// A key prefix for items in a cache.
pub struct CacheItemKeyPrefix<K: IntoBytes> {
    /// The key prefix
    pub key_prefix: K,
}

/// An [IntoBytes] type can be passed in as a CacheItemKeyPrefix.
impl<K: IntoBytes> From<K> for CacheItemKeyPrefix<K> {
    fn from(key_prefix: K) -> Self {
        CacheItemKeyPrefix { key_prefix }
    }
}

/// A component of a [DisposableTokenCachePermission].
/// Specifies the cache item(s) to which the permission applies.
pub enum CacheItemSelector<K: IntoBytes> {
    /// Access to all cache items
    AllCacheItems,
    /// Access to a specific cache item
    CacheItemKey(CacheItemKey<K>),
    /// Access to all cache items with a specific key prefix
    CacheItemKeyPrefix(CacheItemKeyPrefix<K>),
}

/// A permission to be granted to a new disposable access token, specifying
/// access to specific cache items.
pub struct DisposableTokenCachePermission<K: IntoBytes> {
    /// The type of access granted by the permission.
    pub role: CacheRole,
    /// The cache(s) to which the permission applies.
    pub cache: CacheSelector,
    /// The cache item(s) to which the permission applies.
    pub item_selector: CacheItemSelector<K>,
}

/// A set of permissions to be granted to a new disposable access token.
pub struct DisposableTokenCachePermissions<K: IntoBytes> {
    pub(crate) permissions: Vec<DisposableTokenCachePermission<K>>,
}

/// The permission scope for creating a new disposable access token.
pub enum DisposableTokenScope<K: IntoBytes> {
    /// Set of permissions to be granted to a new token on the level of a cache or topic
    Permissions(Permissions),
    /// Set of permissions to be granted to a new token on the level of a cache item (key or key prefix)
    DisposableTokenPermissions(DisposableTokenCachePermissions<K>),
}
