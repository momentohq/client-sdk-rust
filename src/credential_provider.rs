use crate::MomentoResult;
use crate::{MomentoError, MomentoErrorCode};
use base64::Engine;
use log::warn;
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::{Debug, Display, Formatter};

#[derive(Serialize, Deserialize)]
struct V1Token {
    pub api_key: String,
    pub endpoint: String,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) enum EndpointSecurity {
    Insecure,
    Unverified,
    Tls,
    TlsOverride,
}

/// Function arguments for creating a CredentialProvider from a v2 API key and Momento service endpoint
/// stored in the specified environment variables
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct FromEnvVarV2Args {
    /// Name of the environment variable from which to read the v2 api key
    pub api_key_env_var: String,
    /// Name of the environment variable from which to read the Momento service endpoint
    pub endpoint_env_var: String,
}

/// Function arguments for creating a CredentialProvider from a v2 API key string and Momento service endpoint
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct FromApiKeyV2Args {
    /// Momento v2 API key
    pub api_key: String,
    /// Momento service endpoint
    pub endpoint: String,
}

/// Provides information that the client needs in order to establish a connection to and
/// authenticate with the Momento service.
#[derive(PartialEq, Eq, Clone)]
pub struct CredentialProvider {
    pub(crate) auth_token: String,
    pub(crate) tls_cache_endpoint: String,
    pub(crate) control_endpoint: String,
    pub(crate) cache_endpoint: String,
    pub(crate) cache_http_endpoint: String,
    pub(crate) token_endpoint: String,
    pub(crate) endpoint_security: EndpointSecurity,
    pub(crate) use_private_endpoints: bool,
    pub(crate) use_endpoints_http_api: bool,
}

impl Display for CredentialProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CredentialProvider {{ auth_token: <redacted>, tls_cache_endpoint: {}, cache_endpoint: {}, control_endpoint: {}, token_endpoint: {} }}",
            self.tls_cache_endpoint, self.cache_endpoint, self.control_endpoint, self.token_endpoint
        )
    }
}

impl Debug for CredentialProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CredentialProvider")
            .field("auth_token", &"<redacted>")
            .field("tls_cache_endpoint", &self.tls_cache_endpoint)
            .field("cache_endpoint", &self.cache_endpoint)
            .field("control_endpoint", &self.control_endpoint)
            .field("token_endpoint", &self.token_endpoint)
            .field("endpoint_security", &self.endpoint_security)
            .field("use_private_endpoints", &self.use_private_endpoints)
            .field("use_endpoints_http_api", &self.use_endpoints_http_api)
            .finish()
    }
}

impl CredentialProvider {
    /// Returns a Credential Provider using an API key stored in the specified
    /// environment variable
    ///
    /// # Arguments
    ///
    /// * `env_var_name` - Name of the environment variable to read token from
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// use momento::CredentialProvider;
    /// let credential_provider = CredentialProvider::from_env_var("MOMENTO_API_KEY")
    ///     .expect("MOMENTO_API_KEY must be set");
    /// # })
    /// ```
    ///
    #[deprecated(since = "0.59.0", note = "Please use `from_env_var_v2` instead")]
    pub fn from_env_var(env_var_name: impl Into<String>) -> MomentoResult<CredentialProvider> {
        let env_var_name = env_var_name.into();
        let token_to_process = match env::var(&env_var_name) {
            Ok(auth_token) => auth_token,
            Err(e) => {
                return Err(MomentoError {
                    message: format!("Env var {env_var_name} must be set"),
                    error_code: MomentoErrorCode::InvalidArgumentError,
                    inner_error: Some(crate::ErrorSource::Unknown(Box::new(e))),
                });
            }
        };

        if is_v2_api_key(&token_to_process) {
            return Err(MomentoError {
                message: "Received a v2 API key. Are you using the correct key? Or did you mean to use `from_env_var_v2()` instead?".into(),
                error_code: MomentoErrorCode::InvalidArgumentError,
                inner_error: None,
            });
        }

        decode_auth_token(token_to_process)
    }

    /// Returns the hostname that can be used with momento HTTP apis
    pub fn cache_http_endpoint(&self) -> &str {
        &self.cache_http_endpoint
    }

