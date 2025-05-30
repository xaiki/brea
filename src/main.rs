use brea_core::{
    PropertyDisplay, PropertyType, Result, BreaError,
    Database,
};
use brea_core::db::migrations::{apply_migrations, rollback_migration, get_applied_migrations};
use brea_core::db::types::{DbPropertyStatus, STATUS_ACTIVE, STATUS_SOLD, STATUS_REMOVED};
use brea_scrapers::{ScraperType, ScrapeQuery, ScraperFactory};
use clap::{Parser, Subcommand, ValueEnum};
use csv::Writer;
use std::path::PathBuf;
use tracing::{info, Level};
use std::sync::Arc;
use std::str::FromStr;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scrape property listings from various sources
    #[command(about = "Scrape property listings from various sources")]
    #[command(long_about = "Scrape property listings from various sources. Currently supports ArgenProp.")]
    Scrape(ScrapeCommand),
    
    /// List properties from the database
    #[command(about = "List properties from the database")]
    #[command(long_about = "List properties from the database with optional filtering and sorting.")]
    List(ListCommand),
    
    /// Export property data to CSV
    #[command(about = "Export property data to CSV")]
    #[command(long_about = "Export property data to a CSV file for external analysis.")]
    Export(ExportCommand),

    /// Update existing properties with fresh data
    #[command(about = "Update existing properties with fresh data")]
    #[command(long_about = "Update existing properties by re-scraping their listings. Uses the original property type and location for efficient updates.")]
    Update(UpdateCommand),

    /// Manage database migrations
    #[command(about = "Manage database migrations")]
    #[command(long_about = "Apply or rollback database migrations, and view migration status.")]
    Database(DatabaseCommand),
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CliScraperType {
    Argenprop,
}

