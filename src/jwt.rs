use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;

use crate::response::error::MomentoError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub c: Option<String>,
    pub cp: Option<String>,
}

pub fn decode_jwt(jwt: &str, momento_endpoint: Option<String>) -> Result<Claims, MomentoError> {
    if jwt.is_empty() {
        return Err(MomentoError::ClientSdkError(
            "Malformed Auth Token".to_string(),
        ));
    }
    let key = DecodingKey::from_secret("".as_ref());
    let mut validation = Validation::new(Algorithm::HS256);
    validation.required_spec_claims.clear();
    validation.required_spec_claims.insert("sub".to_string());

    validation.validate_exp = false;
    validation.insecure_disable_signature_validation();

    match decode(jwt, &key, &validation) {
        Ok(token) => {
            let token_claims: Claims = token.claims;

            if !is_valid_endpoint_claims(token_claims.borrow()) && momento_endpoint.is_none() {
                log::debug!("Momento Endpoint is none and auth token is missing endpoints");
                Err(token_parsing_error())
            } else {
                Ok(token_claims)
            }
        }
        Err(_) => Err(token_parsing_error()),
    }
}

fn is_valid_endpoint_claims(claims: &Claims) -> bool {
    // Endpoints in a token are valid only if both are present
    claims.c.is_some() && claims.cp.is_some()
}

fn token_parsing_error() -> MomentoError {
    MomentoError::ClientSdkError(
        "Could not parse token. Please ensure a valid token was entered correctly.".to_string(),
    )
}

#[cfg(test)]
mod tests {
    use crate::response::error::MomentoError;

    use super::decode_jwt;

    #[test]
    fn valid_jwt() {
        let valid_jwt = "eyJhbGciOiJIUzUxMiJ9.eyJzdWIiOiJzcXVpcnJlbCIsImNwIjoiY29udHJvbCBwbGFuZSBlbmRwb2ludCIsImMiOiJkYXRhIHBsYW5lIGVuZHBvaW50In0.zsTsEXFawetTCZI";
        let claims = decode_jwt(valid_jwt, None).unwrap();
        assert_eq!(claims.c.unwrap(), "data plane endpoint");
        assert_eq!(claims.cp.unwrap(), "control plane endpoint");
    }

    #[test]
    fn empty_jwt() {
        let e = decode_jwt("", None).unwrap_err();
        let _err_msg = "Malformed Auth Token".to_owned();
        assert!(matches!(e, MomentoError::ClientSdkError(_err_msg)));
    }

    #[test]
    fn invalid_jwt() {
        let e = decode_jwt("wfheofhriugheifweif", None).unwrap_err();
        let _err_msg =
            "Could not parse token. Please ensure a valid token was entered correctly.".to_owned();
        assert!(matches!(e, MomentoError::ClientSdkError(_err_msg)));
    }

    #[test]
    fn validate_no_c_cp_claims_jwt_with_momento_endpoint() {
        let claims = decode_jwt("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhYmNkIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.PTgxba", Some("help.com".to_string())).unwrap();
        assert_eq!(claims.sub, "abcd");
        assert!(claims.c.is_none());
        assert!(claims.cp.is_none());
    }

    #[test]
    fn invalid_no_c_cp_claims_jwt_with_no_momento_endpoint() {
        let e = decode_jwt("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhYmNkIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.PTgxba", None).unwrap_err();
        let _err_msg =
            "Could not parse token. Please ensure a valid token was entered correctly.".to_owned();
        assert!(matches!(e, MomentoError::ClientSdkError(_err_msg)));
    }

    #[test]
    fn invalid_when_only_one_endpoint_claim_present() {
        let token_only_cp_claim = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwiY3AiOiJjb250cm9sLmNvbSIsImlhdCI6MTUxNjIzOTAyMn0.IqldLffaca24NwjjNckNWS6PBTEUbsyeGGVd7ciASb0".to_string();
        let _err_msg =
            "Could not parse token. Please ensure a valid token was entered correctly.".to_owned();

        let e_no_c_claim = decode_jwt(token_only_cp_claim.as_str(), None).unwrap_err();
        assert!(matches!(
            e_no_c_claim,
            MomentoError::ClientSdkError(_err_msg)
        ));

        let token_only_c_claim: String = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwiYyI6ImNhY2hlLmNvbSIsImlhdCI6MTUxNjIzOTAyMn0.bKycvlL5OdixDyPvGJumBip0g4O83pbJxBujkG1Ju44".to_string();
        let e_no_cp_claim = decode_jwt(token_only_c_claim.as_str(), None).unwrap_err();
        assert!(matches!(
            e_no_cp_claim,
            MomentoError::ClientSdkError(_err_msg)
        ));
    }
}
