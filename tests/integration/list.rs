use brea_core::{Database, Property, PropertyType, PropertyStatus};
use brea_scrapers::{ArgenPropScraper, Scraper};
use tempfile::tempdir;
use std::sync::Arc;

#[tokio::test]
async fn test_list_with_filters() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path).await.unwrap();

    // Create and save test properties with different attributes
    let properties = vec![
        Property {
            id: None,
            external_id: "test-1".to_string(),
            source: "test".to_string(),
            property_type: Some(PropertyType::Apartment),
            district: "test".to_string(),
            title: "Test Property 1".to_string(),
            description: None,
            price_usd: 100000.0,
            address: "Test Address 1".to_string(),
            covered_size: Some(100.0),
            rooms: Some(2),
            antiquity: Some(5),
            url: "https://example.com/1".parse().unwrap(),
            status: PropertyStatus::Active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        Property {
            id: None,
            external_id: "test-2".to_string(),
            source: "test".to_string(),
            property_type: Some(PropertyType::House),
            district: "test".to_string(),
            title: "Test Property 2".to_string(),
            description: None,
            price_usd: 200000.0,
            address: "Test Address 2".to_string(),
            covered_size: Some(200.0),
            rooms: Some(3),
            antiquity: Some(10),
            url: "https://example.com/2".parse().unwrap(),
            status: PropertyStatus::Sold,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ];

    for mut property in properties {
        db.save_property(&mut property).await.unwrap();
    }

    // Test price filtering
    let filtered = db.list_properties(
        None,
        Some(150000.0),
        Some(250000.0),
        None,
        None,
        None,
        None,
        None,
        false,
    ).await.unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].price_usd, 200000.0);

    // Test size filtering
    let filtered = db.list_properties(
        None,
        None,
        None,
        Some(150.0),
        Some(250.0),
        None,
        None,
        None,
        false,
    ).await.unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].covered_size, Some(200.0));

    // Test source filtering
    let filtered = db.list_properties(
        Some("test"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
    ).await.unwrap();
    assert_eq!(filtered.len(), 2);

    // Test sorting
    let sorted = db.list_properties(
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some("price_usd"),
        true,
    ).await.unwrap();
    assert_eq!(sorted[0].price_usd, 200000.0);
    assert_eq!(sorted[1].price_usd, 100000.0);

    // Test pagination
    let paginated = db.list_properties(
        None,
        None,
        None,
        None,
        None,
        Some(1),
        Some(1),
        None,
        false,
    ).await.unwrap();
    assert_eq!(paginated.len(), 1);
}

