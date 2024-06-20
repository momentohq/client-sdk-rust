use momento_protos::control_client::scs_control_client::ScsControlClient;
use momento_protos::store::store_client::StoreClient;
use tonic::codegen::InterceptedService;
use tonic::transport::Channel;

use crate::grpc::header_interceptor::HeaderInterceptor;
use crate::storage::messages::control::create_store::{CreateStoreRequest, CreateStoreResponse};
use crate::storage::messages::control::delete_store::{DeleteStoreRequest, DeleteStoreResponse};
use crate::storage::messages::control::list_stores::{ListStoresRequest, ListStoresResponse};
use crate::storage::messages::data::get::{GetRequest, GetResponse};
use crate::storage::messages::momento_store_request::MomentoStorageRequest;
use crate::storage::messages::store_value::StoreValue;
use crate::storage::preview_storage_client_builder::{
    NeedsConfiguration, PreviewStorageClientBuilder,
};
use crate::storage::{Configuration, DeleteRequest, DeleteResponse, PutRequest, PutResponse};
use crate::MomentoResult;

/// Preview client to work with Momento Storage.
///
/// These preview APIs are not final and are subject to change.
///
/// # Example
/// To instantiate a [PreviewStorageClient], you need to provide a configuration and a [CredentialProvider](crate::CredentialProvider).
/// Prebuilt configurations tuned for different environments are available in the [storage::configurations](crate::storage::configurations) module.
///
/// ```
/// # fn main() -> anyhow::Result<()> {
/// # tokio_test::block_on(async {
/// use momento::{CredentialProvider, PreviewStorageClient, storage::configurations};
///
/// let storage_client = match PreviewStorageClient::builder()
///     .configuration(configurations::Laptop::latest())
///     .credential_provider(
///         CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string())
///             .expect("API key should be valid"),
///     )
///     .build()
/// {
///     Ok(client) => client,
///     Err(err) => panic!("{err}"),
/// };
/// # Ok(())
/// # })
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct PreviewStorageClient {
    pub(crate) storage_client: StoreClient<InterceptedService<Channel, HeaderInterceptor>>,
    pub(crate) control_client: ScsControlClient<InterceptedService<Channel, HeaderInterceptor>>,
    pub(crate) configuration: Configuration,
}

impl PreviewStorageClient {
    /// Constructs a PreviewStorageClient to use Momento Store.
    ///
    /// # Arguments
    /// - `configuration` - Prebuilt configurations tuned for different environments are available in the [storage::configurations](crate::storage::configurations) module.
    /// - `credential_provider` - A [CredentialProvider](crate::CredentialProvider) to use for authenticating with Momento.
    ///
    /// # Example
    ///
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # tokio_test::block_on(async {
    /// use momento::{storage::configurations, CredentialProvider, PreviewStorageClient};
    ///
    /// let storage_client = match PreviewStorageClient::builder()
    ///     .configuration(configurations::Laptop::latest())
    ///     .credential_provider(
    ///         CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string())
    ///             .expect("API key should be valid"),
    ///     )
    ///     .build()
    /// {
    ///     Ok(client) => client,
    ///     Err(err) => panic!("{err}"),
    /// };
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    pub fn builder() -> PreviewStorageClientBuilder<NeedsConfiguration> {
        PreviewStorageClientBuilder(NeedsConfiguration)
    }

    /// Creates a store with the given name.
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of the store to be created.
    ///
    /// # Examples
    /// Assumes that a PreviewStorageClient named `storage_client` has been created and is available.
    /// ```no_run
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_storage_client;
    /// # tokio_test::block_on(async {
    /// use momento::storage::CreateStoreResponse;
    /// # let (storage_client, store_name) = create_doctest_storage_client();
    ///
    /// match storage_client.create_store(&store_name).await? {
    ///     CreateStoreResponse::Created => println!("Store {} created", &store_name),
    ///     CreateStoreResponse::AlreadyExists => println!("Store {} already exists", &store_name),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](PreviewStorageClient::send_request) method to create a store using a [CreateStoreRequest].
    pub async fn create_store(
        &self,
        store_name: impl Into<String>,
    ) -> MomentoResult<CreateStoreResponse> {
        let request = CreateStoreRequest::new(store_name);
        request.send(self).await
    }

