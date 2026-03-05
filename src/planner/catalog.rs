//! Lesson 23: Catalog
//!
//! Database catalog for tracking tables and their schemas.

use crate::types::LogicalType;
use crate::chunk::DataChunk;
use std::collections::HashMap;

/// Information about a table column.
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: LogicalType,
    pub nullable: bool,
    pub column_index: usize,
}

/// Information about a table.
#[derive(Debug, Clone)]
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
}

impl TableInfo {
    pub fn find_column(&self, name: &str) -> Option<&ColumnInfo> {
        self.columns.iter().find(|c| c.name == name)
    }

    pub fn schema_types(&self) -> Vec<LogicalType> {
        self.columns.iter().map(|c| c.data_type.clone()).collect()
    }
}

/// Database catalog managing table definitions and data.
pub struct Catalog {
    tables: HashMap<String, TableInfo>,
    /// In-memory table storage.
    table_data: HashMap<String, Vec<DataChunk>>,
}

impl Catalog {
    /// Create a new empty catalog.
    pub fn new() -> Self {
        todo!()
    }

    /// Create a table.
    pub fn create_table(&mut self, info: TableInfo) -> Result<(), String> {
        todo!()
    }

    /// Get table info.
    pub fn get_table(&self, name: &str) -> Option<&TableInfo> {
        todo!()
    }

    /// Drop a table.
    pub fn drop_table(&mut self, name: &str) -> Result<(), String> {
        todo!()
    }

    /// Insert data into a table.
    pub fn insert_data(&mut self, table_name: &str, chunk: DataChunk) -> Result<(), String> {
        todo!()
    }

    /// Get all data for a table.
    pub fn get_table_data(&self, table_name: &str) -> Option<&[DataChunk]> {
        todo!()
    }

    /// List all table names.
    pub fn table_names(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }
}

impl Default for Catalog {
    fn default() -> Self {
        Self::new()
    }
}
