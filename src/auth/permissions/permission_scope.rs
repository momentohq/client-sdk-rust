pub enum CacheRole {
    ReadWrite,
    ReadOnly,
    WriteOnly,
}

pub enum CacheSelector {
    AllCaches,
    CacheName { name: String },
}

impl From<String> for CacheSelector {
    fn from(name: String) -> Self {
        CacheSelector::CacheName { name }
    }
}

impl From<&str> for CacheSelector {
    fn from(name: &str) -> Self {
        CacheSelector::CacheName {
            name: name.to_string(),
        }
    }
}

pub struct CachePermission {
    pub role: CacheRole,
    pub cache: CacheSelector,
}

pub enum TopicRole {
    PublishSubscribe,
    SubscribeOnly,
    PublishOnly,
}

pub enum TopicSelector {
    AllTopics,
    TopicName { name: String },
}

impl From<String> for TopicSelector {
    fn from(name: String) -> Self {
        TopicSelector::TopicName { name }
    }
}

impl From<&str> for TopicSelector {
    fn from(name: &str) -> Self {
        TopicSelector::TopicName {
            name: name.to_string(),
        }
    }
}

pub struct TopicPermission {
    pub role: TopicRole,
    pub cache: CacheSelector,
    pub topic: TopicSelector,
}

pub enum Permission {
    CachePermission(CachePermission),
    TopicPermission(TopicPermission),
}

pub struct Permissions {
    pub permissions: Vec<Permission>,
}

impl Permissions {
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

pub enum PermissionScope {
    Permissions(Permissions),
    PredefinedScope,
}
