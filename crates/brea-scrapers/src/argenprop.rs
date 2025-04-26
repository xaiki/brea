use async_trait::async_trait;
use brea_core::{BreaError, Property, PropertyImage, PropertyType, PropertyStatus, Result};
use crate::{PropertyTypeTranslator, Scraper, ScrapeQuery};
use chrono::Utc;
use reqwest::Client;
use scraper::{Html, Selector};
use url::Url;
use std::path::PathBuf;
use tracing::{debug, info};
use std::sync::Mutex;
use regex;

#[derive(Debug)]
pub struct ArgenPropScraper {
    client: Client,
    html_parser: Mutex<()>,
}

// Make ArgenPropScraper thread-safe
unsafe impl Send for ArgenPropScraper {}
unsafe impl Sync for ArgenPropScraper {}

impl ArgenPropScraper {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            html_parser: Mutex::new(()),
        }
    }

    fn parse_selector(selector: &str) -> Result<Selector> {
        Selector::parse(selector).map_err(|e| BreaError::Scraping(e.to_string()))
    }

    fn create_selectors() -> Result<(
        Selector,  // listing_item
        Selector,  // title
        Selector,  // price
        Selector,  // address
        Selector,  // features
        Selector,  // description
        Selector,  // images
        Selector,  // next_page
    )> {
        Ok((
            Self::parse_selector(".listing__item")?,
            Self::parse_selector(".card__title")?,
            Self::parse_selector(".card__price")?,
            Self::parse_selector(".card__address")?,
            Self::parse_selector(".card__main-features li")?,
            Self::parse_selector(".card__description")?,
            Self::parse_selector(".card__photos img")?,
            Self::parse_selector(".pagination__page-next")?,
        ))
    }

    async fn fetch_page(&self, url: &str) -> Result<String> {
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| BreaError::Scraping(e.to_string()))?;

        response
            .text()
            .await
            .map_err(|e| BreaError::Scraping(e.to_string()))
    }

    fn parse_price(&self, price_text: &str) -> Option<f64> {
        let cleaned = price_text
            .trim()
            .replace("USD", "")
            .replace("U$S", "")
            .replace("$", "")
            .replace(".", "")
            .replace(",", "")
            .trim()
            .to_string();
        
        if cleaned.is_empty() {
            return None;
        }
        
        cleaned.parse::<f64>().ok()
    }

    fn extract_dimensions(&self, text: &str) -> Option<f64> {
        // Match patterns like "13 x 26", "13x26", "13.5 x 26.5", "13m x 26m"
        let dim_regex = regex::Regex::new(r"(\d+(?:\.\d+)?)\s*(?:m|mts|metros)?\s*[xX]\s*(\d+(?:\.\d+)?)\s*(?:m|mts|metros)?").ok()?;
        if let Some(caps) = dim_regex.captures(text) {
            let width: f64 = caps[1].parse().unwrap_or(0.0);
            let length: f64 = caps[2].parse().unwrap_or(0.0);
            if width > 0.0 && length > 0.0 {
                return Some(width * length);
            }
        }
        None
    }

    fn extract_size_from_text(&self, text: &str) -> Option<f64> {
        // First try to find dimensions and calculate area
        if let Some(area) = self.extract_dimensions(text) {
            return Some(area);
        }

        // Clean the text but preserve numbers and units
        let text = text.to_lowercase()
            .replace(".", "")
            .replace(",", ".");

        // Match size patterns with various units
        let size_patterns = [
            r"(\d+(?:\.\d+)?)\s*(?:m²|m2|mts²|mts2|metros\s*cuadrados|metros2|mtrs2|mtrs²)",
            r"(\d+(?:\.\d+)?)\s*(?:m|mts|metros)\s*x\s*(\d+(?:\.\d+)?)",
            r"superficie(?:\s*total)?\s*(?:de)?\s*(\d+(?:\.\d+)?)",
            r"(\d+(?:\.\d+)?)\s*(?:m|mts|metros)\s*de\s*superficie",
            r"(\d+(?:\.\d+)?)\s*(?:m|mts|metros)\s*cubiertos",
            r"(\d+(?:\.\d+)?)\s*(?:m|mts|metros)\s*totales",
        ];

        for pattern in size_patterns {
            let regex = regex::Regex::new(pattern).ok()?;
            if let Some(caps) = regex.captures(&text) {
                if caps.len() > 2 {
                    // Handle "X x Y" format
                    let width: f64 = caps[1].parse().unwrap_or(0.0);
                    let length: f64 = caps[2].parse().unwrap_or(0.0);
                    if width > 0.0 && length > 0.0 {
                        return Some(width * length);
                    }
                } else {
                    // Handle direct size specification
                    if let Ok(size) = caps[1].parse::<f64>() {
                        if size > 0.0 && size < 10000.0 { // Sanity check for reasonable sizes
                            return Some(size);
                        }
                    }
                }
            }
        }

        // Handle ranges like "100-150 m²"
        let range_regex = regex::Regex::new(r"(\d+(?:\.\d+)?)\s*-\s*(\d+(?:\.\d+)?)\s*(?:m²|m2|mts²|mts2)").ok()?;
        if let Some(caps) = range_regex.captures(&text) {
            let min: f64 = caps[1].parse().unwrap_or(0.0);
            let max: f64 = caps[2].parse().unwrap_or(0.0);
            if min > 0.0 && max > min {
                return Some((min + max) / 2.0);
            }
        }

        None
    }

    fn extract_rooms_from_text(&self, text: &str) -> Option<i32> {
        let text = text.to_lowercase();
        
        // Match room patterns with various terms
        let room_patterns = [
            r"(\d+)\s*(?:amb(?:ientes)?|dormitorios?|habitaciones?|dorm\.?|hab\.?|cuartos?)",
            r"monoambiente",
            r"(\d+)\s*(?:amb\.?|dorm\.?|hab\.?)",
            r"(\d+)\s*(?:deptos?|departamentos?)",
        ];

        for pattern in room_patterns {
            let regex = regex::Regex::new(pattern).ok()?;
            if pattern.contains("monoambiente") && regex.is_match(&text) {
                return Some(1);
            }
            if let Some(caps) = regex.captures(&text) {
                if let Ok(rooms) = caps[1].parse::<i32>() {
                    if rooms > 0 && rooms < 20 { // Sanity check for reasonable room counts
                        return Some(rooms);
                    }
                }
            }
        }

        // Handle ranges like "2-3 ambientes"
        let range_regex = regex::Regex::new(r"(\d+)\s*-\s*(\d+)\s*(?:amb(?:ientes)?|dormitorios?|deptos?)").ok()?;
        if let Some(caps) = range_regex.captures(&text) {
            if let Ok(min) = caps[1].parse::<i32>() {
                if min > 0 && min < 20 {
                    return Some(min);
                }
            }
        }

        None
    }

    fn extract_features(&self, element: scraper::ElementRef) -> Result<(Option<f64>, Option<i32>, Option<i32>)> {
        let mut covered_size = None;
        let mut rooms = None;
        let mut antiquity = None;

        // First try to extract from dedicated feature elements
        let feature_selector = Self::parse_selector(".card__main-features li, .card__features li")?;
        
        for feature in element.select(&feature_selector) {
            let text = feature.text().collect::<String>().trim().to_string();
            debug!("Processing feature text: {}", text);

            // Try to extract from structured elements first
            if let Some(size) = self.extract_size_from_text(&text) {
                covered_size = Some(size);
                debug!("Extracted covered size from feature element: {:?}", covered_size);
                continue;
            }

            if let Some(room_count) = self.extract_rooms_from_text(&text) {
                rooms = Some(room_count);
                debug!("Extracted rooms from feature element: {:?}", rooms);
                continue;
            }

            // Try to extract antiquity
            if text.contains("años") || text.contains("año") {
                let age_text = text
                    .replace("años", "")
                    .replace("año", "")
                    .trim()
                    .to_string();
                if let Ok(age) = age_text.parse::<i32>() {
                    antiquity = Some(age);
                    debug!("Extracted antiquity: {:?} from text: {}", antiquity, text);
                }
            }
        }

        // Only if we didn't find size/rooms in features, try title
        if covered_size.is_none() || rooms.is_none() {
            if let Some(title) = element.select(&Self::parse_selector(".card__title")?).next() {
                let title_text = title.text().collect::<String>().trim().to_string();
                debug!("Processing title text: {}", title_text);

                // Try to extract covered size from title
                if covered_size.is_none() {
                    if let Some(size) = self.extract_size_from_text(&title_text) {
                        covered_size = Some(size);
                        debug!("Extracted covered size from title: {:?}", covered_size);
                    }
                }

                // Try to extract rooms from title
                if rooms.is_none() {
                    if let Some(room_count) = self.extract_rooms_from_text(&title_text) {
                        rooms = Some(room_count);
                        debug!("Extracted rooms from title: {:?}", rooms);
                    }
                }
            }
        }

        // Finally, try description as last resort
        if covered_size.is_none() || rooms.is_none() {
            if let Some(description) = element.select(&Self::parse_selector(".card__description")?).next() {
                let desc_text = description.text().collect::<String>().trim().to_string();
                debug!("Processing description text: {}", desc_text);

                // Try to extract covered size from description
                if covered_size.is_none() {
                    if let Some(size) = self.extract_size_from_text(&desc_text) {
                        covered_size = Some(size);
                        debug!("Extracted covered size from description: {:?}", covered_size);
                    }
                }

                // Try to extract rooms from description
                if rooms.is_none() {
                    if let Some(room_count) = self.extract_rooms_from_text(&desc_text) {
                        rooms = Some(room_count);
                        debug!("Extracted rooms from description: {:?}", rooms);
                    }
                }
            }
        }

        debug!(
            "Final extracted features - covered_size: {:?}, rooms: {:?}, antiquity: {:?}",
            covered_size, rooms, antiquity
        );

        Ok((covered_size, rooms, antiquity))
    }

    fn has_next_page(&self, html: &str) -> Result<bool> {
        if html.trim().is_empty() {
            return Err(BreaError::Scraping("Empty HTML provided".to_string()));
        }

        let _guard = self.html_parser.lock().unwrap();
        let document = Html::parse_document(html);
        
        // Check if there's a disabled next page button
        let disabled_next = document
            .select(&Self::parse_selector(".pagination__page-next.pagination__page--disable")?)
            .next()
            .is_some();

        // If there's a disabled next page button, there are no more pages
        if disabled_next {
            info!("Found disabled next page button, no more pages");
            return Ok(false);
        }

        // Check if there's a next page button
        let next_page = document
            .select(&Self::parse_selector(".pagination__page-next")?)
            .next()
            .is_some();

        debug!("Next page button found: {}", next_page);
        Ok(next_page)
    }
}