    /// Returns a Credential Provider from the provided API key
    ///
    /// # Arguments
    ///
    /// * `api_key` - Momento API key
    /// # Examples
    ///
    /// ```
    /// # use momento::MomentoResult;
    /// # fn main() -> () {
    /// # tokio_test::block_on(async {
    /// use momento::CredentialProvider;
    ///
    /// let api_key = "YOUR API KEY GOES HERE";
    /// let credential_provider = match CredentialProvider::from_string(api_key) {
    ///    Ok(credential_provider) => credential_provider,
    ///    Err(e) => {
    ///         println!("Error while creating credential provider: {}", e);
    ///         return // probably you will do something else here
    ///    }
    /// };
    /// # ()
    /// # })
    /// #
    /// # }
    /// ```
    #[deprecated(
        since = "0.59.0",
        note = "Please use `from_api_key_v2` or `from_disposable_token` instead"
    )]
    pub fn from_string(auth_token: impl Into<String>) -> MomentoResult<CredentialProvider> {
        let auth_token = auth_token.into();

        if is_v2_api_key(&auth_token) {
            return Err(MomentoError {
                message: "Received a v2 API key. Are you using the correct key? Or did you mean to use `from_api_key_v2()` or `from_disposable_token()` instead?".into(),
                error_code: MomentoErrorCode::InvalidArgumentError,
                inner_error: None,
            });
        }

        let token_to_process = {
            if auth_token.is_empty() {
                return Err(MomentoError {
                    message: "Auth token string cannot be empty".into(),
                    error_code: MomentoErrorCode::InvalidArgumentError,
                    inner_error: None,
                });
            };
            auth_token
        };

        decode_auth_token(token_to_process)
    }

    /// Returns a Credential Provider from the provided v2 API key and Momento service endpoint.
    ///
    /// # Arguments
    /// * `args` - Momento v2 API key and service endpoint provided as a [FromApiKeyV2Args] struct
    ///
    /// # Examples
    /// ```
    /// # use momento::MomentoResult;
    /// # fn main() -> () {
    /// # tokio_test::block_on(async {
    /// use momento::CredentialProvider;
    /// use momento::FromApiKeyV2Args;
    ///
    /// let args = FromApiKeyV2Args {
    ///    api_key: "YOUR V2 API KEY".to_string(),
    ///   endpoint: "YOUR MOMENTO ENDPOINT".to_string(),
    /// };
    /// let credential_provider = CredentialProvider::from_api_key_v2(args);
    /// # ()
    /// # })
    /// # }
    /// ```
    pub fn from_api_key_v2(args: FromApiKeyV2Args) -> MomentoResult<CredentialProvider> {
        let auth_token = args.api_key;
        let endpoint = args.endpoint;

        if auth_token.is_empty() {
            return Err(MomentoError {
                message: "API key cannot be empty".into(),
                error_code: MomentoErrorCode::InvalidArgumentError,
                inner_error: None,
            });
        };

        if endpoint.is_empty() {
            return Err(MomentoError {
                message: "Endpoint string cannot be empty".into(),
                error_code: MomentoErrorCode::InvalidArgumentError,
                inner_error: None,
            });
        };

        if !is_v2_api_key(&auth_token) {
            return Err(MomentoError {
                message: "Received an invalid v2 API key. Are you using the correct key? Or did you mean to use `from_string()` with a legacy key instead?".into(),
                error_code: MomentoErrorCode::InvalidArgumentError,
                inner_error: None,
            });
        }

        Ok(CredentialProvider {
            auth_token,
            tls_cache_endpoint: https_endpoint(get_cache_endpoint(&endpoint)),
            cache_endpoint: https_endpoint(get_cache_endpoint(&endpoint)),
            cache_http_endpoint: https_endpoint(get_cache_http_endpoint(&endpoint)),
            control_endpoint: https_endpoint(get_control_endpoint(&endpoint)),
            token_endpoint: https_endpoint(get_token_endpoint(&endpoint)),
            endpoint_security: EndpointSecurity::Tls,
            use_private_endpoints: false,
            use_endpoints_http_api: false,
        })
    }

    /// Returns a Credential Provider using an API key and Momento service endpoint
    /// stored in the specified environment variables
    ///
    /// # Arguments
    /// * `args` - Names of the environment variables from which to read the v2 api key and service endpoint, provided as a [FromEnvVarV2Args] struct
    ///
    /// # Examples
    /// ```
    /// # use momento::MomentoResult;
    /// # fn main() -> () {
    /// # tokio_test::block_on(async {
    /// use momento::CredentialProvider;
    /// use momento::FromEnvVarV2Args;
    ///
    /// let args = FromEnvVarV2Args {
    ///    api_key_env_var: "YOUR ENV VAR NAME GOES HERE".to_string(),
    ///    endpoint_env_var: "YOUR MOMENTO ENDPOINT GOES HERE".to_string(),
    /// };
    /// let credential_provider = CredentialProvider::from_env_var_v2(args);
    /// # ()
    /// # })
    /// # }
    /// ```
    pub fn from_env_var_v2(args: FromEnvVarV2Args) -> MomentoResult<CredentialProvider> {
        let api_key_env_var_name = args.api_key_env_var;
        if api_key_env_var_name.is_empty() {
            return Err(MomentoError {
                message: "API key env var name cannot be empty".into(),
                error_code: MomentoErrorCode::InvalidArgumentError,
                inner_error: None,
            });
        };

        let api_key = match env::var(&api_key_env_var_name) {
            Ok(api_key) => {
                if api_key.is_empty() {
                    return Err(MomentoError {
                        message: format!("Env var {api_key_env_var_name} must be set"),
                        error_code: MomentoErrorCode::InvalidArgumentError,
                        inner_error: None,
                    });
                };
                api_key
            }
            Err(e) => {
                return Err(MomentoError {
                    message: format!("Env var {api_key_env_var_name} must be set"),
                    error_code: MomentoErrorCode::InvalidArgumentError,
                    inner_error: Some(crate::ErrorSource::Unknown(Box::new(e))),
                });
            }
        };

        if !is_v2_api_key(&api_key) {
            return Err(MomentoError {
                message: "Received an invalid v2 API key. Are you using the correct key? Or did you mean to use `from_env_var()` with a legacy key instead?".into(),
                error_code: MomentoErrorCode::InvalidArgumentError,
                inner_error: None,
            });
        }

        let endpoint_env_var_name = args.endpoint_env_var;
        if endpoint_env_var_name.is_empty() {
            return Err(MomentoError {
                message: "Endpoint env var name cannot be empty".into(),
                error_code: MomentoErrorCode::InvalidArgumentError,
                inner_error: None,
            });
        };

        let endpoint = match env::var(&endpoint_env_var_name) {
            Ok(endpoint) => {
                if endpoint.is_empty() {
                    return Err(MomentoError {
                        message: format!("Env var {endpoint_env_var_name} must be set"),
                        error_code: MomentoErrorCode::InvalidArgumentError,
                        inner_error: None,
                    });
                };
                endpoint
            }
            Err(e) => {
                return Err(MomentoError {
                    message: format!("Env var {endpoint_env_var_name} must be set"),
                    error_code: MomentoErrorCode::InvalidArgumentError,
                    inner_error: Some(crate::ErrorSource::Unknown(Box::new(e))),
                });
            }
        };

        CredentialProvider::from_api_key_v2(FromApiKeyV2Args { api_key, endpoint })
    }

    /// Returns a Credential Provider from the provided disposable auth token
    ///
    /// # Arguments
    ///
    /// * `auth_token` - Momento disposable auth token
    /// # Examples
    ///
    /// ```
    /// # use momento::MomentoResult;
    /// # fn main() -> () {
    /// # tokio_test::block_on(async {
    /// use momento::CredentialProvider;
    ///
    /// let credential_provider = match CredentialProvider::from_disposable_token("YOUR DISPOSABLE AUTH TOKEN") {
    ///    Ok(credential_provider) => credential_provider,
    ///    Err(e) => {
    ///         println!("Error while creating credential provider: {}", e);
    ///         return // probably you will do something else here
    ///    }
    /// };
    /// # ()
    /// # })
    /// #
    /// # }
    /// ```
    pub fn from_disposable_token(
        auth_token: impl Into<String>,
    ) -> MomentoResult<CredentialProvider> {
        let auth_token = auth_token.into();

        if is_v2_api_key(&auth_token) {
            return Err(MomentoError {
                message: "Received a v2 API key. Are you using the correct key? Or did you mean to use `from_api_key_v2()` instead?".into(),
                error_code: MomentoErrorCode::InvalidArgumentError,
                inner_error: None,
            });
        }

        let token_to_process = {
            if auth_token.is_empty() {
                return Err(MomentoError {
                    message: "Auth token cannot be empty".into(),
                    error_code: MomentoErrorCode::InvalidArgumentError,
                    inner_error: None,
                });
            };
            auth_token
        };

        decode_auth_token(token_to_process)
    }

    /// Overrides the base endpoint. The control, cache, and token endpoints will be set to
    /// control.{supplied endpoint}, cache.{supplied endpoint}, token.{supplied endpoint}, etc.
    ///
    /// Does not change the endpoint security type, so connections will still be established
    /// using TLS by default. gRPC connections will use the new endpoints and protosocket
    /// connections will use the new cache http endpoint to look up addresses to connect to.
    pub fn base_endpoint(mut self, endpoint: &str) -> CredentialProvider {
        self.control_endpoint = https_endpoint(get_control_endpoint(endpoint));
        self.cache_endpoint = https_endpoint(get_cache_endpoint(endpoint));
        self.cache_http_endpoint = https_endpoint(get_cache_http_endpoint(endpoint));
        self.token_endpoint = https_endpoint(get_token_endpoint(endpoint));
        self
    }

    /// Overrides the control, cache, and token endpoints with the supplied endpoint. They will all
    /// be equal to each other once this is done.
    ///
    /// Does not change the endpoint security type, so connections will still be established
    /// using TLS by default. gRPC connections will use the new endpoints and protosocket
    /// connections will use the new cache http endpoint to look up addresses to connect to.
    pub fn full_endpoint_override(self, endpoint: &str) -> CredentialProvider {
        self.endpoint_override(endpoint, None)
    }

    /// Overrides the control, cache, and token endpoints with the supplied endpoint. They will all
    /// be equal to each other once this is done.
    ///
    /// Changes the endpoint security type to TlsOverride. TLS will still use the endpoint from the
    /// API key. gRPC connections will use the new endpoints and protosocket connections will
    /// use the new endpoints directly instead of looking up addresses.
    pub fn secure_endpoint_override(self, endpoint: &str) -> CredentialProvider {
        self.endpoint_override(endpoint, Some(EndpointSecurity::TlsOverride))
    }

    /// Overrides the control, cache, and token endpoints with the supplied endpoint. They will all
    /// be equal to each other once this is done.
    ///
    /// Changes the endpoint security type to Insecure. TLS will not be used when establishing
    /// connections to Momento. gRPC connections will use the new endpoints and protosocket
    /// connections will use the new endpoints directly instead of looking up addresses.
    pub fn insecure_endpoint_override(self, endpoint: &str) -> CredentialProvider {
        self.endpoint_override(endpoint, Some(EndpointSecurity::Insecure))
    }

    /// Overrides the control, cache, and token endpoints with the supplied endpoint. They will all
    /// be equal to each other once this is done.
    ///
    /// Changes the endpoint security type to Unverified. Connections will be established using a
    /// self-signed TLS certificate. gRPC connections will use the new endpoints and protosocket
    /// connections will use the new endpoints directly instead of looking up addresses.
    pub fn unverified_tls_endpoint_override(self, endpoint: &str) -> CredentialProvider {
        self.endpoint_override(endpoint, Some(EndpointSecurity::Unverified))
    }

    /// Directs the ProtosocketCacheClient to look up private endpoints when discovering
    /// addresses to connect to.
    pub fn with_private_endpoints(mut self) -> CredentialProvider {
        self.use_private_endpoints = true;
        self.use_endpoints_http_api = true;
        self
    }
    /// Directs the ProtosocketCacheClient to look up public endpoints when discovering
    /// addresses to connect to.
    pub fn with_endpoints(mut self) -> CredentialProvider {
        self.use_private_endpoints = false;
        self.use_endpoints_http_api = true;
        self
    }

    fn endpoint_override(
        mut self,
        endpoint: &str,
        endpoint_security: Option<EndpointSecurity>,
    ) -> CredentialProvider {
        self.control_endpoint = endpoint.to_string();
        self.cache_endpoint = endpoint.to_string();
        self.cache_http_endpoint = endpoint.to_string();
        self.token_endpoint = endpoint.to_string();
        if let Some(es) = endpoint_security {
            self.endpoint_security = es
        };

        self
    }
}

