use crate::{auth::PermissionScope, IntoBytes};

use super::permission_scope::{CacheRole, CacheSelector, Permissions};
use derive_more::Display;

/// A key for a specific item in a cache.
#[derive(Debug, Clone, PartialEq)]
pub struct CacheItemKey {
    /// The cache item key
    pub key: Vec<u8>,
}

// An [IntoBytes] type can be passed in as a CacheItemKey.
impl<K: IntoBytes> From<K> for CacheItemKey {
    fn from(key: K) -> Self {
        CacheItemKey {
            key: key.into_bytes(),
        }
    }
}

/// A key prefix for items in a cache.
#[derive(Debug, Clone, PartialEq)]
pub struct CacheItemKeyPrefix {
    /// The key prefix
    pub key_prefix: Vec<u8>,
}

/// An [IntoBytes] type can be passed in as a CacheItemKeyPrefix.
impl<K: IntoBytes> From<K> for CacheItemKeyPrefix {
    fn from(key_prefix: K) -> Self {
        CacheItemKeyPrefix {
            key_prefix: key_prefix.into_bytes(),
        }
    }
}

/// A component of a [DisposableTokenCachePermission].
/// Specifies the cache item(s) to which the permission applies.
#[derive(Debug, Clone, PartialEq)]
pub enum CacheItemSelector {
    /// Access to all cache items
    AllCacheItems,
    /// Access to a specific cache item
    CacheItemKey(CacheItemKey),
    /// Access to all cache items with a specific key prefix
    CacheItemKeyPrefix(CacheItemKeyPrefix),
}

/// A permission to be granted to a new disposable access token, specifying
/// access to specific cache items.
#[derive(Debug, Clone, PartialEq)]
pub struct DisposableTokenCachePermission {
    /// The type of access granted by the permission.
    pub role: CacheRole,
    /// The cache(s) to which the permission applies.
    pub cache: CacheSelector,
    /// The cache item(s) to which the permission applies.
    pub item_selector: CacheItemSelector,
}

/// A set of permissions to be granted to a new disposable access token.
#[derive(Debug, Clone, PartialEq)]
pub struct DisposableTokenCachePermissions {
    pub(crate) permissions: Vec<DisposableTokenCachePermission>,
}

/// The permission scope for creating a new disposable access token.
#[derive(Debug, Clone, PartialEq)]
pub enum DisposableTokenScope {
    /// Set of permissions to be granted to a new token on the level of a cache or topic
    Permissions(Permissions),
    /// Set of permissions to be granted to a new token on the level of a cache item (key or key prefix)
    DisposableTokenPermissions(DisposableTokenCachePermissions),
}

impl From<Permissions> for DisposableTokenScope {
    fn from(permissions: Permissions) -> Self {
        DisposableTokenScope::Permissions(permissions)
    }
}

impl From<DisposableTokenCachePermissions> for DisposableTokenScope {
    fn from(permissions: DisposableTokenCachePermissions) -> Self {
        DisposableTokenScope::DisposableTokenPermissions(permissions)
    }
}

impl From<PermissionScope> for DisposableTokenScope {
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
    use crate::auth::{
        CacheItemKey, CacheItemKeyPrefix, CacheItemSelector, CachePermission, CacheRole,
        CacheSelector, DisposableTokenCachePermission, DisposableTokenCachePermissions,
        DisposableTokenScope, DisposableTokenScopes, Permission, PermissionScopes, Permissions,
        TopicPermission, TopicRole, TopicSelector,
    };
    use crate::IntoBytes;

    #[test]
    fn should_support_assignment_from_all_data_read_write() {
        let scope: DisposableTokenScope<String> = Permissions::all_data_read_write().into();
        let expected_permissions = vec![
            Permission::CachePermission(crate::auth::CachePermission {
                role: crate::auth::CacheRole::ReadWrite,
                cache: crate::auth::CacheSelector::AllCaches,
            }),
            Permission::TopicPermission(TopicPermission {
                role: TopicRole::PublishSubscribe,
                cache: crate::auth::CacheSelector::AllCaches,
                topic: crate::auth::TopicSelector::AllTopics,
            }),
        ];
        assert_eq!(
            DisposableTokenScope::Permissions(Permissions {
                permissions: expected_permissions
            }),
            scope
        );
    }

    mod from_permissions_literals {
        use super::*;

