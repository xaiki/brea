use chrono::{DateTime, Utc};
use std::fmt;
use thiserror::Error;
use sqlx::{Type, Encode, sqlite::{Sqlite, SqliteArgumentValue}, Database, Decode};
use serde::{Serialize, Deserialize};

#[derive(Error, Debug)]
pub enum TimestampError {
    #[error("Failed to parse timestamp: {0}")]
    Parse(#[from] chrono::ParseError),
}

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Invalid status: {0}")]
    InvalidStatus(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DbPropertyStatus(pub String);

impl DbPropertyStatus {
    pub fn new(status: &str) -> Self {
        Self(status.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DbPropertyStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<DB: Database> Type<DB> for DbPropertyStatus
where
    String: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        <String as Type<DB>>::type_info()
    }
}

impl<'q, DB: Database> Encode<'q, DB> for DbPropertyStatus
where
    String: Encode<'q, DB>,
{
    fn encode_by_ref(&self, buf: &mut <DB as sqlx::database::HasArguments<'q>>::ArgumentBuffer) -> sqlx::encode::IsNull {
        self.0.encode_by_ref(buf)
    }
}

impl<'r, DB: Database> Decode<'r, DB> for DbPropertyStatus
where
    String: Decode<'r, DB>,
{
    fn decode(value: <DB as sqlx::database::HasValueRef<'r>>::ValueRef) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let s = <String as Decode<DB>>::decode(value)?;
        Ok(Self(s))
    }
}

pub const STATUS_ACTIVE: &str = "active";
pub const STATUS_SOLD: &str = "sold";
pub const STATUS_REMOVED: &str = "removed";

pub static VALID_STATUSES: &[&str] = &[STATUS_ACTIVE, STATUS_SOLD, STATUS_REMOVED];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbTimestamp(DateTime<Utc>);

impl DbTimestamp {
    pub fn now() -> Self {
        Self(Utc::now())
    }
    
    pub fn from_rfc3339(s: &str) -> Result<Self, TimestampError> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| Self(dt.with_timezone(&Utc)))
            .map_err(TimestampError::Parse)
    }

    pub fn inner(&self) -> &DateTime<Utc> {
        &self.0
    }
}

impl fmt::Display for DbTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_rfc3339())
    }
}

// Type-safe column definitions
pub trait ColumnType {
    fn sql_type() -> &'static str;
}

impl ColumnType for DbPropertyStatus {
    fn sql_type() -> &'static str {
        "TEXT"
    }
}

impl ColumnType for DbTimestamp {
    fn sql_type() -> &'static str {
        "TEXT"
    }
}

impl Type<Sqlite> for DbTimestamp {
    fn type_info() -> <Sqlite as Database>::TypeInfo {
        <String as Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for DbTimestamp {
    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'q>>) -> sqlx::encode::IsNull {
        let s = self.0.to_rfc3339();
        buf.push(SqliteArgumentValue::Text(s.into()));
        sqlx::encode::IsNull::No
    }
}

impl<'r> Decode<'r, Sqlite> for DbTimestamp {
    fn decode(value: <Sqlite as sqlx::database::HasValueRef<'r>>::ValueRef) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let s = <String as Decode<Sqlite>>::decode(value)?;
        let dt = DateTime::parse_from_rfc3339(&s)?.with_timezone(&Utc);
        Ok(DbTimestamp(dt))
    }
} 