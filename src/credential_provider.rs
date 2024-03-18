use crate::credential_provider::AuthTokenSource::{EnvironmentVariable, LiteralToken};
use crate::{MomentoError, MomentoResult};
use base64::Engine;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::{Debug, Formatter};

#[derive(Debug, Serialize, Deserialize)]
struct JwtClaims {
    #[serde(rename = "sub")]
    subject: String,
    #[serde(rename = "c")]
    cache_endpoint: Option<String>,
    #[serde(rename = "cp")]
    control_endpoint: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct V1Token {
    pub api_key: String,
    pub endpoint: String,
}

/// Provides information that the client needs in order to establish a connection to and
/// authenticate with the Momento service.
#[derive(Clone)]
pub struct CredentialProvider {
    pub auth_token: String,
    pub control_endpoint: String,
    pub cache_endpoint: String,
}

impl CredentialProvider {
    /// Returns a builder to produce a Credential Provider using an API key stored in the specified
    /// environment variable
    ///
    /// # Arguments
    ///
    /// * `env_var_name` - Name of the environment variable to read token from
    /// # Examples
    ///
    /// ```
    /// # tokio_test::block_on(async {
    ///     use momento::CredentialProvider;
    ///     let credential_provider = CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string())
    ///         .expect("MOMENTO_API_KEY must be set");
    /// # })
    /// ```
    ///
    pub fn from_env_var(env_var_name: String) -> MomentoResult<CredentialProvider> {
        CredentialProviderBuilder::from_environment_variable(env_var_name).build()
    }

    /// Returns a builder to produce a Credential Provider from the provided API key
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
    ///     use momento::CredentialProvider;
    ///
    ///     let api_key = "YOUR API KEY GOES HERE";
    ///     let credential_provider = match(CredentialProvider::from_string(api_key.to_string())) {
    ///        Ok(credential_provider) => credential_provider,
    ///        Err(e) => {
    ///             println!("Error while creating credential provider: {}", e);
    ///             return // probably you will do something else here
    ///        }
    ///     };
    ///
    /// # ()
    /// # })
    /// #
    /// }
    /// ```
    pub fn from_string(auth_token: String) -> MomentoResult<CredentialProvider> {
        CredentialProviderBuilder::from_string(auth_token).build()
    }
}

impl Debug for CredentialProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CredentialProvider")
            .field("auth_token", &"<redacted>")
            .field("cache_endpoint", &self.cache_endpoint)
            .field("control_endpoint", &self.control_endpoint)
            .finish()
    }
}

enum AuthTokenSource {
    EnvironmentVariable(String),
    LiteralToken(String),
}

impl Debug for AuthTokenSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            EnvironmentVariable(env_var_name) => {
                f.write_fmt(format_args!("EnvironmentVariable(\"{}\")", env_var_name))
            }
            LiteralToken(_) => f.write_str("LiteralToken(<redacted>)"),
        }
    }
}

#[derive(Debug)]
struct CredentialProviderBuilder {
    auth_token_source: AuthTokenSource,
    cache_endpoint_override: Option<String>,
    control_endpoint_override: Option<String>,
}

impl CredentialProviderBuilder {
    fn from_environment_variable(env_var_name: String) -> Self {
        CredentialProviderBuilder {
            auth_token_source: EnvironmentVariable(env_var_name),
            cache_endpoint_override: None,
            control_endpoint_override: None,
        }
    }

    fn from_string(auth_token: String) -> Self {
        CredentialProviderBuilder {
            auth_token_source: LiteralToken(auth_token),
            cache_endpoint_override: None,
            control_endpoint_override: None,
        }
    }

    // /// Override the data plane endpoint
    // /// # Arguments
    // ///
    // /// * `cache_endpoint_override` - The host which the Momento client will connect to for Momento data plane operations
    // ///
    // /// # Examples
    // /// ```
    // /// # tokio_test::block_on(async {
    // ///     use momento::CredentialProviderBuilder;
    // ///     let credential_provider = CredentialProviderBuilder::from_environment_variable("MOMENTO_API_KEY".to_string())
    // ///         .with_cache_endpoint("my_cache_endpoint.com".to_string())
    // ///         .build()
    // ///         .expect("MOMENTO_API_KEY must be set");
    // ///      assert_eq!("https://my_cache_endpoint.com", credential_provider.cache_endpoint);
    // /// # })
    // /// ```
    // ///
    // fn with_cache_endpoint(mut self, cache_endpoint_override: String) -> Self {
    //     self.cache_endpoint_override = Some(cache_endpoint_override);
    //     self
    // }

