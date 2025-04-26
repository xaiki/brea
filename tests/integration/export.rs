use brea_core::{Database, Property, PropertyType, PropertyStatus};
use tempfile::tempdir;
use std::fs;
use std::path::Path;

#[tokio::test]
async fn test_export_to_csv() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path).await.unwrap();
    let export_path = temp_dir.path().join("export.csv");

    // Create and save test properties
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

    // Export properties to CSV
    let mut writer = csv::Writer::from_path(&export_path).unwrap();
    let properties = db.list_properties(None, None, None, None, None, None, None, None, false).await.unwrap();
    
    for property in properties {
        writer.serialize(property).unwrap();
    }
    writer.flush().unwrap();

    // Verify CSV file exists and has content
    assert!(export_path.exists());
    let content = fs::read_to_string(&export_path).unwrap();
    assert!(!content.is_empty());
    assert!(content.contains("Test Property 1"));
    assert!(content.contains("Test Property 2"));
}

#[tokio::test]
async fn test_export_with_status_filter() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path).await.unwrap();
    let export_path = temp_dir.path().join("export.csv");

    // Create and save test properties
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

    // Export only active properties to CSV
    let mut writer = csv::Writer::from_path(&export_path).unwrap();
    let properties = db.list_properties(None, None, None, None, None, None, None, None, false).await.unwrap();
    let active_properties: Vec<_> = properties.into_iter().filter(|p| p.status == PropertyStatus::Active).collect();
    
    for property in active_properties {
        writer.serialize(property).unwrap();
    }
    writer.flush().unwrap();

    // Verify CSV file exists and has only active properties
    assert!(export_path.exists());
    let content = fs::read_to_string(&export_path).unwrap();
    assert!(!content.is_empty());
    assert!(content.contains("Test Property 1"));
    assert!(!content.contains("Test Property 2"));
}

#[tokio::test]
async fn test_export_error_handling() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path).await.unwrap();
    let export_path = temp_dir.path().join("export.csv");

    // Try to export from empty database
    let mut writer = csv::Writer::from_path(&export_path).unwrap();
    let properties = db.list_properties(None, None, None, None, None, None, None, None, false).await.unwrap();
    
    for property in properties {
        writer.serialize(property).unwrap();
    }
    writer.flush().unwrap();

    // Verify CSV file exists but is empty (only headers)
    assert!(export_path.exists());
    let content = fs::read_to_string(&export_path).unwrap();
    assert!(!content.is_empty());
    assert!(!content.contains("Test Property"));
} 