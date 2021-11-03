use serde::{Serialize, Deserialize};
use jsonwebtoken::{dangerous_insecure_decode};

use crate::response::momento_error::MomentoError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub c: String,
    pub cp: String
}

pub fn decode_jwt(jwt: &str) -> Result<Claims, MomentoError> {
    let token = dangerous_insecure_decode::<Claims>(jwt)?;
    Ok(token.claims)
}