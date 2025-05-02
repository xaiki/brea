use super::types::{DbPropertyStatus, DbTimestamp};
use sqlx::sqlite::SqlitePool;
use std::fmt;

#[derive(Clone, Debug)]
pub struct Migration {
    version: i32,
    up: &'static str,
    down: &'static str,
}

impl Migration {
    pub const fn new(version: i32, up: &'static str, down: &'static str) -> Self {
        Self { version, up, down }
    }
}

impl fmt::Display for Migration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Migration {}", self.version)
    }
}

pub const MIGRATIONS: &[Migration] = &[
    Migration::new(
        1,
        r#"
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
            covered_size REAL,
            rooms INTEGER,
            antiquity INTEGER,
            url TEXT NOT NULL,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            UNIQUE(source, external_id)
        )
        "#,
        "DROP TABLE IF EXISTS properties",
    ),
    Migration::new(
        2,
        r#"
        -- Create a temporary table with the new schema
        CREATE TABLE properties_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            external_id TEXT NOT NULL,
            source TEXT NOT NULL,
            property_type TEXT,
            district TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            price_usd REAL NOT NULL,
            address TEXT NOT NULL,
            covered_size REAL,
            rooms INTEGER,
            antiquity INTEGER,
            url TEXT NOT NULL,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            UNIQUE(source, external_id)
        );

        -- Copy data from the old table to the new one
        INSERT INTO properties_new 
        SELECT * FROM properties;

        -- Drop the old table
        DROP TABLE properties;

        -- Rename the new table to the original name
        ALTER TABLE properties_new RENAME TO properties;
        "#,
        r#"
        -- Create a temporary table with the old schema
        CREATE TABLE properties_old (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            external_id TEXT NOT NULL,
            source TEXT NOT NULL,
            property_type TEXT,
            district TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            price_usd REAL NOT NULL,
            address TEXT NOT NULL,
            covered_size REAL,
            rooms INTEGER,
            antiquity INTEGER,
            url TEXT NOT NULL,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            UNIQUE(source, external_id)
        );

        -- Copy data from the current table to the old one
        INSERT INTO properties_old 
        SELECT * FROM properties;

        -- Drop the current table
        DROP TABLE properties;

        -- Rename the old table to the original name
        ALTER TABLE properties_old RENAME TO properties;
        "#,
    ),
    Migration::new(
        3,
        r#"
        CREATE TABLE IF NOT EXISTS property_price_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            property_id INTEGER NOT NULL,
            price_usd REAL NOT NULL,
            observed_at DATETIME NOT NULL,
            FOREIGN KEY(property_id) REFERENCES properties(id),
            UNIQUE(property_id, observed_at)
        )
        "#,
        "DROP TABLE IF EXISTS property_price_history",
    ),
    Migration::new(
        4,
        r#"
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
        "DROP TABLE IF EXISTS property_images",
    ),
    Migration::new(
        5,
        r#"
        -- Add status column with default value 'active'
        ALTER TABLE properties ADD COLUMN status TEXT NOT NULL DEFAULT 'active';
        
        -- Create index for status
        CREATE INDEX idx_properties_status ON properties(status);
        "#,
        r#"
        -- Drop the status index
        DROP INDEX IF EXISTS idx_properties_status;
        
        -- Remove the status column
        ALTER TABLE properties DROP COLUMN status;
        "#,
    ),
    Migration::new(
        6,
        r#"
        -- Disable foreign key constraints
        PRAGMA foreign_keys = OFF;

        -- Drop foreign key constraints
        DROP TABLE IF EXISTS property_images;
        DROP TABLE IF EXISTS property_price_history;

        -- Create a temporary table with the new schema (covered_size as nullable)
        CREATE TABLE properties_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            external_id TEXT NOT NULL,
            source TEXT NOT NULL,
            property_type TEXT,
            district TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            price_usd REAL NOT NULL,
            address TEXT NOT NULL,
            covered_size REAL,
            rooms INTEGER,
            antiquity INTEGER,
            url TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'active',
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            UNIQUE(source, external_id)
        );

        -- Copy data from the old table to the new one
        INSERT INTO properties_new 
        SELECT * FROM properties;

        -- Drop the old table
        DROP TABLE properties;

        -- Rename the new table to the original name
        ALTER TABLE properties_new RENAME TO properties;

        -- Recreate the status index
        CREATE INDEX idx_properties_status ON properties(status);

        -- Recreate property_images table with foreign key
        CREATE TABLE property_images (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            property_id INTEGER NOT NULL,
            url TEXT NOT NULL,
            local_path TEXT NOT NULL,
            hash BLOB NOT NULL,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            FOREIGN KEY(property_id) REFERENCES properties(id),
            UNIQUE(property_id, url)
        );

        -- Recreate property_price_history table with foreign key
        CREATE TABLE property_price_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            property_id INTEGER NOT NULL,
            price_usd REAL NOT NULL,
            observed_at DATETIME NOT NULL,
            FOREIGN KEY(property_id) REFERENCES properties(id),
            UNIQUE(property_id, observed_at)
        );

        -- Re-enable foreign key constraints
        PRAGMA foreign_keys = ON;
        "#,
        r#"
        -- Disable foreign key constraints
        PRAGMA foreign_keys = OFF;

        -- Drop foreign key constraints
        DROP TABLE IF EXISTS property_images;
        DROP TABLE IF EXISTS property_price_history;

        -- Create a temporary table with the old schema
        CREATE TABLE properties_old (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            external_id TEXT NOT NULL,
            source TEXT NOT NULL,
            property_type TEXT,
            district TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            price_usd REAL NOT NULL,
            address TEXT NOT NULL,
            covered_size REAL,
            rooms INTEGER,
            antiquity INTEGER,
            url TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'active',
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            UNIQUE(source, external_id)
        );

        -- Copy data from the current table to the old one
        INSERT INTO properties_old 
        SELECT * FROM properties;

        -- Drop the current table
        DROP TABLE properties;

        -- Rename the old table to the original name
        ALTER TABLE properties_old RENAME TO properties;

        -- Recreate the status index
        CREATE INDEX idx_properties_status ON properties(status);

        -- Recreate property_images table with foreign key
        CREATE TABLE property_images (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            property_id INTEGER NOT NULL,
            url TEXT NOT NULL,
            local_path TEXT NOT NULL,
            hash BLOB NOT NULL,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            FOREIGN KEY(property_id) REFERENCES properties(id),
            UNIQUE(property_id, url)
        );

        -- Recreate property_price_history table with foreign key
        CREATE TABLE property_price_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            property_id INTEGER NOT NULL,
            price_usd REAL NOT NULL,
            observed_at DATETIME NOT NULL,
            FOREIGN KEY(property_id) REFERENCES properties(id),
            UNIQUE(property_id, observed_at)
        );

        -- Re-enable foreign key constraints
        PRAGMA foreign_keys = ON;
        "#,
    ),
    Migration::new(
        7,
        r#"
        -- Create a temporary table with the new schema
        CREATE TABLE properties_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            external_id TEXT NOT NULL,
            source TEXT NOT NULL,
            property_type TEXT,
            district TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            price_usd REAL NOT NULL,
            address TEXT NOT NULL,
            covered_size REAL,
            rooms INTEGER,
            antiquity INTEGER,
            url TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'active',
            created_at DATETIME NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE(source, external_id)
        );

        -- Copy data from the old table to the new one
        INSERT INTO properties_new 
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
            status,
            created_at,
            CASE 
                WHEN typeof(updated_at) = 'datetime' THEN datetime(updated_at)
                ELSE updated_at
            END as updated_at
        FROM properties;

        -- Drop the old table
        DROP TABLE properties;

        -- Rename the new table to the original name
        ALTER TABLE properties_new RENAME TO properties;
        "#,
        r#"
        -- Create a temporary table with the old schema
        CREATE TABLE properties_old (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            external_id TEXT NOT NULL,
            source TEXT NOT NULL,
            property_type TEXT,
            district TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            price_usd REAL NOT NULL,
            address TEXT NOT NULL,
            covered_size REAL,
            rooms INTEGER,
            antiquity INTEGER,
            url TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'active',
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            UNIQUE(source, external_id)
        );

        -- Copy data from the current table to the old one
        INSERT INTO properties_old 
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
            status,
            created_at,
            CASE 
                WHEN typeof(updated_at) = 'text' THEN datetime(updated_at)
                ELSE updated_at
            END as updated_at
        FROM properties;

        -- Drop the current table
        DROP TABLE properties;

        -- Rename the old table to the original name
        ALTER TABLE properties_old RENAME TO properties;
        "#,
    ),
];