    // /// Override the control plane endpoint
    // /// # Arguments
    // ///
    // /// * `control_endpoint_override` - The host which the Momento client will connect to for Momento control plane operations
    // ///
    // /// # Examples
    // /// ```
    // /// # tokio_test::block_on(async {
    // ///     use momento::CredentialProviderBuilder;
    // ///     let credential_provider = CredentialProviderBuilder::from_environment_variable("MOMENTO_API_KEY".to_string())
    // ///         .with_control_endpoint("my_control_endpoint.com".to_string())
    // ///         .build()
    // ///         .expect("MOMENTO_API_KEY must be set");
    // ///      assert_eq!("https://my_control_endpoint.com", credential_provider.control_endpoint);
    // /// # })
    // /// ```
    // ///
    // fn with_control_endpoint(mut self, control_endpoint_override: String) -> Self {
    //     self.control_endpoint_override = Some(control_endpoint_override);
    //     self
    // }

    // /// Override both control plane and data plane endpoints
    // /// # Arguments
    // ///
    // /// * `endpoint_override` - The host which will be used to build control and data plane endpoints by prepending `control` and `cache` subdomains.
    // ///
    // /// # Examples
    // /// ```
    // /// # tokio_test::block_on(async {
    // ///     use momento::CredentialProviderBuilder;
    // ///     let credential_provider = CredentialProviderBuilder::from_environment_variable("MOMENTO_API_KEY".to_string())
    // ///         .with_momento_endpoint("my_endpoint.com".to_string())
    // ///         .build()
    // ///         .expect("MOMENTO_API_KEY must be set");
    // ///      assert_eq!("https://cache.my_endpoint.com", credential_provider.cache_endpoint);
    // ///      assert_eq!("https://control.my_endpoint.com", credential_provider.control_endpoint);
    // /// # })
    // /// ```
    // ///
    // fn with_momento_endpoint(mut self, endpoint_override: String) -> Self {
    //     self.cache_endpoint_override = Some(CredentialProviderBuilder::get_cache_endpoint(
    //         &endpoint_override,
    //     ));
    //     self.control_endpoint_override = Some(CredentialProviderBuilder::get_control_endpoint(
    //         &endpoint_override,
    //     ));
    //     self
    // }

    fn build(self) -> MomentoResult<CredentialProvider> {
        let token_to_process = match self.auth_token_source {
            EnvironmentVariable(env_var_name) => match env::var(&env_var_name) {
                Ok(auth_token) => auth_token,
                Err(e) => {
                    return Err(MomentoError::InvalidArgument {
                        description: format!("Env var {env_var_name} must be set").into(),
                        source: Some(crate::ErrorSource::Unknown(Box::new(e))),
                    })
                }
            },
            LiteralToken(auth_token_string) => {
                if auth_token_string.is_empty() {
                    return Err(MomentoError::InvalidArgument {
                        description: "Auth token string cannot be empty".into(),
                        source: None,
                    });
                }
                auth_token_string
            }
        };

        CredentialProviderBuilder::decode_auth_token(
            token_to_process,
            self.cache_endpoint_override,
            self.control_endpoint_override,
        )
    }

    fn decode_auth_token(
        auth_token: String,
        cache_endpoint_override: Option<String>,
        control_endpoint_override: Option<String>,
    ) -> MomentoResult<CredentialProvider> {
        match base64::engine::general_purpose::URL_SAFE.decode(&auth_token) {
            Ok(auth_token_bytes) => CredentialProviderBuilder::process_v1_token(
                auth_token_bytes,
                cache_endpoint_override,
                control_endpoint_override,
            ),
            Err(_) => CredentialProviderBuilder::process_jwt_token(
                auth_token,
                cache_endpoint_override,
                control_endpoint_override,
            ),
        }
    }

    fn process_v1_token(
        auth_token_bytes: Vec<u8>,
        cache_endpoint_override: Option<String>,
        control_endpoint_override: Option<String>,
    ) -> MomentoResult<CredentialProvider> {
        let json: V1Token = serde_json::from_slice(&auth_token_bytes)
            .map_err(|e| token_parsing_error(Box::new(e)))?;

        // If endpoint override is present then that always takes precedence over the
        // endpoint from the token
        let cache_endpoint = cache_endpoint_override.unwrap_or(
            CredentialProviderBuilder::get_cache_endpoint(&json.endpoint),
        );
        let control_endpoint = control_endpoint_override.unwrap_or(
            CredentialProviderBuilder::get_control_endpoint(&json.endpoint),
        );

        Ok(CredentialProvider {
            auth_token: json.api_key,
            cache_endpoint: CredentialProviderBuilder::https_endpoint(cache_endpoint),
            control_endpoint: CredentialProviderBuilder::https_endpoint(control_endpoint),
        })
    }

