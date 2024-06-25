/// Contains the request and response types for storage operations.
pub mod messages;

pub use messages::MomentoStorageRequest;
pub use messages::StorageValue;

pub use messages::control::create_store::{CreateStoreRequest, CreateStoreResponse};
pub use messages::control::delete_store::{DeleteStoreRequest, DeleteStoreResponse};
pub use messages::control::list_stores::{ListStoresRequest, ListStoresResponse, StoreInfo};

pub use messages::data::delete::{DeleteRequest, DeleteResponse};
pub use messages::data::get::{GetRequest, GetResponse};
pub use messages::data::put::{PutRequest, PutResponse};

// Similar re-exporting with config::configuration and config::configurations
// so import paths can be simplified to "momento::storage::Configuration" and
// "use momento::storage::configurations::laptop"
mod config;

pub use config::configuration::Configuration;
pub use config::configurations;

mod preview_storage_client;
mod preview_storage_client_builder;

pub use preview_storage_client::PreviewStorageClient;