pub async fn apply_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Create migrations table if it doesn't exist
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS migrations (
            version INTEGER PRIMARY KEY,
            applied_at DATETIME NOT NULL
        )"
    )
    .execute(pool)
    .await?;

    // Get applied migrations
    let applied_versions: Vec<i32> = sqlx::query_scalar("SELECT version FROM migrations ORDER BY version")
        .fetch_all(pool)
        .await?;

    // Apply pending migrations
    for migration in MIGRATIONS {
        if !applied_versions.contains(&migration.version) {
            // Apply migration
            sqlx::query(migration.up)
                .execute(pool)
                .await?;

            // Record migration
            sqlx::query(
                "INSERT INTO migrations (version, applied_at) VALUES (?, ?)"
            )
            .bind(migration.version)
            .bind(chrono::Utc::now())
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}

pub async fn rollback_migration(pool: &SqlitePool, version: i32) -> Result<(), sqlx::Error> {
    // Find the migration
    let migration = MIGRATIONS
        .iter()
        .find(|m| m.version == version)
        .ok_or_else(|| sqlx::Error::Decode("Migration not found".into()))?;

    // Apply down migration
    sqlx::query(migration.down)
        .execute(pool)
        .await?;

    // Remove migration record
    sqlx::query("DELETE FROM migrations WHERE version = ?")
        .bind(version)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_applied_migrations(pool: &SqlitePool) -> Result<Vec<Migration>, sqlx::Error> {
    // Get applied migrations
    let applied_versions: Vec<i32> = sqlx::query_scalar("SELECT version FROM migrations ORDER BY version")
        .fetch_all(pool)
        .await?;

    // Return applied migrations
    Ok(MIGRATIONS
        .iter()
        .filter(|m| applied_versions.contains(&m.version))
        .cloned()
        .collect())
} 