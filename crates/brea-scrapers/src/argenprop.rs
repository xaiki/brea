use async_trait::async_trait;
use brea_core::{BreaError, Property, PropertyImage, PropertyType, PropertyTypeTranslator, Result, Scraper, ListingUrlBuilder};
use chrono::Utc;
use reqwest::Client;
use scraper::{Html, Selector};
use url::Url;
use std::path::PathBuf;
use tracing::{debug, info};

#[derive(Debug)]
pub struct ArgenPropScraper {
    client: Client,
}

impl ArgenPropScraper {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
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
            Selector::parse(".listing__item").map_err(|e| BreaError::Scraping(e.to_string()))?,
            Selector::parse("a.card .card__title").map_err(|e| BreaError::Scraping(e.to_string()))?,
            Selector::parse(".card__price").map_err(|e| BreaError::Scraping(e.to_string()))?,
            Selector::parse(".card__address").map_err(|e| BreaError::Scraping(e.to_string()))?,
            Selector::parse(".card__main-features").map_err(|e| BreaError::Scraping(e.to_string()))?,
            Selector::parse(".card__info").map_err(|e| BreaError::Scraping(e.to_string()))?,
            Selector::parse(".card__photos img").map_err(|e| BreaError::Scraping(e.to_string()))?,
            Selector::parse(".pagination__page-next").map_err(|e| BreaError::Scraping(e.to_string()))?,
        ))
    }

    fn create_detail_selectors() -> Result<(
        Selector, Selector, Selector, Selector,
        Selector, Selector
    )> {
        Ok((
            Selector::parse(".card__title").map_err(|e| BreaError::Scraping(e.to_string()))?,
            Selector::parse(".card__price").map_err(|e| BreaError::Scraping(e.to_string()))?,
            Selector::parse(".card__address").map_err(|e| BreaError::Scraping(e.to_string()))?,
            Selector::parse(".card__info").map_err(|e| BreaError::Scraping(e.to_string()))?,
            Selector::parse(".card__main-features li span").map_err(|e| BreaError::Scraping(e.to_string()))?,
            Selector::parse(".card__photos img").map_err(|e| BreaError::Scraping(e.to_string()))?,
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

    fn parse_size(&self, size_text: &str) -> Option<f64> {
        let cleaned = size_text
            .trim()
            .replace("m² cubie.", "")
            .replace("m² cub.", "")
            .replace("m²", "")
            .replace("m2", "")
            .trim()
            .to_string();
            
        if cleaned.is_empty() {
            return None;
        }
        
        cleaned.parse::<f64>().ok()
    }

    fn parse_rooms(&self, rooms_text: &str) -> Option<i32> {
        let cleaned = rooms_text
            .trim()
            .replace("dorm.", "")
            .replace("dormitorios", "")
            .replace("dormitorio", "")
            .replace("amb.", "")
            .replace("ambientes", "")
            .replace("ambiente", "")
            .trim()
            .to_string();
            
        if cleaned.is_empty() {
            return None;
        }
        
        cleaned.parse::<i32>().ok()
    }

    fn parse_antiquity(&self, antiquity_text: &str) -> Option<i32> {
        let cleaned = antiquity_text
            .trim()
            .replace("años", "")
            .replace("año", "")
            .replace("a estrenar", "0")
            .replace("en construcción", "0")
            .trim()
            .to_string();
            
        if cleaned.is_empty() {
            return None;
        }
        
        cleaned.parse::<i32>().ok()
    }

    fn extract_features(&self, element: scraper::ElementRef) -> (f64, i32, i32) {
        let mut covered_size = 0.0;
        let mut rooms = 0;
        let mut antiquity = 0;

        for feature in element.select(&Selector::parse("li").unwrap()) {
            let text = feature.text().collect::<String>().to_lowercase();
            if text.contains("m²") {
                covered_size = self.parse_size(&text).unwrap_or(0.0);
            } else if text.contains("dorm") || text.contains("amb") {
                rooms = self.parse_rooms(&text).unwrap_or(0);
            } else if text.contains("año") || text.contains("estrenar") || text.contains("construcción") {
                antiquity = self.parse_antiquity(&text).unwrap_or(0);
            }
        }

        (covered_size, rooms, antiquity)
    }

    fn has_next_page(&self, html: &str) -> Result<bool> {
        let document = Html::parse_document(html);
        
        // Check if there's a disabled next page button
        let disabled_next = document
            .select(&Selector::parse(".pagination__page-next.pagination__page--disable").map_err(|e| BreaError::Scraping(e.to_string()))?)
            .next()
            .is_some();

        // If there's a disabled next page button, there are no more pages
        if disabled_next {
            info!("Found disabled next page button, no more pages");
            return Ok(false);
        }

        // Check if there's a next page button
        let next_page = document
            .select(&Selector::parse(".pagination__page-next").map_err(|e| BreaError::Scraping(e.to_string()))?)
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
            PropertyType::Ph => "depto-tipo-casa",
            PropertyType::Local => "local",
            PropertyType::Field => "campo",
            PropertyType::Garage => "cochera",
            PropertyType::CommercialPremises => "fondo-comercio",
            PropertyType::Warehouse => "galpon",
            PropertyType::Hotel => "hotel",
            PropertyType::SpecialBusiness => "negocio-especial",
            PropertyType::Office => "oficina",
            PropertyType::CountryHouse => "quinta",
        }
    }
}

impl ListingUrlBuilder for ArgenPropScraper {
    fn build_listing_url(
        &self,
        district: &str,
        property_type: &PropertyType,
        min_price: Option<f64>,
        max_price: Option<f64>,
        min_size: Option<f64>,
        max_size: Option<f64>,
    ) -> String {
        // Convert district to lowercase and handle special cases
        let district = district.to_lowercase();
        let district = district
            .strip_prefix("la ")
            .unwrap_or(&district)
            .replace(' ', "-");
        
        // Build the base URL
        let mut url = format!(
            "https://www.argenprop.com/{}/venta/{}",
            self.property_type_to_str(property_type),
            district
        );

        // Add price filters if provided
        if min_price.is_some() || max_price.is_some() {
            url.push_str("?precio=");
            if let Some(min) = min_price {
                url.push_str(&format!("{}", min as i64));
            }
            url.push('-');
            if let Some(max) = max_price {
                url.push_str(&format!("{}", max as i64));
            }
        }

        // Add size filters if provided
        if min_size.is_some() || max_size.is_some() {
            if url.contains('?') {
                url.push('&');
            } else {
                url.push('?');
            }
            url.push_str("superficie=");
            if let Some(min) = min_size {
                url.push_str(&format!("{}", min as i64));
            }
            url.push('-');
            if let Some(max) = max_size {
                url.push_str(&format!("{}", max as i64));
            }
        }

        url
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

    async fn scrape_page(&self, url: &str) -> Result<(Vec<(Property, Vec<PropertyImage>)>, bool)> {
        info!("Scraping page: {}", url);
        let html = self.fetch_page(url).await?;
        let document = Html::parse_document(&html);
        
        // Extract property type from URL
        let property_type = url.split('/')
            .nth(3)
            .and_then(|t| match t {
                "casas" => Some(PropertyType::House),
                "departamentos" => Some(PropertyType::Apartment),
                "terrenos" => Some(PropertyType::Land),
                "depto-tipo-casa" => Some(PropertyType::Ph),
                "local" => Some(PropertyType::Local),
                "campo" => Some(PropertyType::Field),
                "cochera" => Some(PropertyType::Garage),
                "fondo-comercio" => Some(PropertyType::CommercialPremises),
                "galpon" => Some(PropertyType::Warehouse),
                "hotel" => Some(PropertyType::Hotel),
                "negocio-especial" => Some(PropertyType::SpecialBusiness),
                "oficina" => Some(PropertyType::Office),
                "quinta" => Some(PropertyType::CountryHouse),
                _ => None,
            })
            .ok_or_else(|| BreaError::Scraping("Could not determine property type from URL".to_string()))?;
        
        let (
            listing_item_selector,
            title_selector,
            price_selector,
            address_selector,
            features_selector,
            description_selector,
            images_selector,
            _next_page_selector,
        ) = Self::create_selectors()?;

        let mut property_data = Vec::new();
        let now = Utc::now();

        for item in document.select(&listing_item_selector) {
            let title = item.select(&title_selector)
                .next()
                .map(|el| el.text().collect::<String>())
                .map(|text| text.trim().to_string())
                .unwrap_or_default();

            let property_url = item.select(&Selector::parse("a.card").map_err(|e| BreaError::Scraping(e.to_string()))?)
                .next()
                .and_then(|el| el.value().attr("href"))
                .map(|href| format!("https://www.argenprop.com{}", href))
                .unwrap_or_default();

            let price_usd = item.select(&price_selector)
                .next()
                .map(|el| el.text().collect::<String>())
                .and_then(|price| self.parse_price(&price))
                .unwrap_or(0.0);

            let address = item.select(&address_selector)
                .next()
                .map(|el| el.text().collect::<String>())
                .map(|addr| addr.trim().to_string())
                .unwrap_or_default();

            let (covered_size, rooms, antiquity) = item
                .select(&features_selector)
                .next()
                .map(|el| self.extract_features(el))
                .unwrap_or((0.0, 0, 0));

            let description = item.select(&description_selector)
                .next()
                .map(|el| el.text().collect::<String>())
                .map(|desc| desc.trim().to_string());

            let property = Property {
                id: None,
                external_id: property_url.clone(),
                source: "argenprop".to_string(),
                property_type: Some(property_type.clone()),
                title,
                description,
                price_usd,
                address,
                covered_size,
                rooms,
                antiquity,
                url: Url::parse(&property_url).map_err(|e| BreaError::Scraping(e.to_string()))?,
                created_at: now,
                updated_at: now,
            };

            // Extract images
            let mut images = Vec::new();
            for img in item.select(&images_selector) {
                if let Some(img_url) = img.value().attr("src").or_else(|| img.value().attr("data-src")) {
                    if let Ok(url) = Url::parse(img_url) {
                        let image = PropertyImage {
                            id: None,
                            property_id: 0, // This will be set after property is inserted
                            url,
                            local_path: PathBuf::new(), // This will be set when downloading
                            hash: Vec::new(), // This will be set when downloading
                            created_at: now,
                        };
                        images.push(image);
                    }
                }
            }

            property_data.push((property, images));
        }

        // Check if there's a next page
        let has_next = self.has_next_page(&html)?;
        debug!("Has next page: {}", has_next);

        Ok((property_data, has_next))
    }

    async fn get_next_page_url(&self, current_url: &str) -> Result<Option<String>> {
        let url = Url::parse(current_url)
            .map_err(|e| BreaError::Scraping(format!("Failed to parse URL: {}", e)))?;

        // Get current page from query or default to 1
        let current_page = url.query_pairs()
            .find(|(key, _)| key.starts_with("pagina-"))
            .and_then(|(key, _)| {
                let key_str = key.to_string();
                let page_str = key_str.split('-').nth(1)?.to_string();
                page_str.parse::<u32>().ok()
            })
            .unwrap_or(1);

        let next_page = current_page + 1;
        let mut new_url = url.clone();
        new_url.set_query(Some(&format!("pagina-{}", next_page)));
        
        info!("Building next page URL: {}", new_url.as_str());
        Ok(Some(new_url.to_string()))
    }

    async fn scrape_property(&self, url: &str) -> Result<(Property, Vec<PropertyImage>)> {
        let html = self.fetch_page(url).await?;
        let document = Html::parse_document(&html);

        // Extract property type from URL
        let property_type = url.split('/')
            .nth(3)
            .and_then(|t| match t {
                "casas" => Some(PropertyType::House),
                "departamentos" => Some(PropertyType::Apartment),
                "terrenos" => Some(PropertyType::Land),
                "depto-tipo-casa" => Some(PropertyType::Ph),
                "local" => Some(PropertyType::Local),
                "campo" => Some(PropertyType::Field),
                "cochera" => Some(PropertyType::Garage),
                "fondo-comercio" => Some(PropertyType::CommercialPremises),
                "galpon" => Some(PropertyType::Warehouse),
                "hotel" => Some(PropertyType::Hotel),
                "negocio-especial" => Some(PropertyType::SpecialBusiness),
                "oficina" => Some(PropertyType::Office),
                "quinta" => Some(PropertyType::CountryHouse),
                _ => None,
            })
            .ok_or_else(|| BreaError::Scraping("Could not determine property type from URL".to_string()))?;

        let (
            title_selector,
            price_selector,
            address_selector,
            description_selector,
            features_selector,
            images_selector,
        ) = Self::create_detail_selectors()?;

        // Extract property details
        let title = document.select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();

        let price_text = document.select(&price_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();
        let price_usd = self.parse_price(&price_text).unwrap_or(0.0);

        let address = document.select(&address_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();

        let description = document.select(&description_selector)
            .next()
            .map(|el| el.text().collect::<String>());

        let mut covered_size = 0.0;
        let mut rooms = 0;
        let mut antiquity = 0;

        // Extract features
        for feature in document.select(&features_selector) {
            let text = feature.text().collect::<String>();
            if text.contains("m² cubie.") {
                covered_size = self.parse_size(&text).unwrap_or(0.0);
            } else if text.contains("dorm.") {
                rooms = self.parse_rooms(&text).unwrap_or(0);
            } else if text.contains("años") {
                antiquity = self.parse_antiquity(&text).unwrap_or(0);
            }
        }

        // Extract images
        let mut images = Vec::new();
        for img in document.select(&images_selector) {
            if let Some(img_url) = img.value().attr("src").or_else(|| img.value().attr("data-src")) {
                if let Ok(url) = Url::parse(img_url) {
                    let image = PropertyImage {
                        id: None,
                        property_id: 0, // This will be set after property is inserted
                        url,
                        local_path: PathBuf::new(), // This will be set when downloading
                        hash: Vec::new(), // This will be set when downloading
                        created_at: Utc::now(),
                    };
                    images.push(image);
                }
            }
        }

        let external_id = url.split('/')
            .last()
            .unwrap_or_default()
            .to_string();

        let property = Property {
            id: None,
            external_id,
            source: "argenprop".to_string(),
            property_type: Some(property_type),
            title,
            description,
            price_usd,
            address,
            covered_size,
            rooms,
            antiquity,
            url: Url::parse(url).map_err(|e| BreaError::Scraping(e.to_string()))?,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        Ok((property, images))
    }
} 