fn decode_auth_token(auth_token: String) -> MomentoResult<CredentialProvider> {
    let auth_token_bytes = base64::engine::general_purpose::URL_SAFE
        .decode(auth_token)
        .map_err(|e| token_parsing_error(Box::new(e)))?;
    process_v1_token(auth_token_bytes)
}

fn process_v1_token(auth_token_bytes: Vec<u8>) -> MomentoResult<CredentialProvider> {
    let json: V1Token =
        serde_json::from_slice(&auth_token_bytes).map_err(|e| token_parsing_error(Box::new(e)))?;

    Ok(CredentialProvider {
        auth_token: json.api_key,
        tls_cache_endpoint: https_endpoint(get_cache_endpoint(&json.endpoint)),
        cache_endpoint: https_endpoint(get_cache_endpoint(&json.endpoint)),
        cache_http_endpoint: https_endpoint(get_cache_http_endpoint(&json.endpoint)),
        control_endpoint: https_endpoint(get_control_endpoint(&json.endpoint)),
        token_endpoint: https_endpoint(get_token_endpoint(&json.endpoint)),
        endpoint_security: EndpointSecurity::Tls,
        use_private_endpoints: false,
        use_endpoints_http_api: false,
    })
}

fn get_cache_endpoint(endpoint: &str) -> String {
    format!("cache.{endpoint}")
}