#[tokio::test]
async fn test_list_with_status() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path).await.unwrap();

    // Create and save test properties with different statuses
    let properties = vec![
        Property {
            id: None,
            external_id: "test-1".to_string(),
            source: "test".to_string(),
            property_type: Some(PropertyType::Apartment),
            district: "test".to_string(),
            title: "Test Property 1".to_string(),
            description: None,
            price_usd: 100000.0,
            address: "Test Address 1".to_string(),
            covered_size: Some(100.0),
            rooms: Some(2),
            antiquity: Some(5),
            url: "https://example.com/1".parse().unwrap(),
            status: PropertyStatus::Active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        Property {
            id: None,
            external_id: "test-2".to_string(),
            source: "test".to_string(),
            property_type: Some(PropertyType::House),
            district: "test".to_string(),
            title: "Test Property 2".to_string(),
            description: None,
            price_usd: 200000.0,
            address: "Test Address 2".to_string(),
            covered_size: Some(200.0),
            rooms: Some(3),
            antiquity: Some(10),
            url: "https://example.com/2".parse().unwrap(),
            status: PropertyStatus::Sold,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ];

    for mut property in properties {
        db.save_property(&mut property).await.unwrap();
    }

    // Test listing active properties
    let active = db.list_properties(
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
    ).await.unwrap();
    let active_count = active.iter().filter(|p| p.status == PropertyStatus::Active).count();
    assert_eq!(active_count, 1);

    // Test listing sold properties
    let sold = db.list_properties(
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
    ).await.unwrap();
    let sold_count = sold.iter().filter(|p| p.status == PropertyStatus::Sold).count();
    assert_eq!(sold_count, 1);
}

#[tokio::test]
async fn test_list_with_property_type() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path).await.unwrap();

    // Create and save test properties with different property types
    let properties = vec![
        Property {
            id: None,
            external_id: "test-1".to_string(),
            source: "test".to_string(),
            property_type: Some(PropertyType::Apartment),
            district: "test".to_string(),
            title: "Test Property 1".to_string(),
            description: None,
            price_usd: 100000.0,
            address: "Test Address 1".to_string(),
            covered_size: Some(100.0),
            rooms: Some(2),
            antiquity: Some(5),
            url: "https://example.com/1".parse().unwrap(),
            status: PropertyStatus::Active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        Property {
            id: None,
            external_id: "test-2".to_string(),
            source: "test".to_string(),
            property_type: Some(PropertyType::House),
            district: "test".to_string(),
            title: "Test Property 2".to_string(),
            description: None,
            price_usd: 200000.0,
            address: "Test Address 2".to_string(),
            covered_size: Some(200.0),
            rooms: Some(3),
            antiquity: Some(10),
            url: "https://example.com/2".parse().unwrap(),
            status: PropertyStatus::Active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ];

    for mut property in properties {
        db.save_property(&mut property).await.unwrap();
    }

    // Test listing apartments
    let apartments = db.list_properties(
        Some(PropertyType::Apartment),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
    ).await.unwrap();
    assert_eq!(apartments.len(), 1);
    assert_eq!(apartments[0].property_type, Some(PropertyType::Apartment));

    // Test listing houses
    let houses = db.list_properties(
        Some(PropertyType::House),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
    ).await.unwrap();
    assert_eq!(houses.len(), 1);
    assert_eq!(houses[0].property_type, Some(PropertyType::House));
}

#[tokio::test]
async fn test_list_with_district() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path).await.unwrap();

    // Create and save test properties with different districts
    let properties = vec![
        Property {
            id: None,
            external_id: "test-1".to_string(),
            source: "test".to_string(),
            property_type: Some(PropertyType::Apartment),
            district: "Palermo".to_string(),
            title: "Test Property 1".to_string(),
            description: None,
            price_usd: 100000.0,
            address: "Test Address 1".to_string(),
            covered_size: Some(100.0),
            rooms: Some(2),
            antiquity: Some(5),
            url: "https://example.com/1".parse().unwrap(),
            status: PropertyStatus::Active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
        Property {
            id: None,
            external_id: "test-2".to_string(),
            source: "test".to_string(),
            property_type: Some(PropertyType::Apartment),
            district: "Recoleta".to_string(),
            title: "Test Property 2".to_string(),
            description: None,
            price_usd: 200000.0,
            address: "Test Address 2".to_string(),
            covered_size: Some(200.0),
            rooms: Some(3),
            antiquity: Some(10),
            url: "https://example.com/2".parse().unwrap(),
            status: PropertyStatus::Active,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    ];

    for mut property in properties {
        db.save_property(&mut property).await.unwrap();
    }

    // Test listing properties in Palermo
    let palermo = db.list_properties(
        None,
        Some("Palermo".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        false,
    ).await.unwrap();
    assert_eq!(palermo.len(), 1);
    assert_eq!(palermo[0].district, "Palermo");

    // Test listing properties in Recoleta
    let recoleta = db.list_properties(
        None,
        Some("Recoleta".to_string()),
        None,
        None,
        None,
        None,
        None,
        None,
        false,
    ).await.unwrap();
    assert_eq!(recoleta.len(), 1);
    assert_eq!(recoleta[0].district, "Recoleta");
}

#[tokio::test]
async fn test_database_schema_errors() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path).await.unwrap();

    // Create a property with invalid status
    let mut property = Property {
        id: None,
        external_id: "test-1".to_string(),
        source: "test".to_string(),
        property_type: Some(PropertyType::Apartment),
        district: "test".to_string(),
        title: "Test Property 1".to_string(),
        description: None,
        price_usd: 100000.0,
        address: "Test Address 1".to_string(),
        covered_size: Some(100.0),
        rooms: Some(2),
        antiquity: Some(5),
        url: "https://example.com/1".parse().unwrap(),
        status: PropertyStatus::Active,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    // Save the property
    db.save_property(&mut property).await.unwrap();

    // Manually update the status to an invalid value
    sqlx::query(
        "UPDATE properties SET status = ? WHERE id = ?",
    )
    .bind(chrono::Utc::now().to_string())
    .bind(property.id.unwrap())
    .execute(&db.pool)
    .await
    .unwrap();

    // Try to list properties and verify we get a proper error
    let result = db.list_properties(
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        false,
    ).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Unknown property status"));
} 