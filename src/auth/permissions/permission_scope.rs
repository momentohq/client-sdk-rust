use derive_more::Display;

/// A component of a [CachePermission].
/// Type of access granted by the permission.
#[derive(Debug, Display, Clone, PartialEq)]
pub enum CacheRole {
    /// Allows read-write access to a cache
    ReadWrite,
    /// Allows read-only access to a cache
    ReadOnly,
    /// Allows write-only access to a cache
    WriteOnly,
}

/// A component of a [CachePermission].
/// A permission can be restricted to a specific cache or to all caches.
#[derive(Debug, Display, Clone, PartialEq)]
pub enum CacheSelector {
    /// Apply permission to all caches
    AllCaches,
    /// Apply permission to a specific cache
    CacheName {
        /// The name of the cache
        name: String,
    },
}

/// A String can be passed in as a CacheSelector::CacheName
impl From<String> for CacheSelector {
    fn from(name: String) -> Self {
        CacheSelector::CacheName { name }
    }
}

/// A string literal can be passed in as a CacheSelector::CacheName
impl From<&str> for CacheSelector {
    fn from(name: &str) -> Self {
        CacheSelector::CacheName {
            name: name.to_string(),
        }
    }
}

/// Defines access permissions for a cache.
#[derive(Debug, Clone, PartialEq)]
pub struct CachePermission {
    /// The type of access granted by the permission.
    pub role: CacheRole,
    /// The cache(s) to which the permission applies.
    pub cache: CacheSelector,
}

impl std::fmt::Display for CachePermission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CachePermission {{ role: {}, cache: {} }}",
            self.role, self.cache
        )
    }
}

/// A component of a [TopicPermission].
/// Type of access granted by the permission.
#[derive(Debug, Display, Clone, PartialEq)]
pub enum TopicRole {
    /// Allows both publishing and subscribing to a topic
    PublishSubscribe,
    /// Allows only subscribing to a topic
    SubscribeOnly,
    /// Allows only publishing to a topic
    PublishOnly,
}

/// A component of a [TopicPermission].
/// A permission can be restricted to a specific topic or to all topics in a cache.
#[derive(Debug, Display, Clone, PartialEq)]
pub enum TopicSelector {
    /// Apply permission to all topics
    AllTopics,
    /// Apply permission to a specific topic
    TopicName {
        /// The name of the topic
        name: String,
    },
}

// A String can be passed in as a TopicSelector::TopicName
impl From<String> for TopicSelector {
    fn from(name: String) -> Self {
        TopicSelector::TopicName { name }
    }
}

/// A string literal can be passed in as a TopicSelector::TopicName
impl From<&str> for TopicSelector {
    fn from(name: &str) -> Self {
        TopicSelector::TopicName {
            name: name.to_string(),
        }
    }
}

/// Defines access permissions for a topic in a cache.
#[derive(Debug, Clone, PartialEq)]
pub struct TopicPermission {
    /// The type of access granted by the permission.
    pub role: TopicRole,
    /// The cache(s) to which the permission applies.
    pub cache: CacheSelector,
    /// The topic(s) to which the permission applies.
    pub topic: TopicSelector,
}

impl std::fmt::Display for TopicPermission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TopicPermission {{ role: {}, cache: {}, topic: {} }}",
            self.role, self.cache, self.topic
        )
    }
}

/// A component of a [PermissionScope].
#[derive(Debug, Display, Clone, PartialEq)]
pub enum Permission {
    /// Defines the permissions for a cache.
    CachePermission(CachePermission),
    /// Defines the permissions for a topic in a cache.
    TopicPermission(TopicPermission),
}

/// Permissions object contains the set of permissions to be granted to a new API key.
#[derive(Debug, Clone, PartialEq)]
pub struct Permissions {
    /// The set of permissions to be granted to a new API key.
    pub permissions: Vec<Permission>,
}

