use momento_protos::permission_messages::{
    self, permissions,
    permissions_type::{
        self, cache_item_selector, cache_permissions, cache_selector,
        topic_permissions::{self},
        topic_selector, All,
    },
    ExplicitPermissions, PermissionsType,
};

use crate::{
    auth::permissions::{
        disposable_token_scope::{
            CacheItemKey, CacheItemKeyPrefix, CacheItemSelector, DisposableTokenCachePermission,
            DisposableTokenScope,
        },
        permission_scope::{
            CachePermission, CacheRole, CacheSelector, Permission, TopicPermission, TopicRole,
            TopicSelector,
        },
    },
    IntoBytes,
};

// Create protobuf Permissions from DisposableTokenScope
pub(crate) fn permissions_from_disposable_token_scope(
    scope: DisposableTokenScope<impl IntoBytes>,
) -> permission_messages::Permissions {
    match scope {
        DisposableTokenScope::Permissions(permissions) => permission_messages::Permissions {
            kind: Some(permissions::Kind::Explicit(ExplicitPermissions {
                permissions: permissions
                    .permissions
                    .into_iter()
                    .map(token_permission_to_grpc_permission)
                    .collect(),
            })),
        },
        DisposableTokenScope::DisposableTokenPermissions(permissions) => {
            permission_messages::Permissions {
                kind: Some(permissions::Kind::Explicit(ExplicitPermissions {
                    permissions: permissions
                        .permissions
                        .into_iter()
                        .map(disposable_token_permission_to_grpc_permission)
                        .collect(),
                })),
            }
        }
    }
}

fn token_permission_to_grpc_permission(permission: Permission) -> PermissionsType {
    match permission {
        Permission::CachePermission(cache_perm) => cache_permission_to_grpc_permission(cache_perm),
        Permission::TopicPermission(topic_perm) => topic_permission_to_grpc_permission(topic_perm),
    }
}

fn cache_permission_to_grpc_permission(permission: CachePermission) -> PermissionsType {
    let grpc_cache_perm = permissions_type::CachePermissions {
        role: assign_cache_role(permission.role).into(),
        cache: Some(assign_cache_selector_for_cache_permission(permission.cache)),
        cache_item: None,
    };
    permission_messages::PermissionsType {
        kind: Some(permissions_type::Kind::CachePermissions(grpc_cache_perm)),
    }
}

fn assign_cache_role(role: CacheRole) -> permission_messages::CacheRole {
    match role {
        CacheRole::ReadWrite => permission_messages::CacheRole::CacheReadWrite,
        CacheRole::ReadOnly => permission_messages::CacheRole::CacheReadOnly,
        CacheRole::WriteOnly => permission_messages::CacheRole::CacheWriteOnly,
    }
}

fn assign_cache_selector_for_cache_permission(
    cache_selector: CacheSelector,
) -> cache_permissions::Cache {
    match cache_selector {
        CacheSelector::AllCaches => cache_permissions::Cache::AllCaches(All {}),
        CacheSelector::CacheName { name } => {
            cache_permissions::Cache::CacheSelector(permissions_type::CacheSelector {
                kind: Some(cache_selector::Kind::CacheName(name)),
            })
        }
    }
}

fn assign_cache_selector_for_topic_permission(
    cache_selector: CacheSelector,
) -> topic_permissions::Cache {
    match cache_selector {
        CacheSelector::AllCaches => topic_permissions::Cache::AllCaches(All {}),
        CacheSelector::CacheName { name } => {
            topic_permissions::Cache::CacheSelector(permissions_type::CacheSelector {
                kind: Some(cache_selector::Kind::CacheName(name)),
            })
        }
    }
}

fn assign_topic_role(role: TopicRole) -> permission_messages::TopicRole {
    match role {
        TopicRole::PublishSubscribe => permission_messages::TopicRole::TopicReadWrite,
        TopicRole::SubscribeOnly => permission_messages::TopicRole::TopicReadOnly,
        TopicRole::PublishOnly => permission_messages::TopicRole::TopicWriteOnly,
    }
}

fn assign_topic_selector(topic_selector: TopicSelector) -> topic_permissions::Topic {
    match topic_selector {
        TopicSelector::AllTopics => topic_permissions::Topic::AllTopics(All {}),
        TopicSelector::TopicName { name } => {
            topic_permissions::Topic::TopicSelector(permissions_type::TopicSelector {
                kind: Some(topic_selector::Kind::TopicName(name)),
            })
        }
    }
}

fn topic_permission_to_grpc_permission(permission: TopicPermission) -> PermissionsType {
    let grpc_topics_perm = permissions_type::TopicPermissions {
        role: assign_topic_role(permission.role).into(),
        cache: Some(assign_cache_selector_for_topic_permission(permission.cache)),
        topic: Some(assign_topic_selector(permission.topic)),
    };
    permission_messages::PermissionsType {
        kind: Some(permissions_type::Kind::TopicPermissions(grpc_topics_perm)),
    }
}

fn assign_cache_item_selector(
    item_selector: CacheItemSelector<impl IntoBytes>,
) -> cache_permissions::CacheItem {
    match item_selector {
        CacheItemSelector::AllCacheItems => cache_permissions::CacheItem::AllItems(All {}),
        CacheItemSelector::CacheItemKey(CacheItemKey { key }) => {
            cache_permissions::CacheItem::ItemSelector(permissions_type::CacheItemSelector {
                kind: Some(cache_item_selector::Kind::Key(key.into_bytes())),
            })
        }
        CacheItemSelector::CacheItemKeyPrefix(CacheItemKeyPrefix { key_prefix }) => {
            cache_permissions::CacheItem::ItemSelector(permissions_type::CacheItemSelector {
                kind: Some(cache_item_selector::Kind::KeyPrefix(
                    key_prefix.into_bytes(),
                )),
            })
        }
    }
}

fn disposable_token_permission_to_grpc_permission(
    permission: DisposableTokenCachePermission<impl IntoBytes>,
) -> PermissionsType {
    let grpc_perm = permissions_type::CachePermissions {
        role: assign_cache_role(permission.role).into(),
        cache: Some(assign_cache_selector_for_cache_permission(permission.cache)),
        cache_item: Some(assign_cache_item_selector(permission.item_selector)),
    };
    permission_messages::PermissionsType {
        kind: Some(permissions_type::Kind::CachePermissions(grpc_perm)),
    }
}
