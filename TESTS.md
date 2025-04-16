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
- [ ] Concurrent access handling
- [ ] Transaction rollback scenarios
- [ ] Database migration tests

### 1.3 Error Handling
- [x] Custom error type display
- [ ] Error propagation
- [ ] Error recovery scenarios
- [ ] Database error handling
- [ ] Scraper error handling

## 2. Scraper Testing

### 2.1 ArgenProp Scraper
- [ ] URL construction
  - [ ] District name processing
  - [ ] Property type translation
  - [ ] Query parameter handling
- [ ] HTML parsing
  - [ ] Property listing extraction
  - [ ] Price parsing
  - [ ] Size parsing
  - [ ] Feature extraction
- [ ] Pagination handling
  - [ ] Next page detection
  - [ ] Page number increment
  - [ ] Last page detection
- [ ] Error scenarios
  - [ ] Network failures
  - [ ] HTML structure changes
  - [ ] Rate limiting
  - [ ] Invalid responses

### 2.2 Scraper Factory
- [ ] Scraper creation
- [ ] Scraper type validation
- [ ] Error handling for invalid types

## 3. CLI Testing

### 3.1 Command Parsing
- [ ] Scrape command options
- [ ] List command options
- [ ] Export command options
- [ ] Update command options
- [ ] Invalid command handling

### 3.2 Command Execution
- [ ] Scrape command execution
- [ ] List command execution
- [ ] Export command execution
- [ ] Update command execution
- [ ] Error handling and reporting

## 4. Integration Tests

### 4.1 End-to-End Scenarios
- [ ] Full property scraping workflow
- [ ] Property update workflow
- [ ] Data export workflow
- [ ] Price history tracking
- [ ] Image downloading and hashing

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
# Run all tests
cargo test

# Run specific test module
cargo test module_name

# Run with logging
RUST_LOG=debug cargo test

# Run with test database
DATABASE_URL=sqlite:test.db cargo test
```

### 7.2 Test Reports
- Test coverage reports
- Test duration tracking
- Error reporting
- Performance metrics

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