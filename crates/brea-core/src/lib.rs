use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use std::path::PathBuf;
use url::Url;
use std::str::FromStr;
use colored::Colorize;
use crate::db::types::{DbPropertyStatus, DbTimestamp, STATUS_ACTIVE, STATUS_SOLD, STATUS_REMOVED};
use std::fmt;

pub mod db;
mod graph;
mod display;
pub use db::Database;
pub use graph::PriceHistory;

pub type Result<T> = std::result::Result<T, BreaError>;

#[derive(Debug, thiserror::Error)]
pub enum BreaError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Scraping error: {0}")]
    Scraping(String),
    #[error("Invalid property type: {0}")]
    InvalidPropertyType(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PropertyType {
    House,
    Apartment,
    Land,
    Ph,
    Local,
    Field,
    Garage,
    CommercialPremises,
    Warehouse,
    Hotel,
    SpecialBusiness,
    Office,
    CountryHouse,
}

impl sqlx::Type<sqlx::Sqlite> for PropertyType {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for PropertyType {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> std::result::Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let text = <&str as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        match text.to_lowercase().as_str() {
            "house" | "houses" | "casa" | "casas" => Ok(PropertyType::House),
            "apartment" | "apartments" | "departamento" | "departamentos" => Ok(PropertyType::Apartment),
            "land" | "lands" | "terreno" | "terrenos" => Ok(PropertyType::Land),
            "ph" => Ok(PropertyType::Ph),
            "local" | "locales" => Ok(PropertyType::Local),
            "field" | "fields" | "campo" | "campos" => Ok(PropertyType::Field),
            "garage" | "garages" | "cochera" | "cocheras" => Ok(PropertyType::Garage),
            "commercial" | "commercial-premises" | "fondo-comercio" | "commercial premises" => Ok(PropertyType::CommercialPremises),
            "warehouse" | "warehouses" | "galpon" | "galpones" => Ok(PropertyType::Warehouse),
            "hotel" | "hotels" => Ok(PropertyType::Hotel),
            "special-business" | "special-businesses" | "negocio-especial" | "special business" => Ok(PropertyType::SpecialBusiness),
            "office" | "offices" | "oficina" | "oficinas" => Ok(PropertyType::Office),
            "country-house" | "country-houses" | "quinta" | "quintas" => Ok(PropertyType::CountryHouse),
            _ => Err(format!("Unknown property type: {}", text).into()),
        }
    }
}

impl sqlx::Encode<'_, sqlx::Sqlite> for PropertyType {
    fn encode_by_ref(&self, args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'_>>) -> sqlx::encode::IsNull {
        let text = match self {
            PropertyType::House => "house",
            PropertyType::Apartment => "apartment",
            PropertyType::Land => "land",
            PropertyType::Ph => "ph",
            PropertyType::Local => "local",
            PropertyType::Field => "field",
            PropertyType::Garage => "garage",
            PropertyType::CommercialPremises => "commercial premises",
            PropertyType::Warehouse => "warehouse",
            PropertyType::Hotel => "hotel",
            PropertyType::SpecialBusiness => "special business",
            PropertyType::Office => "office",
            PropertyType::CountryHouse => "country house",
        };
        args.push(sqlx::sqlite::SqliteArgumentValue::Text(text.into()));
        sqlx::encode::IsNull::No
    }
}

impl std::fmt::Display for PropertyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropertyType::House => write!(f, "House"),
            PropertyType::Apartment => write!(f, "Apartment"),
            PropertyType::Land => write!(f, "Land"),
            PropertyType::Ph => write!(f, "PH"),
            PropertyType::Local => write!(f, "Local"),
            PropertyType::Field => write!(f, "Field"),
            PropertyType::Garage => write!(f, "Garage"),
            PropertyType::CommercialPremises => write!(f, "Commercial Premises"),
            PropertyType::Warehouse => write!(f, "Warehouse"),
            PropertyType::Hotel => write!(f, "Hotel"),
            PropertyType::SpecialBusiness => write!(f, "Special Business"),
            PropertyType::Office => write!(f, "Office"),
            PropertyType::CountryHouse => write!(f, "Country House"),
        }
    }
}

