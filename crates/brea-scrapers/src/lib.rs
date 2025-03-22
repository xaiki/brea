pub mod argenprop;

use brea_core::{PropertyType, Scraper, PropertyTypeTranslator};
use std::sync::Arc;

pub use argenprop::ArgenPropScraper;

/// Enum representing different property listing sources
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScraperType {
    /// ArgenProp - Argentine real estate listings
    Argenprop,
    // Add more scrapers here as we implement them
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