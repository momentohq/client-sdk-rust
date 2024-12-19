use crate::{MomentoError, MomentoErrorCode, MomentoResult};
use derive_more::Display;

pub trait Expiration {
    fn does_expire(&self) -> bool;
}

#[derive(Debug, Display, PartialEq, Eq, Clone)]
pub struct ExpiresIn {
    // u64::MAX means it never expires
    valid_for_seconds: u64,
}

impl Expiration for ExpiresIn {
    fn does_expire(&self) -> bool {
        self.valid_for_seconds != u64::MAX
    }
}

impl ExpiresIn {
    pub fn to_seconds(&self) -> u64 {
        self.valid_for_seconds
    }

    pub fn never() -> Self {
        Self {
            valid_for_seconds: u64::MAX,
        }
    }

    pub fn seconds(valid_for_seconds: u64) -> Self {
        Self { valid_for_seconds }
    }

    pub fn minutes(valid_for_minutes: u64) -> Self {
        Self {
            valid_for_seconds: valid_for_minutes * 60,
        }
    }

    pub fn hours(valid_for_hours: u64) -> Self {
        Self {
            valid_for_seconds: valid_for_hours * 3600,
        }
    }

    pub fn days(valid_for_days: u64) -> Self {
        Self {
            valid_for_seconds: valid_for_days * 86400,
        }
    }

    pub fn epoch(expires_by: u64) -> MomentoResult<Self> {
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(duration) => {
                let current_epoch = duration.as_secs();
                let seconds_until_epoch = expires_by - current_epoch;
                Ok(Self {
                    valid_for_seconds: seconds_until_epoch,
                })
            }
            Err(_) => Err(MomentoError {
                message: "Unable to convert epoch timestamp into valid expiry".into(),
                error_code: MomentoErrorCode::InvalidArgumentError,
                inner_error: None,
                details: None,
            }),
        }
    }
}

#[derive(Debug, Display, PartialEq, Eq, Clone)]
pub struct ExpiresAt {
    // u64::MAX means it never expires
    valid_until: u64,
}

impl Expiration for ExpiresAt {
    fn does_expire(&self) -> bool {
        self.valid_until != u64::MAX
    }
}

impl ExpiresAt {
    pub fn new(epoch_timestamp: Option<u64>) -> Self {
        match epoch_timestamp {
            Some(epoch_timestamp) => Self {
                valid_until: epoch_timestamp,
            },
            None => Self {
                valid_until: u64::MAX,
            },
        }
    }

    pub fn from_epoch(epoch_timestamp: u64) -> Self {
        Self {
            valid_until: epoch_timestamp,
        }
    }

    pub fn epoch(&self) -> u64 {
        self.valid_until
    }
}
