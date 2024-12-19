use crate::IntoBytes;

use super::permission_scope::{CacheRole, CacheSelector, Permissions};

pub struct CacheItemKey<K: IntoBytes> {
    pub key: K,
}

// Can accept a String or &str or bytes as CacheItemKey
// rather than constructing CacheItemKey manually
impl<K: IntoBytes> From<K> for CacheItemKey<K> {
    fn from(key: K) -> Self {
        CacheItemKey { key }
    }
}

pub struct CacheItemKeyPrefix<K: IntoBytes> {
    pub key_prefix: K,
}

impl<K: IntoBytes> From<K> for CacheItemKeyPrefix<K> {
    fn from(key_prefix: K) -> Self {
        CacheItemKeyPrefix { key_prefix }
    }
}

pub enum CacheItemSelector<K: IntoBytes> {
    AllCacheItems,
    CacheItemKey(CacheItemKey<K>),
    CacheItemKeyPrefix(CacheItemKeyPrefix<K>),
}

pub struct DisposableTokenCachePermission<K: IntoBytes> {
    pub role: CacheRole,
    pub cache: CacheSelector,
    pub item_selector: CacheItemSelector<K>,
}

pub struct DisposableTokenCachePermissions<K: IntoBytes> {
    pub(crate) permissions: Vec<DisposableTokenCachePermission<K>>,
}
pub enum DisposableTokenScope<K: IntoBytes> {
    Permissions(Permissions),
    // PredefinedScope, // does this need to exist?
    DisposableTokenPermissions(DisposableTokenCachePermissions<K>),
}