        #[test]
        fn cache_selectors_in_cache_permission() {
            let expected_permissions = vec![Permission::CachePermission(CachePermission {
                cache: CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                role: CacheRole::ReadWrite,
            })];
            let expected_scope = DisposableTokenScope::Permissions(Permissions {
                permissions: expected_permissions,
            });

            // Accepts string literal as cache selector
            let perm_literal = Permissions {
                permissions: vec![Permission::CachePermission(CachePermission {
                    cache: "my-cache".into(),
                    role: CacheRole::ReadWrite,
                })],
            };
            let disp_token_scope: DisposableTokenScope<String> = perm_literal.into();
            assert_eq!(expected_scope, disp_token_scope);

            // Accepts CacheName as cache selector
            let perm_literal = Permissions {
                permissions: vec![Permission::CachePermission(CachePermission {
                    cache: CacheSelector::CacheName {
                        name: "my-cache".into(),
                    },
                    role: CacheRole::ReadWrite,
                })],
            };
            let disp_token_scope: DisposableTokenScope<String> = perm_literal.into();
            assert_eq!(expected_scope, disp_token_scope);

            // Accepts AllCaches as cache selector
            let expected_permissions = vec![Permission::CachePermission(CachePermission {
                cache: CacheSelector::AllCaches,
                role: CacheRole::ReadWrite,
            })];
            let expected_scope = DisposableTokenScope::Permissions(Permissions {
                permissions: expected_permissions,
            });
            let perm_literal = Permissions {
                permissions: vec![Permission::CachePermission(CachePermission {
                    cache: CacheSelector::AllCaches,
                    role: CacheRole::ReadWrite,
                })],
            };
            let disp_token_scope: DisposableTokenScope<String> = perm_literal.into();
            assert_eq!(expected_scope, disp_token_scope);
        }

        #[test]
        fn item_selectors_in_cache_permission() {
            // Accepts string literal as cache key selector
            let expected_permissions = vec![DisposableTokenCachePermission {
                cache: CacheSelector::AllCaches,
                role: CacheRole::ReadWrite,
                item_selector: CacheItemSelector::CacheItemKey(CacheItemKey { key: "my-key" }),
            }];
            let expected_scope =
                DisposableTokenScope::DisposableTokenPermissions(DisposableTokenCachePermissions {
                    permissions: expected_permissions,
                });
            let perm_literal = DisposableTokenCachePermissions {
                permissions: vec![DisposableTokenCachePermission {
                    cache: CacheSelector::AllCaches,
                    role: CacheRole::ReadWrite,
                    item_selector: CacheItemSelector::CacheItemKey("my-key".into()),
                }],
            };
            let disp_token_scope: DisposableTokenScope<&str> =
                DisposableTokenScope::DisposableTokenPermissions(perm_literal);
            assert_eq!(expected_scope, disp_token_scope);

            // Accepts string literal as cache key prefix selector
            let expected_permissions = vec![DisposableTokenCachePermission {
                cache: CacheSelector::AllCaches,
                role: CacheRole::ReadWrite,
                item_selector: CacheItemSelector::CacheItemKeyPrefix(CacheItemKeyPrefix::<&str> {
                    key_prefix: "my-key-prefix",
                }),
            }];
            let expected_scope =
                DisposableTokenScope::DisposableTokenPermissions(DisposableTokenCachePermissions {
                    permissions: expected_permissions,
                });
            let perm_literal = DisposableTokenCachePermissions {
                permissions: vec![DisposableTokenCachePermission {
                    cache: CacheSelector::AllCaches,
                    role: CacheRole::ReadWrite,
                    item_selector: CacheItemSelector::CacheItemKeyPrefix("my-key-prefix".into()),
                }],
            };
            let disp_token_scope: DisposableTokenScope<&str> =
                DisposableTokenScope::DisposableTokenPermissions(perm_literal);
            assert_eq!(expected_scope, disp_token_scope);

            // Accepts CacheItemKey as cache selector
            let expected_permissions = vec![DisposableTokenCachePermission {
                cache: CacheSelector::AllCaches,
                role: CacheRole::ReadWrite,
                item_selector: CacheItemSelector::CacheItemKey(CacheItemKey::<String> {
                    key: "my-key".into(),
                }),
            }];
            let expected_scope =
                DisposableTokenScope::DisposableTokenPermissions(DisposableTokenCachePermissions {
                    permissions: expected_permissions,
                });
            let perm_literal = DisposableTokenCachePermissions {
                permissions: vec![DisposableTokenCachePermission {
                    cache: CacheSelector::AllCaches,
                    role: CacheRole::ReadWrite,
                    item_selector: CacheItemSelector::CacheItemKey(CacheItemKey {
                        key: "my-key".into(),
                    }),
                }],
            };
            let disp_token_scope: DisposableTokenScope<String> = perm_literal.into();
            assert_eq!(expected_scope, disp_token_scope);

            // Accepts CacheItemKeyPrefix as cache selector
            let expected_permissions = vec![DisposableTokenCachePermission {
                cache: CacheSelector::AllCaches,
                role: CacheRole::ReadWrite,
                item_selector: CacheItemSelector::CacheItemKeyPrefix(
                    CacheItemKeyPrefix::<String> {
                        key_prefix: "my-key-prefix".into(),
                    },
                ),
            }];
            let expected_scope =
                DisposableTokenScope::DisposableTokenPermissions(DisposableTokenCachePermissions {
                    permissions: expected_permissions,
                });
            let perm_literal = DisposableTokenCachePermissions {
                permissions: vec![DisposableTokenCachePermission {
                    cache: CacheSelector::AllCaches,
                    role: CacheRole::ReadWrite,
                    item_selector: CacheItemSelector::CacheItemKeyPrefix(CacheItemKeyPrefix {
                        key_prefix: "my-key-prefix".into(),
                    }),
                }],
            };
            let disp_token_scope: DisposableTokenScope<String> = perm_literal.into();
            assert_eq!(expected_scope, disp_token_scope);

            // Accepts AllCacheItems as item selector
            let expected_permissions = vec![DisposableTokenCachePermission {
                cache: CacheSelector::AllCaches,
                role: CacheRole::ReadWrite,
                item_selector: CacheItemSelector::AllCacheItems,
            }];
            let expected_scope =
                DisposableTokenScope::DisposableTokenPermissions(DisposableTokenCachePermissions {
                    permissions: expected_permissions,
                });
            let perm_literal = DisposableTokenCachePermissions {
                permissions: vec![DisposableTokenCachePermission {
                    cache: CacheSelector::AllCaches,
                    role: CacheRole::ReadWrite,
                    item_selector: CacheItemSelector::AllCacheItems,
                }],
            };
            let disp_token_scope: DisposableTokenScope<String> = perm_literal.into();
            assert_eq!(expected_scope, disp_token_scope);
        }

