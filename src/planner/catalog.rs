//! # Lesson 23: Query Planning — Catalog (File 1 of 2)
//!
//! This file implements the database catalog, the central metadata store that
//! tracks table definitions and their in-memory data. The binder queries it to
//! resolve table/column names, and the physical plan builder reads table data
//! from it.
//!
//! It works together with:
//! - `binder.rs` — name resolution and type checking that uses the catalog to
//!   look up table schemas and resolve column references.
//!
//! **Start here**: Implement `catalog.rs` first, then `binder.rs`. The binder
//! depends on `Catalog` methods like `get_table()` to resolve names, so having
//! the catalog working first lets you test the binder incrementally.
//!
//! **Key idea:** A `HashMap<String, TableInfo>` stores table schemas, and
//! a parallel `HashMap<String, Vec<DataChunk>>` stores the actual row data.
//! DDL operations (CREATE/DROP TABLE) modify the schema map; DML operations
//! (INSERT) modify the data map.

use crate::types::LogicalType;
use crate::chunk::DataChunk;
use std::collections::HashMap;

/// Metadata about a single table column.
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    /// Column name.
    pub name: String,
    /// Column data type.
    pub data_type: LogicalType,
    /// Whether the column allows NULL values.
    pub nullable: bool,
    /// Zero-based position of this column in the table.
    pub column_index: usize,
}

/// Metadata about a table (name and column definitions).
#[derive(Debug, Clone)]
pub struct TableInfo {
    /// Table name.
    pub name: String,
    /// Ordered list of column definitions.
    pub columns: Vec<ColumnInfo>,
}

impl TableInfo {
    /// Look up a column by name, returning its [`ColumnInfo`] if found.
    pub fn find_column(&self, name: &str) -> Option<&ColumnInfo> {
        self.columns.iter().find(|c| c.name == name)
    }

    /// Return the data types of all columns, in column order.
    pub fn schema_types(&self) -> Vec<LogicalType> {
        self.columns.iter().map(|c| c.data_type.clone()).collect()
    }
}

/// The database catalog: manages table definitions and in-memory row storage.
pub struct Catalog {
    /// Table name -> table schema.
    tables: HashMap<String, TableInfo>,
    /// Table name -> stored data chunks.
    table_data: HashMap<String, Vec<DataChunk>>,
}

impl Catalog {
    /// Create a new empty catalog with no tables.
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            table_data: HashMap::new(),
        }
    }

    /// Register a new table in the catalog.
    ///
    /// Returns an error if a table with the same name already exists.
    pub fn create_table(&mut self, info: TableInfo) -> Result<(), String> {
        if self.tables.contains_key(&info.name) {
            return Err(format!("Table '{}' already exists", info.name));
        }
        let name = info.name.clone();
        self.tables.insert(name.clone(), info);
        self.table_data.insert(name, Vec::new());
        Ok(())
    }

    /// Look up a table's metadata by name.
    pub fn get_table(&self, name: &str) -> Option<&TableInfo> {
        self.tables.get(name)
    }

    /// Remove a table and its data from the catalog.
    ///
    /// Returns an error if the table does not exist.
    pub fn drop_table(&mut self, name: &str) -> Result<(), String> {
        if self.tables.remove(name).is_none() {
            return Err(format!("Table '{}' does not exist", name));
        }
        self.table_data.remove(name);
        Ok(())
    }

    /// Append a data chunk to the named table's storage.
    ///
    /// Returns an error if the table does not exist.
    pub fn insert_data(&mut self, table_name: &str, chunk: DataChunk) -> Result<(), String> {
        let data = self.table_data.get_mut(table_name)
            .ok_or_else(|| format!("Table '{}' does not exist", table_name))?;
        data.push(chunk);
        Ok(())
    }

    /// Retrieve all stored data chunks for a table.
    ///
    /// Returns `None` if the table does not exist.
    pub fn get_table_data(&self, table_name: &str) -> Option<&[DataChunk]> {
        self.table_data.get(table_name).map(|v| v.as_slice())
    }

    /// Return a list of all table names in the catalog.
    pub fn table_names(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }
}

impl Default for Catalog {
    fn default() -> Self {
        Self::new()
    }
}
