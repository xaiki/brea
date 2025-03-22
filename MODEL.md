# Common Mistakes and Patterns

This file is automatically updated whenever new best practices or common patterns are discovered during development.

## Process Learnings
1. **Always Clean Up First**
   - Mistake: Jumping into new features or refactoring without cleaning up
   - Pattern: Technical debt accumulates when we don't clean up
   - Solution: 
     - Remove unused code and dead paths first
     - Clean up duplicate implementations
     - Document what was removed and why
     - Commit cleanup changes separately from feature work

2. **Evaluate Before Refactoring**
   - Mistake: Immediately jumping into massive refactoring
   - Pattern: Need to evaluate simpler solutions first
   - Solution:
     - Start with the smallest possible change that could work
     - Evaluate current implementation thoroughly
     - Consider costs vs benefits of refactoring
     - Document why simpler solutions wouldn't work
     - Break large refactors into smaller, reversible steps

3. **Consult Before Implementing**
   - Mistake: Jumping straight into implementation without discussion
   - Pattern: Making assumptions about direction without verification
   - Solution:
     - Present options before implementing any solution
     - Discuss trade-offs and implications
     - Get agreement on approach
     - Document decisions and reasoning
     - Break work into clear, discussable steps

4. **Stay Focused on the Task**
   - Mistake: Making unrelated changes during feature implementation
   - Pattern: Accidentally modifying code that isn't part of the current task
   - Solution:
     - Keep changes strictly related to the current task
     - Review diffs carefully before committing
     - If unrelated issues are found, track them separately
     - Maintain separate commits for unrelated changes
     - When in doubt, leave existing code behavior unchanged

## Database Issues
1. **SQLite JSON Handling**
   - Mistake: Trying to directly decode JSON fields into Rust types or using raw strings
   - Pattern: Create a custom type with proper SQLite trait implementations
   - Solution: Create a custom type (e.g. SQLiteJson) that implements:
     - Type<Sqlite> for type information
     - Decode<'_, Sqlite> for value decoding
     - From<String> and From<Option<String>> for conversions
     - Handle null values by returning default values (e.g. "[]" for arrays)

2. **Database File Management**
   - Mistake: Not properly handling SQLite WAL files
   - Pattern: Need to clean up both `.db` and `.db-wal` files
   - Solution: Use `rm -rf` for both files when resetting

## Rust Toolchain Issues
1. **Toolchain Switching**
   - Mistake: Switching between stable and nightly without proper consideration
   - Pattern: Need to stay on nightly for Leptos
   - Solution: Keep nightly toolchain and fix issues within nightly

## Move Issues in UI
1. **Closure Ownership**
   - Mistake: Moving values into closures without proper cloning
   - Pattern: Need to clone values before using in multiple places
   - Solution: Clone values before closure and use cloned versions

## Code Organization
1. **Error Handling**
   - Mistake: Not properly propagating errors
   - Pattern: Need to use proper error types and conversions
   - Solution: Use `map_err` with appropriate error types

2. **Type Safety**
   - Mistake: Not properly handling nullable fields
   - Pattern: Need to use proper Option types
   - Solution: Use `Option<T>` for nullable fields

3. **Code Duplication**
   - Mistake: Repeating similar code patterns
   - Pattern: Need to identify common patterns and abstract them
   - Solution: Use traits, macros, and shared functions to reduce duplication

4. **Project Structure**
   - Mistake: Changing project names or structure unnecessarily
   - Pattern: Keep project names and structure consistent
   - Solution: Maintain original project names and structure