fn get_cache_http_endpoint(endpoint: &str) -> String {
    format!("api.cache.{endpoint}")
}

fn get_control_endpoint(endpoint: &str) -> String {
    format!("control.{endpoint}")
}

fn get_token_endpoint(endpoint: &str) -> String {
    format!("token.{endpoint}")
}

fn https_endpoint(hostname: String) -> String {
    format!("https://{hostname}")
}

fn token_parsing_error(e: Box<dyn std::error::Error + Send + Sync>) -> MomentoError {
    MomentoError {
        message: "Could not parse token. Please ensure a valid token was entered correctly.".into(),
        error_code: MomentoErrorCode::InvalidArgumentError,
        inner_error: Some(crate::ErrorSource::Unknown(e)),
    }
}

fn is_base64_encoded(s: &str) -> bool {
    base64::engine::general_purpose::URL_SAFE.decode(s).is_ok()
}

fn is_v2_api_key(api_key: &str) -> bool {
    // only v1 api keys are entirely b64 encoded
    // v2 keys are JWTs with b64 encoded segments
    if is_base64_encoded(api_key) {
        warn!("did not expect v2 api key to be entirely base64 encoded");
        return false;
    }

    // do not use jwt parsing library, just b64 decode middle segment to check "t" claim
    let segments: Vec<&str> = api_key.split('.').collect();
    if segments.len() != 3 {
        warn!("token does not have three segments");
        return false;
    }

    let b64_encoded_claims = segments[1];
    let b64_decoded_claims_bytes =
        match base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(b64_encoded_claims) {
            Ok(bytes) => bytes,
            Err(_) => {
                warn!("could not decode jwt claims segment");
                return false;
            }
        };
    let json_claims: serde_json::Map<String, serde_json::Value> =
        match serde_json::from_slice(&b64_decoded_claims_bytes) {
            Ok(json) => json,
            Err(_) => {
                warn!("could not parse jwt claims segment as utf8 string");
                return false;
            }
        };
    let t_claim = match json_claims.get("t") {
        Some(serde_json::Value::String(t)) => t,
        _ => {
            warn!("could not find 't' claim in jwt claims");
            return false;
        }
    };
    t_claim == "g"
}

