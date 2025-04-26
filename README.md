# BREA Real Estate Analyzer

A powerful tool for analyzing real estate properties in Argentina, with a focus on price trends and market analysis.

## Features

- **Property Scraping**: Collect property data from various sources (currently supporting ArgenProp)
- **Price History Tracking**: Monitor price changes over time
- **Data Export**: Export property data to CSV for external analysis
- **Flexible Filtering**: Filter properties by price, size, location, and more
- **Visual Analysis**: ASCII graphs showing price trends
- **Multiple Property Types**: Support for various property types:
  - Houses (casas)
  - Apartments (departamentos)
  - Land (terrenos)
  - PH (depto-tipo-casa)
  - Commercial (local)
  - Fields (campo)
  - Garages (cochera)
  - Commercial Premises (fondo-comercio)
  - Warehouses (galpón)
  - Hotels
  - Special Business (negocio-especial)
  - Offices (oficina)
  - Country Houses (quinta)

## Installation

```bash
cargo install --path .
```

## Usage

### Scraping Properties

```bash
# Scrape all property types in La Plata
brea scrape -n "la plata"

# Scrape specific property types
brea scrape -n "buenos aires" -t house -t apartment

# Scrape multiple property types with comma separation
brea scrape -n "cordoba" -t house,apartment,land

# Scrape apartments with price range
brea scrape -n "buenos aires" -t apartment -p 100000 -P 200000

# Scrape land with size filters
brea scrape -n "cordoba" -t land -m 100 -M 500

# Scrape multiple pages
brea scrape -n "rosario" -t house -c 3
```

### Listing Properties

```bash
# List all properties
brea list

# List with filters
brea list -p 100000 -P 200000 -m 100 -M 200

# Sort by price (ascending)
brea list -s price_usd -r asc

# Show price history graph
brea list -g 5
```

### Exporting Data

```bash
# Export to CSV
brea export -o properties.csv
```

### Database Management

BREA uses SQLite for data storage and includes a migration system to manage database schema changes. You can use the following commands to manage your database:

```bash
# List applied migrations
brea db list

# Apply pending migrations
brea db migrate

# Rollback to a specific version
brea db rollback --version 1

# Rollback all migrations
brea db rollback --all
```

Options:
- `--action <ACTION>`: Action to perform (list, migrate, rollback)
- `--version <VERSION>`: Target version for rollback
- `--all`: Rollback all migrations

## Architecture

BREA is built with a modular architecture:

- `brea-core`: Core domain models and database operations
- `brea-scrapers`: Property data collection from various sources
  - ArgenProp scraper with support for all property types
  - Extensible design for adding more scrapers

Each scraper implements:
- `Scraper` trait for property data collection
- `PropertyTypeTranslator` trait for handling property type-specific URL formats

## Development

### Adding a New Scraper

1. Create a new module in `brea-scrapers/src/`
2. Implement the `Scraper` and `PropertyTypeTranslator` traits
3. Add the scraper type to the `ScraperType` enum in `src/main.rs`
4. Update the CLI to handle the new scraper

### Running Tests

```bash
cargo test
```

## License

MIT 