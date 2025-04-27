# Type-Safe Database Design in BREA

## Overview
This document outlines our approach to making database operations in BREA more type-safe, particularly focusing on handling property statuses, timestamps, and other critical data types.

## Goals
1. Catch type mismatches at compile time rather than runtime
2. Ensure database schema and Rust types are always in sync
3. Make impossible states unrepresentable
4. Provide clear error messages during compilation

## Implementation Strategy

### 1. Strong Types for Database Fields

#### Property Status
Instead of using raw strings or enums directly in the database, we'll create wrapper types:
```rust
// In db/types.rs
pub struct DbPropertyStatus(String);

impl DbPropertyStatus {
    // Validates at compile time that the status is one of the known values
    pub const fn new(status: &'static str) -> Option<Self> {
        match status {
            "active" | "sold" | "removed" => Some(Self(status.to_string())),
            _ => None
        }
    }
}

// Usage in migrations
const VALID_STATUSES: &[DbPropertyStatus] = &[
    DbPropertyStatus::new("active").unwrap(),
    DbPropertyStatus::new("sold").unwrap(),
    DbPropertyStatus::new("removed").unwrap(),
];
```

#### Timestamps
We'll create wrapper types for timestamps to ensure proper formatting and timezone handling:
```rust
pub struct DbTimestamp(DateTime<Utc>);

impl DbTimestamp {
    pub fn now() -> Self {
        Self(Utc::now())
    }
    
    pub fn from_rfc3339(s: &str) -> Result<Self, TimestampError> {
        DateTime::parse_from_rfc3339(s)
            .map(|dt| Self(dt.with_timezone(&Utc)))
            .map_err(TimestampError::Parse)
    }
}
```

### 2. Type-Safe Migrations

We'll create a type-safe migration builder that ensures:
- Column types match their Rust counterparts
- Foreign key constraints are validated at compile time
- Default values are type-checked

```rust
// Example usage in migrations
let migration = Migration::new("create_properties")
    .create_table("properties", |t| {
        t.column("status", DbPropertyStatus::column())
         .column("updated_at", DbTimestamp::column())
         // ... other columns
    });
```

### 3. Type-Safe Queries

We'll implement a query builder that leverages Rust's type system:
```rust
// Example of type-safe query building
let query = Query::select()
    .from::<Property>()
    .where_("status", Eq(DbPropertyStatus::new("active").unwrap()))
    .and_where("updated_at", Gt(DbTimestamp::now()));
```

## Benefits

1. **Compile-Time Validation**
   - Invalid property statuses caught at compile time
   - Timestamp format mismatches prevented
   - Foreign key constraints validated during compilation

2. **Runtime Safety**
   - No runtime panics from invalid database values
   - Clear error handling for edge cases
   - Type conversion handled automatically

3. **Developer Experience**
   - IDE autocompletion for valid values
   - Clear compile errors for invalid usage
   - Self-documenting code through types

## Implementation Phases

1. **Phase 1: Basic Types**
   - Move database code to modular structure
   - Implement basic wrapper types
   - Add compile-time validation

2. **Phase 2: Migration System**
   - Create type-safe migration builder
   - Implement validation for schema changes
   - Add tests for migration system

3. **Phase 3: Query Builder**
   - Implement type-safe query builder
   - Add support for complex queries
   - Ensure type safety in joins

4. **Phase 4: Integration**
   - Update existing code to use new types
   - Add migration tests
   - Document type system usage

## Testing Strategy

1. **Unit Tests**
   - Test each wrapper type
   - Verify compile-time validation
   - Check error handling

2. **Integration Tests**
   - Test database operations
   - Verify migration system
   - Check query builder

3. **Compile Tests**
   - Ensure invalid states don't compile
   - Test error messages
   - Verify type inference 