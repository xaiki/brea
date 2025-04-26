# BREA Test Plan

## Overview

This test plan covers the testing strategy for the BREA Real Estate Analyzer project. The plan is organized by component and test type, with specific test cases and implementation details.

## 1. Core Components Testing

### 1.1 Property Model Tests
- [x] Property serialization/deserialization
  - Basic property serialization/deserialization
  - Edge cases (empty strings, min/max values)
  - URL handling
- [x] Property type conversion
  - String to PropertyType conversion
  - PropertyType to string conversion
  - Invalid type handling
- [x] Property display formatting
  - Basic property display
  - Price formatting
  - Size and room formatting
  - Address formatting
- [x] Property validation rules
  - Negative price validation
  - Negative size validation
  - Negative rooms validation
  - Negative antiquity validation
- [x] Property comparison logic
  - Price comparison
  - Size comparison
  - Room count comparison
  - Antiquity comparison

### 1.2 Database Operations
- [x] Database connection and initialization
- [x] Property CRUD operations
- [x] Property image CRUD operations
- [x] Price history cleanup
- [x] Concurrent access handling
- [x] Transaction rollback scenarios
- [x] Database migration tests

### 1.3 Error Handling
- [x] Custom error type display
- [x] Error propagation
- [x] Error recovery scenarios
- [x] Database error handling
- [ ] Scraper error handling

## 2. Scraper Testing

### 2.1 ArgenProp Scraper
- [x] URL construction
  - [x] District name processing
  - [x] Property type translation
  - [x] Query parameter handling
- [x] HTML parsing
  - [x] Property listing extraction
  - [x] Price parsing
  - [x] Size parsing
  - [x] Feature extraction
- [x] Pagination handling
  - [x] Next page detection
  - [x] Page number increment
  - [x] Last page detection
- [x] Error scenarios
  - [x] Network failures
  - [x] HTML structure changes
  - [x] Rate limiting
  - [x] Invalid responses

### 2.2 Scraper Factory
- [x] Scraper creation
- [x] Scraper type validation
- [x] Error handling for invalid types

## 3. CLI Testing

### 3.1 Command Parsing
- [x] Scrape command options
- [x] List command options
- [x] Export command options
- [x] Update command options
- [x] Invalid command handling

### 3.2 Command Execution
- [x] Scrape command execution
- [x] List command execution
- [x] Export command execution
  - [x] CSV file generation
  - [x] Optional field handling
  - [x] Error handling
- [x] Update command execution
- [x] Error handling and reporting

## 4. Integration Tests

### 4.1 End-to-End Scenarios
- [ ] Full property scraping workflow
  - [ ] Single property scraping
  - [ ] Multiple properties scraping
  - [ ] Property update detection
- [ ] Property update workflow
  - [ ] Price change detection
  - [ ] Status change detection
  - [ ] Image update detection
- [ ] Data export workflow
  - [ ] CSV export with all fields
  - [ ] CSV export with selected fields
  - [ ] Error handling during export
- [ ] Price history tracking
  - [ ] Price change recording
  - [ ] Price history cleanup
  - [ ] Price trend analysis
- [ ] Image downloading and hashing
  - [ ] Image download success
  - [ ] Image download failure
  - [ ] Duplicate image detection

### 4.2 Performance Tests
- [ ] Scraping performance
- [ ] Database query performance
- [ ] Memory usage
- [ ] Concurrent operation handling

## 5. Test Implementation Guidelines

### 5.1 Unit Tests
- Use `#[cfg(test)]` for test modules
- Place tests in the same file as the code being tested
- Use `#[tokio::test]` for async tests
- Mock external dependencies where appropriate
- Test both success and failure cases
- Include edge cases and boundary conditions
- Use descriptive test names
- Group related tests in modules

### 5.2 Integration Tests
- Create separate test files in `tests/` directory
- Use test databases for database operations
- Mock HTTP requests for scraper tests
- Clean up test data after each test
- Test the full workflow
- Include error scenarios
- Test concurrent operations

### 5.3 Test Data
- Use realistic test data
- Include edge cases
- Cover error scenarios
- Document test data assumptions
- Use constants for test data
- Generate random test data where appropriate

## 6. Test Environment Setup

### 6.1 Required Tools
- Rust toolchain
- SQLite
- HTTP mocking tools
- Test database setup

### 6.2 Configuration
- Test database location
- Mock server configuration
- Logging levels
- Test data directories

## 7. Test Execution

### 7.1 Running Tests
```bash
# Run all tests in all crates
cargo test --all

# Run tests for a specific crate
cargo test -p brea-core
cargo test -p brea-scrapers
cargo test -p brea-cli

# Run specific test module
cargo test module_name

# Run with logging
RUST_LOG=debug cargo test

# Run with test database
DATABASE_URL=sqlite:test.db cargo test

# Run tests in parallel (default)
cargo test -- --test-threads=1

# Show test output for passing tests
cargo test -- --show-output

# Run tests with specific features
cargo test --features test-utils
```

### 7.2 Test Organization
- Unit tests are placed in the same file as the code being tested using `#[cfg(test)]` modules
- Integration tests are placed in the `tests/` directory of each crate
- Test data files are stored in the `test_data/` directory
- Mock responses are stored in the `debug/` directory
- Each test module should clean up its test data after execution

### 7.3 Test Reports
- Test coverage reports (using cargo-tarpaulin)
```bash
cargo tarpaulin --all-features --workspace
```
- Test duration tracking
```bash
cargo test -- --report-time
```
- Error reporting
```bash
cargo test -- --nocapture
```
- Performance metrics
```bash
cargo test -- --bench
```

## 8. Maintenance

### 8.1 Regular Tasks
- Update test data
- Review test coverage
- Update mock responses
- Monitor test performance

### 8.2 Documentation
- Keep test documentation updated
- Document test assumptions
- Update test plan as needed
- Track test-related issues 