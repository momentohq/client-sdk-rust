use crate::MomentoResult;
use crate::{MomentoError, MomentoErrorCode};
use base64::Engine;
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
    pub fn from_string(auth_token: impl Into<String>) -> MomentoResult<CredentialProvider> {
        let auth_token = auth_token.into();

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

#[cfg(test)]
mod tests {
    use crate::{CredentialProvider, MomentoResult};
    use std::env;

    #[test]
    fn env_var() {
        let env_var_name = "TEST_ENV_VAR_CREDENTIAL_PROVIDER";
        let v1_token = "eyJlbmRwb2ludCI6Im1vbWVudG9fZW5kcG9pbnQiLCJhcGlfa2V5IjoiZXlKaGJHY2lPaUpJVXpJMU5pSjkuZXlKemRXSWlPaUowWlhOMElITjFZbXBsWTNRaUxDSjJaWElpT2pFc0luQWlPaUlpZlEuaGcyd01iV2Utd2VzUVZ0QTd3dUpjUlVMalJwaFhMUXdRVFZZZlFMM0w3YyJ9Cg==".to_string();
        env::set_var(env_var_name, v1_token);
        let credential_provider = CredentialProvider::from_env_var(env_var_name)
            .expect("should be able to build credential provider");
        env::remove_var(env_var_name);

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
        env::set_var(env_var_name, "");
        let _err_msg = "Could not parse token. Please ensure a valid token was entered correctly.";
        let e = CredentialProvider::from_env_var(env_var_name).unwrap_err();

        assert_eq!(e.to_string(), _err_msg);
    }

    #[test]
    fn empty_token() {
        let e = CredentialProvider::from_string("").unwrap_err();
        let _err_msg = "Auth token string cannot be empty".to_owned();
        assert_eq!(e.to_string(), _err_msg);
    }

    #[test]
    fn invalid_token() {
        let e = CredentialProvider::from_string("wfheofhriugheifweif").unwrap_err();
        let _err_msg =
            "Could not parse token. Please ensure a valid token was entered correctly.".to_owned();
        assert_eq!(e.to_string(), _err_msg);
    }

    #[test]
    fn valid_v1_token() {
        let v1_token = "eyJlbmRwb2ludCI6Im1vbWVudG9fZW5kcG9pbnQiLCJhcGlfa2V5IjoiZXlKaGJHY2lPaUpJVXpJMU5pSjkuZXlKemRXSWlPaUowWlhOMElITjFZbXBsWTNRaUxDSjJaWElpT2pFc0luQWlPaUlpZlEuaGcyd01iV2Utd2VzUVZ0QTd3dUpjUlVMalJwaFhMUXdRVFZZZlFMM0w3YyJ9Cg==".to_string();

        let credential_provider =
            CredentialProvider::from_string(v1_token).expect("failed to parse token");
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
        let v1_token = "eyJlbmRwb2ludCI6Im1vbWVudG9fZW5kcG9pbnQiLCJhcGlfa2V5IjoiZXlKaGJHY2lPaUpJVXpJMU5pSjkuZXlKemRXSWlPaUowWlhOMElITjFZbXBsWTNRaUxDSjJaWElpT2pFc0luQWlPaUlpZlEuaGcyd01iV2Utd2VzUVZ0QTd3dUpjUlVMalJwaFhMUXdRVFZZZlFMM0w3YyJ9Cg==".to_string();

        let credential_provider =
            CredentialProvider::from_string(v1_token)?.base_endpoint("foo.com");
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
}
