use momento::config::configuration::Configuration;
use momento::config::grpc_configuration::GrpcConfiguration;
use momento::config::transport_strategy::TransportStrategy;
use momento::MomentoError;
use std::time::Duration;

// const CACHE_NAME: &str = "cache";

#[tokio::main]
pub async fn main() -> Result<(), MomentoError> {
    let config = Configuration::builder()
        .transport_strategy(
            TransportStrategy::builder()
                .grpc_configuration(
                    GrpcConfiguration::builder()
                        .deadline(Duration::from_secs(60))
                        .build(),
                )
                .build(),
        )
        .build();
    
    println!("{:?}", config);

    // // Credential Provider builders | this one needs a builder because it will parse the tokens and
    // //                              | stuff when you call 'build'. we can have some factory fns that
    // //                              | skip the exposure to the builder in the common case, or force
    // //                              | people to see the builder for consistency
    // let cred_provider = CredentialProvider::builder()
    //     .from_env_var("MOMENTO_API_KEY".to_string())
    //     .base_endpoint("foo.com")
    //     .build()?;
    //
    // // Cache Client builders | this one will need a builder, because the 'build' function gates the
    // //                       | establishment of connections etc. can probably be similar to the config builder
    // let cache_client = CacheClient::builder()
    //     // kenny's pref was to do a phased builder here. if we do, default_ttl needs to be first, because it's the least likely one
    //     // to become optional in the future
    //     .default_ttl(Duration::from_secs(60))
    //     // I think we would do config next because i don't really care if we ever make it optional, whereas cred provider i would
    //     // definitely like to make optional
    //     .config(config)
    //     // When we have a default credential chain (to look in env vars, config files, etc for creds), this one could become optional. Which
    //     // would simply mean that we add a `.build` function to the builder from the previous phase, even though it won't have one in the first launch.
    //     .credential_provider(cred_provider)
    //     // we won't be able to add required args in the future but we can add optional ones on the phased builder at this level.
    //     .build()?;
    //
    // // Request builders | these won't really need to have a builder because they don't need a 'build' function,
    // //                  | because there are no resources that need to be initialized on 'build'. but we could
    // //                  | make builders anyway just for consistency. currently thinking that since people will be interacting with these more often than
    // //                  | the config / client constructors, that consistency is probably not a compelling argument to force them to use builders for these.
    // let sorted_set_put_elements_request = SortedSetPutElementsRequest::new(
    //     "cache".to_string(),
    //     "key".to_string(),
    //     vec![]
    // ).with_ttl(CollectionTtl::of(Duration::from_secs(60)));

    Ok(())
}