#[cfg(test)]
#[allow(deprecated)] // we'll still test the legacy methods
mod tests {
    use crate::{CredentialProvider, FromApiKeyV2Args, FromEnvVarV2Args, MomentoResult};

    const TEST_V1_API_KEY: &str = "eyJlbmRwb2ludCI6Im1vbWVudG9fZW5kcG9pbnQiLCJhcGlfa2V5IjoiZXlKaGJHY2lPaUpJVXpJMU5pSjkuZXlKemRXSWlPaUowWlhOMElITjFZbXBsWTNRaUxDSjJaWElpT2pFc0luQWlPaUlpZlEuaGcyd01iV2Utd2VzUVZ0QTd3dUpjUlVMalJwaFhMUXdRVFZZZlFMM0w3YyJ9Cg==";
    const TEST_V2_API_KEY: &str = "eyJhbGciOiJIUzUxMiIsInR5cCI6IkpXVCJ9.eyJ0IjoiZyIsImp0aSI6InNvbWUtaWQifQ.GMr9nA6HE0ttB6llXct_2Sg5-fOKGFbJCdACZFgNbN1fhT6OPg_hVc8ThGzBrWC_RlsBpLA1nzqK3SOJDXYxAw";

    const TEST_ENDPOINT: &str = "test_endpoint";
    const KEY_ENV_VAR_NAME: &str = "MOMENTO_TEST_V2_API_KEY";
    const ENDPOINT_ENV_VAR_NAME: &str = "MOMENTO_TEST_ENDPOINT";

