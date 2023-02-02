use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::response::MomentoError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub c: Option<String>,
    pub cp: Option<String>,
}

pub fn decode_jwt(jwt: &str, momento_endpoint: Option<String>) -> Result<Claims, MomentoError> {
    if jwt.is_empty() {
        return Err(MomentoError::InvalidArgument {
            description: "missing auth token".into(),
            source: None,
        });
    }
    let key = DecodingKey::from_secret("".as_ref());
    let mut validation = Validation::new(Algorithm::HS256);
    validation.required_spec_claims.clear();
    validation.required_spec_claims.insert("sub".to_string());

    validation.validate_exp = false;
    validation.insecure_disable_signature_validation();

    let token = decode(jwt, &key, &validation).map_err(token_parsing_error)?;
    let token_claims: Claims = token.claims;

    // If Momento Endpoint is not provided, then `c` and `cp` claims must be present.
    // If Momento Endpoint is present then that always takes precedence over the c and cp
    // claims in the JWT, hence, there is no need to look for all possibilities.
    if momento_endpoint.is_none() && (token_claims.c.is_none() || token_claims.cp.is_none()) {
        Err(MomentoError::InvalidArgument {
            description: "momento endpoint is none and auth token is missing endpoints. One or the other must be provided".into(),
            source: None,
        })
    } else {
        Ok(token_claims)
    }
}

fn token_parsing_error(e: jsonwebtoken::errors::Error) -> MomentoError {
    MomentoError::ClientSdkError {
        description: "Could not parse token. Please ensure a valid token was entered correctly."
            .into(),
        source: crate::ErrorSource::Unknown(Box::new(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::decode_jwt;

    #[test]
    fn valid_jwt() {
        let valid_jwt = "eyJhbGciOiJIUzUxMiJ9.eyJzdWIiOiJzcXVpcnJlbCIsImNwIjoiY29udHJvbCBwbGFuZSBlbmRwb2ludCIsImMiOiJkYXRhIHBsYW5lIGVuZHBvaW50In0.zsTsEXFawetTCZI";
        let claims = decode_jwt(valid_jwt, None).expect("couldn't decode jwt");
        assert_eq!(claims.c.expect("c wasn't present"), "data plane endpoint");
        assert_eq!(
            claims.cp.expect("cp wasn't present"),
            "control plane endpoint"
        );
    }

    #[test]
    fn empty_jwt() {
        let e = decode_jwt("", None).unwrap_err();
        let _err_msg = "Malformed Auth Token".to_owned();
        assert!(matches!(e.to_string(), _err_msg));
    }

    #[test]
    fn invalid_jwt() {
        let e = decode_jwt("wfheofhriugheifweif", None).unwrap_err();
        let _err_msg =
            "Could not parse token. Please ensure a valid token was entered correctly.".to_owned();
        assert!(matches!(e.to_string(), _err_msg));
    }

    #[test]
    fn validate_no_c_cp_claims_jwt_with_momento_endpoint() {
        let claims = decode_jwt("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhYmNkIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.PTgxba", Some("help.com".to_string())).expect("result not returned from jwt");
        assert_eq!(claims.sub, "abcd");
        assert!(claims.c.is_none());
        assert!(claims.cp.is_none());
    }

    #[test]
    fn invalid_no_c_cp_claims_jwt_with_no_momento_endpoint() {
        let e = decode_jwt("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJhYmNkIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.PTgxba", None).unwrap_err();
        let _err_msg =
            "Could not parse token. Please ensure a valid token was entered correctly.".to_owned();
        assert!(matches!(e.to_string(), _err_msg));
    }
}
