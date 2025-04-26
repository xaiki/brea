use brea_core::{Database, PropertyType};
use tempfile::tempdir;

#[tokio::test]
async fn test_rollback_to_v1() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary database
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path).await?;

    // Apply all migrations
    db.migrate().await?;

    // Verify we're at the latest version
    let migrations = db.get_applied_migrations().await?;
    assert!(!migrations.is_empty());
    assert_eq!(migrations.last().unwrap().version, 2);

    // Rollback to v1
    db.rollback_to(1).await?;

    // Verify we're at v1
    let migrations = db.get_applied_migrations().await?;
    assert_eq!(migrations.len(), 1);
    assert_eq!(migrations[0].version, 1);

    Ok(())
}

#[tokio::test]
async fn test_migration_sequence() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary database
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path).await?;

    // Apply all migrations
    db.migrate().await?;

    // Verify migration sequence
    let migrations = db.get_applied_migrations().await?;
    assert_eq!(migrations.len(), 2);
    assert_eq!(migrations[0].version, 1);
    assert_eq!(migrations[1].version, 2);

    Ok(())
}

#[tokio::test]
async fn test_rollback_all() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary database
    let temp_dir = tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(&db_path).await?;

    // Apply all migrations
    db.migrate().await?;

    // Rollback all migrations
    db.rollback_all().await?;

    // Verify no migrations are applied
    let migrations = db.get_applied_migrations().await?;
    assert!(migrations.is_empty());

    Ok(())
} 