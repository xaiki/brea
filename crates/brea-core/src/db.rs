use crate::{
    Property, PropertyImage, Result, BreaError,
};
use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqlitePool, QueryBuilder, Row, Sqlite, Column, TypeInfo};
use std::path::Path;
use std::fs;
use tracing::{debug, info};
use serde_json;
use hex;

#[cfg(test)]
use chrono::Duration;

#[cfg(test)]
use crate::PropertyType;

struct Migration {
    version: i32,
    up: &'static str,
    down: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        up: r#"
        CREATE TABLE IF NOT EXISTS properties (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            external_id TEXT NOT NULL,
            source TEXT NOT NULL,
            property_type TEXT,
            district TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            price_usd REAL NOT NULL,
            address TEXT NOT NULL,
            covered_size REAL NOT NULL,
            rooms INTEGER NOT NULL,
            antiquity INTEGER NOT NULL,
            url TEXT NOT NULL,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            UNIQUE(source, external_id)
        )
        "#,
        down: "DROP TABLE IF EXISTS properties",
    },
    Migration {
        version: 2,
        up: r#"
        CREATE TABLE IF NOT EXISTS property_price_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            property_id INTEGER NOT NULL,
            price_usd REAL NOT NULL,
            observed_at DATETIME NOT NULL,
            FOREIGN KEY(property_id) REFERENCES properties(id),
            UNIQUE(property_id, observed_at)
        )
        "#,
        down: "DROP TABLE IF EXISTS property_price_history",
    },
    Migration {
        version: 3,
        up: r#"
        CREATE TABLE IF NOT EXISTS property_images (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            property_id INTEGER NOT NULL,
            url TEXT NOT NULL,
            local_path TEXT NOT NULL,
            hash BLOB NOT NULL,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            FOREIGN KEY(property_id) REFERENCES properties(id),
            UNIQUE(property_id, url)
        )
        "#,
        down: "DROP TABLE IF EXISTS property_images",
    },
];

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let db_path = db_path.as_ref();
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create empty database file if it doesn't exist
        if !db_path.exists() {
            fs::write(db_path, "")?;
        }

        // Convert path to URL format
        let db_url = format!("sqlite:{}", db_path.to_string_lossy());
        
        let pool = SqlitePool::connect(&db_url).await?;
        let db = Self { pool };
        db.migrate().await?;

        Ok(db)
    }

    // For testing purposes only
    #[cfg(test)]
    pub(crate) async fn test_connection() -> Result<Self> {
        Self::new(":memory:").await
    }

    async fn create_migrations_table(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS migrations (
                version INTEGER PRIMARY KEY,
                applied_at DATETIME NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_applied_migrations(&self) -> Result<Vec<i32>> {
        let versions: Vec<i32> = sqlx::query_scalar("SELECT version FROM migrations ORDER BY version")
            .fetch_all(&self.pool)
            .await?;
        Ok(versions)
    }

    pub async fn migrate(&self) -> Result<()> {
        debug!("Running database migrations");
        
        // Create migrations table if it doesn't exist
        self.create_migrations_table().await?;

        // Get list of applied migrations
        let applied = self.get_applied_migrations().await?;

        // Apply pending migrations
        for migration in MIGRATIONS {
            if !applied.contains(&migration.version) {
                info!("Applying migration {}", migration.version);
                
                // Start transaction
                let mut tx = self.pool.begin().await?;
                
                // Apply migration
                sqlx::query(migration.up)
                    .execute(&mut *tx)
                    .await?;
                
                // Record migration
                sqlx::query(
                    "INSERT INTO migrations (version, applied_at) VALUES (?, ?)",
                )
                .bind(migration.version)
                .bind(Utc::now())
                .execute(&mut *tx)
                .await?;
                
                // Commit transaction
                tx.commit().await?;
                
                info!("Successfully applied migration {}", migration.version);
            }
        }

        // Clean up any redundant price history records
        let cleaned = self.cleanup_price_history().await?;
        if cleaned > 0 {
            debug!("Cleaned up {} redundant price history records", cleaned);
        }

        Ok(())
    }

    pub async fn rollback(&self, version: i32) -> Result<()> {
        debug!("Rolling back to version {}", version);
        
        // Get list of applied migrations
        let applied = self.get_applied_migrations().await?;
        
        // Find migrations to rollback
        let migrations_to_rollback: Vec<_> = MIGRATIONS
            .iter()
            .filter(|m| m.version > version && applied.contains(&m.version))
            .collect();
        
        // Rollback in reverse order
        for migration in migrations_to_rollback.iter().rev() {
            info!("Rolling back migration {}", migration.version);
            
            // Start transaction
            let mut tx = self.pool.begin().await?;
            
            // Rollback migration
            sqlx::query(migration.down)
                .execute(&mut *tx)
                .await?;
            
            // Remove migration record
            sqlx::query("DELETE FROM migrations WHERE version = ?")
                .bind(migration.version)
                .execute(&mut *tx)
                .await?;
            
            // Commit transaction
            tx.commit().await?;
            
            info!("Successfully rolled back migration {}", migration.version);
        }
        
        Ok(())
    }

    /// Dump all tables from the database
    pub async fn dump_tables(&self) -> Result<serde_json::Value> {
        let mut tables = serde_json::Map::new();
        
        // Get all table names
        let table_query = "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'";
        let table_rows: Vec<(String,)> = sqlx::query_as(table_query)
            .fetch_all(&self.pool)
            .await?;
            
        for (table_name,) in table_rows {
            // Get all rows from the table
            let rows = sqlx::query(&format!("SELECT * FROM {}", table_name))
                .fetch_all(&self.pool)
                .await?;
                
            let mut table_data = Vec::new();
            for row in rows {
                let mut row_obj = serde_json::Map::new();
                
                for (i, column) in row.columns().iter().enumerate() {
                    let column_name = column.name();
                    let value = match column.type_info().name() {
                        "TEXT" => serde_json::Value::String(row.get::<String, _>(i)),
                        "INTEGER" => serde_json::Value::Number(serde_json::Number::from(row.get::<i64, _>(i))),
                        "REAL" => {
                            let val: f64 = row.get(i);
                            if let Some(num) = serde_json::Number::from_f64(val) {
                                serde_json::Value::Number(num)
                            } else {
                                serde_json::Value::Null
                            }
                        },
                        "BLOB" => {
                            let bytes: Vec<u8> = row.get(i);
                            serde_json::Value::String(hex::encode(bytes))
                        },
                        _ => serde_json::Value::Null
                    };
                    row_obj.insert(column_name.to_string(), value);
                }
                table_data.push(serde_json::Value::Object(row_obj));
            }
            tables.insert(table_name, serde_json::Value::Array(table_data));
        }
        
        Ok(serde_json::Value::Object(tables))
    }

    /// Check database integrity
    pub async fn check_integrity(&self) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        // Check SQLite integrity
        let integrity_check: Vec<String> = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_all(&self.pool)
            .await?;
        
        if integrity_check.len() != 1 || integrity_check[0] != "ok" {
            issues.extend(integrity_check);
        }

        // Check foreign key constraints
        let foreign_key_violations: Vec<(i64, String, i64, String)> = sqlx::query_as(
            "PRAGMA foreign_key_check"
        )
        .fetch_all(&self.pool)
        .await?;

        if !foreign_key_violations.is_empty() {
            for (table_id, table_name, row_id, parent) in foreign_key_violations {
                issues.push(format!(
                    "Foreign key violation in table {} (id: {}) at row {} referencing {}",
                    table_name, table_id, row_id, parent
                ));
            }
        }

        // Check for orphaned images
        let orphaned_images: Vec<(i64,)> = sqlx::query_as(
            "SELECT i.id FROM property_images i LEFT JOIN properties p ON i.property_id = p.id WHERE p.id IS NULL"
        )
        .fetch_all(&self.pool)
        .await?;

        if !orphaned_images.is_empty() {
            issues.push(format!(
                "Found {} orphaned images (IDs: {})",
                orphaned_images.len(),
                orphaned_images
                    .iter()
                    .map(|(id,)| id.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        // Check for duplicate properties (same external_id and source)
        let duplicates: Vec<(String, String, i64)> = sqlx::query_as(
            "SELECT external_id, source, COUNT(*) as count 
             FROM properties 
             GROUP BY external_id, source 
             HAVING count > 1"
        )
        .fetch_all(&self.pool)
        .await?;

        if !duplicates.is_empty() {
            for (external_id, source, count) in duplicates {
                issues.push(format!(
                    "Found {} duplicate entries for property {} from {}",
                    count, external_id, source
                ));
            }
        }

        Ok(issues)
    }

    /// Dump all tables from the database
    pub async fn save_property(&self, property: &mut Property) -> Result<()> {
        let now = Utc::now();

        // Start a transaction
        let mut tx = self.pool.begin().await?;

        // Check if property already exists
        let existing = sqlx::query_as::<Sqlite, (i64, Option<f64>)>(
            r#"
            SELECT id, price_usd
            FROM properties
            WHERE source = ? AND external_id = ?
            "#
        )
        .bind(&property.source)
        .bind(&property.external_id)
        .fetch_optional(&mut *tx)
        .await?;

        match existing {
            Some((id, last_price)) => {
                // Update existing property
                property.id = Some(id);
                sqlx::query(
                    r#"
                    UPDATE properties 
                    SET property_type = ?,
                        district = ?,
                        title = ?,
                        description = ?,
                        price_usd = ?,
                        address = ?,
                        covered_size = ?,
                        rooms = ?,
                        antiquity = ?,
                        url = ?,
                        updated_at = ?
                    WHERE id = ?
                    "#
                )
                .bind(&property.property_type)
                .bind(&property.district)
                .bind(&property.title)
                .bind(&property.description)
                .bind(property.price_usd)
                .bind(&property.address)
                .bind(property.covered_size)
                .bind(property.rooms)
                .bind(property.antiquity)
                .bind(property.url.as_str())
                .bind(now)
                .bind(id)
                .execute(&mut *tx)
                .await?;

                // Save price history if price has changed
                if last_price != Some(property.price_usd) {
                    sqlx::query(
                        r#"
                        INSERT INTO property_price_history (property_id, price_usd, observed_at)
                        VALUES (?, ?, ?)
                        "#
                    )
                    .bind(id)
                    .bind(property.price_usd)
                    .bind(now)
                    .execute(&mut *tx)
                    .await?;
                }
            }
            None => {
                // Insert new property
                let result = sqlx::query(
                    r#"
                    INSERT INTO properties (
                        external_id, source, property_type, district, title,
                        description, price_usd, address, covered_size, rooms,
                        antiquity, url, created_at, updated_at
                    )
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    "#
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
                .bind(property.url.as_str())
                .bind(now)
                .bind(now)
                .execute(&mut *tx)
                .await?;

                property.id = Some(result.last_insert_rowid());

                // Save initial price history
                sqlx::query(
                    r#"
                    INSERT INTO property_price_history (property_id, price_usd, observed_at)
                    VALUES (?, ?, ?)
                    "#
                )
                .bind(property.id.unwrap())
                .bind(property.price_usd)
                .bind(now)
                .execute(&mut *tx)
                .await?;
            }
        }

        // Commit the transaction
        tx.commit().await?;

        Ok(())
    }

    pub async fn save_property_image(&self, image: &mut PropertyImage) -> Result<()> {
        let now = Utc::now();
        let path_str = image.local_path.to_str().ok_or_else(|| BreaError::Database(sqlx::Error::Protocol("Invalid path".into())))?;

        let id = sqlx::query(
            r#"
            INSERT INTO property_images (property_id, url, local_path, hash, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(property_id, url) DO UPDATE SET
                local_path = excluded.local_path,
                hash = excluded.hash,
                updated_at = excluded.updated_at
            RETURNING id
            "#,
        )
        .bind(image.property_id)
        .bind(image.url.as_str())
        .bind(path_str)
        .bind(&image.hash)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await?
        .get::<i64, _>(0);

        image.id = Some(id);
        Ok(())
    }

    pub async fn list_properties(
        &self,
        source: Option<&str>,
        min_price: Option<f64>,
        max_price: Option<f64>,
        min_size: Option<f64>,
        max_size: Option<f64>,
    ) -> Result<Vec<Property>> {
        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"
            SELECT 
                id,
                external_id,
                source,
                property_type,
                district,
                title,
                description,
                price_usd,
                address,
                covered_size,
                rooms,
                antiquity,
                url,
                created_at,
                updated_at
            FROM properties 
            WHERE 1=1
            "#,
        );

        if let Some(source) = source {
            query_builder.push(" AND source = ");
            query_builder.push_bind(source.to_string());
        }

        if let Some(min_price) = min_price {
            query_builder.push(" AND price_usd >= ");
            query_builder.push_bind(min_price);
        }

        if let Some(max_price) = max_price {
            query_builder.push(" AND price_usd <= ");
            query_builder.push_bind(max_price);
        }

        if let Some(min_size) = min_size {
            query_builder.push(" AND covered_size >= ");
            query_builder.push_bind(min_size);
        }

        if let Some(max_size) = max_size {
            query_builder.push(" AND covered_size <= ");
            query_builder.push_bind(max_size);
        }

        query_builder.push(" ORDER BY created_at DESC");

        let query = query_builder.build_query_as::<Property>();
        let properties = query.fetch_all(&self.pool).await?;

        Ok(properties)
    }

    pub async fn get_price_history(&self, property_id: i64) -> Result<Vec<(f64, DateTime<Utc>)>> {
        let history = sqlx::query(
            r#"
            SELECT price_usd, observed_at
            FROM property_price_history
            WHERE property_id = ?
            ORDER BY observed_at DESC
            "#,
        )
        .bind(property_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(history
            .into_iter()
            .map(|row| (row.get::<f64, _>("price_usd"), row.get::<DateTime<Utc>, _>("observed_at")))
            .collect())
    }

    pub async fn cleanup_price_history(&self) -> Result<usize> {
        // This query will:
        // 1. Keep the first record of each day
        // 2. Keep records where price changed from the previous record
        // 3. Delete all other records
        let deleted = sqlx::query(
            r#"
            WITH ranked_prices AS (
                SELECT 
                    id,
                    property_id,
                    price_usd,
                    observed_at,
                    LAG(price_usd) OVER (PARTITION BY property_id ORDER BY observed_at) as prev_price,
                    strftime('%Y-%m-%d', observed_at) as obs_date,
                    ROW_NUMBER() OVER (
                        PARTITION BY property_id, strftime('%Y-%m-%d', observed_at) 
                        ORDER BY observed_at
                    ) as daily_rank
                FROM property_price_history
            )
            DELETE FROM property_price_history
            WHERE id IN (
                SELECT id 
                FROM ranked_prices
                WHERE daily_rank > 1  -- Not the first record of the day
                AND (
                    prev_price IS NULL  -- Keep first record for each property
                    OR ABS(price_usd - prev_price) < prev_price * 0.001  -- Delete if price didn't change
                )
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(deleted.rows_affected() as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use url::Url;

    #[tokio::test]
    async fn test_database_creation() {
        let db = Database::test_connection().await.unwrap();
        sqlx::query("SELECT 1").execute(&db.pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_property_crud() {
        let db = Database::test_connection().await.unwrap();
        let now = Utc::now();

        // Create a test property
        let mut property = Property {
            id: None,
            external_id: "test123".to_string(),
            source: "argenprop".to_string(),
            property_type: Some(PropertyType::House),
            district: "Test District".to_string(),
            title: "Test Property".to_string(),
            description: Some("A test property".to_string()),
            price_usd: 100000.0,
            address: "123 Test St".to_string(),
            covered_size: 100.0,
            rooms: 3,
            antiquity: 5,
            url: Url::from_str("https://example.com/property/123").unwrap(),
            created_at: now,
            updated_at: now,
        };

        // Test insert
        db.save_property(&mut property).await.unwrap();
        assert!(property.id.is_some());
        let id = property.id.unwrap();

        // Test update
        property.price_usd = 110000.0;
        db.save_property(&mut property).await.unwrap();

        // Verify update
        let updated = sqlx::query(
            "SELECT price_usd FROM properties WHERE id = ?"
        )
        .bind(id)
        .fetch_one(&db.pool)
        .await
        .unwrap();
        assert_eq!(updated.get::<f64, _>("price_usd"), 110000.0);

        // Test duplicate handling
        let mut duplicate = property.clone();
        duplicate.id = None;
        db.save_property(&mut duplicate).await.unwrap();
        assert_eq!(duplicate.id, Some(id));
    }

    #[tokio::test]
    async fn test_property_image_crud() {
        let db = Database::test_connection().await.unwrap();
        let now = Utc::now();

        // First create a property
        let mut property = Property {
            id: None,
            external_id: "test123".to_string(),
            source: "argenprop".to_string(),
            property_type: Some(PropertyType::House),
            district: "Test District".to_string(),
            title: "Test Property".to_string(),
            description: Some("A test property".to_string()),
            price_usd: 100000.0,
            address: "123 Test St".to_string(),
            covered_size: 100.0,
            rooms: 3,
            antiquity: 5,
            url: Url::from_str("https://example.com/property/123").unwrap(),
            created_at: now,
            updated_at: now,
        };
        db.save_property(&mut property).await.unwrap();

        // Create a test image
        let mut image = PropertyImage {
            id: None,
            property_id: property.id.unwrap(),
            url: Url::from_str("https://example.com/image.jpg").unwrap(),
            local_path: std::path::PathBuf::from("/tmp/images/test.jpg"),
            hash: vec![1, 2, 3, 4],
            created_at: now,
        };

        // Test insert
        db.save_property_image(&mut image).await.unwrap();
        assert!(image.id.is_some());

        // Test duplicate handling
        let mut duplicate = image.clone();
        duplicate.id = None;
        db.save_property_image(&mut duplicate).await.unwrap();
        assert_eq!(duplicate.id, image.id);
    }

    #[tokio::test]
    async fn test_price_history_cleanup() {
        let db = Database::test_connection().await.unwrap();
        let now = Utc::now();

        // Create a test property
        let mut property = Property {
            id: None,
            external_id: "test123".to_string(),
            source: "argenprop".to_string(),
            property_type: Some(PropertyType::House),
            district: "Test District".to_string(),
            title: "Test Property".to_string(),
            description: Some("A test property".to_string()),
            price_usd: 100000.0,
            address: "123 Test St".to_string(),
            covered_size: 100.0,
            rooms: 3,
            antiquity: 5,
            url: Url::from_str("https://example.com/property/123").unwrap(),
            created_at: now,
            updated_at: now,
        };

        // Save initial property
        db.save_property(&mut property).await.unwrap();
        let id = property.id.unwrap();

        // Insert test price history records
        for i in 0..24 {
            // Create 24 records, one per hour for a day
            let time = now - Duration::hours(i);
            sqlx::query(
                r#"
                INSERT INTO property_price_history (property_id, price_usd, observed_at)
                VALUES (?, ?, ?)
                "#,
            )
            .bind(id)
            .bind(100000.0) // Same price
            .bind(time)
            .execute(&db.pool)
            .await
            .unwrap();
        }

        // Add one record with a price change
        sqlx::query(
            r#"
            INSERT INTO property_price_history (property_id, price_usd, observed_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(110000.0) // Price change
        .bind(now - Duration::hours(12))
        .execute(&db.pool)
        .await
        .unwrap();

        // Count initial records
        let initial_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM property_price_history")
            .fetch_one(&db.pool)
            .await
            .unwrap();
        assert_eq!(initial_count, 25); // 24 same price + 1 price change

        // Run cleanup
        let cleaned = db.cleanup_price_history().await.unwrap();
        assert!(cleaned > 0);

        // Verify we kept only the important records
        let final_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM property_price_history")
            .fetch_one(&db.pool)
            .await
            .unwrap();
        
        // We should have kept:
        // 1. The first record of the day
        // 2. The record where price changed
        assert_eq!(final_count, 2);
    }
} 