    #[test]
    fn env_var() {
        let env_var_name = "TEST_ENV_VAR_CREDENTIAL_PROVIDER";
        temp_env::with_var(env_var_name, Some(TEST_V1_API_KEY), || {
            let credential_provider = CredentialProvider::from_env_var(env_var_name)
                .expect("should be able to build credential provider");

            assert_eq!(
                "https://cache.momento_endpoint",
                credential_provider.cache_endpoint
            );
            assert_eq!(
                "https://control.momento_endpoint",
                credential_provider.control_endpoint
            );
            assert_eq!(
                "https://token.momento_endpoint",
                credential_provider.token_endpoint
            );

            assert_eq!("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ0ZXN0IHN1YmplY3QiLCJ2ZXIiOjEsInAiOiIifQ.hg2wMbWe-wesQVtA7wuJcRULjRphXLQwQTVYfQL3L7c", credential_provider.auth_token);
        });
    }

    #[test]
    fn env_var_not_set() {
        let env_var_name = "TEST_ENV_VAR_CREDENTIAL_PROVIDER_NOT_SET";
        let _err_msg = format!("Env var {env_var_name} must be set");
        let e = CredentialProvider::from_env_var(env_var_name).unwrap_err();

        assert_eq!(e.to_string(), _err_msg);
    }

    #[test]
    fn env_var_empty_string() {
        let env_var_name = "TEST_ENV_VAR_CREDENTIAL_PROVIDER_EMPTY_STRING";
        temp_env::with_var(env_var_name, Some(""), || {
            let _err_msg =
                "Could not parse token. Please ensure a valid token was entered correctly.";
            let e = CredentialProvider::from_env_var(env_var_name).unwrap_err();
            assert_eq!(e.to_string(), _err_msg);
        });
    }

    #[test]
    fn empty_token() {
        let e = CredentialProvider::from_string("").unwrap_err();
        let _err_msg = "Auth token string cannot be empty".to_owned();
        assert_eq!(e.to_string(), _err_msg);
    }

