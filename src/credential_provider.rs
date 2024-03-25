use crate::response::{MomentoErrorCode, SdkError};
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
    pub(crate) auth_token: String,
    pub(crate) control_endpoint: String,
    pub(crate) cache_endpoint: String,
    pub(crate) token_endpoint: String,
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
    /// let credential_provider = CredentialProvider::from_env_var("MOMENTO_API_KEY".to_string())
    ///     .expect("MOMENTO_API_KEY must be set");
    /// # })
    /// ```
    ///
    pub fn from_env_var(env_var_name: String) -> MomentoResult<CredentialProvider> {
        let token_to_process = match env::var(&env_var_name) {
            Ok(auth_token) => auth_token,
            Err(e) => {
                return Err(MomentoError::InvalidArgument(SdkError {
                    message: format!("Env var {env_var_name} must be set").into(),
                    error_code: MomentoErrorCode::InvalidArgumentError,
                    inner_error: Some(crate::ErrorSource::Unknown(Box::new(e))),
                    details: None,
                }));
            }
        };

        decode_auth_token(token_to_process)
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
    /// let credential_provider = match(CredentialProvider::from_string(api_key.to_string())) {
    ///    Ok(credential_provider) => credential_provider,
    ///    Err(e) => {
    ///         println!("Error while creating credential provider: {}", e);
    ///         return // probably you will do something else here
    ///    }
    /// };
    ///
    /// # ()
    /// # })
    /// #
    /// }
    /// ```
    pub fn from_string(auth_token: String) -> MomentoResult<CredentialProvider> {
        let token_to_process = {
            if auth_token.is_empty() {
                return Err(MomentoError::InvalidArgument(SdkError {
                    message: "Auth token string cannot be empty".into(),
                    error_code: MomentoErrorCode::InvalidArgumentError,
                    inner_error: None,
                    details: None,
                }));
            };
            auth_token
        };

        decode_auth_token(token_to_process)
    }

    pub fn base_endpoint(mut self, endpoint: &str) -> CredentialProvider {
        self.control_endpoint = https_endpoint(get_control_endpoint(endpoint));
        self.cache_endpoint = https_endpoint(get_cache_endpoint(endpoint));
        self.token_endpoint = https_endpoint(get_token_endpoint(endpoint));
        self
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

fn decode_auth_token(auth_token: String) -> MomentoResult<CredentialProvider> {
    match base64::engine::general_purpose::URL_SAFE.decode(&auth_token) {
        Ok(auth_token_bytes) => process_v1_token(auth_token_bytes),
        Err(_) => process_jwt_token(auth_token),
    }
}

fn process_v1_token(auth_token_bytes: Vec<u8>) -> MomentoResult<CredentialProvider> {
    let json: V1Token =
        serde_json::from_slice(&auth_token_bytes).map_err(|e| token_parsing_error(Box::new(e)))?;

    Ok(CredentialProvider {
        auth_token: json.api_key,
        cache_endpoint: https_endpoint(get_cache_endpoint(&json.endpoint)),
        control_endpoint: https_endpoint(get_control_endpoint(&json.endpoint)),
        token_endpoint: https_endpoint(get_token_endpoint(&json.endpoint)),
    })
}

fn process_jwt_token(auth_token: String) -> MomentoResult<CredentialProvider> {
    let key = DecodingKey::from_secret(b"");
    let mut validation = Validation::new(Algorithm::HS256);
    validation.required_spec_claims.clear();
    validation.required_spec_claims.insert("sub".to_string());

    validation.validate_exp = false;
    validation.insecure_disable_signature_validation();

    let token =
        decode(&auth_token, &key, &validation).map_err(|e| token_parsing_error(Box::new(e)))?;
    let token_claims: JwtClaims = token.claims;

    let cache_endpoint = token_claims.cache_endpoint
    .ok_or_else(|| MomentoError::InvalidArgument(SdkError {
        message: "auth token is missing cache endpoint and endpoint override is missing. One or the other must be provided".into(),
        error_code: MomentoErrorCode::InvalidArgumentError,
        inner_error: None,
        details: None
    }))?;
    let control_endpoint = token_claims.control_endpoint
    .ok_or_else(|| MomentoError::InvalidArgument(SdkError {
        message: "auth token is missing control endpoint and endpoint override is missing. One or the other must be provided.".into(),
        error_code: MomentoErrorCode::InvalidArgumentError,
        inner_error: None,
        details: None
    }))?;
    let token_endpoint = cache_endpoint.clone();

    Ok(CredentialProvider {
        auth_token,
        cache_endpoint: https_endpoint(cache_endpoint),
        control_endpoint: https_endpoint(control_endpoint),
        token_endpoint: https_endpoint(token_endpoint),
    })
}

fn get_cache_endpoint(endpoint: &str) -> String {
    format!("cache.{endpoint}")
}

fn get_control_endpoint(endpoint: &str) -> String {
    format!("control.{endpoint}")
}

fn get_token_endpoint(endpoint: &str) -> String {
    format!("token.{endpoint}")
}

fn https_endpoint(hostname: String) -> String {
    format!("https://{}", hostname)
}

fn token_parsing_error(e: Box<dyn std::error::Error + Send + Sync>) -> MomentoError {
    MomentoError::ClientSdkError(SdkError {
        message: "Could not parse token. Please ensure a valid token was entered correctly.".into(),
        error_code: MomentoErrorCode::InvalidArgumentError,
        inner_error: Some(crate::ErrorSource::Unknown(e)),
        details: None,
    })
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
        let credential_provider = CredentialProvider::from_env_var(env_var_name.to_string())
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
        let _err_msg = format!("InvalidArgument: Env var {env_var_name} must be set");
        let e = CredentialProvider::from_env_var(env_var_name.to_string()).unwrap_err();

        assert_eq!(e.to_string(), _err_msg);
    }

    #[test]
    fn env_var_empty_string() {
        let env_var_name = "TEST_ENV_VAR_CREDENTIAL_PROVIDER_EMPTY_STRING";
        env::set_var(env_var_name, "");
        let _err_msg = "ClientSdkError: Could not parse token. Please ensure a valid token was entered correctly.";
        let e = CredentialProvider::from_env_var(env_var_name.to_string()).unwrap_err();

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
        let credential_provider = CredentialProvider::from_string(legacy_jwt.clone())
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
        let e = CredentialProvider::from_string("".to_string()).unwrap_err();
        let _err_msg = "InvalidArgument: Auth token string cannot be empty".to_owned();
        assert_eq!(e.to_string(), _err_msg);
    }

    #[test]
    fn invalid_token() {
        let e = CredentialProvider::from_string("wfheofhriugheifweif".to_string()).unwrap_err();
        let _err_msg =
            "ClientSdkError: Could not parse token. Please ensure a valid token was entered correctly.".to_owned();
        assert_eq!(e.to_string(), _err_msg);
    }

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
            "InvalidArgument: auth token is missing cache endpoint and endpoint override is missing. One or the other must be provided".to_string();
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
        assert_eq!("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ0ZXN0IHN1YmplY3QiLCJ2ZXIiOjEsInAiOiIifQ.hg2wMbWe-wesQVtA7wuJcRULjRphXLQwQTVYfQL3L7c", credential_provider.auth_token);
    }

    #[test]
    fn v1_token_with_base_endpoint_override() -> MomentoResult<()> {
        let v1_token = "eyJlbmRwb2ludCI6Im1vbWVudG9fZW5kcG9pbnQiLCJhcGlfa2V5IjoiZXlKaGJHY2lPaUpJVXpJMU5pSjkuZXlKemRXSWlPaUowWlhOMElITjFZbXBsWTNRaUxDSjJaWElpT2pFc0luQWlPaUlpZlEuaGcyd01iV2Utd2VzUVZ0QTd3dUpjUlVMalJwaFhMUXdRVFZZZlFMM0w3YyJ9Cg==".to_string();

        let credential_provider =
            CredentialProvider::from_string(v1_token)?.base_endpoint("foo.com");
        assert_eq!("https://cache.foo.com", credential_provider.cache_endpoint);
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
        let e = CredentialProvider::from_string(auth_token.to_string()).unwrap_err();
        let _err_msg =
            "ClientSdkError: Could not parse token. Please ensure a valid token was entered correctly.".to_string();
        assert_eq!(e.to_string(), _err_msg);
    }
}
