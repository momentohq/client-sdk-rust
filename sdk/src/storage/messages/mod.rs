/// Control messages for storage
pub mod control;

/// Data messages for storage
pub mod data;
mod momento_storage_request;
pub use momento_storage_request::MomentoStorageRequest;
mod storage_value;
pub use storage_value::StorageValue;
