use std::convert::{TryFrom, TryInto};

use momento_protos::store::store_value::Value;
use momento_protos::store::StoreValue;

use crate::{MomentoError, MomentoErrorCode};

/// An enum representing an item in a Momento store.
#[derive(Debug, Clone, PartialEq)]
pub enum StorageValue {
    /// A storage value containing a byte array.
    Bytes(Vec<u8>),
    /// A storage value containing a string.
    String(String),
    /// A storage value containing a 64-bit integer.
    Integer(i64),
    /// A storage value containing a double.
    Double(f64),
}

impl From<Vec<u8>> for StorageValue {
    fn from(bytes: Vec<u8>) -> Self {
        StorageValue::Bytes(bytes)
    }
}

impl From<&[u8]> for StorageValue {
    fn from(bytes: &[u8]) -> Self {
        StorageValue::Bytes(bytes.to_vec())
    }
}

impl From<String> for StorageValue {
    fn from(string: String) -> Self {
        StorageValue::String(string)
    }
}

impl From<&str> for StorageValue {
    fn from(string: &str) -> Self {
        StorageValue::String(string.to_string())
    }
}

impl From<&String> for StorageValue {
    fn from(string: &String) -> Self {
        StorageValue::String(string.clone())
    }
}

impl From<i64> for StorageValue {
    fn from(integer: i64) -> Self {
        StorageValue::Integer(integer)
    }
}

impl From<&i64> for StorageValue {
    fn from(integer: &i64) -> Self {
        StorageValue::Integer(*integer)
    }
}

impl From<i32> for StorageValue {
    fn from(integer: i32) -> Self {
        StorageValue::Integer(integer as i64)
    }
}

impl From<&i32> for StorageValue {
    fn from(integer: &i32) -> Self {
        StorageValue::Integer(*integer as i64)
    }
}

impl From<f64> for StorageValue {
    fn from(double: f64) -> Self {
        StorageValue::Double(double)
    }
}

impl From<&f64> for StorageValue {
    fn from(double: &f64) -> Self {
        StorageValue::Double(*double)
    }
}

impl From<Value> for StorageValue {
    fn from(value: Value) -> Self {
        match value {
            Value::BytesValue(bytes) => StorageValue::Bytes(bytes),
            Value::StringValue(string) => StorageValue::String(string),
            Value::IntegerValue(integer) => StorageValue::Integer(integer),
            Value::DoubleValue(double) => StorageValue::Double(double),
        }
    }
}

impl From<StorageValue> for StoreValue {
    fn from(value: StorageValue) -> StoreValue {
        match value {
            StorageValue::Bytes(bytes) => StoreValue {
                value: Some(Value::BytesValue(bytes)),
            },
            StorageValue::String(string) => StoreValue {
                value: Some(Value::StringValue(string)),
            },
            StorageValue::Integer(integer) => StoreValue {
                value: Some(Value::IntegerValue(integer)),
            },
            StorageValue::Double(double) => StoreValue {
                value: Some(Value::DoubleValue(double)),
            },
        }
    }
}

impl std::fmt::Display for StorageValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<StorageValue> for Vec<u8> {
    type Error = MomentoError;

    fn try_from(value: StorageValue) -> Result<Self, Self::Error> {
        match value {
            StorageValue::Bytes(s) => Ok(s),
            _ => Err(MomentoError {
                message: "item is not a byte array".to_string(),
                error_code: MomentoErrorCode::TypeError,
                inner_error: None,
                details: None,
            }),
        }
    }
}

impl TryFrom<StorageValue> for String {
    type Error = MomentoError;

    fn try_from(value: StorageValue) -> Result<Self, Self::Error> {
        match value {
            StorageValue::String(s) => Ok(s),
            _ => Err(MomentoError {
                message: "item is not a utf-8 string".to_string(),
                error_code: MomentoErrorCode::TypeError,
                inner_error: None,
                details: None,
            }),
        }
    }
}

impl TryFrom<StorageValue> for i64 {
    type Error = MomentoError;

    fn try_from(value: StorageValue) -> Result<Self, Self::Error> {
        match value {
            StorageValue::Integer(s) => Ok(s),
            _ => Err(MomentoError {
                message: "item is not an i64".to_string(),
                error_code: MomentoErrorCode::TypeError,
                inner_error: None,
                details: None,
            }),
        }
    }
}

impl TryFrom<StorageValue> for i32 {
    type Error = MomentoError;

    fn try_from(value: StorageValue) -> Result<Self, Self::Error> {
        match value {
            StorageValue::Integer(s) => match s.try_into() {
                Ok(converted) => Ok(converted),
                Err(_) => Err(MomentoError {
                    message: "item is out of range for i32".to_string(),
                    error_code: MomentoErrorCode::TypeError,
                    inner_error: None,
                    details: None,
                }),
            },
            _ => Err(MomentoError {
                message: "item is not an i64".to_string(),
                error_code: MomentoErrorCode::TypeError,
                inner_error: None,
                details: None,
            }),
        }
    }
}

impl TryFrom<StorageValue> for f64 {
    type Error = MomentoError;

    fn try_from(value: StorageValue) -> Result<Self, Self::Error> {
        match value {
            StorageValue::Double(s) => Ok(s),
            _ => Err(MomentoError {
                message: "item is not an f64".to_string(),
                error_code: MomentoErrorCode::TypeError,
                inner_error: None,
                details: None,
            }),
        }
    }
}

impl TryFrom<StorageValue> for f32 {
    type Error = MomentoError;

    fn try_from(value: StorageValue) -> Result<Self, Self::Error> {
        match value {
            StorageValue::Double(s) => {
                let converted: f32 = s as f32;
                if converted.is_infinite() && s.is_finite() {
                    Err(MomentoError {
                        message: "item is out of range for f32".to_string(),
                        error_code: MomentoErrorCode::TypeError,
                        inner_error: None,
                        details: None,
                    })
                } else {
                    Ok(converted)
                }
            }
            _ => Err(MomentoError {
                message: "item is not an f64".to_string(),
                error_code: MomentoErrorCode::TypeError,
                inner_error: None,
                details: None,
            }),
        }
    }
}
