use momento::{
  storage, CredentialProvider, MomentoResult, PreviewStorageClient,
};

#[tokio::main]
async fn main() -> MomentoResult<()> {
  let storage_client = PreviewStorageClient::builder()
  .configuration(storage::configurations::Laptop::latest())
  .credential_provider(
      CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string())
          .expect("API key should be valid"),
  )
  .build()?;

  // ...

  Ok(())
}