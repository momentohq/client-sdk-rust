use crate::{MomentoError, MomentoErrorCode, MomentoResult};
use derive_more::Display;

/// Trait for determining if an object expires.
pub trait Expiration {
    /// Returns true if the object expires.
    fn does_expire(&self) -> bool;
}

/// Represents the time remaining before an object expires.
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
    /// Returns the number of seconds until the object expires.
    /// If the object never expires, it will return u64::MAX.
    pub fn to_seconds(&self) -> u64 {
        self.valid_for_seconds
    }

    /// Creates a new instance of ExpiresIn that never expires.
    pub fn never() -> Self {
        Self {
            valid_for_seconds: u64::MAX,
        }
    }

    /// Creates a new instance of ExpiresIn from a number of seconds.
    pub fn seconds(valid_for_seconds: u64) -> Self {
        Self { valid_for_seconds }
    }

    /// Creates a new instance of ExpiresIn from a number of minutes.
    pub fn minutes(valid_for_minutes: u64) -> Self {
        Self {
            valid_for_seconds: valid_for_minutes * 60,
        }
    }

    /// Creates a new instance of ExpiresIn from a number of hours.
    pub fn hours(valid_for_hours: u64) -> Self {
        Self {
            valid_for_seconds: valid_for_hours * 3600,
        }
    }

    /// Creates a new instance of ExpiresIn from a number of days.
    pub fn days(valid_for_days: u64) -> Self {
        Self {
            valid_for_seconds: valid_for_days * 86400,
        }
    }

    /// Creates a new instance of ExpiresIn from an epoch timestamp.
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
            }),
        }
    }
}

/// Represents an expiration time for an object.
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
    /// Creates a new instance of ExpiresAt.
    /// If no epoch timestamp is provided, the object will never expire.
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

    /// Creates a new instance of ExpiresAt from an epoch timestamp.
    pub fn from_epoch(epoch_timestamp: u64) -> Self {
        Self {
            valid_until: epoch_timestamp,
        }
    }

    /// Returns the epoch timestamp of when the object expires.
    pub fn epoch(&self) -> u64 {
        self.valid_until
    }
}

#[cfg(test)]
mod tests {
    use crate::auth::Expiration;

    #[test]
    fn expires_in_never_is_valid() {
        let expires_in = super::ExpiresIn::never();
        assert_eq!(expires_in.to_seconds(), u64::MAX);
        assert!(!expires_in.does_expire());
    }

    #[test]
    fn expires_in_seconds_is_valid() {
        let expires_in = super::ExpiresIn::seconds(10);
        assert_eq!(expires_in.to_seconds(), 10);
        assert!(expires_in.does_expire());
    }

    #[test]
    fn expires_in_minutes_is_valid() {
        let expires_in = super::ExpiresIn::minutes(10);
        assert_eq!(expires_in.to_seconds(), 600);
        assert!(expires_in.does_expire());
    }

    #[test]
    fn expires_in_hours_is_valid() {
        let expires_in = super::ExpiresIn::hours(10);
        assert_eq!(expires_in.to_seconds(), 36000);
        assert!(expires_in.does_expire());
    }

    #[test]
    fn expires_in_days_is_valid() {
        let expires_in = super::ExpiresIn::days(10);
        assert_eq!(expires_in.to_seconds(), 864000);
        assert!(expires_in.does_expire());
    }

    #[test]
    fn expires_at_from_epoch_when_epoch_is_none() {
        let expires_at = super::ExpiresAt::new(None);
        assert_eq!(expires_at.epoch(), u64::MAX);
        assert!(!expires_at.does_expire());
    }

    #[test]
    fn expires_at_from_epoch_when_epoch_is_some() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let expires_at = super::ExpiresAt::new(Some(now));
        assert_eq!(expires_at.epoch(), now);
        assert!(expires_at.does_expire());
    }
}
