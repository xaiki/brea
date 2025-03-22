use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};
use std::path::PathBuf;
use url::Url;
use std::str::FromStr;
use colored::Colorize;

mod db;
mod graph;
mod display;
pub use db::Database;
pub use graph::PriceHistory;
pub use display::{PropertyTableRow, create_property_table};

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
        match text {
            "house" => Ok(PropertyType::House),
            "apartment" => Ok(PropertyType::Apartment),
            "land" => Ok(PropertyType::Land),
            "ph" => Ok(PropertyType::Ph),
            "local" => Ok(PropertyType::Local),
            "field" => Ok(PropertyType::Field),
            "garage" => Ok(PropertyType::Garage),
            "commercial_premises" => Ok(PropertyType::CommercialPremises),
            "warehouse" => Ok(PropertyType::Warehouse),
            "hotel" => Ok(PropertyType::Hotel),
            "special_business" => Ok(PropertyType::SpecialBusiness),
            "office" => Ok(PropertyType::Office),
            "country_house" => Ok(PropertyType::CountryHouse),
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
            PropertyType::CommercialPremises => "commercial_premises",
            PropertyType::Warehouse => "warehouse",
            PropertyType::Hotel => "hotel",
            PropertyType::SpecialBusiness => "special_business",
            PropertyType::Office => "office",
            PropertyType::CountryHouse => "country_house",
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
            "commercial" | "commercial-premises" | "fondo-comercio" => Ok(PropertyType::CommercialPremises),
            "warehouse" | "warehouses" | "galpon" | "galpones" => Ok(PropertyType::Warehouse),
            "hotel" | "hotels" => Ok(PropertyType::Hotel),
            "special-business" | "special-businesses" | "negocio-especial" => Ok(PropertyType::SpecialBusiness),
            "office" | "offices" | "oficina" | "oficinas" => Ok(PropertyType::Office),
            "country-house" | "country-houses" | "quinta" | "quintas" => Ok(PropertyType::CountryHouse),
            _ => Err(format!(
                "Invalid property type: {}. Valid options are: house/casa, apartment/departamento, land/terreno, ph, local, field/campo, garage/cochera, commercial/fondo-comercio, warehouse/galpon, hotel, special-business/negocio-especial, office/oficina, country-house/quinta",
                s
            )),
        }
    }
}

// Property with SQLx support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub id: Option<i64>,
    pub external_id: String,
    pub source: String,
    pub property_type: Option<PropertyType>,
    pub title: String,
    pub description: Option<String>,
    pub price_usd: f64,
    pub address: String,
    pub covered_size: f64,
    pub rooms: i32,
    pub antiquity: i32,
    pub url: Url,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Implement FromRow manually for Property