    #[test]
    fn invalid_token() {
        let b64_encoded_invalid_token = base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE,
            "wfheofhriugheifweif",
        );
        let e = CredentialProvider::from_string(b64_encoded_invalid_token).unwrap_err();
        let _err_msg =
            "Could not parse token. Please ensure a valid token was entered correctly.".to_owned();
        assert_eq!(e.to_string(), _err_msg);
    }

    #[test]
    fn valid_v1_token() {
        let credential_provider =
            CredentialProvider::from_string(TEST_V1_API_KEY).expect("failed to parse token");
        assert_eq!(
            "https://control.momento_endpoint",
            credential_provider.control_endpoint
        );
        assert_eq!(
            "https://cache.momento_endpoint",
            credential_provider.cache_endpoint
        );
        assert_eq!(
            "https://token.momento_endpoint",
            credential_provider.token_endpoint
        );
        assert_eq!("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ0ZXN0IHN1YmplY3QiLCJ2ZXIiOjEsInAiOiIifQ.hg2wMbWe-wesQVtA7wuJcRULjRphXLQwQTVYfQL3L7c", credential_provider.auth_token);
    }

    #[test]
    fn v1_token_with_base_endpoint_override() -> MomentoResult<()> {
        let credential_provider =
            CredentialProvider::from_string(TEST_V1_API_KEY)?.base_endpoint("foo.com");
        assert_eq!("https://cache.foo.com", credential_provider.cache_endpoint);
        assert_eq!(
            "https://api.cache.foo.com",
            credential_provider.cache_http_endpoint
        );
        assert_eq!(
            "https://control.foo.com",
            credential_provider.control_endpoint
        );
        assert_eq!("https://token.foo.com", credential_provider.token_endpoint);
        assert_eq!("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ0ZXN0IHN1YmplY3QiLCJ2ZXIiOjEsInAiOiIifQ.hg2wMbWe-wesQVtA7wuJcRULjRphXLQwQTVYfQL3L7c", credential_provider.auth_token);

        Ok(())
    }

    #[test]
    fn invalid_v1_token_json() {
        let auth_token = "eyJmb28iOiJiYXIifQo=";
        let e = CredentialProvider::from_string(auth_token).unwrap_err();
        let _err_msg =
            "Could not parse token. Please ensure a valid token was entered correctly.".to_string();
        assert_eq!(e.to_string(), _err_msg);
    }

    #[test]
    fn from_env_var_v2() -> MomentoResult<()> {
        temp_env::with_vars(
            [
                (KEY_ENV_VAR_NAME, Some(TEST_V2_API_KEY)),
                (ENDPOINT_ENV_VAR_NAME, Some(TEST_ENDPOINT)),
            ],
            || {
                let credential_provider = CredentialProvider::from_env_var_v2(FromEnvVarV2Args {
                    api_key_env_var: KEY_ENV_VAR_NAME.into(),
                    endpoint_env_var: ENDPOINT_ENV_VAR_NAME.into(),
                })
                .expect("should be able to build credential provider");
                assert!(credential_provider.auth_token == TEST_V2_API_KEY);
                assert!(credential_provider.cache_endpoint == "https://cache.test_endpoint");
                assert!(credential_provider.control_endpoint == "https://control.test_endpoint");
                assert!(credential_provider.token_endpoint == "https://token.test_endpoint");
                assert!(
                    credential_provider.cache_http_endpoint == "https://api.cache.test_endpoint"
                );
            },
        );
        Ok(())
    }

    #[test]
    fn from_api_key_v2() -> MomentoResult<()> {
        let credential_provider = CredentialProvider::from_api_key_v2(FromApiKeyV2Args {
            api_key: TEST_V2_API_KEY.into(),
            endpoint: TEST_ENDPOINT.into(),
        })?;
        assert!(credential_provider.auth_token == TEST_V2_API_KEY);
        assert!(credential_provider.cache_endpoint == "https://cache.test_endpoint");
        assert!(credential_provider.control_endpoint == "https://control.test_endpoint");
        assert!(credential_provider.token_endpoint == "https://token.test_endpoint");
        assert!(credential_provider.cache_http_endpoint == "https://api.cache.test_endpoint");
        Ok(())
    }

    #[test]
    fn from_api_key_v2_empty_endpoint() {
        let empty_endpoint_err = CredentialProvider::from_api_key_v2(FromApiKeyV2Args {
            api_key: TEST_V2_API_KEY.into(),
            endpoint: "".into(),
        })
        .unwrap_err();
        let _err_msg = "Endpoint string cannot be empty".to_owned();
        assert_eq!(empty_endpoint_err.to_string(), _err_msg);
    }

    #[test]
    fn from_api_key_v2_empty_api_key() {
        let empty_key_err = CredentialProvider::from_api_key_v2(FromApiKeyV2Args {
            api_key: "".into(),
            endpoint: TEST_ENDPOINT.into(),
        })
        .unwrap_err();
        let _err_msg = "API key cannot be empty".to_owned();
        assert_eq!(empty_key_err.to_string(), _err_msg);
    }

    #[test]
    fn from_env_var_v2_empty_endpoint_env_var_name() {
        temp_env::with_vars([(KEY_ENV_VAR_NAME, Some(TEST_V2_API_KEY))], || {
            let empty_endpoint_err = CredentialProvider::from_env_var_v2(FromEnvVarV2Args {
                api_key_env_var: KEY_ENV_VAR_NAME.into(),
                endpoint_env_var: "".into(),
            })
            .unwrap_err();
            let _err_msg = "Endpoint env var name cannot be empty".to_owned();
            assert_eq!(empty_endpoint_err.to_string(), _err_msg);
        });
    }

    #[test]
    fn from_env_var_v2_empty_key_env_var_name() {
        temp_env::with_var(ENDPOINT_ENV_VAR_NAME, Some(TEST_ENDPOINT), || {
            let empty_env_var_name_err = CredentialProvider::from_env_var_v2(FromEnvVarV2Args {
                api_key_env_var: "".into(),
                endpoint_env_var: ENDPOINT_ENV_VAR_NAME.into(),
            })
            .unwrap_err();
            let _err_msg = "API key env var name cannot be empty".to_owned();
            assert_eq!(empty_env_var_name_err.to_string(), _err_msg);
        });
    }

    #[test]
    fn from_env_var_v2_empty_key_env_var() {
        temp_env::with_vars(
            [
                (KEY_ENV_VAR_NAME, Some("")),
                (ENDPOINT_ENV_VAR_NAME, Some(TEST_ENDPOINT)),
            ],
            || {
                let empty_env_var_err = CredentialProvider::from_env_var_v2(FromEnvVarV2Args {
                    api_key_env_var: KEY_ENV_VAR_NAME.into(),
                    endpoint_env_var: ENDPOINT_ENV_VAR_NAME.into(),
                })
                .unwrap_err();
                let _err_msg = format!("Env var {KEY_ENV_VAR_NAME} must be set").to_owned();
                assert_eq!(empty_env_var_err.to_string(), _err_msg);
            },
        );
    }

    #[test]
    fn from_env_var_v2_empty_endpoint_env_var() {
        temp_env::with_vars(
            [
                (KEY_ENV_VAR_NAME, Some(TEST_V2_API_KEY)),
                (ENDPOINT_ENV_VAR_NAME, Some("")),
            ],
            || {
                let empty_env_var_err = CredentialProvider::from_env_var_v2(FromEnvVarV2Args {
                    api_key_env_var: KEY_ENV_VAR_NAME.into(),
                    endpoint_env_var: ENDPOINT_ENV_VAR_NAME.into(),
                })
                .unwrap_err();
                let _err_msg = format!("Env var {ENDPOINT_ENV_VAR_NAME} must be set").to_owned();
                assert_eq!(empty_env_var_err.to_string(), _err_msg);
            },
        );
    }

    #[test]
    fn v1_token_given_to_from_api_key_v2() {
        let err = CredentialProvider::from_api_key_v2(FromApiKeyV2Args {
            api_key: TEST_V1_API_KEY.into(),
            endpoint: TEST_ENDPOINT.into(),
        })
        .unwrap_err();
        let _err_msg = "Received an invalid v2 API key. Are you using the correct key? Or did you mean to use `from_string()` with a legacy key instead?".to_owned();
        assert_eq!(err.to_string(), _err_msg);
    }

    #[test]
    fn v1_token_given_to_from_env_var_v2() {
        temp_env::with_vars(
            [
                (KEY_ENV_VAR_NAME, Some(TEST_V1_API_KEY)),
                (ENDPOINT_ENV_VAR_NAME, Some(TEST_ENDPOINT)),
            ],
            || {
                let err = CredentialProvider::from_env_var_v2(FromEnvVarV2Args {
                    api_key_env_var: KEY_ENV_VAR_NAME.into(),
                    endpoint_env_var: ENDPOINT_ENV_VAR_NAME.into(),
                })
                .unwrap_err();
                let _err_msg = "Received an invalid v2 API key. Are you using the correct key? Or did you mean to use `from_env_var()` with a legacy key instead?".to_owned();
                assert_eq!(err.to_string(), _err_msg);
            },
        );
    }

    #[test]
    fn v2_key_given_to_from_string() {
        let err = CredentialProvider::from_string(TEST_V2_API_KEY).unwrap_err();
        let _err_msg = "Received a v2 API key. Are you using the correct key? Or did you mean to use `from_api_key_v2()` or `from_disposable_token()` instead?".to_owned();
        assert_eq!(err.to_string(), _err_msg);
    }

    #[test]
    fn v2_key_given_to_from_env_var() {
        temp_env::with_var(KEY_ENV_VAR_NAME, Some(TEST_V2_API_KEY), || {
            let err = CredentialProvider::from_env_var(KEY_ENV_VAR_NAME).unwrap_err();
            let _err_msg = "Received a v2 API key. Are you using the correct key? Or did you mean to use `from_env_var_v2()` instead?".to_owned();
            assert_eq!(err.to_string(), _err_msg);
        });
    }

    #[test]
    fn v2_key_given_to_from_disposable_token() {
        let err = CredentialProvider::from_disposable_token(TEST_V2_API_KEY).unwrap_err();
        let _err_msg = "Received a v2 API key. Are you using the correct key? Or did you mean to use `from_api_key_v2()` instead?".to_owned();
        assert_eq!(err.to_string(), _err_msg);
    }

    #[test]
    fn from_disposable_token() -> MomentoResult<()> {
        let credential_provider = CredentialProvider::from_disposable_token(TEST_V1_API_KEY)?;
        assert!(credential_provider.cache_endpoint == "https://cache.momento_endpoint");
        assert!(credential_provider.control_endpoint == "https://control.momento_endpoint");
        assert!(credential_provider.token_endpoint == "https://token.momento_endpoint");
        assert!(credential_provider.cache_http_endpoint == "https://api.cache.momento_endpoint");
        Ok(())
    }
}