impl From<CliScraperType> for ScraperType {
    fn from(scraper_type: CliScraperType) -> Self {
        match scraper_type {
            CliScraperType::Argenprop => ScraperType::Argenprop,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CliPropertyStatus {
    Active,
    Sold,
    Removed,
}

impl From<CliPropertyStatus> for DbPropertyStatus {
    fn from(status: CliPropertyStatus) -> Self {
        match status {
            CliPropertyStatus::Active => DbPropertyStatus::new(STATUS_ACTIVE),
            CliPropertyStatus::Sold => DbPropertyStatus::new(STATUS_SOLD),
            CliPropertyStatus::Removed => DbPropertyStatus::new(STATUS_REMOVED),
        }
    }
}

#[derive(Parser)]
#[command(about = "Scrape property listings")]
#[command(long_about = "Scrape property listings from various sources. Currently supports ArgenProp.")]
struct ScrapeCommand {
    /// The scraper to use (-x, --scraper)
    #[arg(short = 'x', long, value_enum, default_value_t = CliScraperType::Argenprop)]
    scraper: CliScraperType,

    /// The district to search in (-n, --district)
    #[arg(short = 'n', long)]
    district: String,

    /// Type of property (-t, --property-type). Can be specified multiple times.
    #[arg(short = 't', long, value_enum, num_args = 1.., value_delimiter = ',')]
    property_type: Vec<PropertyType>,

    /// Minimum price in USD (-p, --min-price)
    #[arg(short = 'p', long)]
    min_price: Option<f64>,

    /// Maximum price in USD (-P, --max-price)
    #[arg(short = 'P', long)]
    max_price: Option<f64>,

    /// Minimum size in square meters (-m, --min-size)
    #[arg(short = 'm', long)]
    min_size: Option<f64>,

    /// Maximum size in square meters (-M, --max-size)
    #[arg(short = 'M', long)]
    max_size: Option<f64>,

    /// Maximum number of pages to scrape (-c, --max-pages)
    #[arg(short = 'c', long, default_value_t = 1)]
    max_pages: u32,

    /// Database file path (-d, --database)
    #[arg(short = 'd', long, default_value = "brea.db")]
    database: PathBuf,
}

#[derive(Parser)]
#[command(about = "List properties from the database")]
#[command(long_about = "List properties from the database with optional filtering and sorting.")]
struct ListCommand {
    /// Database file path (-d, --database)
    #[arg(short = 'd', long, default_value = "brea.db")]
    database: PathBuf,

    /// Source to filter by (-f, --source)
    #[arg(short = 'f', long)]
    source: Option<String>,

    /// Minimum price in USD (-p, --min-price)
    #[arg(short = 'p', long)]
    min_price: Option<f64>,

    /// Maximum price in USD (-P, --max-price)
    #[arg(short = 'P', long)]
    max_price: Option<f64>,

    /// Minimum size in square meters (-m, --min-size)
    #[arg(short = 'm', long)]
    min_size: Option<f64>,

    /// Maximum size in square meters (-M, --max-size)
    #[arg(short = 'M', long)]
    max_size: Option<f64>,

    /// Maximum number of properties to display (-l, --limit)
    #[arg(short = 'l', long, default_value_t = 10)]
    limit: i64,

    /// Number of properties to skip (-o, --offset)
    #[arg(short = 'o', long, default_value_t = 0)]
    offset: i64,

    /// Field to sort by (-s, --sort-by)
    #[arg(short = 's', long, default_value = "price_usd")]
    sort_by: String,

    /// Sort order (-r, --sort-order)
    #[arg(short = 'r', long, value_enum, default_value_t = SortOrder::Desc)]
    sort_order: SortOrder,

    /// Height of the price history graph in lines (-g, --graph-height)
    #[arg(short = 'g', long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..=5))]
    graph_height: u8,

    /// Property status to filter by (-S, --status)
    #[arg(short = 'S', long, value_enum, default_value_t = CliPropertyStatus::Active)]
    status: CliPropertyStatus,
}

#[derive(Parser)]
#[command(about = "Export property data to CSV")]
#[command(long_about = "Export property data to a CSV file for external analysis.")]
struct ExportCommand {
    /// Output file path (-o, --output)
    #[arg(short = 'o', long, default_value = "properties.csv")]
    output: PathBuf,

    /// Database file path (-d, --database)
    #[arg(short = 'd', long, default_value = "brea.db")]
    database: PathBuf,

    /// Property status to filter by (-S, --status)
    #[arg(short = 'S', long, value_enum, default_value_t = CliPropertyStatus::Active)]
    status: CliPropertyStatus,
}

#[derive(Parser)]
#[command(about = "Update properties from the database")]
struct UpdateCommand {
    /// The scraper to use (-x, --scraper)
    #[arg(short = 'x', long, value_enum, default_value_t = CliScraperType::Argenprop)]
    scraper: CliScraperType,

    /// Maximum number of pages to scrape (-c, --max-pages)
    #[arg(short = 'c', long)]
    max_pages: Option<u32>,

    /// Database file path (-d, --database)
    #[arg(short = 'd', long, default_value = "brea.db")]
    database: PathBuf,
}

#[derive(Debug, clap::ValueEnum, Clone, PartialEq)]
enum SortOrder {
    Asc,
    Desc,
}

async fn scrape_properties(cmd: &ScrapeCommand, db: Arc<Database>) -> Result<()> {
    let scraper = ScraperFactory::create_scraper(cmd.scraper.into());
    let query = ScrapeQuery::new(
        cmd.district.clone(),
        cmd.property_type[0].clone(),
        cmd.min_price,
        cmd.max_price,
        cmd.min_size,
        cmd.max_size,
        Some(Arc::clone(&db)),
    );

    let results = scraper.scrape_listing(query, cmd.max_pages).await?;
    let results_len = results.len();

    let mut displays = Vec::new();
    for (property, _images) in &results {
        let price_history = db.get_price_history(property.id).await?;
        displays.push(PropertyDisplay::new(property.clone(), price_history));
    }
    for display in &displays {
        println!("{}", display.to_string());
    }

    info!("Found {} properties", results_len);
    Ok(())
}

async fn update_properties(cmd: &UpdateCommand, db: Arc<Database>) -> Result<()> {
    let scraper = ScraperFactory::create_scraper(cmd.scraper.into());
    let properties = db.get_properties().await?;

    for property in properties {
        if let Some(property_type) = property.property_type.as_ref().and_then(|t| PropertyType::from_str(t).ok()) {
            let query = ScrapeQuery::new(
                property.district.clone(),
                property_type,
                None, // No price filters for updates
                None,
                None, // No size filters for updates
                None,
                Some(Arc::clone(&db)),
            );

            let mut results = scraper.scrape_listing(query, cmd.max_pages.unwrap_or(1)).await?;
            
            // Save updated properties first
            for (ref mut property, _images) in &mut results {
                db.save_property(property).await?;
            }
            
            // Display updated properties in the same format as the list command
            let mut displays = Vec::new();
            for (property, _images) in &results {
                let price_history = db.get_price_history(property.id).await?;
                displays.push(PropertyDisplay::new(property.clone(), price_history));
            }
            for display in &displays {
                println!("{}", display.to_string());
            }

            info!("Updated {} properties", results.len());
        }
    }
    Ok(())
}

#[derive(Parser)]
#[command(about = "Manage database migrations")]
struct DatabaseCommand {
    /// Database file path (-d, --database)
    #[arg(short = 'd', long, default_value = "brea.db")]
    database: PathBuf,

    /// Migration action to perform (-a, --action)
    #[arg(short = 'a', long, value_enum)]
    action: DatabaseAction,

    /// Target migration version for rollback (-t, --target-version)
    #[arg(short = 't', long = "target-version")]
    target_version: Option<i32>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum DatabaseAction {
    /// Apply all pending migrations
    Up,
    /// Rollback to a specific version
    Down,
    /// List applied migrations
    List,
}

async fn handle_migrations(cmd: &DatabaseCommand) -> Result<()> {
    match cmd.action {
        DatabaseAction::Up => {
            let db = Database::new(&cmd.database).await?;
            info!("Applying all pending migrations...");
            apply_migrations(db.pool()).await?;
            info!("All migrations applied successfully.");
        }
        DatabaseAction::Down => {
            let db = Database::new(&cmd.database).await?;
            let version = cmd.target_version.ok_or_else(|| {
                BreaError::InvalidPropertyType("Target version is required for rollback".to_string())
            })?;
            info!("Rolling back to version {}...", version);
            rollback_migration(db.pool(), version).await?;
            info!("Rollback completed successfully.");
        }
        DatabaseAction::List => {
            let db = Database::new_without_migrations(&cmd.database).await?;
            let migrations = get_applied_migrations(db.pool()).await?;
            if migrations.is_empty() {
                info!("No migrations have been applied.");
            } else {
                info!("Applied migrations:");
                for migration in migrations {
                    println!("{}", migration);
                }
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive(Level::INFO.into()))
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Scrape(cmd) => {
            let db = Arc::new(Database::new(&cmd.database).await?);
            scrape_properties(cmd, db).await
        }
        Commands::List(cmd) => {
            let db = Database::new(&cmd.database).await?;
            list_properties(cmd, &db).await
        }
        Commands::Export(cmd) => {
            let db = Database::new(&cmd.database).await?;
            export_properties(cmd, &db).await
        }
        Commands::Update(cmd) => {
            let db = Arc::new(Database::new(&cmd.database).await?);
            update_properties(cmd, db).await
        }
        Commands::Database(cmd) => {
            handle_migrations(cmd).await
        }
    }
}

async fn list_properties(cmd: &ListCommand, db: &Database) -> Result<()> {
    let properties = db.get_properties().await?;
    let mut displays = Vec::new();
    for property in properties.iter() {
        if property.status == DbPropertyStatus::from(cmd.status) {
            let price_history = db.get_price_history(property.id).await?;
            displays.push(PropertyDisplay::new(property.clone(), price_history));
        }
    }
    for display in &displays {
        println!("{}", display.to_string());
    }

    info!("Listed {} properties", displays.len());
    Ok(())
}

async fn export_properties(cmd: &ExportCommand, db: &Database) -> Result<()> {
    let properties = db.get_properties().await?;
    let mut writer = Writer::from_path(&cmd.output)?;
    let properties_len = properties.len();

    for property in properties {
        if let Some(_) = property.property_type.as_ref().and_then(|t| PropertyType::from_str(t).ok()) {
            writer.serialize(&property)?;
        }
    }

    writer.flush()?;
    info!("Exported {} properties to {}", properties_len, cmd.output.display());
    Ok(())
} 