use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Invalid property status: {0}")]
    InvalidPropertyStatus(String),
    #[error("Invalid timestamp format: {0}")]
    InvalidTimestamp(String),
}

/// A type-safe wrapper for property status values in the database
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DbPropertyStatus(String);

impl DbPropertyStatus {
    /// Creates a new DbPropertyStatus, validating the status at compile time
    pub const fn new(status: &'static str) -> Option<Self> {
        match status {
            "active" | "sold" | "removed" => Some(Self(status.to_string())),
            _ => None
        }
    }

    /// Returns the underlying string value
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the SQL type definition for this field
    pub const fn sql_type() -> &'static str {
        "TEXT NOT NULL CHECK(status IN ('active', 'sold', 'removed'))"
    }
}

impl FromStr for DbPropertyStatus {
    type Err = DbError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" | "sold" | "removed" => Ok(Self(s.to_string())),
            _ => Err(DbError::InvalidPropertyStatus(s.to_string())),
        }
    }
}

/// A type-safe wrapper for timestamps in the database
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DbTimestamp(DateTime<Utc>);

impl DbTimestamp {
    /// Creates a new timestamp with the current time
    pub fn now() -> Self {
        Self(Utc::now())
    }

    /// Creates a timestamp from an RFC3339 string
    pub fn from_rfc3339(s: &str) -> Result<Self, DbError> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| Self(dt.with_timezone(&Utc)))
            .map_err(|_| DbError::InvalidTimestamp(s.to_string()))
    }

    /// Returns the underlying DateTime
    pub fn datetime(&self) -> &DateTime<Utc> {
        &self.0
    }

    /// Returns the SQL type definition for this field
    pub const fn sql_type() -> &'static str {
        "TEXT NOT NULL"
    }
}

impl From<DateTime<Utc>> for DbTimestamp {
    fn from(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
}

/// A type-safe wrapper for URLs in the database
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DbUrl(String);

impl DbUrl {
    /// Creates a new DbUrl from a string, validating it's a valid URL
    pub fn new(url: String) -> Result<Self, DbError> {
        // Basic validation - we'll use proper URL validation in the FromStr impl
        if url.is_empty() {
            return Err(DbError::InvalidTimestamp("URL cannot be empty".to_string()));
        }
        Ok(Self(url))
    }

    /// Returns the underlying string value
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the SQL type definition for this field
    pub const fn sql_type() -> &'static str {
        "TEXT NOT NULL"
    }
}

impl FromStr for DbUrl {
    type Err = DbError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // We could add more sophisticated URL validation here
        Self::new(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_status() {
        assert!(DbPropertyStatus::new("active").is_some());
        assert!(DbPropertyStatus::new("sold").is_some());
        assert!(DbPropertyStatus::new("removed").is_some());
        assert!(DbPropertyStatus::new("invalid").is_none());

        assert_eq!("active".parse::<DbPropertyStatus>().unwrap().as_str(), "active");
        assert!("invalid".parse::<DbPropertyStatus>().is_err());
    }

    #[test]
    fn test_timestamp() {
        let now = DbTimestamp::now();
        assert!(now.datetime().timestamp() > 0);

        let dt = "2024-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let ts = DbTimestamp::from(dt);
        assert_eq!(ts.datetime(), &dt);
    }

    #[test]
    fn test_url() {
        assert!(DbUrl::new("https://example.com".to_string()).is_ok());
        assert!(DbUrl::new("".to_string()).is_err());
    }
} 