        #[test]
        fn cache_selectors_in_topic_permission() {
            let expected_permissions = vec![Permission::TopicPermission(TopicPermission {
                cache: CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                topic: "my-topic".into(),
                role: TopicRole::PublishSubscribe,
            })];
            let expected_scope = DisposableTokenScope::Permissions(Permissions {
                permissions: expected_permissions,
            });

            // Accepts string literal as cache selector
            let perm_literal = Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: "my-cache".into(),
                    topic: "my-topic".into(),
                    role: TopicRole::PublishSubscribe,
                })],
            };
            let disp_token_scope: DisposableTokenScope<String> = perm_literal.into();
            assert_eq!(expected_scope, disp_token_scope);

            // Accepts CacheName as cache selector
            let perm_literal = Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: CacheSelector::CacheName {
                        name: "my-cache".into(),
                    },
                    topic: "my-topic".into(),
                    role: TopicRole::PublishSubscribe,
                })],
            };
            let disp_token_scope: DisposableTokenScope<String> = perm_literal.into();
            assert_eq!(expected_scope, disp_token_scope);

            // Accepts AllCaches as cache selector
            let expected_permissions = vec![Permission::TopicPermission(TopicPermission {
                cache: CacheSelector::AllCaches,
                topic: "my-topic".into(),
                role: TopicRole::PublishSubscribe,
            })];
            let expected_scope = DisposableTokenScope::Permissions(Permissions {
                permissions: expected_permissions,
            });
            let perm_literal = Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: CacheSelector::AllCaches,
                    topic: "my-topic".into(),
                    role: TopicRole::PublishSubscribe,
                })],
            };
            let disp_token_scope: DisposableTokenScope<String> = perm_literal.into();
            assert_eq!(expected_scope, disp_token_scope);
        }

        #[test]
        fn topics_selectors_in_topic_permission() {
            let expected_permissions = vec![Permission::TopicPermission(TopicPermission {
                cache: CacheSelector::AllCaches,
                topic: TopicSelector::TopicName {
                    name: "my-topic".into(),
                },
                role: TopicRole::PublishSubscribe,
            })];
            let expected_scope = DisposableTokenScope::Permissions(Permissions {
                permissions: expected_permissions,
            });

            // Accepts string literal as topic selector
            let perm_literal = Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: CacheSelector::AllCaches,
                    topic: "my-topic".into(),
                    role: TopicRole::PublishSubscribe,
                })],
            };
            let disp_token_scope: DisposableTokenScope<String> = perm_literal.into();
            assert_eq!(expected_scope, disp_token_scope);

            // Accepts TopicName as cache selector
            let perm_literal = Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: CacheSelector::AllCaches,
                    topic: TopicSelector::TopicName {
                        name: "my-topic".into(),
                    },
                    role: TopicRole::PublishSubscribe,
                })],
            };
            let disp_token_scope: DisposableTokenScope<String> = perm_literal.into();
            assert_eq!(expected_scope, disp_token_scope);

            // Accepts AllTopics as topic selector
            let expected_permissions = vec![Permission::TopicPermission(TopicPermission {
                cache: CacheSelector::AllCaches,
                topic: TopicSelector::AllTopics,
                role: TopicRole::PublishSubscribe,
            })];
            let expected_scope = DisposableTokenScope::Permissions(Permissions {
                permissions: expected_permissions,
            });
            let perm_literal = Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: CacheSelector::AllCaches,
                    topic: TopicSelector::AllTopics,
                    role: TopicRole::PublishSubscribe,
                })],
            };
            let disp_token_scope: DisposableTokenScope<String> = perm_literal.into();
            assert_eq!(expected_scope, disp_token_scope);
        }

        #[test]
        fn mixed_cache_and_topic_permissions() {
            let expected_permissions = vec![
                Permission::CachePermission(CachePermission {
                    cache: CacheSelector::CacheName {
                        name: "my-cache".into(),
                    },
                    role: CacheRole::ReadOnly,
                }),
                Permission::TopicPermission(TopicPermission {
                    cache: CacheSelector::CacheName {
                        name: "my-cache".into(),
                    },
                    topic: TopicSelector::AllTopics,
                    role: TopicRole::SubscribeOnly,
                }),
            ];

            let expected_scope = DisposableTokenScope::Permissions(Permissions {
                permissions: expected_permissions,
            });

            let perm_literal = Permissions {
                permissions: vec![
                    Permission::CachePermission(CachePermission {
                        cache: "my-cache".into(),
                        role: CacheRole::ReadOnly,
                    }),
                    Permission::TopicPermission(TopicPermission {
                        cache: "my-cache".into(),
                        topic: TopicSelector::AllTopics,
                        role: TopicRole::SubscribeOnly,
                    }),
                ],
            };
            let disp_token_scope: DisposableTokenScope<String> = perm_literal.into();
            assert_eq!(expected_scope, disp_token_scope);
        }
    }

    mod from_permission_scopes_factory_functions {
        use super::*;

        #[test]
        fn cache_read_write() {
            let expected_permissions = vec![Permission::CachePermission(CachePermission {
                cache: CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                role: CacheRole::ReadWrite,
            })];
            let expected_scope = DisposableTokenScope::Permissions(Permissions {
                permissions: expected_permissions,
            });

            // String literal cache name
            let disp_token_scope: DisposableTokenScope<String> =
                PermissionScopes::cache_read_write("my-cache").into();
            assert_eq!(expected_scope, disp_token_scope);

            // CacheName
            let disp_token_scope: DisposableTokenScope<String> =
                PermissionScopes::cache_read_write(CacheSelector::CacheName {
                    name: "my-cache".into(),
                })
                .into();
            assert_eq!(expected_scope, disp_token_scope);

            // AllCaches
            let expected_permissions = vec![Permission::CachePermission(CachePermission {
                cache: CacheSelector::AllCaches,
                role: CacheRole::ReadWrite,
            })];
            let expected_scope = DisposableTokenScope::Permissions(Permissions {
                permissions: expected_permissions,
            });
            let disp_token_scope: DisposableTokenScope<String> =
                PermissionScopes::cache_read_write(CacheSelector::AllCaches).into();
            assert_eq!(expected_scope, disp_token_scope);
        }

        #[test]
        fn cache_write_only() {
            let expected_permissions = vec![Permission::CachePermission(CachePermission {
                cache: CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                role: CacheRole::WriteOnly,
            })];
            let expected_scope = DisposableTokenScope::Permissions(Permissions {
                permissions: expected_permissions,
            });

            // String literal cache name
            let disp_token_scope: DisposableTokenScope<String> =
                PermissionScopes::cache_write_only("my-cache").into();
            assert_eq!(expected_scope, disp_token_scope);

            // CacheName
            let disp_token_scope: DisposableTokenScope<String> =
                PermissionScopes::cache_write_only(CacheSelector::CacheName {
                    name: "my-cache".into(),
                })
                .into();
            assert_eq!(expected_scope, disp_token_scope);

            // AllCaches
            let expected_permissions = vec![Permission::CachePermission(CachePermission {
                cache: CacheSelector::AllCaches,
                role: CacheRole::WriteOnly,
            })];
            let expected_scope = DisposableTokenScope::Permissions(Permissions {
                permissions: expected_permissions,
            });
            let disp_token_scope: DisposableTokenScope<String> =
                PermissionScopes::cache_write_only(CacheSelector::AllCaches).into();
            assert_eq!(expected_scope, disp_token_scope);
        }

        #[test]
        fn cache_read_only() {
            let expected_permissions = vec![Permission::CachePermission(CachePermission {
                cache: CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                role: CacheRole::ReadOnly,
            })];
            let expected_scope = DisposableTokenScope::Permissions(Permissions {
                permissions: expected_permissions,
            });

            // String literal cache name
            let disp_token_scope: DisposableTokenScope<String> =
                PermissionScopes::cache_read_only("my-cache").into();
            assert_eq!(expected_scope, disp_token_scope);

            // CacheName
            let disp_token_scope: DisposableTokenScope<String> =
                PermissionScopes::cache_read_only(CacheSelector::CacheName {
                    name: "my-cache".into(),
                })
                .into();
            assert_eq!(expected_scope, disp_token_scope);

            // AllCaches
            let expected_permissions = vec![Permission::CachePermission(CachePermission {
                cache: CacheSelector::AllCaches,
                role: CacheRole::ReadOnly,
            })];
            let expected_scope = DisposableTokenScope::Permissions(Permissions {
                permissions: expected_permissions,
            });
            let disp_token_scope: DisposableTokenScope<String> =
                PermissionScopes::cache_read_only(CacheSelector::AllCaches).into();
            assert_eq!(expected_scope, disp_token_scope);
        }

        #[test]
        fn topic_publish_subscribe() {
            let expected_permissions = vec![Permission::TopicPermission(TopicPermission {
                cache: CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                topic: TopicSelector::TopicName {
                    name: "my-topic".into(),
                },
                role: TopicRole::PublishSubscribe,
            })];
            let expected_scope = DisposableTokenScope::Permissions(Permissions {
                permissions: expected_permissions,
            });

            // String literal  topic name
            let disp_token_scope: DisposableTokenScope<String> =
                PermissionScopes::topic_publish_subscribe("my-cache", "my-topic").into();
            assert_eq!(expected_scope, disp_token_scope);

            // TopicName
            let disp_token_scope: DisposableTokenScope<String> =
                PermissionScopes::topic_publish_subscribe(
                    CacheSelector::CacheName {
                        name: "my-cache".into(),
                    },
                    TopicSelector::TopicName {
                        name: "my-topic".into(),
                    },
                )
                .into();
            assert_eq!(expected_scope, disp_token_scope);

            // AllTopics
            let expected_permissions = vec![Permission::TopicPermission(TopicPermission {
                cache: CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                topic: TopicSelector::AllTopics,
                role: TopicRole::PublishSubscribe,
            })];
            let expected_scope = DisposableTokenScope::Permissions(Permissions {
                permissions: expected_permissions,
            });
            let disp_token_scope: DisposableTokenScope<String> =
                PermissionScopes::topic_publish_subscribe("my-cache", TopicSelector::AllTopics)
                    .into();
            assert_eq!(expected_scope, disp_token_scope);
        }

        #[test]
        fn topic_publish_only() {
            let expected_permissions = vec![Permission::TopicPermission(TopicPermission {
                cache: CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                topic: TopicSelector::TopicName {
                    name: "my-topic".into(),
                },
                role: TopicRole::PublishOnly,
            })];
            let expected_scope = DisposableTokenScope::Permissions(Permissions {
                permissions: expected_permissions,
            });

            // String literal topic name
            let disp_token_scope: DisposableTokenScope<String> =
                PermissionScopes::topic_publish_only("my-cache", "my-topic").into();
            assert_eq!(expected_scope, disp_token_scope);

            // TopicName
            let disp_token_scope: DisposableTokenScope<String> =
                PermissionScopes::topic_publish_only(
                    CacheSelector::CacheName {
                        name: "my-cache".into(),
                    },
                    TopicSelector::TopicName {
                        name: "my-topic".into(),
                    },
                )
                .into();
            assert_eq!(expected_scope, disp_token_scope);

            // AllTopics
            let expected_permissions = vec![Permission::TopicPermission(TopicPermission {
                cache: CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                topic: TopicSelector::AllTopics,
                role: TopicRole::PublishOnly,
            })];
            let expected_scope = DisposableTokenScope::Permissions(Permissions {
                permissions: expected_permissions,
            });
            let disp_token_scope: DisposableTokenScope<String> =
                PermissionScopes::topic_publish_only("my-cache", TopicSelector::AllTopics).into();
            assert_eq!(expected_scope, disp_token_scope);
        }

        #[test]
        fn topic_subscribe_only() {
            let expected_permissions = vec![Permission::TopicPermission(TopicPermission {
                cache: CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                topic: TopicSelector::TopicName {
                    name: "my-topic".into(),
                },
                role: TopicRole::SubscribeOnly,
            })];
            let expected_scope = DisposableTokenScope::Permissions(Permissions {
                permissions: expected_permissions,
            });

            // String literal topic name
            let disp_token_scope: DisposableTokenScope<String> =
                PermissionScopes::topic_subscribe_only("my-cache", "my-topic").into();
            assert_eq!(expected_scope, disp_token_scope);

            // TopicName
            let disp_token_scope: DisposableTokenScope<String> =
                PermissionScopes::topic_subscribe_only(
                    CacheSelector::CacheName {
                        name: "my-cache".into(),
                    },
                    TopicSelector::TopicName {
                        name: "my-topic".into(),
                    },
                )
                .into();
            assert_eq!(expected_scope, disp_token_scope);

            // AllTopics
            let expected_permissions = vec![Permission::TopicPermission(TopicPermission {
                cache: CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                topic: TopicSelector::AllTopics,
                role: TopicRole::SubscribeOnly,
            })];
            let expected_scope = DisposableTokenScope::Permissions(Permissions {
                permissions: expected_permissions,
            });
            let disp_token_scope: DisposableTokenScope<String> =
                PermissionScopes::topic_subscribe_only("my-cache", TopicSelector::AllTopics).into();
            assert_eq!(expected_scope, disp_token_scope);
        }
    }

    mod from_disposable_token_scopes_factory_functions {
        use super::*;

        // Note: usage of `<impl IntoBytes>` means we cannot use assert statements as we have above
        // (IntoBytes does not implement Debug, which is needed to use asserts) and it seems we cannot
        // compare `DisposableTokenScope<impl IntoBytes>` (returned by the factory functions) with a
        // manually crafted `DisposableTokenScope<String>`.

        fn assert_eq_prefix_disposable_token_scope(
            scope: DisposableTokenScope<impl IntoBytes>,
            expected_cache_selector: CacheSelector,
            expected_cache_role: CacheRole,
            expected_item: impl IntoBytes,
        ) {
            match scope {
                DisposableTokenScope::DisposableTokenPermissions(perms) => {
                    let mut perms_list = perms.permissions;
                    assert_eq!(perms_list.len(), 1);
                    let cache_perm = perms_list.pop().unwrap();

                    assert_eq!(cache_perm.cache, expected_cache_selector);
                    assert_eq!(cache_perm.role, expected_cache_role);

                    let item_selector = cache_perm.item_selector;
                    match item_selector {
                        CacheItemSelector::CacheItemKeyPrefix(CacheItemKeyPrefix {
                            key_prefix,
                        }) => {
                            assert_eq!(key_prefix.into_bytes(), expected_item.into_bytes());
                        }
                        _ => panic!("Did not receive CacheItemKeyPrefix"),
                    }
                }
                DisposableTokenScope::Permissions(_) => {
                    panic!("Did not recieve DisposableTokenScope::DisposableTokenPermissions")
                }
            }
        }

        fn assert_eq_key_disposable_token_scope(
            scope: DisposableTokenScope<impl IntoBytes>,
            expected_cache_selector: CacheSelector,
            expected_cache_role: CacheRole,
            expected_item: impl IntoBytes,
        ) {
            match scope {
                DisposableTokenScope::DisposableTokenPermissions(perms) => {
                    let mut perms_list = perms.permissions;
                    assert_eq!(perms_list.len(), 1);
                    let cache_perm = perms_list.pop().unwrap();

                    assert_eq!(cache_perm.cache, expected_cache_selector);
                    assert_eq!(cache_perm.role, expected_cache_role);

                    let item_selector = cache_perm.item_selector;
                    match item_selector {
                        CacheItemSelector::CacheItemKey(CacheItemKey { key }) => {
                            assert_eq!(key, expected_item.into_bytes());
                        }
                        _ => panic!("Did not receive CacheItemKey"),
                    }
                }
                DisposableTokenScope::Permissions(_) => {
                    panic!("Did not recieve DisposableTokenScope::DisposableTokenPermissions")
                }
            }
        }

        #[test]
        fn cache_key_prefix_read_write() {
            // String literal cache name
            let disp_token_scope =
                DisposableTokenScopes::cache_key_prefix_read_write("my-cache", "my-key-prefix");
            assert_eq_prefix_disposable_token_scope(
                disp_token_scope,
                CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                CacheRole::ReadWrite,
                "my-key-prefix",
            );

            // CacheName
            let disp_token_scope = DisposableTokenScopes::cache_key_prefix_read_write(
                CacheSelector::CacheName {
                    name: "my-cache".to_string(),
                },
                "my-key-prefix",
            );
            assert_eq_prefix_disposable_token_scope(
                disp_token_scope,
                CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                CacheRole::ReadWrite,
                "my-key-prefix",
            );

            // AllCaches
            let disp_token_scope = DisposableTokenScopes::cache_key_prefix_read_write(
                CacheSelector::AllCaches,
                "my-key-prefix",
            );
            assert_eq_prefix_disposable_token_scope(
                disp_token_scope,
                CacheSelector::AllCaches,
                CacheRole::ReadWrite,
                "my-key-prefix",
            );

            // Byte key prefix
            let disp_token_scope = DisposableTokenScopes::cache_key_prefix_read_write(
                CacheSelector::AllCaches,
                "my-key-prefix".as_bytes(),
            );
            assert_eq_prefix_disposable_token_scope(
                disp_token_scope,
                CacheSelector::AllCaches,
                CacheRole::ReadWrite,
                "my-key-prefix".as_bytes(),
            );
        }

        #[test]
        fn cache_key_prefix_write_only() {
            // String literal cache name
            let disp_token_scope =
                DisposableTokenScopes::cache_key_prefix_write_only("my-cache", "my-key-prefix");
            assert_eq_prefix_disposable_token_scope(
                disp_token_scope,
                CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                CacheRole::WriteOnly,
                "my-key-prefix",
            );

            // CacheName
            let disp_token_scope = DisposableTokenScopes::cache_key_prefix_write_only(
                CacheSelector::CacheName {
                    name: "my-cache".to_string(),
                },
                "my-key-prefix",
            );
            assert_eq_prefix_disposable_token_scope(
                disp_token_scope,
                CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                CacheRole::WriteOnly,
                "my-key-prefix",
            );

            // AllCaches
            let disp_token_scope = DisposableTokenScopes::cache_key_prefix_write_only(
                CacheSelector::AllCaches,
                "my-key-prefix",
            );
            assert_eq_prefix_disposable_token_scope(
                disp_token_scope,
                CacheSelector::AllCaches,
                CacheRole::WriteOnly,
                "my-key-prefix",
            );

            // Byte key prefix
            let disp_token_scope = DisposableTokenScopes::cache_key_prefix_write_only(
                CacheSelector::AllCaches,
                "my-key-prefix".as_bytes(),
            );
            assert_eq_prefix_disposable_token_scope(
                disp_token_scope,
                CacheSelector::AllCaches,
                CacheRole::WriteOnly,
                "my-key-prefix".as_bytes(),
            );
        }

        #[test]
        fn cache_key_prefix_read_only() {
            // String literal cache name
            let disp_token_scope =
                DisposableTokenScopes::cache_key_prefix_read_only("my-cache", "my-key-prefix");
            assert_eq_prefix_disposable_token_scope(
                disp_token_scope,
                CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                CacheRole::ReadOnly,
                "my-key-prefix",
            );

            // CacheName
            let disp_token_scope = DisposableTokenScopes::cache_key_prefix_read_only(
                CacheSelector::CacheName {
                    name: "my-cache".to_string(),
                },
                "my-key-prefix",
            );
            assert_eq_prefix_disposable_token_scope(
                disp_token_scope,
                CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                CacheRole::ReadOnly,
                "my-key-prefix",
            );

            // AllCaches
            let disp_token_scope = DisposableTokenScopes::cache_key_prefix_read_only(
                CacheSelector::AllCaches,
                "my-key-prefix",
            );
            assert_eq_prefix_disposable_token_scope(
                disp_token_scope,
                CacheSelector::AllCaches,
                CacheRole::ReadOnly,
                "my-key-prefix",
            );

            // Byte key prefix
            let disp_token_scope = DisposableTokenScopes::cache_key_prefix_read_only(
                CacheSelector::AllCaches,
                "my-key-prefix".as_bytes(),
            );
            assert_eq_prefix_disposable_token_scope(
                disp_token_scope,
                CacheSelector::AllCaches,
                CacheRole::ReadOnly,
                "my-key-prefix".as_bytes(),
            );
        }

        #[test]
        fn cache_key_read_write() {
            // String literal cache name
            let disp_token_scope =
                DisposableTokenScopes::cache_key_read_write("my-cache", "my-key");
            assert_eq_key_disposable_token_scope(
                disp_token_scope,
                CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                CacheRole::ReadWrite,
                "my-key",
            );

            // CacheName
            let disp_token_scope = DisposableTokenScopes::cache_key_read_write(
                CacheSelector::CacheName {
                    name: "my-cache".to_string(),
                },
                "my-key",
            );
            assert_eq_key_disposable_token_scope(
                disp_token_scope,
                CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                CacheRole::ReadWrite,
                "my-key",
            );
            // AllCaches
            let disp_token_scope =
                DisposableTokenScopes::cache_key_read_write(CacheSelector::AllCaches, "my-key");
            assert_eq_key_disposable_token_scope(
                disp_token_scope,
                CacheSelector::AllCaches,
                CacheRole::ReadWrite,
                "my-key",
            );

            // Byte key
            let disp_token_scope = DisposableTokenScopes::cache_key_read_write(
                CacheSelector::AllCaches,
                "my-key".as_bytes(),
            );
            assert_eq_key_disposable_token_scope(
                disp_token_scope,
                CacheSelector::AllCaches,
                CacheRole::ReadWrite,
                "my-key".as_bytes(),
            );
        }

        #[test]
        fn cache_key_write_only() {
            // String literal cache name
            let disp_token_scope =
                DisposableTokenScopes::cache_key_write_only("my-cache", "my-key");
            assert_eq_key_disposable_token_scope(
                disp_token_scope,
                CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                CacheRole::WriteOnly,
                "my-key",
            );

            // CacheName
            let disp_token_scope = DisposableTokenScopes::cache_key_write_only(
                CacheSelector::CacheName {
                    name: "my-cache".to_string(),
                },
                "my-key",
            );
            assert_eq_key_disposable_token_scope(
                disp_token_scope,
                CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                CacheRole::WriteOnly,
                "my-key",
            );

            // AllCaches
            let disp_token_scope =
                DisposableTokenScopes::cache_key_write_only(CacheSelector::AllCaches, "my-key");
            assert_eq_key_disposable_token_scope(
                disp_token_scope,
                CacheSelector::AllCaches,
                CacheRole::WriteOnly,
                "my-key",
            );

            // Byte key
            let disp_token_scope = DisposableTokenScopes::cache_key_write_only(
                CacheSelector::AllCaches,
                "my-key".as_bytes(),
            );
            assert_eq_key_disposable_token_scope(
                disp_token_scope,
                CacheSelector::AllCaches,
                CacheRole::WriteOnly,
                "my-key".as_bytes(),
            );
        }

        #[test]
        fn cache_key_read_only() {
            // String literal cache name
            let disp_token_scope = DisposableTokenScopes::cache_key_read_only("my-cache", "my-key");
            assert_eq_key_disposable_token_scope(
                disp_token_scope,
                CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                CacheRole::ReadOnly,
                "my-key",
            );

            // CacheName
            let disp_token_scope = DisposableTokenScopes::cache_key_read_only(
                CacheSelector::CacheName {
                    name: "my-cache".to_string(),
                },
                "my-key",
            );
            assert_eq_key_disposable_token_scope(
                disp_token_scope,
                CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                CacheRole::ReadOnly,
                "my-key",
            );

            // AllCaches
            let disp_token_scope =
                DisposableTokenScopes::cache_key_read_only(CacheSelector::AllCaches, "my-key");
            assert_eq_key_disposable_token_scope(
                disp_token_scope,
                CacheSelector::AllCaches,
                CacheRole::ReadOnly,
                "my-key",
            );

            // Byte key
            let disp_token_scope = DisposableTokenScopes::cache_key_read_only(
                CacheSelector::AllCaches,
                "my-key".as_bytes(),
            );
            assert_eq_key_disposable_token_scope(
                disp_token_scope,
                CacheSelector::AllCaches,
                CacheRole::ReadOnly,
                "my-key".as_bytes(),
            );
        }
    }
}