impl std::fmt::Display for Permissions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Permissions {{ permissions: {:?} }}", self.permissions)
    }
}

impl Permissions {
    /// Create a permission scope allowing read-write access to all caches and topics.
    pub fn all_data_read_write() -> Permissions {
        Permissions {
            permissions: vec![
                Permission::CachePermission(CachePermission {
                    role: CacheRole::ReadWrite,
                    cache: CacheSelector::AllCaches,
                }),
                Permission::TopicPermission(TopicPermission {
                    role: TopicRole::PublishSubscribe,
                    cache: CacheSelector::AllCaches,
                    topic: TopicSelector::AllTopics,
                }),
            ],
        }
    }
}

/// The permission scope for creating a new API key.
#[derive(Debug, Display, Clone, PartialEq)]
pub enum PermissionScope {
    /// Set of permissions to be granted to a new API key
    Permissions(Permissions),
}

impl From<Permissions> for PermissionScope {
    fn from(permissions: Permissions) -> Self {
        PermissionScope::Permissions(permissions)
    }
}

#[cfg(test)]
mod tests {
    use crate::auth::{
        CachePermission, CacheRole, CacheSelector, Permission, PermissionScope, PermissionScopes,
        Permissions, TopicPermission, TopicRole, TopicSelector,
    };