impl PropertyTypeTranslator for ArgenPropScraper {
    fn property_type_to_str(&self, property_type: &PropertyType) -> &'static str {
        match property_type {
            PropertyType::House => "casas",
            PropertyType::Apartment => "departamentos",
            PropertyType::Land => "terrenos",
            PropertyType::Ph => "ph",
            PropertyType::Local => "locales",
            PropertyType::Field => "campos",
            PropertyType::Garage => "cocheras",
            PropertyType::CommercialPremises => "locales-comerciales",
            PropertyType::Warehouse => "galpones",
            PropertyType::Hotel => "hoteles",
            PropertyType::SpecialBusiness => "negocios-especiales",
            PropertyType::Office => "oficinas",
            PropertyType::CountryHouse => "quintas",
        }
    }
}

#[async_trait]
impl Scraper for ArgenPropScraper {
    fn supported_property_types(&self) -> Vec<PropertyType> {
        vec![
            PropertyType::House,
            PropertyType::Apartment,
            PropertyType::Land,
            PropertyType::Ph,
            PropertyType::Local,
            PropertyType::Field,
            PropertyType::Garage,
            PropertyType::CommercialPremises,
            PropertyType::Warehouse,
            PropertyType::Hotel,
            PropertyType::SpecialBusiness,
            PropertyType::Office,
            PropertyType::CountryHouse,
        ]
    }

