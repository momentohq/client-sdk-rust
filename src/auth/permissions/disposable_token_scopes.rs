use crate::IntoBytes;

use super::{
    disposable_token_scope::{
        CacheItemKey, CacheItemKeyPrefix, CacheItemSelector, DisposableTokenCachePermission,
        DisposableTokenCachePermissions, DisposableTokenScope,
    },
    permission_scope::{CacheRole, CacheSelector},
};

pub struct DisposableTokenScopes {}

impl DisposableTokenScopes {
    pub fn cache_key_read_write(
        cache_selector: impl Into<CacheSelector>,
        key: impl IntoBytes,
    ) -> DisposableTokenScope<impl IntoBytes> {
        DisposableTokenScope::DisposableTokenPermissions(DisposableTokenCachePermissions {
            permissions: vec![DisposableTokenCachePermission {
                role: CacheRole::ReadWrite,
                cache: cache_selector.into(),
                item_selector: CacheItemSelector::CacheItemKey(CacheItemKey { key }),
            }],
        })
    }

    pub fn cache_key_prefix_read_write(
        cache_selector: impl Into<CacheSelector>,
        key_prefix: impl IntoBytes,
    ) -> DisposableTokenScope<impl IntoBytes> {
        DisposableTokenScope::DisposableTokenPermissions(DisposableTokenCachePermissions {
            permissions: vec![DisposableTokenCachePermission {
                role: CacheRole::ReadWrite,
                cache: cache_selector.into(),
                item_selector: CacheItemSelector::CacheItemKeyPrefix(CacheItemKeyPrefix {
                    key_prefix,
                }),
            }],
        })
    }

    pub fn cache_key_read_only(
        cache_selector: impl Into<CacheSelector>,
        key: impl IntoBytes,
    ) -> DisposableTokenScope<impl IntoBytes> {
        DisposableTokenScope::DisposableTokenPermissions(DisposableTokenCachePermissions {
            permissions: vec![DisposableTokenCachePermission {
                role: CacheRole::ReadOnly,
                cache: cache_selector.into(),
                item_selector: CacheItemSelector::CacheItemKey(CacheItemKey { key }),
            }],
        })
    }

    pub fn cache_key_prefix_read_only(
        cache_selector: impl Into<CacheSelector>,
        key_prefix: impl IntoBytes,
    ) -> DisposableTokenScope<impl IntoBytes> {
        DisposableTokenScope::DisposableTokenPermissions(DisposableTokenCachePermissions {
            permissions: vec![DisposableTokenCachePermission {
                role: CacheRole::ReadOnly,
                cache: cache_selector.into(),
                item_selector: CacheItemSelector::CacheItemKeyPrefix(CacheItemKeyPrefix {
                    key_prefix,
                }),
            }],
        })
    }

    pub fn cache_key_write_only(
        cache_selector: impl Into<CacheSelector>,
        key: impl IntoBytes,
    ) -> DisposableTokenScope<impl IntoBytes> {
        DisposableTokenScope::DisposableTokenPermissions(DisposableTokenCachePermissions {
            permissions: vec![DisposableTokenCachePermission {
                role: CacheRole::WriteOnly,
                cache: cache_selector.into(),
                item_selector: CacheItemSelector::CacheItemKey(CacheItemKey { key }),
            }],
        })
    }

    pub fn cache_key_prefix_write_only(
        cache_selector: impl Into<CacheSelector>,
        key_prefix: impl IntoBytes,
    ) -> DisposableTokenScope<impl IntoBytes> {
        DisposableTokenScope::DisposableTokenPermissions(DisposableTokenCachePermissions {
            permissions: vec![DisposableTokenCachePermission {
                role: CacheRole::WriteOnly,
                cache: cache_selector.into(),
                item_selector: CacheItemSelector::CacheItemKeyPrefix(CacheItemKeyPrefix {
                    key_prefix,
                }),
            }],
        })
    }
}
