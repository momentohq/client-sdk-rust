/// Control plane messages for storage
pub mod control;

/// Data plane messages for storage
pub mod data;
mod momento_storage_request;
pub use momento_storage_request::MomentoStorageRequest;
mod storage_value;
pub use storage_value::StorageValue;
