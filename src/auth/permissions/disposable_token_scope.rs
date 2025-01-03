use crate::{auth::PermissionScope, IntoBytes};

use super::permission_scope::{CacheRole, CacheSelector, Permissions};

/// A key for a specific item in a cache.
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
pub struct DisposableTokenCachePermission<K: IntoBytes> {
    /// The type of access granted by the permission.
    pub role: CacheRole,
    /// The cache(s) to which the permission applies.
    pub cache: CacheSelector,
    /// The cache item(s) to which the permission applies.
    pub item_selector: CacheItemSelector<K>,
}

/// A set of permissions to be granted to a new disposable access token.
#[derive(Debug, Clone, PartialEq)]
pub struct DisposableTokenCachePermissions<K: IntoBytes> {
    pub(crate) permissions: Vec<DisposableTokenCachePermission<K>>,
}

/// The permission scope for creating a new disposable access token.
#[derive(Debug, Clone, PartialEq)]
pub enum DisposableTokenScope<K: IntoBytes> {
    /// Set of permissions to be granted to a new token on the level of a cache or topic
    Permissions(Permissions),
    /// Set of permissions to be granted to a new token on the level of a cache item (key or key prefix)
    DisposableTokenPermissions(DisposableTokenCachePermissions<K>),
}

impl From<Permissions> for DisposableTokenScope<String> {
    fn from(permissions: Permissions) -> Self {
        DisposableTokenScope::Permissions(permissions)
    }
}

impl From<PermissionScope> for DisposableTokenScope<String> {
    fn from(permission_scope: PermissionScope) -> Self {
        match permission_scope {
            PermissionScope::Permissions(permissions) => {
                DisposableTokenScope::Permissions(permissions)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::auth::{DisposableTokenScope, Permissions};

    #[test]
    fn should_support_assignment_from_all_data_read_write() {
        let scope: DisposableTokenScope<String> = Permissions::all_data_read_write().into();
        match scope {
            DisposableTokenScope::Permissions(perms) => {
                assert_eq!(2, perms.permissions.len());
                for p in perms.permissions {
                    match p {
                        crate::auth::Permission::CachePermission(cache_perm) => {
                            assert_eq!(crate::auth::CacheRole::ReadWrite, cache_perm.role);
                            assert_eq!(crate::auth::CacheSelector::AllCaches, cache_perm.cache);
                        }
                        crate::auth::Permission::TopicPermission(topic_perm) => {
                            assert_eq!(crate::auth::TopicRole::PublishSubscribe, topic_perm.role);
                            assert_eq!(crate::auth::CacheSelector::AllCaches, topic_perm.cache);
                            assert_eq!(crate::auth::TopicSelector::AllTopics, topic_perm.topic);
                        }
                    }
                }
            }
            _ => panic!("Expected Permissions"),
        }
    }
}
