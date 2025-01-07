use crate::IntoBytes;

use super::{
    disposable_token_scope::{
        CacheItemKey, CacheItemKeyPrefix, CacheItemSelector, DisposableTokenCachePermission,
        DisposableTokenCachePermissions, DisposableTokenScope,
    },
    permission_scope::{CacheRole, CacheSelector},
};

/// A collection of convenience methods for creating disposable token permission scopes.
pub struct DisposableTokenScopes {}

impl DisposableTokenScopes {
    /// Create a ReadWrite permission scope for a specific key in a specific cache.
    pub fn cache_key_read_write(
        cache_selector: impl Into<CacheSelector>,
        key: impl IntoBytes,
    ) -> DisposableTokenScope {
        DisposableTokenScope::DisposableTokenPermissions(DisposableTokenCachePermissions {
            permissions: vec![DisposableTokenCachePermission {
                role: CacheRole::ReadWrite,
                cache: cache_selector.into(),
                item_selector: CacheItemSelector::CacheItemKey(CacheItemKey { 
                    key: key.into_bytes(),
                }),
            }],
        })
    }

    /// Create a ReadWrite permission scope for all keys matching the key prefix in a specific cache.
    pub fn cache_key_prefix_read_write(
        cache_selector: impl Into<CacheSelector>,
        key_prefix: impl IntoBytes,
    ) -> DisposableTokenScope {
        DisposableTokenScope::DisposableTokenPermissions(DisposableTokenCachePermissions {
            permissions: vec![DisposableTokenCachePermission {
                role: CacheRole::ReadWrite,
                cache: cache_selector.into(),
                item_selector: CacheItemSelector::CacheItemKeyPrefix(CacheItemKeyPrefix {
                    key_prefix: key_prefix.into_bytes(),
                }),
            }],
        })
    }

    /// Create a ReadOnly permission scope for a specific key in a specific cache.
    pub fn cache_key_read_only(
        cache_selector: impl Into<CacheSelector>,
        key: impl IntoBytes,
    ) -> DisposableTokenScope {
        DisposableTokenScope::DisposableTokenPermissions(DisposableTokenCachePermissions {
            permissions: vec![DisposableTokenCachePermission {
                role: CacheRole::ReadOnly,
                cache: cache_selector.into(),
                item_selector: CacheItemSelector::CacheItemKey(CacheItemKey { 
                    key: key.into_bytes(),
                }),
            }],
        })
    }

    /// Create a ReadOnly permission scope for all keys matching the key prefix in a specific cache.
    pub fn cache_key_prefix_read_only(
        cache_selector: impl Into<CacheSelector>,
        key_prefix: impl IntoBytes,
    ) -> DisposableTokenScope {
        DisposableTokenScope::DisposableTokenPermissions(DisposableTokenCachePermissions {
            permissions: vec![DisposableTokenCachePermission {
                role: CacheRole::ReadOnly,
                cache: cache_selector.into(),
                item_selector: CacheItemSelector::CacheItemKeyPrefix(CacheItemKeyPrefix {
                    key_prefix: key_prefix.into_bytes(),
                }),
            }],
        })
    }

    /// Create a WriteOnly permission scope for a specific key in a specific cache.
    pub fn cache_key_write_only(
        cache_selector: impl Into<CacheSelector>,
        key: impl IntoBytes,
    ) -> DisposableTokenScope {
        DisposableTokenScope::DisposableTokenPermissions(DisposableTokenCachePermissions {
            permissions: vec![DisposableTokenCachePermission {
                role: CacheRole::WriteOnly,
                cache: cache_selector.into(),
                item_selector: CacheItemSelector::CacheItemKey(CacheItemKey { 
                    key: key.into_bytes()
                }),
            }],
        })
    }

    /// Create a WriteOnly permission scope for all keys matching the key prefix in a specific cache.
    pub fn cache_key_prefix_write_only(
        cache_selector: impl Into<CacheSelector>,
        key_prefix: impl IntoBytes,
    ) -> DisposableTokenScope {
        DisposableTokenScope::DisposableTokenPermissions(DisposableTokenCachePermissions {
            permissions: vec![DisposableTokenCachePermission {
                role: CacheRole::WriteOnly,
                cache: cache_selector.into(),
                item_selector: CacheItemSelector::CacheItemKeyPrefix(CacheItemKeyPrefix {
                    key_prefix: key_prefix.into_bytes(),
                }),
            }],
        })
    }
}