    fn process_jwt_token(
        auth_token: String,
        cache_endpoint_override: Option<String>,
        control_endpoint_override: Option<String>,
    ) -> MomentoResult<CredentialProvider> {
        let key = DecodingKey::from_secret(b"");
        let mut validation = Validation::new(Algorithm::HS256);
        validation.required_spec_claims.clear();
        validation.required_spec_claims.insert("sub".to_string());

        validation.validate_exp = false;
        validation.insecure_disable_signature_validation();

        let token =
            decode(&auth_token, &key, &validation).map_err(|e| token_parsing_error(Box::new(e)))?;
        let token_claims: JwtClaims = token.claims;

        // If endpoint override is present then that always takes precedence over the c and cp
        // claims in the JWT
        // If endpoint override is not provided, then `c` and `cp` claims must be present.
        let cache_endpoint = cache_endpoint_override
            .or(token_claims.cache_endpoint)
            .ok_or_else(|| MomentoError::InvalidArgument {
                description: "auth token is missing cache endpoint and endpoint override is missing. One or the other must be provided".into(),
                source: None,
            })?;
        let control_endpoint = control_endpoint_override
            .or(token_claims.control_endpoint)
            .ok_or_else(|| MomentoError::InvalidArgument {
                description: "auth token is missing control endpoint and endpoint override is missing. One or the other must be provided.".into(),
                source: None,
            })?;
        Ok(CredentialProvider {
            auth_token,
            cache_endpoint: CredentialProviderBuilder::https_endpoint(cache_endpoint),
            control_endpoint: CredentialProviderBuilder::https_endpoint(control_endpoint),
        })
    }

    fn get_cache_endpoint(endpoint: &str) -> String {
        format!("cache.{endpoint}")
    }

    fn get_control_endpoint(endpoint: &str) -> String {
        format!("control.{endpoint}")
    }

    fn https_endpoint(hostname: String) -> String {
        format!("https://{}", hostname)
    }
}

fn token_parsing_error(e: Box<dyn std::error::Error + Send + Sync>) -> MomentoError {
    MomentoError::ClientSdkError {
        description: "Could not parse token. Please ensure a valid token was entered correctly."
            .into(),
        source: crate::ErrorSource::Unknown(e),
    }
}

#[cfg(test)]
mod tests {
    use crate::credential_provider::CredentialProviderBuilder;
    use crate::CredentialProvider;
    use std::env;

    #[test]
    fn env_var() {
        let env_var_name = "TEST_ENV_VAR_CREDENTIAL_PROVIDER";
        let v1_token = "eyJlbmRwb2ludCI6Im1vbWVudG9fZW5kcG9pbnQiLCJhcGlfa2V5IjoiZXlKaGJHY2lPaUpJVXpJMU5pSjkuZXlKemRXSWlPaUowWlhOMElITjFZbXBsWTNRaUxDSjJaWElpT2pFc0luQWlPaUlpZlEuaGcyd01iV2Utd2VzUVZ0QTd3dUpjUlVMalJwaFhMUXdRVFZZZlFMM0w3YyJ9Cg==".to_string();
        env::set_var(env_var_name, v1_token);
        let credential_provider =
            CredentialProviderBuilder::from_environment_variable(env_var_name.to_string())
                .build()
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
        assert_eq!("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ0ZXN0IHN1YmplY3QiLCJ2ZXIiOjEsInAiOiIifQ.hg2wMbWe-wesQVtA7wuJcRULjRphXLQwQTVYfQL3L7c", credential_provider.auth_token);
    }

    #[test]
    fn env_var_not_set() {
        let env_var_name = "TEST_ENV_VAR_CREDENTIAL_PROVIDER_NOT_SET";
        let _err_msg = format!("invalid argument: Env var {env_var_name} must be set");
        let e = CredentialProviderBuilder::from_environment_variable(env_var_name.to_string())
            .build()
            .unwrap_err();

        assert_eq!(e.to_string(), _err_msg);
    }

