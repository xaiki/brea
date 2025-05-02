use brea_core::{Database, PropertyType};
use brea_core::db::migrations::apply_migrations;
use brea_core::db::types::DbTimestamp;
use brea_scrapers::{ArgenPropScraper, Scraper};
use tempfile::tempdir;
use std::sync::Arc;
use std::fs;

#[tokio::test]
async fn test_single_property_scraping() {
    let db = Database::new(":memory:").await.unwrap();
    apply_migrations(db.pool()).await.unwrap();

    // Create a scraper for ArgenProp
    let scraper = ArgenPropScraper::new();

    // Scrape a single property
    let query = brea_scrapers::ScrapeQuery::new(
        "belgrano".to_string(),
        PropertyType::Apartment,
        None,
        None,
        None,
        None,
        None,
    );
    let properties = scraper
        .scrape_listing(query, 1)
        .await
        .unwrap();

    // Verify that we got properties
    assert!(!properties.is_empty());

    // Save the properties to the database
    for (mut property, _images) in properties {
        db.save_property(&mut property).await.unwrap();
    }

    // Verify that the properties were saved
    let saved_properties = db.get_properties().await.unwrap();
    assert!(!saved_properties.is_empty());
}

#[tokio::test]
async fn test_multiple_properties_scraping() {
    let db = Database::new(":memory:").await.unwrap();
    apply_migrations(db.pool()).await.unwrap();

    // Create a scraper for ArgenProp
    let scraper = ArgenPropScraper::new();

    // Scrape multiple pages of properties
    let mut all_properties = Vec::new();
    for _ in 1..=3 {
        let query = brea_scrapers::ScrapeQuery::new(
            "belgrano".to_string(),
            PropertyType::Apartment,
            None,
            None,
            None,
            None,
            None,
        );
        let properties = scraper
            .scrape_listing(query, 1)
            .await
            .unwrap();
        all_properties.extend(properties);
    }

    // Verify that we got multiple properties
    assert!(all_properties.len() > 1);

    // Save all properties to the database
    for (mut property, _images) in all_properties {
        db.save_property(&mut property).await.unwrap();
    }

    // Verify that all properties were saved
    let saved_properties = db.get_properties().await.unwrap();
    assert!(saved_properties.len() > 1);
}

#[tokio::test]
async fn test_property_update_detection() {
    let db = Database::new(":memory:").await.unwrap();
    apply_migrations(db.pool()).await.unwrap();

    // Create a scraper for ArgenProp
    let scraper = ArgenPropScraper::new();

    // Scrape initial properties
    let query = brea_scrapers::ScrapeQuery::new(
        "belgrano".to_string(),
        PropertyType::Apartment,
        None,
        None,
        None,
        None,
        None,
    );
    let initial_properties = scraper
        .scrape_listing(query, 1)
        .await
        .unwrap();

    // Save initial properties
    for (mut property, _images) in initial_properties {
        db.save_property(&mut property).await.unwrap();
    }

    // Wait a bit to ensure timestamps are different
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Scrape again to detect updates
    let query = brea_scrapers::ScrapeQuery::new(
        "belgrano".to_string(),
        PropertyType::Apartment,
        None,
        None,
        None,
        None,
        None,
    );
    let updated_properties = scraper
        .scrape_listing(query, 1)
        .await
        .unwrap();

    // Save updated properties
    for (mut property, _images) in updated_properties {
        db.save_property(&mut property).await.unwrap();
    }

    // Verify that we can detect updates
    let saved_properties = db.get_properties().await.unwrap();
    assert!(!saved_properties.is_empty());

    // Check that we have price history entries
    for property in saved_properties {
        let price_history = db.get_price_history(property.id).await.unwrap();
        assert!(!price_history.is_empty());
    }
}

#[tokio::test]
async fn test_scraping() -> Result<(), Box<dyn std::error::Error>> {
    let scraper = brea_scrapers::ScraperFactory::create_scraper(brea_scrapers::ScraperType::Argenprop);
    
    let query = brea_scrapers::ScrapeQuery::new(
        "belgrano".to_string(),
        PropertyType::Apartment,
        None,
        None,
        None,
        None,
        None,
    );
    
    let properties = scraper.scrape_listing(query, 1).await?;
    assert!(!properties.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_scraping_with_filters() -> Result<(), Box<dyn std::error::Error>> {
    let scraper = brea_scrapers::ScraperFactory::create_scraper(brea_scrapers::ScraperType::Argenprop);
    
    let query = brea_scrapers::ScrapeQuery::new(
        "belgrano".to_string(),
        PropertyType::Apartment,
        Some(100_000.0),
        Some(200_000.0),
        Some(50.0),
        Some(100.0),
        None,
    );
    
    let properties = scraper.scrape_listing(query, 1).await?;
    assert!(!properties.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_scraping_with_database() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::new(":memory:").await?;
    apply_migrations(db.pool()).await?;
    let db = Arc::new(db);
    let scraper = brea_scrapers::ScraperFactory::create_scraper(brea_scrapers::ScraperType::Argenprop);
    
    let query = brea_scrapers::ScrapeQuery::new(
        "belgrano".to_string(),
        PropertyType::Apartment,
        None,
        None,
        None,
        None,
        Some(Arc::clone(&db)),
    );
    
    let properties = scraper.scrape_listing(query, 1).await?;
    assert!(!properties.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_scraping_with_database_and_filters() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::new(":memory:").await?;
    apply_migrations(db.pool()).await?;
    let db = Arc::new(db);
    let scraper = brea_scrapers::ScraperFactory::create_scraper(brea_scrapers::ScraperType::Argenprop);
    
    let query = brea_scrapers::ScrapeQuery::new(
        "belgrano".to_string(),
        PropertyType::Apartment,
        Some(100_000.0),
        Some(200_000.0),
        Some(50.0),
        Some(100.0),
        Some(Arc::clone(&db)),
    );
    
    let properties = scraper.scrape_listing(query, 1).await?;
    assert!(!properties.is_empty());
    Ok(())
}

fn main() {
    println!("Running integration tests...");
} 