    #[test]
    fn should_support_assignment_from_all_data_read_write() {
        let scope: PermissionScope = Permissions::all_data_read_write().into();
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
            PermissionScope::Permissions(Permissions {
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
            let expected_scope = PermissionScope::Permissions(Permissions {
                permissions: expected_permissions,
            });

            // Accepts string literal as cache selector
            let perm_literal = Permissions {
                permissions: vec![Permission::CachePermission(CachePermission {
                    cache: "my-cache".into(),
                    role: CacheRole::ReadWrite,
                })],
            };
            let perm_scope: PermissionScope = perm_literal.into();
            assert_eq!(expected_scope, perm_scope);

            // Accepts CacheName as cache selector
            let perm_literal = Permissions {
                permissions: vec![Permission::CachePermission(CachePermission {
                    cache: CacheSelector::CacheName {
                        name: "my-cache".into(),
                    },
                    role: CacheRole::ReadWrite,
                })],
            };
            let perm_scope: PermissionScope = perm_literal.into();
            assert_eq!(expected_scope, perm_scope);

            // Accepts AllCaches as cache selector
            let expected_permissions = vec![Permission::CachePermission(CachePermission {
                cache: CacheSelector::AllCaches,
                role: CacheRole::ReadWrite,
            })];
            let expected_scope = PermissionScope::Permissions(Permissions {
                permissions: expected_permissions,
            });
            let perm_literal = Permissions {
                permissions: vec![Permission::CachePermission(CachePermission {
                    cache: CacheSelector::AllCaches,
                    role: CacheRole::ReadWrite,
                })],
            };
            let perm_scope: PermissionScope = perm_literal.into();
            assert_eq!(expected_scope, perm_scope);
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
            let expected_scope = PermissionScope::Permissions(Permissions {
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
            let perm_scope: PermissionScope = perm_literal.into();
            assert_eq!(expected_scope, perm_scope);

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
            let perm_scope: PermissionScope = perm_literal.into();
            assert_eq!(expected_scope, perm_scope);

            // Accepts AllCaches as cache selector
            let expected_permissions = vec![Permission::TopicPermission(TopicPermission {
                cache: CacheSelector::AllCaches,
                topic: "my-topic".into(),
                role: TopicRole::PublishSubscribe,
            })];
            let expected_scope = PermissionScope::Permissions(Permissions {
                permissions: expected_permissions,
            });
            let perm_literal = Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: CacheSelector::AllCaches,
                    topic: "my-topic".into(),
                    role: TopicRole::PublishSubscribe,
                })],
            };
            let perm_scope: PermissionScope = perm_literal.into();
            assert_eq!(expected_scope, perm_scope);
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
            let expected_scope = PermissionScope::Permissions(Permissions {
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
            let perm_scope: PermissionScope = perm_literal.into();
            assert_eq!(expected_scope, perm_scope);

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
            let perm_scope: PermissionScope = perm_literal.into();
            assert_eq!(expected_scope, perm_scope);

            // Accepts AllTopics as topic selector
            let expected_permissions = vec![Permission::TopicPermission(TopicPermission {
                cache: CacheSelector::AllCaches,
                topic: TopicSelector::AllTopics,
                role: TopicRole::PublishSubscribe,
            })];
            let expected_scope = PermissionScope::Permissions(Permissions {
                permissions: expected_permissions,
            });
            let perm_literal = Permissions {
                permissions: vec![Permission::TopicPermission(TopicPermission {
                    cache: CacheSelector::AllCaches,
                    topic: TopicSelector::AllTopics,
                    role: TopicRole::PublishSubscribe,
                })],
            };
            let perm_scope: PermissionScope = perm_literal.into();
            assert_eq!(expected_scope, perm_scope);
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

            let expected_scope = PermissionScope::Permissions(Permissions {
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
            let perm_scope: PermissionScope = perm_literal.into();
            assert_eq!(expected_scope, perm_scope);
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
            let expected_scope = PermissionScope::Permissions(Permissions {
                permissions: expected_permissions,
            });

            // String literal cache name
            let perm_scope: PermissionScope = PermissionScopes::cache_read_write("my-cache");
            assert_eq!(expected_scope, perm_scope);

            // CacheName
            let perm_scope: PermissionScope =
                PermissionScopes::cache_read_write(CacheSelector::CacheName {
                    name: "my-cache".into(),
                });
            assert_eq!(expected_scope, perm_scope);

            // AllCaches
            let expected_permissions = vec![Permission::CachePermission(CachePermission {
                cache: CacheSelector::AllCaches,
                role: CacheRole::ReadWrite,
            })];
            let expected_scope = PermissionScope::Permissions(Permissions {
                permissions: expected_permissions,
            });
            let perm_scope: PermissionScope =
                PermissionScopes::cache_read_write(CacheSelector::AllCaches);
            assert_eq!(expected_scope, perm_scope);
        }

        #[test]
        fn cache_write_only() {
            let expected_permissions = vec![Permission::CachePermission(CachePermission {
                cache: CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                role: CacheRole::WriteOnly,
            })];
            let expected_scope = PermissionScope::Permissions(Permissions {
                permissions: expected_permissions,
            });

            // String literal cache name
            let perm_scope: PermissionScope = PermissionScopes::cache_write_only("my-cache");
            assert_eq!(expected_scope, perm_scope);

            // CacheName
            let perm_scope: PermissionScope =
                PermissionScopes::cache_write_only(CacheSelector::CacheName {
                    name: "my-cache".into(),
                });
            assert_eq!(expected_scope, perm_scope);

            // AllCaches
            let expected_permissions = vec![Permission::CachePermission(CachePermission {
                cache: CacheSelector::AllCaches,
                role: CacheRole::WriteOnly,
            })];
            let expected_scope = PermissionScope::Permissions(Permissions {
                permissions: expected_permissions,
            });
            let perm_scope: PermissionScope =
                PermissionScopes::cache_write_only(CacheSelector::AllCaches);
            assert_eq!(expected_scope, perm_scope);
        }

        #[test]
        fn cache_read_only() {
            let expected_permissions = vec![Permission::CachePermission(CachePermission {
                cache: CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                role: CacheRole::ReadOnly,
            })];
            let expected_scope = PermissionScope::Permissions(Permissions {
                permissions: expected_permissions,
            });

            // String literal cache name
            let perm_scope: PermissionScope = PermissionScopes::cache_read_only("my-cache");
            assert_eq!(expected_scope, perm_scope);

            // CacheName
            let perm_scope: PermissionScope =
                PermissionScopes::cache_read_only(CacheSelector::CacheName {
                    name: "my-cache".into(),
                });
            assert_eq!(expected_scope, perm_scope);

            // AllCaches
            let expected_permissions = vec![Permission::CachePermission(CachePermission {
                cache: CacheSelector::AllCaches,
                role: CacheRole::ReadOnly,
            })];
            let expected_scope = PermissionScope::Permissions(Permissions {
                permissions: expected_permissions,
            });
            let perm_scope: PermissionScope =
                PermissionScopes::cache_read_only(CacheSelector::AllCaches);
            assert_eq!(expected_scope, perm_scope);
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
            let expected_scope = PermissionScope::Permissions(Permissions {
                permissions: expected_permissions,
            });

            // String literal  topic name
            let perm_scope: PermissionScope =
                PermissionScopes::topic_publish_subscribe("my-cache", "my-topic");
            assert_eq!(expected_scope, perm_scope);

            // TopicName
            let perm_scope: PermissionScope = PermissionScopes::topic_publish_subscribe(
                CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                TopicSelector::TopicName {
                    name: "my-topic".into(),
                },
            );
            assert_eq!(expected_scope, perm_scope);

            // AllTopics
            let expected_permissions = vec![Permission::TopicPermission(TopicPermission {
                cache: CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                topic: TopicSelector::AllTopics,
                role: TopicRole::PublishSubscribe,
            })];
            let expected_scope = PermissionScope::Permissions(Permissions {
                permissions: expected_permissions,
            });
            let perm_scope: PermissionScope =
                PermissionScopes::topic_publish_subscribe("my-cache", TopicSelector::AllTopics);
            assert_eq!(expected_scope, perm_scope);
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
            let expected_scope = PermissionScope::Permissions(Permissions {
                permissions: expected_permissions,
            });

            // String literal topic name
            let perm_scope: PermissionScope =
                PermissionScopes::topic_publish_only("my-cache", "my-topic");
            assert_eq!(expected_scope, perm_scope);

            // TopicName
            let perm_scope: PermissionScope = PermissionScopes::topic_publish_only(
                CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                TopicSelector::TopicName {
                    name: "my-topic".into(),
                },
            );
            assert_eq!(expected_scope, perm_scope);

            // AllTopics
            let expected_permissions = vec![Permission::TopicPermission(TopicPermission {
                cache: CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                topic: TopicSelector::AllTopics,
                role: TopicRole::PublishOnly,
            })];
            let expected_scope = PermissionScope::Permissions(Permissions {
                permissions: expected_permissions,
            });
            let perm_scope: PermissionScope =
                PermissionScopes::topic_publish_only("my-cache", TopicSelector::AllTopics);
            assert_eq!(expected_scope, perm_scope);
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
            let expected_scope = PermissionScope::Permissions(Permissions {
                permissions: expected_permissions,
            });

            // String literal topic name
            let perm_scope: PermissionScope =
                PermissionScopes::topic_subscribe_only("my-cache", "my-topic");
            assert_eq!(expected_scope, perm_scope);

            // TopicName
            let perm_scope: PermissionScope = PermissionScopes::topic_subscribe_only(
                CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                TopicSelector::TopicName {
                    name: "my-topic".into(),
                },
            );
            assert_eq!(expected_scope, perm_scope);

            // AllTopics
            let expected_permissions = vec![Permission::TopicPermission(TopicPermission {
                cache: CacheSelector::CacheName {
                    name: "my-cache".into(),
                },
                topic: TopicSelector::AllTopics,
                role: TopicRole::SubscribeOnly,
            })];
            let expected_scope = PermissionScope::Permissions(Permissions {
                permissions: expected_permissions,
            });
            let perm_scope: PermissionScope =
                PermissionScopes::topic_subscribe_only("my-cache", TopicSelector::AllTopics);
            assert_eq!(expected_scope, perm_scope);
        }
    }
}