    #[test]
    fn env_var_empty_string() {
        let env_var_name = "TEST_ENV_VAR_CREDENTIAL_PROVIDER_EMPTY_STRING";
        env::set_var(env_var_name, "");
        let _err_msg = "client error: Could not parse token. Please ensure a valid token was entered correctly.";
        let e = CredentialProviderBuilder::from_environment_variable(env_var_name.to_string())
            .build()
            .unwrap_err();

        assert_eq!(e.to_string(), _err_msg);
    }

    #[test]
    fn valid_legacy_jwt() {
        // Token header
        // ------------
        // {
        //   "alg": "HS512"
        // }
        //
        // Token claims
        // ------------
        // {
        //   "c": "data plane endpoint",
        //   "cp": "control plane endpoint",
        //   "sub": "squirrel"
        // }
        let legacy_jwt = "eyJhbGciOiJIUzUxMiJ9.eyJzdWIiOiJzcXVpcnJlbCIsImNwIjoiY29udHJvbCBwbGFuZSBlbmRwb2ludCIsImMiOiJkYXRhIHBsYW5lIGVuZHBvaW50In0.zsTsEXFawetTCZI".to_owned();
        let credential_provider = CredentialProviderBuilder::from_string(legacy_jwt.clone())
            .build()
            .expect("should be able to build credential provider");
        assert_eq!(
            credential_provider.cache_endpoint,
            "https://data plane endpoint"
        );
        assert_eq!(
            credential_provider.control_endpoint,
            "https://control plane endpoint"
        );
        assert_eq!(credential_provider.auth_token, legacy_jwt);
    }

    #[test]
    fn empty_token() {
        let e = CredentialProviderBuilder::from_string("".to_string())
            .build()
            .unwrap_err();
        let _err_msg = "invalid argument: Auth token string cannot be empty".to_owned();
        assert_eq!(e.to_string(), _err_msg);
    }

    #[test]
    fn invalid_token() {
        let e = CredentialProviderBuilder::from_string("wfheofhriugheifweif".to_string())
            .build()
            .unwrap_err();
        let _err_msg =
            "client error: Could not parse token. Please ensure a valid token was entered correctly.".to_owned();
        assert_eq!(e.to_string(), _err_msg);
    }

    // #[test]
    // fn valid_no_c_cp_claims_jwt_with_endpoint_overrides() {
    //     // Token header
    //     // ------------
    //     // {
    //     //   "typ": "JWT",
    //     //   "alg": "HS256"
    //     // }
    //     //
    //     // Token claims
    //     // ------------
    //     // {
    //     //   "iat": 1516239022,
    //     //   "name": "John Doe",
    //     //   "sub": "abcd"
    //     // }
    //     let auth_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhYmNkIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.PTgxba";
    //     let credential_provider = CredentialProviderBuilder::from_string(auth_token.to_string())
    //         .with_cache_endpoint("cache.help.com".to_string())
    //         .with_control_endpoint("control.help.com".to_string())
    //         .build()
    //         .expect("should be able to get credentials");
    //
    //     assert_eq!(credential_provider.auth_token, auth_token.to_string());
    //     assert_eq!(
    //         credential_provider.control_endpoint,
    //         "https://control.help.com"
    //     );
    //     assert_eq!(credential_provider.cache_endpoint, "https://cache.help.com");
    // }

    // #[test]
    // fn valid_no_c_cp_claims_jwt_with_momento_endpoint_override() {
    //     // Token header
    //     // ------------
    //     // {
    //     //   "typ": "JWT",
    //     //   "alg": "HS256"
    //     // }
    //     //
    //     // Token claims
    //     // ------------
    //     // {
    //     //   "iat": 1516239022,
    //     //   "name": "John Doe",
    //     //   "sub": "abcd"
    //     // }
    //     let auth_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhYmNkIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.PTgxba";
    //     let credential_provider = CredentialProviderBuilder::from_string(auth_token.to_string())
    //         .with_momento_endpoint("help.com".to_string())
    //         .build()
    //         .expect("should be able to get credentials");
    //
    //     assert_eq!(credential_provider.auth_token, auth_token.to_string());
    //     assert_eq!(
    //         credential_provider.control_endpoint,
    //         "https://control.help.com"
    //     );
    //     assert_eq!(credential_provider.cache_endpoint, "https://cache.help.com");
    // }

