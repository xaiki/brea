use super::types::{DbPropertyStatus, DbTimestamp};
use crate::{Property, PropertyImage, Result};
use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqlitePool, QueryBuilder, Row, FromRow, sqlite::Sqlite};

pub struct PropertyQueryBuilder<'a> {
    builder: QueryBuilder<'a, Sqlite>,
}

impl<'a> PropertyQueryBuilder<'a> {
    pub fn new() -> Self {
        let builder = QueryBuilder::new("SELECT * FROM properties WHERE 1=1");
        Self { builder }
    }

    pub fn with_source(mut self, source: &'a str) -> Self {
        self.builder.push(" AND source = ");
        self.builder.push_bind(source);
        self
    }

    pub fn with_status(mut self, status: DbPropertyStatus) -> Self {
        self.builder.push(" AND status = ");
        self.builder.push_bind(status);
        self
    }

    pub fn with_price_range(mut self, min: Option<f64>, max: Option<f64>) -> Self {
        if let Some(min_price) = min {
            self.builder.push(" AND price_usd >= ");
            self.builder.push_bind(min_price);
        }
        if let Some(max_price) = max {
            self.builder.push(" AND price_usd <= ");
            self.builder.push_bind(max_price);
        }
        self
    }

    pub fn with_size_range(mut self, min_size: Option<f64>, max_size: Option<f64>) -> Self {
        if let Some(min) = min_size {
            self.builder.push(" AND covered_size >= ");
            self.builder.push_bind(min);
        }
        if let Some(max) = max_size {
            self.builder.push(" AND covered_size <= ");
            self.builder.push_bind(max);
        }
        self
    }

    pub fn with_limit(mut self, limit: Option<i64>) -> Self {
        if let Some(limit) = limit {
            self.builder.push(" LIMIT ");
            self.builder.push_bind(limit);
        }
        self
    }

    pub fn with_offset(mut self, offset: Option<i64>) -> Self {
        if let Some(offset) = offset {
            self.builder.push(" OFFSET ");
            self.builder.push_bind(offset);
        }
        self
    }

    pub fn order_by(mut self, field: &str, desc: bool) -> Self {
        self.builder.push(" ORDER BY ");
        self.builder.push(field);
        if desc {
            self.builder.push(" DESC");
        }
        self
    }

    pub fn with_external_ids_not_in(mut self, ids: &'a [&'a str]) -> Self {
        if !ids.is_empty() {
            self.builder.push(" AND external_id NOT IN (");
            for (i, id) in ids.iter().enumerate() {
                if i > 0 {
                    self.builder.push(", ");
                }
                self.builder.push_bind(*id);
            }
            self.builder.push(")");
        }
        self
    }

    pub fn with_external_id(mut self, external_id: &'a str) -> Self {
        self.builder.push(" AND external_id = ");
        self.builder.push_bind(external_id);
        self
    }

    pub async fn execute(mut self, pool: &SqlitePool) -> Result<Vec<Property>> {
        let query = self.builder.build_query_as::<Property>();
        let rows = query.fetch_all(pool).await?;
        Ok(rows)
    }
}

pub struct PropertyImageQueryBuilder<'a> {
    builder: QueryBuilder<'a, Sqlite>,
}

impl<'a> PropertyImageQueryBuilder<'a> {
    pub fn new() -> Self {
        let builder = QueryBuilder::new("SELECT * FROM property_images WHERE 1=1");
        Self { builder }
    }

    pub fn with_property_id(mut self, property_id: i64) -> Self {
        self.builder.push(" AND property_id = ");
        self.builder.push_bind(property_id);
        self
    }

    pub async fn execute(mut self, pool: &SqlitePool) -> Result<Vec<PropertyImage>> {
        let query = self.builder.build_query_as::<PropertyImage>();
        let rows = query.fetch_all(pool).await?;
        Ok(rows)
    }
} 