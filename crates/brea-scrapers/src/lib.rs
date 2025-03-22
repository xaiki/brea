pub mod argenprop;

use brea_core::{Property, PropertyImage, PropertyType, Result};
use std::sync::Arc;
use async_trait::async_trait;

pub use argenprop::ArgenPropScraper;

/// Enum representing different property listing sources
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScraperType {
    /// ArgenProp - Argentine real estate listings
    Argenprop,
    // Add more scrapers here as we implement them
}

#[derive(Debug, Clone)]
pub struct ScrapeQuery {
    pub district: String,
    pub property_type: PropertyType,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub min_size: Option<f64>,
    pub max_size: Option<f64>,
    pub page: u32,
}

impl ScrapeQuery {
    pub fn new(
        district: String,
        property_type: PropertyType,
        min_price: Option<f64>,
        max_price: Option<f64>,
        min_size: Option<f64>,
        max_size: Option<f64>,
    ) -> Self {
        Self {
            district,
            property_type,
            min_price,
            max_price,
            min_size,
            max_size,
            page: 1,
        }
    }

    pub fn next_page(&mut self) {
        self.page += 1;
    }
}

/// Trait for translating PropertyType to scraper-specific strings
pub trait PropertyTypeTranslator {
    /// Convert a PropertyType to a string representation for this scraper
    fn property_type_to_str(&self, property_type: &PropertyType) -> &'static str;
}

/// Trait for scraping property listings from various sources
#[async_trait]
pub trait Scraper: Send + Sync + PropertyTypeTranslator {
    /// Scrape a single page of property listings
    async fn scrape_page(&self, query: &ScrapeQuery) -> Result<(Vec<(Property, Vec<PropertyImage>)>, bool)>;

    /// Get all property types supported by this scraper
    fn supported_property_types(&self) -> Vec<PropertyType>;

    /// Scrape multiple pages of property listings
    async fn scrape_listing(&self, mut query: ScrapeQuery, max_pages: u32) -> Result<Vec<(Property, Vec<PropertyImage>)>> {
        let mut all_properties = Vec::new();
        let mut pages_scraped = 0;

        while pages_scraped < max_pages {
            let (properties, has_next) = self.scrape_page(&query).await?;
            all_properties.extend(properties);

            if !has_next {
                break;
            }

            query.next_page();
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
        let supported_types = self.supported_property_types();

        for property_type in supported_types {
            let query = ScrapeQuery::new(
                district.to_string(),
                property_type,
                min_price,
                max_price,
                min_size,
                max_size,
            );

            let properties = self.scrape_listing(query, max_pages).await?;
            all_properties.extend(properties);
        }

        Ok(all_properties)
    }
}

/// Factory for creating scraper instances
pub struct ScraperFactory;

impl ScraperFactory {
    /// Create a new scraper instance based on the specified type
    pub fn create_scraper(scraper_type: ScraperType) -> Arc<dyn Scraper> {
        match scraper_type {
            ScraperType::Argenprop => Arc::new(ArgenPropScraper::new()),
            // Add more cases here as we implement more scrapers
        }
    }
}

/// Get the string representation of a property type for a specific scraper
pub fn property_type_to_str(scraper_type: ScraperType, property_type: &PropertyType) -> &'static str {
    match scraper_type {
        ScraperType::Argenprop => ArgenPropScraper::new().property_type_to_str(property_type),
        // Add more cases here as we implement more scrapers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrape_query() {
        let query = ScrapeQuery::new(
            "test".to_string(),
            PropertyType::House,
            Some(100_000.0),
            Some(200_000.0),
            Some(50.0),
            Some(100.0),
        );

        assert_eq!(query.district, "test");
        assert_eq!(query.property_type, PropertyType::House);
        assert_eq!(query.min_price, Some(100_000.0));
        assert_eq!(query.max_price, Some(200_000.0));
        assert_eq!(query.min_size, Some(50.0));
        assert_eq!(query.max_size, Some(100.0));
        assert_eq!(query.page, 1);
    }

    #[test]
    fn test_scrape_query_next_page() {
        let mut query = ScrapeQuery::new(
            "test".to_string(),
            PropertyType::House,
            None,
            None,
            None,
            None,
        );

        assert_eq!(query.page, 1);
        query.next_page();
        assert_eq!(query.page, 2);
    }
} 