impl FromStr for PropertyType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "house" | "houses" | "casa" | "casas" => Ok(PropertyType::House),
            "apartment" | "apartments" | "departamento" | "departamentos" => Ok(PropertyType::Apartment),
            "land" | "lands" | "terreno" | "terrenos" => Ok(PropertyType::Land),
            "ph" => Ok(PropertyType::Ph),
            "local" | "locales" => Ok(PropertyType::Local),
            "field" | "fields" | "campo" | "campos" => Ok(PropertyType::Field),
            "garage" | "garages" | "cochera" | "cocheras" => Ok(PropertyType::Garage),
            "commercial" | "commercial-premises" | "fondo-comercio" | "commercial premises" => Ok(PropertyType::CommercialPremises),
            "warehouse" | "warehouses" | "galpon" | "galpones" => Ok(PropertyType::Warehouse),
            "hotel" | "hotels" => Ok(PropertyType::Hotel),
            "special-business" | "special-businesses" | "negocio-especial" | "special business" => Ok(PropertyType::SpecialBusiness),
            "office" | "offices" | "oficina" | "oficinas" => Ok(PropertyType::Office),
            "country-house" | "country-houses" | "quinta" | "quintas" => Ok(PropertyType::CountryHouse),
            _ => Err(format!(
                "Invalid property type: {}. Valid options are: house/casa, apartment/departamento, land/terreno, ph, local, field/campo, garage/cochera, commercial/fondo-comercio, warehouse/galpon, hotel, special-business/negocio-especial, office/oficina, country-house/quinta",
                s
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PropertyStatus {
    Active,
    Sold,
    Removed,
}

impl FromStr for PropertyStatus {
    type Err = BreaError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(PropertyStatus::Active),
            "sold" => Ok(PropertyStatus::Sold),
            "removed" => Ok(PropertyStatus::Removed),
            _ => {
                // Check if it's a timestamp
                if chrono::DateTime::parse_from_rfc3339(s).is_ok() {
                    // If it's a timestamp, default to Active
                    Ok(PropertyStatus::Active)
                } else {
                    Err(BreaError::Database(sqlx::Error::ColumnDecode {
                        index: "status".into(),
                        source: format!("Unknown property status: {}", s).into(),
                    }))
                }
            }
        }
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for PropertyStatus {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'r>) -> std::result::Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let text = <&str as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        Self::from_str(text).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
}

impl sqlx::Type<sqlx::Sqlite> for PropertyStatus {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

impl sqlx::Encode<'_, sqlx::Sqlite> for PropertyStatus {
    fn encode_by_ref(&self, args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'_>>) -> sqlx::encode::IsNull {
        let text = match self {
            PropertyStatus::Active => "active",
            PropertyStatus::Sold => "sold",
            PropertyStatus::Removed => "removed",
        };
        args.push(sqlx::sqlite::SqliteArgumentValue::Text(text.into()));
        sqlx::encode::IsNull::No
    }
}

impl std::fmt::Display for PropertyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropertyStatus::Active => write!(f, "Active"),
            PropertyStatus::Sold => write!(f, "Sold"),
            PropertyStatus::Removed => write!(f, "Removed"),
        }
    }
}

// Property with SQLx support
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Property {
    pub id: i64,
    pub external_id: String,
    pub source: String,
    pub property_type: Option<String>,
    pub district: String,
    pub title: String,
    pub description: Option<String>,
    pub price_usd: f64,
    pub address: String,
    pub covered_size: Option<f64>,
    pub rooms: Option<i32>,
    pub antiquity: Option<i32>,
    pub url: String,
    pub status: DbPropertyStatus,
    pub created_at: DbTimestamp,
    pub updated_at: DbTimestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PropertyImage {
    pub id: i64,
    pub property_id: i64,
    pub url: String,
    pub local_path: String,
    pub hash: Vec<u8>,
    pub created_at: DbTimestamp,
    pub updated_at: DbTimestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PropertyPriceHistory {
    pub id: i64,
    pub property_id: i64,
    pub price_usd: f64,
    pub observed_at: DbTimestamp,
}

// Custom serialization for PathBuf
mod path_buf_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::path::PathBuf;

    pub fn serialize<S>(path: &PathBuf, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        path.to_str()
            .ok_or_else(|| serde::ser::Error::custom("Path contains invalid UTF-8 characters"))
            .and_then(|s| s.serialize(serializer))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(PathBuf::from)
    }
}

pub struct PropertyDisplay {
    pub property: Property,
    pub price_history: Option<Vec<(f64, DateTime<Utc>)>>,
}

impl PropertyDisplay {
    pub fn new(property: Property, price_history: Vec<(f64, DateTime<Utc>)>) -> Self {
        Self {
            property,
            price_history: Some(price_history),
        }
    }

    pub fn format(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("Property: {} - {}\n", self.property.title, self.property.district));
        output.push_str(&format!("Address: {}\n", self.property.address));
        output.push_str(&format!("Price: ${:.2}\n", self.property.price_usd));
        if let Some(size) = self.property.covered_size {
            output.push_str(&format!("Size: {:.1} m²\n", size));
        }
        if let Some(rooms) = self.property.rooms {
            output.push_str(&format!("Rooms: {}\n", rooms));
        }
        if let Some(antiquity) = self.property.antiquity {
            output.push_str(&format!("Antiquity: {} years\n", antiquity));
        }
        output.push_str(&format!("Status: {}\n", self.property.status));
        output
    }
}

impl fmt::Display for PropertyDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_serialization() {
        let property = Property {
            id: 1,
            external_id: "test-123".to_string(),
            source: "test".to_string(),
            property_type: Some("apartment".to_string()),
            district: "Test District".to_string(),
            title: "Test Property".to_string(),
            description: Some("Test description".to_string()),
            price_usd: 100000.0,
            address: "123 Test St".to_string(),
            covered_size: Some(100.0),
            rooms: Some(2),
            antiquity: Some(5),
            url: "https://example.com/test".to_string(),
            status: DbPropertyStatus::new("active"),
            created_at: DbTimestamp::from_rfc3339("2024-03-20T00:00:00Z").unwrap(),
            updated_at: DbTimestamp::from_rfc3339("2024-03-20T00:00:00Z").unwrap(),
        };

        let json = serde_json::to_string(&property).unwrap();
        let deserialized: Property = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.external_id, "test-123");
    }

    #[test]
    fn test_property_image_serialization() {
        let image = PropertyImage {
            id: 1,
            property_id: 1,
            url: "https://example.com/image.jpg".to_string(),
            local_path: "/tmp/images/test.jpg".to_string(),
            hash: vec![1, 2, 3, 4],
            created_at: DbTimestamp::from_rfc3339("2024-03-20T00:00:00Z").unwrap(),
            updated_at: DbTimestamp::from_rfc3339("2024-03-20T00:00:00Z").unwrap(),
        };

        let json = serde_json::to_string(&image).unwrap();
        let deserialized: PropertyImage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.url, "https://example.com/image.jpg");
    }

    #[test]
    fn test_error_display() {
        let err = BreaError::InvalidPropertyType("invalid".to_string());
        assert_eq!(err.to_string(), "Invalid property type: invalid");
    }
} 