impl<'r> FromRow<'r, sqlx::sqlite::SqliteRow> for Property {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> std::result::Result<Self, sqlx::Error> {
        let url_str: String = row.try_get("url")?;
        let url = Url::from_str(&url_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

        Ok(Property {
            id: row.try_get("id")?,
            external_id: row.try_get("external_id")?,
            source: row.try_get("source")?,
            property_type: row.try_get("property_type")?,
            title: row.try_get("title")?,
            description: row.try_get("description")?,
            price_usd: row.try_get("price_usd")?,
            address: row.try_get("address")?,
            covered_size: row.try_get("covered_size")?,
            rooms: row.try_get("rooms")?,
            antiquity: row.try_get("antiquity")?,
            url,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyImage {
    pub id: Option<i64>,
    pub property_id: i64,
    pub url: Url,
    #[serde(with = "path_buf_serde")]
    pub local_path: PathBuf,
    pub hash: Vec<u8>,
    pub created_at: DateTime<Utc>,
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

/// Trait for building URLs for property listings
pub trait ListingUrlBuilder {
    /// Build a URL for property listings based on search criteria
    fn build_listing_url(
        &self,
        district: &str,
        property_type: &PropertyType,
        min_price: Option<f64>,
        max_price: Option<f64>,
        min_size: Option<f64>,
        max_size: Option<f64>,
    ) -> String;
}

/// Trait for scraping property listings from various sources
#[async_trait]
pub trait Scraper: ListingUrlBuilder + Send + Sync {
    /// Scrape a single page of property listings
    async fn scrape_page(&self, url: &str) -> Result<(Vec<(Property, Vec<PropertyImage>)>, bool)>;

    /// Get the URL for the next page of listings
    async fn get_next_page_url(&self, current_url: &str) -> Result<Option<String>>;

    /// Scrape a single property's details
    async fn scrape_property(&self, url: &str) -> Result<(Property, Vec<PropertyImage>)>;

    /// Get all property types supported by this scraper
    fn supported_property_types(&self) -> Vec<PropertyType>;

    /// Scrape multiple pages of property listings
    async fn scrape_listing(&self, url: &str, max_pages: u32) -> Result<Vec<(Property, Vec<PropertyImage>)>> {
        let mut all_properties = Vec::new();
        let mut current_url = url.to_string();
        let mut pages_scraped = 0;

        while pages_scraped < max_pages {
            let (properties, has_next) = self.scrape_page(&current_url).await?;
            all_properties.extend(properties);

            if !has_next {
                break;
            }

            if let Some(next_url) = self.get_next_page_url(&current_url).await? {
                current_url = next_url;
            } else {
                break;
            }

            pages_scraped += 1;
        }

        Ok(all_properties)
    }

    /// Scrape all property types for a given district
    async fn scrape_all_types(
        &self,
        district: &str,
        min_price: Option<f64>,
        max_price: Option<f64>,
        min_size: Option<f64>,
        max_size: Option<f64>,
        max_pages: u32,
    ) -> Result<Vec<(Property, Vec<PropertyImage>)>> {
        let mut all_properties = Vec::new();
        let property_types = self.supported_property_types();

        for property_type in property_types {
            let url = self.build_listing_url(
                district,
                &property_type,
                min_price,
                max_price,
                min_size,
                max_size,
            );

            let properties = self.scrape_listing(&url, max_pages).await?;
            all_properties.extend(properties);
        }

        Ok(all_properties)
    }
}

/// Trait for translating PropertyType to scraper-specific strings
pub trait PropertyTypeTranslator {
    /// Convert a PropertyType to a string representation for this scraper
    fn property_type_to_str(&self, property_type: &PropertyType) -> &'static str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDisplay {
    pub property: Property,
    pub price_history: Option<PriceHistory>,
}

impl PropertyDisplay {
    pub fn new(property: Property, price_history: Vec<(f64, DateTime<Utc>)>) -> Self {
        Self {
            property,
            price_history: Some(PriceHistory::new(price_history)),
        }
    }

    pub fn format(&self) -> String {
        let mut result = String::new();
        
        // Format property details
        result.push_str(&format!("{} - ${}k\n", 
            self.property.title.bold(),
            (self.property.price_usd/1000.0).round() as i64
        ));
        result.push_str(&format!("{} - {}m², {} rooms, {} years old\n",
            self.property.address,
            self.property.covered_size,
            self.property.rooms,
            self.property.antiquity
        ));
        
        // Add price history graph if available
        if let Some(history) = &self.price_history {
            result.push_str("\nPrice History:\n");
            result.push_str(&history.to_ascii_graph(40, 5)); // 40 chars width, 5 lines height
            result.push('\n'); // Add a newline after the graph
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::str::FromStr;

    #[test]
    fn test_property_serialization() {
        let property = Property {
            id: Some(1),
            external_id: "test123".to_string(),
            source: "argenprop".to_string(),
            property_type: Some(PropertyType::House),
            title: "Test Property".to_string(),
            description: Some("A test property".to_string()),
            price_usd: 100000.0,
            address: "123 Test St".to_string(),
            covered_size: 100.0,
            rooms: 3,
            antiquity: 5,
            url: Url::from_str("https://example.com/property/123").unwrap(),
            created_at: Utc.with_ymd_and_hms(2024, 3, 21, 0, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2024, 3, 21, 0, 0, 0).unwrap(),
        };

        let json = serde_json::to_string(&property).unwrap();
        let deserialized: Property = serde_json::from_str(&json).unwrap();

        assert_eq!(property.id, deserialized.id);
        assert_eq!(property.external_id, deserialized.external_id);
        assert_eq!(property.price_usd, deserialized.price_usd);
        assert_eq!(property.url.as_str(), deserialized.url.as_str());
    }

    #[test]
    fn test_property_image_serialization() {
        let image = PropertyImage {
            id: Some(1),
            property_id: 1,
            url: Url::from_str("https://example.com/image.jpg").unwrap(),
            local_path: PathBuf::from("/tmp/images/test.jpg"),
            hash: vec![1, 2, 3, 4],
            created_at: Utc.with_ymd_and_hms(2024, 3, 21, 0, 0, 0).unwrap(),
        };

        let json = serde_json::to_string(&image).unwrap();
        let deserialized: PropertyImage = serde_json::from_str(&json).unwrap();

        assert_eq!(image.id, deserialized.id);
        assert_eq!(image.property_id, deserialized.property_id);
        assert_eq!(image.url.as_str(), deserialized.url.as_str());
        assert_eq!(image.local_path, deserialized.local_path);
        assert_eq!(image.hash, deserialized.hash);
    }

    #[test]
    fn test_error_display() {
        let db_error = BreaError::Database(sqlx::Error::RowNotFound);
        assert!(db_error.to_string().contains("Database error"));

        let scraping_error = BreaError::Scraping("Failed to parse".to_string());
        assert!(scraping_error.to_string().contains("Failed to parse"));

        let invalid_type_error = BreaError::InvalidPropertyType("Invalid type".to_string());
        assert!(invalid_type_error.to_string().contains("Invalid property type"));
    }
} 