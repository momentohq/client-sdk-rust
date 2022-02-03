use jsonwebtoken::dangerous_insecure_decode;
use serde::{Deserialize, Serialize};

use crate::response::error::MomentoError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub c: String,
    pub cp: String,
}

pub fn decode_jwt(jwt: &str) -> Result<Claims, MomentoError> {
    if jwt.is_empty() {
        return Err(MomentoError::ClientSdkError(
            "Malformed Auth Token".to_string(),
        ));
    }
    let token = dangerous_insecure_decode::<Claims>(jwt)?;
    Ok(token.claims)
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
    fn invalid_jwt() {
        let e = decode_jwt("").unwrap_err();
        let _err_msg = "Failed to parse Auth Token".to_owned();
        assert!(matches!(e, MomentoError::ClientSdkError(_err_msg)));
    }
}
