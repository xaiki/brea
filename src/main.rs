use brea_core::{
    Database, PropertyType, Result, create_property_table, PropertyDisplay as CorePropertyDisplay,
    Scraper,
};
use brea_scrapers::{ScraperFactory, ScraperType as CoreScraperType};
use clap::{Parser, Subcommand, ValueEnum};
use csv::Writer;
use std::path::PathBuf;
use tracing::{info, Level};
use std::sync::Arc;

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
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliScraperType {
    Argenprop,
}

impl From<CliScraperType> for CoreScraperType {
    fn from(value: CliScraperType) -> Self {
        match value {
            CliScraperType::Argenprop => CoreScraperType::Argenprop,
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
}

#[derive(Parser)]
#[command(about = "Update existing properties")]
#[command(long_about = "Update existing properties by re-scraping their listings. Uses the original property type and location for efficient updates.")]
struct UpdateCommand {
    /// The scraper to use (-x, --scraper)
    #[arg(short = 'x', long, value_enum, default_value_t = CliScraperType::Argenprop)]
    scraper: CliScraperType,

    /// Maximum number of pages to scrape per property type (-c, --max-pages)
    #[arg(short = 'c', long, default_value_t = 1)]
    max_pages: u32,

    /// Database file path (-d, --database)
    #[arg(short = 'd', long, default_value = "brea.db")]
    database: PathBuf,
}

#[derive(Debug, clap::ValueEnum, Clone)]
enum SortOrder {
    Asc,
    Desc,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Scrape(cmd) => {
            let scraper: Arc<dyn Scraper> = ScraperFactory::create_scraper(cmd.scraper.into());
            let db = Database::new(&cmd.database).await?;

            let properties = if cmd.property_type.is_empty() {
                // If no property types specified, scrape all supported types
                scraper.scrape_all_types(
                    &cmd.district,
                    cmd.min_price,
                    cmd.max_price,
                    cmd.min_size,
                    cmd.max_size,
                    cmd.max_pages,
                ).await?
            } else {
                // Scrape only the specified property types
                let mut all_properties = Vec::new();
                for property_type in cmd.property_type {
                    info!("Scraping {} properties in {}", property_type, cmd.district);
                    let url = scraper.build_listing_url(
                        &cmd.district,
                        &property_type,
                        cmd.min_price,
                        cmd.max_price,
                        cmd.min_size,
                        cmd.max_size,
                    );

                    let mut current_page = 1;
                    let mut has_next = true;
                    let mut url = url;

                    while has_next && current_page <= cmd.max_pages {
                        let (mut properties, next) = scraper.scrape_page(&url).await?;
                        all_properties.append(&mut properties);
                        has_next = next;

                        if has_next {
                            if let Some(next_url) = scraper.get_next_page_url(&url).await? {
                                url = next_url;
                                current_page += 1;
                            } else {
                                break;
                            }
                        }
                    }
                }
                all_properties
            };

            for (mut property, mut images) in properties {
                db.save_property(&mut property).await?;
                
                for image in &mut images {
                    image.property_id = property.id.unwrap();
                    db.save_property_image(image).await?;
                }
            }
        }
        Commands::List(cmd) => {
            let db = Database::new(&cmd.database).await?;
            let properties = db.list_properties(
                cmd.source.as_deref(),
                cmd.min_price,
                cmd.max_price,
                cmd.min_size,
                cmd.max_size,
            ).await?;

            let mut displays = Vec::new();
            for property in properties {
                let price_history = db.get_price_history(property.id.unwrap()).await?;
                displays.push(CorePropertyDisplay::new(property, price_history));
            }

            let table = create_property_table(&displays, cmd.graph_height as u8);
            println!("{}", table);
        }
        Commands::Export(cmd) => {
            let db = Database::new(&cmd.database).await?;
            let properties = db.list_properties(None, None, None, None, None).await?;

            let mut writer = Writer::from_path(&cmd.output)?;
            writer.write_record(&["Title", "Price (USD)", "Size (m²)", "Rooms", "Antiquity (years)", "Address"])?;
            for property in properties {
                writer.write_record(&[
                    property.title,
                    property.price_usd.to_string(),
                    property.covered_size.to_string(),
                    property.rooms.to_string(),
                    property.antiquity.to_string(),
                    property.address,
                ])?;
            }
            writer.flush()?;
        }
        Commands::Update(cmd) => {
            let scraper: Arc<dyn Scraper> = ScraperFactory::create_scraper(cmd.scraper.into());
            let db = Database::new(&cmd.database).await?;

            // Get all unique property types and districts from the database
            let properties = db.list_properties(None, None, None, None, None).await?;
            let mut property_types = std::collections::HashSet::new();
            let mut districts = std::collections::HashSet::new();

            for property in &properties {
                if let Some(property_type) = &property.property_type {
                    property_types.insert(property_type.clone());
                }
                // Extract district from address (assuming it's the first part)
                if let Some(district) = property.address.split(',').next() {
                    districts.insert(district.trim().to_string());
                }
            }

            info!("Found {} unique property types and {} districts to update", property_types.len(), districts.len());

            // For each combination of property type and district, scrape fresh data
            for property_type in property_types {
                for district in &districts {
                    info!("Updating {} properties in {}", property_type, district);
                    let url = scraper.build_listing_url(
                        district,
                        &property_type,
                        None, // No price filters for updates
                        None,
                        None, // No size filters for updates
                        None,
                    );

                    let mut current_page = 1;
                    let mut has_next = true;
                    let mut url = url;

                    while has_next && current_page <= cmd.max_pages {
                        let (new_properties, next) = scraper.scrape_page(&url).await?;
                        has_next = next;

                        // Update existing properties with new data
                        for (mut new_property, mut new_images) in new_properties {
                            // Try to find existing property by external_id
                            if let Some(existing_property) = properties.iter().find(|p| p.external_id == new_property.external_id) {
                                // Update existing property with new data
                                new_property.id = existing_property.id;
                                db.save_property(&mut new_property).await?;

                                // Update images
                                for image in &mut new_images {
                                    image.property_id = new_property.id.unwrap();
                                    db.save_property_image(image).await?;
                                }
                            } else {
                                // New property found, save it
                                db.save_property(&mut new_property).await?;
                                for image in &mut new_images {
                                    image.property_id = new_property.id.unwrap();
                                    db.save_property_image(image).await?;
                                }
                            }
                        }

                        if has_next {
                            if let Some(next_url) = scraper.get_next_page_url(&url).await? {
                                url = next_url;
                                current_page += 1;
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
} 