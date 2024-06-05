use std::convert::TryFrom;

use momento_protos::store;
use momento_protos::store::store_value::Value;

use crate::{MomentoError, MomentoErrorCode};

#[derive(Debug, Clone, PartialEq)]
pub enum StoreValue {
    Bytes(Vec<u8>),
    String(String),
    Integer(i64),
    Double(f64),
}

impl From<Vec<u8>> for StoreValue {
    fn from(bytes: Vec<u8>) -> Self {
        StoreValue::Bytes(bytes)
    }
}

impl From<&[u8]> for StoreValue {
    fn from(bytes: &[u8]) -> Self {
        StoreValue::Bytes(bytes.to_vec())
    }
}

impl From<String> for StoreValue {
    fn from(string: String) -> Self {
        StoreValue::String(string)
    }
}

impl From<&str> for StoreValue {
    fn from(string: &str) -> Self {
        StoreValue::String(string.to_string())
    }
}

impl From<&String> for StoreValue {
    fn from(string: &String) -> Self {
        StoreValue::String(string.clone())
    }
}

impl From<i64> for StoreValue {
    fn from(integer: i64) -> Self {
        StoreValue::Integer(integer)
    }
}

impl From<&i64> for StoreValue {
    fn from(integer: &i64) -> Self {
        StoreValue::Integer(*integer)
    }
}

impl From<i32> for StoreValue {
    fn from(integer: i32) -> Self {
        StoreValue::Integer(integer as i64)
    }
}

impl From<&i32> for StoreValue {
    fn from(integer: &i32) -> Self {
        StoreValue::Integer(*integer as i64)
    }
}

impl From<f64> for StoreValue {
    fn from(double: f64) -> Self {
        StoreValue::Double(double)
    }
}

impl From<&f64> for StoreValue {
    fn from(double: &f64) -> Self {
        StoreValue::Double(*double)
    }
}

impl From<Value> for StoreValue {
    fn from(value: Value) -> Self {
        match value {
            Value::BytesValue(bytes) => StoreValue::Bytes(bytes),
            Value::StringValue(string) => StoreValue::String(string),
            Value::IntegerValue(integer) => StoreValue::Integer(integer),
            Value::DoubleValue(double) => StoreValue::Double(double),
        }
    }
}

impl From<StoreValue> for store::StoreValue {
    fn from(value: StoreValue) -> store::StoreValue {
        match value {
            StoreValue::Bytes(bytes) => store::StoreValue {
                value: Some(Value::BytesValue(bytes)),
            },
            StoreValue::String(string) => store::StoreValue {
                value: Some(Value::StringValue(string)),
            },
            StoreValue::Integer(integer) => store::StoreValue {
                value: Some(Value::IntegerValue(integer)),
            },
            StoreValue::Double(double) => store::StoreValue {
                value: Some(Value::DoubleValue(double)),
            },
        }
    }
}

impl std::fmt::Display for StoreValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<StoreValue> for Vec<u8> {
    type Error = MomentoError;

    fn try_from(value: StoreValue) -> Result<Self, Self::Error> {
        match value {
            StoreValue::Bytes(s) => Ok(s),
            _ => Err(MomentoError {
                message: "item is not a byte array".to_string(),
                error_code: MomentoErrorCode::TypeError,
                inner_error: None,
                details: None,
            }),
        }
    }
}

impl TryFrom<StoreValue> for String {
    type Error = MomentoError;

    fn try_from(value: StoreValue) -> Result<Self, Self::Error> {
        match value {
            StoreValue::String(s) => Ok(s),
            _ => Err(MomentoError {
                message: "item is not a utf-8 string".to_string(),
                error_code: MomentoErrorCode::TypeError,
                inner_error: None,
                details: None,
            }),
        }
    }
}

impl TryFrom<StoreValue> for i64 {
    type Error = MomentoError;

    fn try_from(value: StoreValue) -> Result<Self, Self::Error> {
        match value {
            StoreValue::Integer(s) => Ok(s),
            _ => Err(MomentoError {
                message: "item is not an i64".to_string(),
                error_code: MomentoErrorCode::TypeError,
                inner_error: None,
                details: None,
            }),
        }
    }
}

impl TryFrom<StoreValue> for f64 {
    type Error = MomentoError;

    fn try_from(value: StoreValue) -> Result<Self, Self::Error> {
        match value {
            StoreValue::Double(s) => Ok(s),
            _ => Err(MomentoError {
                message: "item is not an f64".to_string(),
                error_code: MomentoErrorCode::TypeError,
                inner_error: None,
                details: None,
            }),
        }
    }
}