    async fn scrape_page(&self, query: &ScrapeQuery) -> Result<(Vec<(Property, Vec<PropertyImage>)>, bool)> {
        // Build the URL for the query
        let district = query.district.to_lowercase();
        let district = district
            .strip_prefix("la ")
            .or_else(|| district.strip_prefix("el "))
            .or_else(|| district.strip_prefix("los "))
            .or_else(|| district.strip_prefix("las "))
            .unwrap_or(&district)
            .replace(' ', "-");
        
        debug!("ScrapeQuery: district={}, property_type={}, page={}", query.district, query.property_type, query.page);
        debug!("Processed district for URL: {}", district);
        
        // Build the base URL
        let mut url = format!(
            "https://www.argenprop.com/{}/venta/{}",
            self.property_type_to_str(&query.property_type),
            district
        );

        // Add price filters if provided
        if query.min_price.is_some() || query.max_price.is_some() {
            url.push_str("?precio=");
            if let Some(min) = query.min_price {
                url.push_str(&format!("{}", min as i64));
            }
            url.push('-');
            if let Some(max) = query.max_price {
                url.push_str(&format!("{}", max as i64));
            }
        }

        // Add size filters if provided
        if query.min_size.is_some() || query.max_size.is_some() {
            if url.contains('?') {
                url.push('&');
            } else {
                url.push('?');
            }
            url.push_str("superficie=");
            if let Some(min) = query.min_size {
                url.push_str(&format!("{}", min as i64));
            }
            url.push('-');
            if let Some(max) = query.max_size {
                url.push_str(&format!("{}", max as i64));
            }
        }

        // Add page number if not first page
        if query.page > 1 {
            if url.contains('?') {
                url.push_str(&format!("&pagina-{}", query.page));
            } else {
                url.push_str(&format!("?pagina-{}", query.page));
            }
        }

        info!("Scraping page: {}", url);
        let html = self.fetch_page(&url).await?;
        
        // Extract property type from URL
        let property_type = query.property_type.clone();
        
        let (
            listing_item_selector,
            title_selector,
            price_selector,
            address_selector,
            _features_selector,
            description_selector,
            images_selector,
            _next_page_selector,
        ) = Self::create_selectors()?;

        let mut properties = Vec::new();
        let mut external_ids = Vec::new();

        // Parse HTML and extract properties
        {
            let _guard = self.html_parser.lock().unwrap();
            let document = Html::parse_document(&html);
            
            for element in document.select(&listing_item_selector) {
                // Extract external ID from the listing URL
                let external_id = element
                    .select(&Self::parse_selector("a")?)
                    .next()
                    .and_then(|a| a.value().attr("href"))
                    .and_then(|href| href.split('/').last())
                    .unwrap_or_default()
                    .to_string();

                external_ids.push(external_id.clone());

                let title = element.select(&title_selector)
                    .next()
                    .map(|el| el.text().collect::<String>())
                    .map(|text| text.trim().to_string())
                    .unwrap_or_default();

                let property_url = element.select(&Self::parse_selector("a.card")?)
                    .next()
                    .and_then(|el| el.value().attr("href"))
                    .map(|href| {
                        if href.starts_with("http") {
                            href.to_string()
                        } else {
                            format!("https://www.argenprop.com{}", href)
                        }
                    })
                    .unwrap_or_default();

                let price_usd = element.select(&price_selector)
                    .next()
                    .map(|el| el.text().collect::<String>())
                    .and_then(|price| self.parse_price(&price))
                    .unwrap_or(0.0);

                let address = element.select(&address_selector)
                    .next()
                    .map(|el| el.text().collect::<String>())
                    .map(|addr| addr.trim().to_string())
                    .unwrap_or_default();

                let (covered_size, rooms, antiquity) = self.extract_features(element)?;

                let description = element.select(&description_selector)
                    .next()
                    .map(|el| el.text().collect::<String>())
                    .map(|desc| desc.trim().to_string());

                let property = Property {
                    id: None,
                    external_id,
                    source: "argenprop".to_string(),
                    property_type: Some(property_type.clone()),
                    district: query.district.clone(),
                    title,
                    description,
                    price_usd,
                    address,
                    covered_size,
                    rooms,
                    antiquity,
                    url: Url::parse(&property_url).map_err(|e| BreaError::Scraping(e.to_string()))?,
                    status: PropertyStatus::Active,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                // Extract images
                let mut images = Vec::new();
                for img in element.select(&images_selector) {
                    if let Some(img_url) = img.value().attr("src").or_else(|| img.value().attr("data-src")) {
                        if let Ok(url) = Url::parse(img_url) {
                            let image = PropertyImage {
                                id: None,
                                property_id: 0,
                                url,
                                local_path: PathBuf::new(),
                                hash: Vec::new(),
                                created_at: Utc::now(),
                            };
                            images.push(image);
                        }
                    }
                }

                properties.push((property, images));
            }
        }

        // Check for sold properties
        if let Some(db) = &query.db {
            let sold_properties = db.detect_sold_properties("argenprop", &external_ids).await?;
            for property in sold_properties {
                info!("Property {} marked as sold", property.external_id);
                db.mark_property_sold(property.id.unwrap()).await?;
            }
        }

        let has_next = self.has_next_page(&html)?;
        Ok((properties, has_next))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScrapeQuery;

    #[tokio::test]
    async fn test_url_construction() {
        let scraper = ArgenPropScraper::new();
        
        // Test basic URL construction
        let query = ScrapeQuery {
            district: "palermo".to_string(),
            property_type: PropertyType::Apartment,
            min_price: None,
            max_price: None,
            min_size: None,
            max_size: None,
            page: 1,
            db: None,
        };
        
        let (properties, _) = scraper.scrape_page(&query).await.unwrap();
        assert!(!properties.is_empty());
        
        // Test with price range
        let query = ScrapeQuery {
            district: "palermo".to_string(),
            property_type: PropertyType::Apartment,
            min_price: Some(100000.0),
            max_price: Some(200000.0),
            min_size: None,
            max_size: None,
            page: 1,
            db: None,
        };
        
        let (properties, _) = scraper.scrape_page(&query).await.unwrap();
        assert!(!properties.is_empty());
        
        // Test with size range
        let query = ScrapeQuery {
            district: "palermo".to_string(),
            property_type: PropertyType::Apartment,
            min_price: None,
            max_price: None,
            min_size: Some(50.0),
            max_size: Some(100.0),
            page: 1,
            db: None,
        };
        
        let (properties, _) = scraper.scrape_page(&query).await.unwrap();
        assert!(!properties.is_empty());
    }

    #[tokio::test]
    async fn test_html_parsing() {
        let scraper = ArgenPropScraper::new();
        
        // Fetch a real page to test parsing
        let query = ScrapeQuery {
            district: "palermo".to_string(),
            property_type: PropertyType::Apartment,
            min_price: None,
            max_price: None,
            min_size: None,
            max_size: None,
            page: 1,
            db: None,
        };
        
        let (properties, _) = scraper.scrape_page(&query).await.unwrap();
        assert!(!properties.is_empty());
        
        // Test price parsing
        let price_text = "USD 100.000";
        let price = scraper.parse_price(price_text);
        assert_eq!(price, Some(100000.0));
        
        // Test feature extraction from a real property
        let property = &properties[0].0;
        assert!(property.covered_size.is_some() || property.rooms.is_some() || property.antiquity.is_some());
    }

    #[tokio::test]
    async fn test_pagination() {
        let scraper = ArgenPropScraper::new();
        
        // Test first page
        let query = ScrapeQuery {
            district: "palermo".to_string(),
            property_type: PropertyType::Apartment,
            min_price: None,
            max_price: None,
            min_size: None,
            max_size: None,
            page: 1,
            db: None,
        };
        
        let (_, has_next) = scraper.scrape_page(&query).await.unwrap();
        assert!(has_next, "First page should have a next page");
        
        // Test last page (using a high page number)
        let query = ScrapeQuery {
            district: "palermo".to_string(),
            property_type: PropertyType::Apartment,
            min_price: None,
            max_price: None,
            min_size: None,
            max_size: None,
            page: 100,
            db: None,
        };
        
        let (_, has_next) = scraper.scrape_page(&query).await.unwrap();
        assert!(!has_next, "Last page should not have a next page");
    }

    #[tokio::test]
    async fn test_error_handling() {
        let scraper = ArgenPropScraper::new();
        
        // Test network error by using a non-existent domain
        let query = ScrapeQuery {
            district: "palermo".to_string(),
            property_type: PropertyType::Apartment,
            min_price: None,
            max_price: None,
            min_size: None,
            max_size: None,
            page: 1,
            db: None,
        };
        
        // Override the client to use a non-existent domain
        let mut scraper = scraper;
        scraper.client = Client::builder()
            .connect_timeout(std::time::Duration::from_millis(100))
            .build()
            .unwrap();
        
        let result = scraper.scrape_page(&query).await;
        assert!(result.is_err(), "Request to non-existent domain should fail");
        
        // Test invalid HTML
        let result = scraper.has_next_page("");
        assert!(result.is_err(), "Empty HTML should return an error");
        
        // Test malformed HTML
        let result = scraper.has_next_page("<not>valid</html>");
        assert!(result.is_ok(), "Malformed HTML should not error, just return no next page");
        assert!(!result.unwrap(), "Malformed HTML should indicate no next page");
    }
} 