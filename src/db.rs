//! Top-level Database facade.
//!
//! Provides a simple interface for executing SQL against QuackDB.

use crate::planner::catalog::Catalog;
use crate::chunk::DataChunk;

/// The top-level QuackDB database.
pub struct Database {
    catalog: Catalog,
}

impl Database {
    /// Create a new empty database.
    pub fn new() -> Self {
        todo!()
    }

    /// Execute a SQL query and return result chunks.
    pub fn execute_sql(&mut self, sql: &str) -> Result<Vec<DataChunk>, String> {
        todo!()
    }

    /// Get a reference to the catalog.
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    /// Get a mutable reference to the catalog.
    pub fn catalog_mut(&mut self) -> &mut Catalog {
        &mut self.catalog
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}
