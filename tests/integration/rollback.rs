use brea_core::db::Database;
use brea_core::Result;
use std::path::Path;
use std::fs;

#[tokio::test]
async fn test_rollback_to_v1() -> Result<()> {
    // Create a temporary database file
    let db_path = Path::new("test_rollback.db");
    if db_path.exists() {
        fs::remove_file(db_path)?;
    }

    // Create a new database
    let db = Database::new(db_path).await?;

    // Verify all migrations are applied
    let applied = db.get_applied_migrations().await?;
    assert_eq!(applied.len(), 7, "Expected 7 migrations to be applied");

    // Rollback to version 1
    db.rollback(1).await?;

    // Verify only version 1 is applied
    let applied = db.get_applied_migrations().await?;
    assert_eq!(applied.len(), 1, "Expected only version 1 to be applied");
    assert_eq!(applied[0], 1, "Expected version 1 to be applied");

    // Run migrations again
    db.migrate().await?;

    // Verify all migrations are applied again
    let applied = db.get_applied_migrations().await?;
    assert_eq!(applied.len(), 7, "Expected 7 migrations to be applied again");

    // Clean up
    fs::remove_file(db_path)?;

    Ok(())
} 