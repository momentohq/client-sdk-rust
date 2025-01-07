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
    scope: DisposableTokenScope,
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
    item_selector: CacheItemSelector,
) -> cache_permissions::CacheItem {
    match item_selector {
        CacheItemSelector::AllCacheItems => cache_permissions::CacheItem::AllItems(All {}),
        CacheItemSelector::CacheItemKey(CacheItemKey { key }) => {
            cache_permissions::CacheItem::ItemSelector(permissions_type::CacheItemSelector {
                kind: Some(cache_item_selector::Kind::Key(key)),
            })
        }
        CacheItemSelector::CacheItemKeyPrefix(CacheItemKeyPrefix { key_prefix }) => {
            cache_permissions::CacheItem::ItemSelector(permissions_type::CacheItemSelector {
                kind: Some(cache_item_selector::Kind::KeyPrefix(
                    key_prefix,
                )),
            })
        }
    }
}

fn disposable_token_permission_to_grpc_permission(
    permission: DisposableTokenCachePermission,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{DisposableTokenCachePermissions, DisposableTokenScope, Permissions};

    #[test]
    fn creates_expected_grpc_permissions_from_all_data_read_write() {
        let sdk_permissions =
            DisposableTokenScope::Permissions::<String>(Permissions::all_data_read_write());
        let converted_permissions = permissions_from_disposable_token_scope(sdk_permissions);
        let expected_permissions = permission_messages::Permissions {
            kind: Some(permission_messages::permissions::Kind::Explicit(
                permission_messages::ExplicitPermissions {
                    permissions: vec![
                        permission_messages::PermissionsType {
                            kind: Some(
                                permission_messages::permissions_type::Kind::CachePermissions(
                                    permission_messages::permissions_type::CachePermissions {
                                        role: permission_messages::CacheRole::CacheReadWrite as i32,
                                        cache: Some(cache_permissions::Cache::AllCaches(All {})),
                                        cache_item: None,
                                    },
                                ),
                            ),
                        },
                        permission_messages::PermissionsType {
                            kind: Some(
                                permission_messages::permissions_type::Kind::TopicPermissions(
                                    permission_messages::permissions_type::TopicPermissions {
                                        role: permission_messages::TopicRole::TopicReadWrite as i32,
                                        cache: Some(topic_permissions::Cache::AllCaches(All {})),
                                        topic: Some(topic_permissions::Topic::AllTopics(All {})),
                                    },
                                ),
                            ),
                        },
                    ],
                },
            )),
        };
        assert_eq!(converted_permissions, expected_permissions);
    }

    #[test]
    fn creates_expected_grpc_permissions_from_mixed_cache_topics_permissions() {
        // Construct sdk permissions object
        let sdk_permissions = DisposableTokenScope::Permissions::<String>(Permissions {
            permissions: vec![
                // read only for all caches
                Permission::CachePermission(CachePermission {
                    role: CacheRole::ReadOnly,
                    cache: CacheSelector::AllCaches,
                }),
                // read write for cache "foo"
                Permission::CachePermission(CachePermission {
                    role: CacheRole::ReadWrite,
                    cache: CacheSelector::CacheName {
                        name: "foo".to_string(),
                    },
                }),
                // subscribe only to all topics in all caches
                Permission::TopicPermission(TopicPermission {
                    role: TopicRole::SubscribeOnly,
                    cache: CacheSelector::AllCaches,
                    topic: TopicSelector::AllTopics,
                }),
                // publish subscribe to all topics in cache "foo"
                Permission::TopicPermission(TopicPermission {
                    role: TopicRole::PublishSubscribe,
                    cache: CacheSelector::CacheName {
                        name: "foo".to_string(),
                    },
                    topic: TopicSelector::AllTopics,
                }),
                // publish subscribe to topic "bar" in all caches
                Permission::TopicPermission(TopicPermission {
                    role: TopicRole::PublishSubscribe,
                    cache: CacheSelector::AllCaches,
                    topic: TopicSelector::TopicName {
                        name: "bar".to_string(),
                    },
                }),
                // publish only to topic "cat" in cache "dog"
                Permission::TopicPermission(TopicPermission {
                    role: TopicRole::PublishOnly,
                    cache: CacheSelector::CacheName {
                        name: "dog".to_string(),
                    },
                    topic: TopicSelector::TopicName {
                        name: "cat".to_string(),
                    },
                }),
            ],
        });

        // Construct expected grpc permissions object

        // read only for all caches
        let read_only_all_caches = permission_messages::PermissionsType {
            kind: Some(
                permission_messages::permissions_type::Kind::CachePermissions(
                    permission_messages::permissions_type::CachePermissions {
                        role: permission_messages::CacheRole::CacheReadOnly as i32,
                        cache: Some(cache_permissions::Cache::AllCaches(All {})),
                        cache_item: None,
                    },
                ),
            ),
        };

        // read write for cache "foo"
        let read_write_foo = permission_messages::PermissionsType {
            kind: Some(
                permission_messages::permissions_type::Kind::CachePermissions(
                    permission_messages::permissions_type::CachePermissions {
                        role: permission_messages::CacheRole::CacheReadWrite as i32,
                        cache: Some(cache_permissions::Cache::CacheSelector(
                            permission_messages::permissions_type::CacheSelector {
                                kind: Some(cache_selector::Kind::CacheName("foo".to_string())),
                            },
                        )),
                        cache_item: None,
                    },
                ),
            ),
        };

        // publish subscribe to all topics in cache "foo"
        let pub_sub_foo = permission_messages::PermissionsType {
            kind: Some(
                permission_messages::permissions_type::Kind::TopicPermissions(
                    permission_messages::permissions_type::TopicPermissions {
                        role: permission_messages::TopicRole::TopicReadWrite as i32,
                        cache: Some(topic_permissions::Cache::CacheSelector(
                            permission_messages::permissions_type::CacheSelector {
                                kind: Some(cache_selector::Kind::CacheName("foo".to_string())),
                            },
                        )),
                        topic: Some(topic_permissions::Topic::AllTopics(All {})),
                    },
                ),
            ),
        };
        // subscribe only to all topics in all caches
        let sub_only_all_topics = permission_messages::PermissionsType {
            kind: Some(
                permission_messages::permissions_type::Kind::TopicPermissions(
                    permission_messages::permissions_type::TopicPermissions {
                        role: permission_messages::TopicRole::TopicReadOnly as i32,
                        cache: Some(topic_permissions::Cache::AllCaches(All {})),
                        topic: Some(topic_permissions::Topic::AllTopics(All {})),
                    },
                ),
            ),
        };

        // publish subscribe to topic "bar" in all caches
        let pub_sub_bar = permission_messages::PermissionsType {
            kind: Some(
                permission_messages::permissions_type::Kind::TopicPermissions(
                    permission_messages::permissions_type::TopicPermissions {
                        role: permission_messages::TopicRole::TopicReadWrite as i32,
                        cache: Some(topic_permissions::Cache::AllCaches(All {})),
                        topic: Some(topic_permissions::Topic::TopicSelector(
                            permission_messages::permissions_type::TopicSelector {
                                kind: Some(topic_selector::Kind::TopicName("bar".to_string())),
                            },
                        )),
                    },
                ),
            ),
        };

        // publish only to topic "cat" in cache "dog"
        let pub_only_cat_dog = permission_messages::PermissionsType {
            kind: Some(
                permission_messages::permissions_type::Kind::TopicPermissions(
                    permission_messages::permissions_type::TopicPermissions {
                        role: permission_messages::TopicRole::TopicWriteOnly as i32,
                        cache: Some(topic_permissions::Cache::CacheSelector(
                            permission_messages::permissions_type::CacheSelector {
                                kind: Some(cache_selector::Kind::CacheName("dog".to_string())),
                            },
                        )),
                        topic: Some(topic_permissions::Topic::TopicSelector(
                            permission_messages::permissions_type::TopicSelector {
                                kind: Some(topic_selector::Kind::TopicName("cat".to_string())),
                            },
                        )),
                    },
                ),
            ),
        };

        let grpc_permissions = permission_messages::Permissions {
            kind: Some(permission_messages::permissions::Kind::Explicit(
                permission_messages::ExplicitPermissions {
                    permissions: vec![
                        read_only_all_caches,
                        read_write_foo,
                        sub_only_all_topics,
                        pub_sub_foo,
                        pub_sub_bar,
                        pub_only_cat_dog,
                    ],
                },
            )),
        };

        let converted_permissions = permissions_from_disposable_token_scope(sdk_permissions);
        assert_eq!(converted_permissions, grpc_permissions);
    }

    #[test]
    fn creates_expected_grpc_permissions_for_key_specific_read_write_cache_permissions() {
        // Construct sdk permissions object
        let sdk_permissions =
            DisposableTokenScope::DisposableTokenPermissions(DisposableTokenCachePermissions {
                permissions: vec![
                    DisposableTokenCachePermission {
                        role: CacheRole::ReadWrite,
                        cache: CacheSelector::AllCaches,
                        item_selector: CacheItemSelector::CacheItemKey(CacheItemKey {
                            key: "specific-key".to_string(),
                        }),
                    },
                    DisposableTokenCachePermission {
                        role: CacheRole::ReadWrite,
                        cache: CacheSelector::CacheName {
                            name: "foo".to_string(),
                        },
                        item_selector: CacheItemSelector::CacheItemKeyPrefix(CacheItemKeyPrefix {
                            key_prefix: "key-prefix".to_string(),
                        }),
                    },
                ],
            });

        // Construct expected grpc permissions object
        let key_perm = permission_messages::PermissionsType {
            kind: Some(
                permission_messages::permissions_type::Kind::CachePermissions(
                    permission_messages::permissions_type::CachePermissions {
                        role: permission_messages::CacheRole::CacheReadWrite as i32,
                        cache: Some(cache_permissions::Cache::AllCaches(All {})),
                        cache_item: Some(cache_permissions::CacheItem::ItemSelector(
                            permission_messages::permissions_type::CacheItemSelector {
                                kind: Some(cache_item_selector::Kind::Key("specific-key".into())),
                            },
                        )),
                    },
                ),
            ),
        };
        let key_prefix_perm = permission_messages::PermissionsType {
            kind: Some(
                permission_messages::permissions_type::Kind::CachePermissions(
                    permission_messages::permissions_type::CachePermissions {
                        role: permission_messages::CacheRole::CacheReadWrite as i32,
                        cache: Some(cache_permissions::Cache::CacheSelector(
                            permission_messages::permissions_type::CacheSelector {
                                kind: Some(cache_selector::Kind::CacheName("foo".to_string())),
                            },
                        )),
                        cache_item: Some(cache_permissions::CacheItem::ItemSelector(
                            permission_messages::permissions_type::CacheItemSelector {
                                kind: Some(cache_item_selector::Kind::KeyPrefix(
                                    "key-prefix".into(),
                                )),
                            },
                        )),
                    },
                ),
            ),
        };
        let grpc_permissions = permission_messages::Permissions {
            kind: Some(permission_messages::permissions::Kind::Explicit(
                permission_messages::ExplicitPermissions {
                    permissions: vec![key_perm, key_prefix_perm],
                },
            )),
        };

        let converted_permissions = permissions_from_disposable_token_scope(sdk_permissions);
        assert_eq!(converted_permissions, grpc_permissions);
    }
}
