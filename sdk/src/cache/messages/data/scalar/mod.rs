/// Contains the request and response types for decreasing the time-to-live of an item in a cache.
pub mod decrease_ttl;
/// Contains the request and response types for deleting an item from a cache.
pub mod delete;
/// Contains the request and response types for getting an item from a cache.
pub mod get;
/// Contains the request and response types for increasing the time-to-live of an item in a cache.
pub mod increase_ttl;
/// Contains the request and response types for incrementing the value of an item in a cache.
pub mod increment;
/// Contains the request and response types for getting the time-to-live of an item in a cache.
pub mod item_get_ttl;
/// Contains the request and response types for getting the type of an item in a cache.
pub mod item_get_type;
/// Contains the request and response types for checking if an item exists in a cache.
pub mod key_exists;
/// Contains the request and response types for checking if multiple items exist in a cache.
pub mod keys_exist;
/// Contains the request and response types for setting an item in a cache.
pub mod set;
/// Contains the request and response types for setting an item in a cache if it is absent from the cache.
pub mod set_if_absent;
/// Contains the request and response types for setting an item in a cache if it is absent or equal to a given value.
pub mod set_if_absent_or_equal;
/// Contains the request and response types for setting an item in a cache if it is equal to a given value.
pub mod set_if_equal;
/// Contains the request and response types for setting an item in a cache if it is not equal to a given value.
pub mod set_if_not_equal;
/// Contains the request and response types for setting an item in a cache if it is present in the cache.
pub mod set_if_present;
/// Contains the request and response types for setting an item in a cache if it is present and not equal to a given value.
pub mod set_if_present_and_not_equal;
/// Contains the request and response types for overwriting the time-to-live of an item in a cache.
pub mod update_ttl;