    /// Deletes the store with the given name.
    ///
    /// # Arguments
    ///
    /// * `store_name` - The name of the store to be deleted.
    ///
    /// # Examples
    /// Assumes that a PreviewStorageClient named `storage_client` has been created and is available.
    /// ```no_run
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_storage_client;
    /// # tokio_test::block_on(async {
    /// use momento::storage::DeleteStoreResponse;
    /// use momento::MomentoErrorCode;
    /// # let (storage_client, store_name) = create_doctest_storage_client();
    ///
    /// match storage_client.delete_store(&store_name).await {
    ///     Ok(_) => println!("Store deleted: {}", &store_name),
    ///     Err(e) => if let MomentoErrorCode::StoreNotFoundError = e.error_code {
    ///         println!("Store not found: {}", &store_name);
    ///     } else {
    ///         eprintln!("Error deleting store {}: {}", &store_name, e);
    ///     }
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](PreviewStorageClient::send_request) method to delete a store using a [DeleteStoreRequest].
    pub async fn delete_store(
        &self,
        store_name: impl Into<String>,
    ) -> MomentoResult<DeleteStoreResponse> {
        let request = DeleteStoreRequest::new(store_name);
        request.send(self).await
    }

    /// Lists all stores in your account.
    ///
    /// # Examples
    /// Assumes that a PreviewStorageClient named `storage_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_storage_client;
    /// # tokio_test::block_on(async {
    /// use momento::storage::ListStoresResponse;
    /// # let (storage_client, store_name) = create_doctest_storage_client();
    ///
    /// match storage_client.list_stores().await {
    ///     Ok(response) => println!("Stores: {:#?}", response.stores),
    ///     Err(e) => eprintln!("Error listing stores: {}", e),
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](PreviewStorageClient::send_request) method to list stores using a [ListStoresRequest].
    pub async fn list_stores(&self) -> MomentoResult<ListStoresResponse> {
        let request = ListStoresRequest {};
        request.send(self).await
    }

    /// Puts an item in a Momento Store
    ///
    /// # Arguments
    ///
    /// * `store_name` - name of the store
    /// * `key` - key of the item whose value we are putting
    /// * `value` - data to stored
    ///
    /// # Examples
    /// Assumes that a PreviewStorageClient named `storage_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_storage_client;
    /// # tokio_test::block_on(async {
    /// # let (storage_client, store_name) = create_doctest_storage_client();
    /// use momento::MomentoErrorCode;
    ///
    /// match storage_client.put(&store_name, "k1", "v1").await {
    ///     Ok(_) => println!("PutResponse successful"),
    ///     Err(e) => if let MomentoErrorCode::StoreNotFoundError = e.error_code {
    ///         println!("Store not found: {}", &store_name);
    ///     } else {
    ///         eprintln!("Error putting value in store {}: {}", &store_name, e);
    ///     }
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](PreviewStorageClient::send_request) method to get an item using a [PutRequest].
    pub async fn put(
        &self,
        store_name: impl Into<String>,
        key: impl Into<String>,
        value: impl Into<StoreValue>,
    ) -> MomentoResult<PutResponse> {
        let request = PutRequest::new(store_name, key, value);
        request.send(self).await
    }

    /// Gets an item from a Momento Store
    ///
    /// # Arguments
    ///
    /// * `store_name` - name of the store
    /// * `key` - key of entry within the store.
    ///
    /// # Examples
    /// Assumes that a PreviewStorageClient named `storage_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento::storage::GetResponse;
    /// use momento_test_util::create_doctest_storage_client;
    /// # tokio_test::block_on(async {
    /// # let (storage_client, store_name) = create_doctest_storage_client();
    /// use std::convert::TryInto;
    /// # storage_client.put(&store_name, "key", "value").await?;
    ///
    /// let item: String = storage_client.get(&store_name, "key").await?.try_into().expect("I stored a string!");
    /// # assert_eq!(item, "value");
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](PreviewStorageClient::send_request) method to get an item using a [GetRequest].
    ///
    /// For more examples of handling the response, see [GetResponse].
    pub async fn get(
        &self,
        store_name: impl Into<String>,
        key: impl Into<String>,
    ) -> MomentoResult<GetResponse> {
        let request = GetRequest::new(store_name.into(), key.into());
        request.send(self).await
    }

    /// Deletes an item in a Momento Store
    ///
    /// # Arguments
    ///
    /// * `store_name` - name of the store
    /// * `key` - key of the item to delete
    ///
    /// # Examples
    /// Assumes that a PreviewStorageClient named `storage_client` has been created and is available.
    /// ```
    /// # fn main() -> anyhow::Result<()> {
    /// # use momento_test_util::create_doctest_storage_client;
    /// # tokio_test::block_on(async {
    /// # let (storage_client, store_name) = create_doctest_storage_client();
    /// use momento::storage::DeleteResponse;
    /// use momento::MomentoErrorCode;
    ///
    /// match storage_client.delete(&store_name, "key").await {
    ///     Ok(_) => println!("DeleteResponse successful"),
    ///     Err(e) => if let MomentoErrorCode::StoreNotFoundError = e.error_code {
    ///         println!("Store not found: {}", &store_name);
    ///     } else {
    ///         eprintln!("Error deleting value in store {}: {}", &store_name, e);
    ///     }
    /// }
    /// # Ok(())
    /// # })
    /// # }
    /// ```
    /// You can also use the [send_request](PreviewStorageClient::send_request) method to delete an item using a [DeleteRequest].
    pub async fn delete(
        &self,
        store_name: impl Into<String>,
        key: impl Into<String>,
    ) -> MomentoResult<DeleteResponse> {
        let request = DeleteRequest::new(store_name, key);
        request.send(self).await
    }

    /// Lower-level API to send any type of MomentoRequest to the server.
    pub async fn send_request<R: MomentoStorageRequest>(
        &self,
        request: R,
    ) -> MomentoResult<R::Response> {
        request.send(self).await
    }
}
