/// A component of a [CachePermission].
/// Type of access granted by the permission.
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
pub struct CachePermission {
    /// The type of access granted by the permission.
    pub role: CacheRole,
    /// The cache(s) to which the permission applies.
    pub cache: CacheSelector,
}

/// A component of a [TopicPermission].
/// Type of access granted by the permission.
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
pub struct TopicPermission {
    /// The type of access granted by the permission.
    pub role: TopicRole,
    /// The cache(s) to which the permission applies.
    pub cache: CacheSelector,
    /// The topic(s) to which the permission applies.
    pub topic: TopicSelector,
}

/// A component of a [PermissionScope].
pub enum Permission {
    /// Defines the permissions for a cache.
    CachePermission(CachePermission),
    /// Defines the permissions for a topic in a cache.
    TopicPermission(TopicPermission),
}

/// Permissions object contains the set of permissions to be granted to a new API key.
pub struct Permissions {
    /// The set of permissions to be granted to a new API key.
    pub permissions: Vec<Permission>,
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
pub enum PermissionScope {
    /// Set of permissions to be granted to a new API key
    Permissions(Permissions),
    /// PredefinedScope for internal use
    PredefinedScope,
}