    #[test]
    fn invalid_no_cache_claim_jwt_with_no_endpoint_override() {
        // Token header
        // ------------
        // {
        //   "typ": "JWT",
        //   "alg": "HS256"
        // }
        //
        // Token claims
        // ------------
        // {
        //   "iat": 1516239022,
        //   "name": "John Doe",
        //   "sub": "abcd"
        // }
        let auth_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhYmNkIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.PTgxba";
        let e = CredentialProvider::from_string(auth_token.to_string()).unwrap_err();
        let _err_msg =
            "invalid argument: auth token is missing cache endpoint and endpoint override is missing. One or the other must be provided".to_string();
        assert_eq!(e.to_string(), _err_msg);
    }

    // #[test]
    // fn invalid_no_control_claim_jwt_with_no_endpoint_override() {
    //     // Token header
    //     // ------------
    //     // {
    //     //   "typ": "JWT",
    //     //   "alg": "HS256"
    //     // }
    //     //
    //     // Token claims
    //     // ------------
    //     // {
    //     //   "iat": 1516239022,
    //     //   "name": "John Doe",
    //     //   "sub": "abcd"
    //     // }
    //     let auth_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhYmNkIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.PTgxba";
    //     let e = CredentialProvider::from_string(auth_token.to_string()).unwrap_err();
    //     let _err_msg =
    //     "invalid argument: auth token is missing control endpoint and endpoint override is missing. One or the other must be provided.".to_string();
    //     assert_eq!(e.to_string(), _err_msg);
    // }

    #[test]
    fn valid_v1_token() {
        let v1_token = "eyJlbmRwb2ludCI6Im1vbWVudG9fZW5kcG9pbnQiLCJhcGlfa2V5IjoiZXlKaGJHY2lPaUpJVXpJMU5pSjkuZXlKemRXSWlPaUowWlhOMElITjFZbXBsWTNRaUxDSjJaWElpT2pFc0luQWlPaUlpZlEuaGcyd01iV2Utd2VzUVZ0QTd3dUpjUlVMalJwaFhMUXdRVFZZZlFMM0w3YyJ9Cg==".to_string();

        let credential_provider = CredentialProviderBuilder::from_string(v1_token)
            .build()
            .expect("failed to parse token");
        assert_eq!(
            "https://control.momento_endpoint",
            credential_provider.control_endpoint
        );
        assert_eq!(
            "https://cache.momento_endpoint",
            credential_provider.cache_endpoint
        );
        assert_eq!("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ0ZXN0IHN1YmplY3QiLCJ2ZXIiOjEsInAiOiIifQ.hg2wMbWe-wesQVtA7wuJcRULjRphXLQwQTVYfQL3L7c", credential_provider.auth_token);
    }

    // #[test]
    // fn v1_token_with_overrides() {
    //     let v1_token = "eyJlbmRwb2ludCI6Im1vbWVudG9fZW5kcG9pbnQiLCJhcGlfa2V5IjoiZXlKaGJHY2lPaUpJVXpJMU5pSjkuZXlKemRXSWlPaUowWlhOMElITjFZbXBsWTNRaUxDSjJaWElpT2pFc0luQWlPaUlpZlEuaGcyd01iV2Utd2VzUVZ0QTd3dUpjUlVMalJwaFhMUXdRVFZZZlFMM0w3YyJ9Cg==".to_string();
    //
    //     let credential_provider = CredentialProviderBuilder::from_string(v1_token)
    //         .with_cache_endpoint("cache.foo.com".to_string())
    //         .with_control_endpoint("control.foo.com".to_string())
    //         .build()
    //         .expect("failed to parse token");
    //     assert_eq!("https://cache.foo.com", credential_provider.cache_endpoint);
    //     assert_eq!(
    //         "https://control.foo.com",
    //         credential_provider.control_endpoint
    //     );
    //     assert_eq!("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ0ZXN0IHN1YmplY3QiLCJ2ZXIiOjEsInAiOiIifQ.hg2wMbWe-wesQVtA7wuJcRULjRphXLQwQTVYfQL3L7c", credential_provider.auth_token);
    // }

    #[test]
    fn invalid_v1_token_json() {
        let auth_token = "eyJmb28iOiJiYXIifQo=";
        let e = CredentialProviderBuilder::from_string(auth_token.to_string())
            .build()
            .unwrap_err();
        let _err_msg =
            "client error: Could not parse token. Please ensure a valid token was entered correctly.".to_string();
        assert_eq!(e.to_string(), _err_msg);
    }
}
