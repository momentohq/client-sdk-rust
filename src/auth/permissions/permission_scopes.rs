use super::permission_scope::{
    CachePermission, CacheRole, CacheSelector, Permission, PermissionScope, Permissions,
    TopicPermission, TopicRole, TopicSelector,
};

pub struct PermissionScopes {}

impl PermissionScopes {
    pub fn cache_read_write(cache_selector: impl Into<CacheSelector>) -> PermissionScope {
        PermissionScope::Permissions(Permissions {
            permissions: vec![Permission::CachePermission(CachePermission {
                role: CacheRole::ReadWrite,
                cache: cache_selector.into(),
            })],
        })
    }

    pub fn cache_read_only(cache_selector: impl Into<CacheSelector>) -> PermissionScope {
        PermissionScope::Permissions(Permissions {
            permissions: vec![Permission::CachePermission(CachePermission {
                role: CacheRole::ReadOnly,
                cache: cache_selector.into(),
            })],
        })
    }

    pub fn cache_write_only(cache_selector: impl Into<CacheSelector>) -> PermissionScope {
        PermissionScope::Permissions(Permissions {
            permissions: vec![Permission::CachePermission(CachePermission {
                role: CacheRole::WriteOnly,
                cache: cache_selector.into(),
            })],
        })
    }

    pub fn topic_publish_subscribe(
        cache_selector: impl Into<CacheSelector>,
        topic_selector: impl Into<TopicSelector>,
    ) -> PermissionScope {
        PermissionScope::Permissions(Permissions {
            permissions: vec![Permission::TopicPermission(TopicPermission {
                role: TopicRole::PublishSubscribe,
                cache: cache_selector.into(),
                topic: topic_selector.into(),
            })],
        })
    }

    pub fn topic_subscribe_only(
        cache_selector: impl Into<CacheSelector>,
        topic_selector: impl Into<TopicSelector>,
    ) -> PermissionScope {
        PermissionScope::Permissions(Permissions {
            permissions: vec![Permission::TopicPermission(TopicPermission {
                role: TopicRole::SubscribeOnly,
                cache: cache_selector.into(),
                topic: topic_selector.into(),
            })],
        })
    }

    pub fn topic_publish_only(
        cache_selector: impl Into<CacheSelector>,
        topic_selector: impl Into<TopicSelector>,
    ) -> PermissionScope {
        PermissionScope::Permissions(Permissions {
            permissions: vec![Permission::TopicPermission(TopicPermission {
                role: TopicRole::PublishOnly,
                cache: cache_selector.into(),
                topic: topic_selector.into(),
            })],
        })
    }
}