## Best Practices to Follow
1. Always use `rm -rf` for file cleanup
2. Stay on nightly toolchain for Leptos
3. Clone values before using in closures
4. Use proper JSON handling in SQLite queries
5. Handle all error cases explicitly
6. Use proper type annotations in SQL queries
7. Never rename the project or change its structure unnecessarily
8. Keep linting fixes separate from functional changes
9. Abstract common patterns into traits and shared functions
10. Use DRY (Don't Repeat Yourself) principles
11. Maintain consistent naming across the project

## SQLx Query Generation

### Mistakes
- Manually mapping database rows to structs
- Using raw `query!` when `query_as!` would be more appropriate
- Not handling JSON fields properly in SQL queries
- Not using proper type annotations for database fields

### Patterns
1. Use `query_as!` for automatic struct mapping:
   ```rust
   // Instead of:
   let properties = sqlx::query!(...)
       .fetch_all(&pool)
       .await?
       .into_iter()
       .map(|row| Property { ... })
       .collect();

   // Use:
   let properties = sqlx::query_as!(
       Property,
       "SELECT * FROM properties"
   )
   .fetch_all(&pool)
   .await?;
   ```

2. Handle JSON fields properly:
   ```rust
   // In the struct:
   #[serde(deserialize_with = "deserialize_json_string")]
   pub image_urls: Vec<String>,

   // In the query:
   serde_json::to_string(&property.image_urls).unwrap_or_default()
   ```

3. Use proper type annotations:
   ```rust
   // Instead of:
   id as "id!",
   
   // Use:
   id as "id!: i64",
   ```

### Solutions to Follow
1. Always use `query_as!` when mapping to a struct
2. Create custom types for special database fields (like JSON)
3. Implement proper SQLx traits for custom types
4. Use explicit type annotations in SQL queries
5. Keep database schema and Rust structs in sync
6. Use proper error handling for JSON operations
7. Handle null values gracefully in custom types

## Auto-Update Note

This file is automatically updated whenever new best practices or common patterns are discovered during development.

# WASM Code Organization Best Practices

## Project Structure
1. Keep client-side code in a separate module (e.g., `src/client/`)
2. Use feature flags to conditionally compile client/server code
3. Avoid duplicating components across files
4. Keep shared types and models in a `shared` module

## Dependencies
1. Separate client and server dependencies using feature flags
2. Mark WASM-specific dependencies as optional
3. Use appropriate crate types (`cdylib` and `rlib`) for WASM compilation

## Component Organization
1. Keep component definitions in dedicated files (e.g., `app.rs`)
2. Export shared utilities and types from module root
3. Use a single WASM entry point
4. Keep state management logic close to components

## Build Process
1. Use `trunk` for development and building
2. Configure proper MIME types and CORS headers
3. Ensure all static assets are properly served
4. Handle WASM initialization in a clean way

## Common Pitfalls
1. Avoid duplicate component definitions
2. Be careful with dependencies that don't support WASM
3. Handle errors gracefully in WASM context
4. Properly initialize logging and panic hooks

## Project Structure
1. Always verify the current directory before creating or modifying files
2. When creating a workspace with multiple crates, ensure all crates are in the correct directory structure
3. Double-check paths when moving files to avoid duplicates or incorrect locations

## Code Generation
1. Always read the HTML structure carefully before implementing selectors
2. Use data attributes when available as they are more stable than class names
3. Implement proper error handling and parsing for numeric values
4. Add helper methods for parsing common data formats
5. Use strong typing and avoid unwrap() where possible
6. Implement proper logging for debugging purposes

## Web Scraping
1. Handle missing data gracefully with Option types
2. Use appropriate selectors that are least likely to change
3. Parse numeric values carefully, removing currency symbols and separators
4. Handle different text formats for the same type of data
5. Consider rate limiting and user agent strings for production use

## Database
1. Use migrations for database schema changes
2. Include foreign key constraints for related tables
3. Use appropriate data types for each column
4. Include created_at and updated_at timestamps for all records

## Testing
1. Add unit tests for parsing functions
2. Create integration tests with sample HTML files
3. Test edge cases and error conditions
4. Mock external services in tests

## Future Improvements
1. Implement image downloading and deduplication
2. Add support for pagination in listing pages
3. Implement proper error recovery and retry logic
4. Add rate limiting and respect robots.txt
5. Implement proper logging and monitoring
6. Add support for proxies to avoid IP blocks

# BREA Development Model and Learnings

## Project Structure

The project is organized into multiple crates to maintain separation of concerns:

- `brea`: Main application and CLI interface
- `brea-core`: Core functionality, models, and database operations
- `brea-scrapers`: Web scraping implementations for different property websites

## Design Decisions

### CLI Design

1. **Scraper Abstraction**
   - Each scraper should be self-contained and implement the `Scraper` trait
   - URL construction is handled by the scraper itself, not exposed to users
   - Common parameters (district, property type, price range, size range) are standardized across scrapers
   - Enum-based scraper selection allows for compile-time verification of supported scrapers

2. **Parameter Standardization**
   - Property types are standardized across scrapers (house, apartment, land)
   - Price is always in USD for consistency
   - Size is always in square meters
   - Districts are normalized (lowercase, hyphen-separated)

3. **Database Management**
   - Automatic database creation and migration
   - Upsert-based property storage to handle duplicates
   - Timestamps for creation and updates
   - Property images linked to properties with deduplication

### Error Handling

1. **Database Errors**
   - Graceful handling of missing database files
   - Automatic table creation
   - Clear error messages for database operations

2. **Scraping Errors**
   - Specific error types for scraping issues
   - Proper error propagation through async code
   - Logging of failed scraping attempts

### Future Considerations

1. **Additional Scrapers**
   - Design allows easy addition of new scrapers
   - Common parameter mapping can be shared between scrapers
   - URL construction patterns can be reused

2. **Data Analysis**
   - Current schema supports time-series analysis
   - Property deduplication enables price tracking
   - Image hashing allows duplicate listing detection

3. **Performance**
   - Consider parallel scraping for multiple pages
   - Implement rate limiting per scraper
   - Cache frequently accessed data

4. **Data Quality**
   - Normalize property attributes (size units, currency)
   - Validate and clean input data
   - Handle missing or invalid data gracefully

## Common Mistakes and Solutions

1. **Directory Structure**
   - Keep workspace organization clean with clear crate boundaries
   - Use proper module organization within crates
   - Maintain clear dependency relationships

2. **Database Operations**
   - Use transactions for related operations
   - Handle concurrent access properly
   - Implement proper migration strategies

3. **Error Handling**
   - Use custom error types with proper context
   - Provide meaningful error messages
   - Log errors with appropriate detail levels

4. **Code Organization**
   - Keep scraper-specific logic within scraper implementations
   - Share common functionality through traits
   - Use proper visibility modifiers

## Best Practices

1. **Documentation**
   - Keep README.md updated with latest features
   - Document CLI usage with examples
   - Maintain MODEL.md for architectural decisions

2. **Testing**
   - Unit tests for core functionality
   - Integration tests for scrapers
   - Mock external services in tests

3. **Code Quality**
   - Use strong typing with enums for fixed choices
   - Implement proper trait bounds
   - Follow Rust idioms and patterns

4. **Maintenance**
   - Regular dependency updates
   - Monitor scraper compatibility
   - Track website changes that affect scrapers

### Licensing

1. **Choice of AGPLv3**
   - Ensures that any modifications or improvements to the codebase remain open source
   - Requires network service providers to share their modifications
   - Protects against proprietary forks of the project
   - Encourages community contributions and transparency
   - Particularly important for web services and data collection tools

2. **Implications**
   - All derivative works must also be licensed under AGPLv3
   - Source code must be made available when the software is used over a network
   - Changes and improvements benefit the entire community
   - Commercial use is allowed but must comply with license terms

3. **Compliance Requirements**
   - Include license notice in all source files
   - Provide access to complete source code
   - Document any modifications made to the code
   - Maintain copyright notices

# Project Learnings and Best Practices

## Code Quality and Linting

- Always fix linting issues before running any commands or making database changes
- SQLx query macros require proper setup with `DATABASE_URL` and `cargo sqlx prepare`
- Error handling should be comprehensive with proper trait implementations for all error types
- Use clippy and rustfmt to maintain consistent code style

## Project Structure

- Keep a clear separation between core functionality and specific implementations
- Use workspace structure for better module organization
- Maintain proper directory hierarchy
- Document directory structure changes in commit messages

## Web Scraping

- Use stable selectors that are less likely to change
- Handle missing data gracefully
- Implement rate limiting and respect robots.txt
- Cache responses when appropriate
- Add proper error handling for network issues

## Database Practices

- Use migrations for schema changes
- Keep database URL configuration consistent
- Handle database errors appropriately
- Use transactions for related operations
- Test with in-memory database for unit tests

## Testing

- Write comprehensive unit tests
- Use integration tests for scrapers
- Mock external services when appropriate
- Test error conditions and edge cases
- Use test fixtures for complex data

## Future Improvements

- Add metrics and monitoring
- Implement caching layer
- Add retry mechanisms for failed requests
- Improve error reporting
- Add data validation layer

# Model Learnings

1. Never use `cargo check` or similar commands to find errors and warnings. Instead:
   - Analyze the code files in the provided context
   - Look for linter messages in function results
   - Use the conversation history and summaries
   - Trust that the context contains all necessary information about code issues

2. When fixing errors:
   - Address one issue at a time
   - Verify the fix worked by checking the function results
   - If new errors appear, handle them systematically
   - Don't make assumptions about errors not shown in the context

3. Database Schema Changes:
   - NEVER use DROP TABLE as it loses data
   - Always use ALTER TABLE for schema modifications
   - Check if columns exist before adding them
   - Use migrations to handle schema changes systematically
   - Provide default values when adding NOT NULL columns
   - Keep existing data intact during schema updates

4. Code Review Best Practices:
   - Look at the full context before making changes
   - Consider data preservation in all operations
   - Test for edge cases in schema migrations
   - Use proper error handling for all database operations
   - Document schema changes in migrations 

# BREA Model Design

## Property Types

The system uses a centralized `PropertyType` enum in `brea-core` to represent different types of real estate properties. This enum is designed to be:

1. **Domain-Focused**: The enum represents property types in a domain-agnostic way, without any knowledge of how they're used in specific scrapers or URLs.
2. **Extensible**: New property types can be easily added to the enum.
3. **User-Friendly**: Supports both English and Spanish names through the `FromStr` implementation.

### Property Type Translation

To handle scraper-specific property type representations (like URL segments), we use the `PropertyTypeTranslator` trait:

```rust
pub trait PropertyTypeTranslator {
    fn property_type_to_str(&self, property_type: &PropertyType) -> &'static str;
}
```

This design provides several benefits:

1. **Separation of Concerns**: Each scraper can implement its own translation logic without polluting the core domain model.
2. **Flexibility**: Different scrapers can use different string representations for the same property type.
3. **Maintainability**: URL format changes in a specific scraper only require updating that scraper's implementation.
4. **Type Safety**: The compiler ensures that all property types are handled in translations.

Example usage in a scraper:
```rust
impl PropertyTypeTranslator for ArgenPropScraper {
    fn property_type_to_str(&self, property_type: &PropertyType) -> &'static str {
        match property_type {
            PropertyType::House => "casas",
            PropertyType::Apartment => "departamentos",
            // ...
        }
    }
}
```

## Database Schema 