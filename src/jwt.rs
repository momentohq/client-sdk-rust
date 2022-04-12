use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::response::error::MomentoError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub c: String,
    pub cp: String,
}

pub fn decode_jwt(jwt: &str) -> Result<Claims, MomentoError> {
    if jwt.is_empty() {
        return Err(MomentoError::ClientSdkError(
            "Malformed Auth Token".to_string(),
        ));
    }
    let key = DecodingKey::from_secret("".as_ref());
    let mut validation = Validation::new(Algorithm::HS256);
    validation.required_spec_claims.clear();
    validation.required_spec_claims.insert("sub".to_string());
    validation.required_spec_claims.insert("c".to_string());
    validation.required_spec_claims.insert("cp".to_string());
    validation.validate_exp = false;
    validation.insecure_disable_signature_validation();
    match decode(jwt, &key, &validation) {
        Ok(token) => Ok(token.claims),
        Err(_) => Err(MomentoError::ClientSdkError(
            "Please make sure entered auth token matches your auth token".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::response::error::MomentoError;

    use super::decode_jwt;

    #[test]
    fn valid_jwt() {
        let valid_jwt = "eyJhbGciOiJIUzUxMiJ9.eyJzdWIiOiJzcXVpcnJlbCIsImNwIjoiY29udHJvbCBwbGFuZSBlbmRwb2ludCIsImMiOiJkYXRhIHBsYW5lIGVuZHBvaW50In0.zsTsEXFawetTCZI";
        let claims = decode_jwt(valid_jwt).unwrap();
        assert_eq!(claims.c, "data plane endpoint");
        assert_eq!(claims.cp, "control plane endpoint");
    }

    #[test]
    fn empty_jwt() {
        let e = decode_jwt("").unwrap_err();
        let _err_msg = "Malformed Auth Token".to_owned();
        assert!(matches!(e, MomentoError::ClientSdkError(_err_msg)));
    }

    #[test]
    fn invalid_jwt() {
        let e = decode_jwt("wfheofhriugheifweif").unwrap_err();
        let _err_msg = "Please make sure entered auth token matches your auth token".to_owned();
        assert!(matches!(e, MomentoError::ClientSdkError(_err_msg)));
    }
}
