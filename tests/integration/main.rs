use brea_core::{Database, PropertyType};
use brea_scrapers::{ArgenPropScraper, Scraper};
use tempfile::tempdir;

#[tokio::test]
async fn test_single_property_scraping() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path).await.unwrap();

    // Create a scraper for ArgenProp
    let scraper = ArgenPropScraper::new();

    // Scrape a single property
    let properties = scraper
        .scrape_listing(
            brea_scrapers::ScrapeQuery::new(
                "belgrano".to_string(),
                PropertyType::Apartment,
                None,
                None,
                None,
                None,
            ),
            1,
        )
        .await
        .unwrap();

    // Verify that we got properties
    assert!(!properties.is_empty());

    // Save the properties to the database
    for (mut property, _images) in properties {
        db.save_property(&mut property).await.unwrap();
    }

    // Verify that the properties were saved
    let saved_properties = db.list_properties(None, None, None, None, None, None, None, None, false).await.unwrap();
    assert!(!saved_properties.is_empty());
}

#[tokio::test]
async fn test_multiple_properties_scraping() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path).await.unwrap();

    // Create a scraper for ArgenProp
    let scraper = ArgenPropScraper::new();

    // Scrape multiple pages of properties
    let mut all_properties = Vec::new();
    for _ in 1..=3 {
        let properties = scraper
            .scrape_listing(
                brea_scrapers::ScrapeQuery::new(
                    "belgrano".to_string(),
                    PropertyType::Apartment,
                    None,
                    None,
                    None,
                    None,
                ),
                1,
            )
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
    let saved_properties = db.list_properties(None, None, None, None, None, None, None, None, false).await.unwrap();
    assert!(saved_properties.len() > 1);
}

#[tokio::test]
async fn test_property_update_detection() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path).await.unwrap();

    // Create a scraper for ArgenProp
    let scraper = ArgenPropScraper::new();

    // Scrape initial properties
    let initial_properties = scraper
        .scrape_listing(
            brea_scrapers::ScrapeQuery::new(
                "belgrano".to_string(),
                PropertyType::Apartment,
                None,
                None,
                None,
                None,
            ),
            1,
        )
        .await
        .unwrap();

    // Save initial properties
    for (mut property, _images) in initial_properties {
        db.save_property(&mut property).await.unwrap();
    }

    // Wait a bit to ensure timestamps are different
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Scrape again to detect updates
    let updated_properties = scraper
        .scrape_listing(
            brea_scrapers::ScrapeQuery::new(
                "belgrano".to_string(),
                PropertyType::Apartment,
                None,
                None,
                None,
                None,
            ),
            1,
        )
        .await
        .unwrap();

    // Save updated properties
    for (mut property, _images) in updated_properties {
        db.save_property(&mut property).await.unwrap();
    }

    // Verify that we can detect updates
    let saved_properties = db.list_properties(None, None, None, None, None, None, None, None, false).await.unwrap();
    assert!(!saved_properties.is_empty());

    // Check that we have price history entries
    for property in saved_properties {
        let price_history = db.get_price_history(property.id.unwrap()).await.unwrap();
        assert!(!price_history.is_empty());
    }
}

fn main() {
    println!("Running integration tests...");
} 