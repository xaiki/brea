pub mod migrations;
pub mod queries;
pub mod types;

pub use migrations::apply_migrations;
pub use queries::{PropertyQueryBuilder, PropertyImageQueryBuilder};
pub use types::{DbPropertyStatus, STATUS_ACTIVE, STATUS_SOLD, STATUS_REMOVED};

use crate::{Property, PropertyImage, Result};
use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqlitePool, Row};
use std::path::Path;
use tempfile::NamedTempFile;
use std::path::PathBuf;
use crate::db::migrations::Migration;
use crate::db::types::DbTimestamp;

#[derive(Clone, Debug)]
pub struct Database {
    pool: SqlitePool,
    migrations: Vec<Migration>,
}

impl Database {
    pub async fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let pool = SqlitePool::connect(&format!("sqlite:{}", db_path.as_ref().display())).await?;
        let migrations = Vec::new();
        Ok(Self { pool, migrations })
    }

    pub async fn new_without_migrations(db_path: impl AsRef<Path>) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let pool = SqlitePool::connect(&format!("sqlite:{}", db_path.as_ref().display())).await?;
        let migrations = Vec::new();
        Ok(Self { pool, migrations })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn save_property(&self, property: &mut Property) -> Result<()> {
        // First try to find an existing property with the same source and external_id
        let existing_property = sqlx::query_as::<_, Property>(
            "SELECT * FROM properties WHERE source = ? AND external_id = ?"
        )
        .bind(&property.source)
        .bind(&property.external_id)
        .fetch_optional(&self.pool)
        .await?;

        match existing_property {
            Some(existing) => {
                // Update the property's ID to match the existing one
                property.id = existing.id;
                // Record price history if the price has changed
                if existing.price_usd != property.price_usd {
                    self.record_price_history(
                        existing.id,
                        property.price_usd,
                        DbTimestamp::now()
                    ).await?;
                }
                // Update the existing property
                self.update_property(property).await
            }
            None => {
                // Insert as a new property
                let id = sqlx::query(
                    r#"
                    INSERT INTO properties (
                        external_id, source, property_type, district, title,
                        description, price_usd, address, covered_size, rooms,
                        antiquity, url, status, created_at, updated_at
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    "#,
                )
                .bind(&property.external_id)
                .bind(&property.source)
                .bind(&property.property_type)
                .bind(&property.district)
                .bind(&property.title)
                .bind(&property.description)
                .bind(property.price_usd)
                .bind(&property.address)
                .bind(property.covered_size)
                .bind(property.rooms)
                .bind(property.antiquity)
                .bind(&property.url)
                .bind(&property.status)
                .bind(&property.created_at)
                .bind(&property.updated_at)
                .execute(&self.pool)
                .await?
                .last_insert_rowid();

                property.id = id;

                // Record initial price history
                self.record_price_history(
                    id,
                    property.price_usd,
                    DbTimestamp::now()
                ).await?;

                Ok(())
            }
        }
    }

    pub async fn update_property(&self, property: &Property) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE properties SET
                external_id = ?,
                source = ?,
                property_type = ?,
                district = ?,
                title = ?,
                description = ?,
                price_usd = ?,
                address = ?,
                covered_size = ?,
                rooms = ?,
                antiquity = ?,
                url = ?,
                status = ?,
                created_at = ?,
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&property.external_id)
        .bind(&property.source)
        .bind(&property.property_type)
        .bind(&property.district)
        .bind(&property.title)
        .bind(&property.description)
        .bind(property.price_usd)
        .bind(&property.address)
        .bind(property.covered_size)
        .bind(property.rooms)
        .bind(property.antiquity)
        .bind(&property.url)
        .bind(&property.status)
        .bind(&property.created_at)
        .bind(&property.updated_at)
        .bind(property.id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_property(&self, id: i64) -> Result<Option<Property>> {
        let property = sqlx::query_as::<_, Property>(
            "SELECT * FROM properties WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(property)
    }

    pub async fn get_property_by_external_id(&self, external_id: &str) -> Result<Option<Property>> {
        let property = sqlx::query_as::<_, Property>(
            "SELECT * FROM properties WHERE external_id = ?"
        )
        .bind(external_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(property)
    }

    pub async fn get_properties(&self) -> Result<Vec<Property>> {
        let properties = sqlx::query_as::<_, Property>(
            "SELECT * FROM properties ORDER BY id DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(properties)
    }

    pub async fn get_active_properties(&self) -> Result<Vec<Property>> {
        PropertyQueryBuilder::new()
            .with_status(DbPropertyStatus::new(STATUS_ACTIVE))
            .execute(&self.pool)
            .await
    }

    pub async fn get_sold_properties(&self) -> Result<Vec<Property>> {
        PropertyQueryBuilder::new()
            .with_status(DbPropertyStatus::new(STATUS_SOLD))
            .execute(&self.pool)
            .await
    }

    pub async fn get_removed_properties(&self) -> Result<Vec<Property>> {
        PropertyQueryBuilder::new()
            .with_status(DbPropertyStatus::new(STATUS_REMOVED))
            .execute(&self.pool)
            .await
    }

    pub async fn get_price_history(&self, property_id: i64) -> Result<Vec<(f64, DateTime<Utc>)>> {
        let rows = sqlx::query(
            "SELECT price_usd, observed_at FROM property_price_history WHERE property_id = ? ORDER BY observed_at DESC"
        )
        .bind(property_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| Ok((row.get("price_usd"), row.get("observed_at"))))
            .collect::<Result<Vec<_>>>()?)
    }

    pub async fn save_property_image(&self, image: &mut PropertyImage) -> Result<()> {
        let id = sqlx::query(
            r#"
            INSERT INTO property_images (
                property_id, url, local_path, hash,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(image.property_id)
        .bind(&image.url)
        .bind(&image.local_path)
        .bind(&image.hash)
        .bind(&image.created_at)
        .bind(&image.updated_at)
        .execute(&self.pool)
        .await?
        .last_insert_rowid();

        image.id = id;
        Ok(())
    }

    pub async fn update_property_image(&self, image: &PropertyImage) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE property_images SET
                property_id = ?,
                url = ?,
                local_path = ?,
                hash = ?,
                created_at = ?,
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(image.property_id)
        .bind(&image.url)
        .bind(&image.local_path)
        .bind(&image.hash)
        .bind(&image.created_at)
        .bind(&image.updated_at)
        .bind(image.id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_property_images(&self, property_id: i64) -> Result<Vec<PropertyImage>> {
        PropertyImageQueryBuilder::new()
            .with_property_id(property_id)
            .execute(&self.pool)
            .await
    }

    pub async fn get_primary_property_image(&self, property_id: i64) -> Result<Option<PropertyImage>> {
        let image = sqlx::query_as::<_, PropertyImage>(
            "SELECT * FROM property_images WHERE property_id = ? AND is_primary = 1"
        )
        .bind(property_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(image)
    }

    pub async fn detect_sold_properties(&self, current_external_ids: &[&str]) -> Result<Vec<Property>> {
        PropertyQueryBuilder::new()
            .with_status(DbPropertyStatus::new(STATUS_ACTIVE))
            .with_external_ids_not_in(current_external_ids)
            .execute(&self.pool)
            .await
    }

    pub async fn mark_property_as_sold(&self, property_id: i64) -> Result<()> {
        sqlx::query(
            "UPDATE properties SET status = ?, updated_at = ? WHERE id = ?"
        )
        .bind(DbPropertyStatus::new(STATUS_SOLD))
        .bind(DbTimestamp::now())
        .bind(property_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn mark_property_as_removed(&self, property_id: i64) -> Result<()> {
        sqlx::query(
            "UPDATE properties SET status = ?, updated_at = ? WHERE id = ?"
        )
        .bind(DbPropertyStatus::new(STATUS_REMOVED))
        .bind(DbTimestamp::now())
        .bind(property_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn cleanup_price_history(&self) -> Result<usize> {
        let result = sqlx::query(
            r#"
            DELETE FROM property_price_history
            WHERE id NOT IN (
                SELECT id FROM (
                    SELECT id, ROW_NUMBER() OVER (
                        PARTITION BY property_id
                        ORDER BY observed_at DESC
                    ) as rn
                    FROM property_price_history
                ) WHERE rn <= 10
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as usize)
    }

    async fn record_price_history(&self, property_id: i64, price_usd: f64, observed_at: DbTimestamp) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO property_price_history (property_id, price_usd, observed_at)
            VALUES (?, ?, ?)
            ON CONFLICT(property_id, observed_at) DO NOTHING
            "#
        )
        .bind(property_id)
        .bind(price_usd)
        .bind(&observed_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    async fn test_connection() -> Database {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let db = Database { pool, migrations: Vec::new() };
        apply_migrations(&db.pool).await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_database_creation() {
        let _db = test_connection().await;
    }

    #[tokio::test]
    async fn test_property_crud() {
        let db = test_connection().await;
        let now = DbTimestamp::now();

        // Create a property
        let mut property = Property {
            id: 0,
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
            status: DbPropertyStatus::new(STATUS_ACTIVE),
            created_at: now.clone(),
            updated_at: now,
        };

        // Save property
        db.save_property(&mut property).await.unwrap();
        assert!(property.id > 0);

        // Get property
        let retrieved = db.get_property(property.id).await.unwrap().unwrap();
        assert_eq!(retrieved.external_id, "test-123");
        assert_eq!(retrieved.price_usd, 100000.0);

        // Update property
        property.price_usd = 150000.0;
        property.updated_at = DbTimestamp::now();
        db.update_property(&property).await.unwrap();

        // Verify update
        let updated = db.get_property(property.id).await.unwrap().unwrap();
        assert_eq!(updated.price_usd, 150000.0);
    }

    #[tokio::test]
    async fn test_property_image_crud() {
        let db = test_connection().await;
        let now = DbTimestamp::now();

        // Create a property first
        let mut property = Property {
            id: 0,
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
            status: DbPropertyStatus::new(STATUS_ACTIVE),
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        db.save_property(&mut property).await.unwrap();

        // Create an image
        let mut image = PropertyImage {
            id: 0,
            property_id: property.id,
            url: "https://example.com/image.jpg".to_string(),
            local_path: "/tmp/images/test.jpg".to_string(),
            hash: vec![1, 2, 3, 4],
            created_at: now.clone(),
            updated_at: now,
        };

        // Save image
        db.save_property_image(&mut image).await.unwrap();
        assert!(image.id > 0);

        // Get images
        let images = db.get_property_images(property.id).await.unwrap();
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].url, "https://example.com/image.jpg");
    }

    #[tokio::test]
    async fn test_price_history_cleanup() {
        let db = test_connection().await;
        let now = DbTimestamp::now();

        // Create a property
        let mut property = Property {
            id: 0,
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
            status: DbPropertyStatus::new(STATUS_ACTIVE),
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        db.save_property(&mut property).await.unwrap();

        // Record some price history
        for i in 0..15 {
            let price = 100000.0 + (i as f64 * 10000.0);
            let timestamp = DbTimestamp::now();
            db.record_price_history(property.id, price, timestamp).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        // Cleanup price history
        let removed = db.cleanup_price_history().await.unwrap();
        assert!(removed > 0, "Should have removed some price history entries");

        // Verify cleanup
        let history = db.get_price_history(property.id).await.unwrap();
        assert!(history.len() <= 10, "Should have at most 10 price history entries");
        
        // Verify the entries are ordered by date descending
        for i in 1..history.len() {
            assert!(history[i-1].1 > history[i].1, "Price history should be ordered by date descending");
        }
    }

    #[tokio::test]
    async fn test_type_safe_query_builder() {
        let db = test_connection().await;
        let now = DbTimestamp::now();

        // Create some properties
        let mut property1 = Property {
            id: 0,
            external_id: "test-1".to_string(),
            source: "test".to_string(),
            property_type: Some("apartment".to_string()),
            district: "Test District".to_string(),
            title: "Test Property 1".to_string(),
            description: Some("Test description 1".to_string()),
            price_usd: 100000.0,
            address: "123 Test St".to_string(),
            covered_size: Some(100.0),
            rooms: Some(2),
            antiquity: Some(5),
            url: "https://example.com/test1".to_string(),
            status: DbPropertyStatus::new(STATUS_ACTIVE),
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        let mut property2 = Property {
            id: 0,
            external_id: "test-2".to_string(),
            source: "test".to_string(),
            property_type: Some("house".to_string()),
            district: "Test District".to_string(),
            title: "Test Property 2".to_string(),
            description: Some("Test description 2".to_string()),
            price_usd: 200000.0,
            address: "456 Test St".to_string(),
            covered_size: Some(150.0),
            rooms: Some(3),
            antiquity: Some(10),
            url: "https://example.com/test2".to_string(),
            status: DbPropertyStatus::new(STATUS_SOLD),
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        db.save_property(&mut property1).await.unwrap();
        db.save_property(&mut property2).await.unwrap();

        // Test query builder
        let active_properties = PropertyQueryBuilder::new()
            .with_status(DbPropertyStatus::new(STATUS_ACTIVE))
            .execute(&db.pool)
            .await
            .unwrap();

        assert_eq!(active_properties.len(), 1);
        assert_eq!(active_properties[0].external_id, "test-1");

        let sold_properties = PropertyQueryBuilder::new()
            .with_status(DbPropertyStatus::new(STATUS_SOLD))
            .execute(&db.pool)
            .await
            .unwrap();

        assert_eq!(sold_properties.len(), 1);
        assert_eq!(sold_properties[0].external_id, "test-2");
    }

    #[tokio::test]
    async fn test_type_safe_status_transitions() {
        let db = test_connection().await;
        let now = DbTimestamp::now();

        // Create a property
        let mut property = Property {
            id: 0,
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
            status: DbPropertyStatus::new(STATUS_ACTIVE),
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        db.save_property(&mut property).await.unwrap();

        // Test status transitions
        db.mark_property_as_sold(property.id).await.unwrap();
        let sold = db.get_property(property.id).await.unwrap().unwrap();
        assert_eq!(sold.status.as_str(), STATUS_SOLD);

        db.mark_property_as_removed(property.id).await.unwrap();
        let removed = db.get_property(property.id).await.unwrap().unwrap();
        assert_eq!(removed.status.as_str(), STATUS_REMOVED);
    }

    #[tokio::test]
    async fn test_type_safe_timestamp() {
        let db = test_connection().await;
        let now = DbTimestamp::now();

        // Create a property
        let mut property = Property {
            id: 0,
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
            status: DbPropertyStatus::new(STATUS_ACTIVE),
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        db.save_property(&mut property).await.unwrap();

        // Test timestamp handling
        let retrieved = db.get_property(property.id).await.unwrap().unwrap();
        assert_eq!(retrieved.created_at.to_string(), now.to_string());
        assert_eq!(retrieved.updated_at.to_string(), now.to_string());
    }
} 