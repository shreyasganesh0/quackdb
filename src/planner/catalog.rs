//! Lesson 23: Catalog
//!
//! Database catalog for tracking table definitions and their in-memory data.
//! The catalog is the central metadata store: the binder queries it to
//! resolve table/column names, and the physical plan builder reads table
//! data from it.
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
        // Hint: initialize both HashMaps as empty.
        todo!()
    }

    /// Register a new table in the catalog.
    ///
    /// Returns an error if a table with the same name already exists.
    pub fn create_table(&mut self, info: TableInfo) -> Result<(), String> {
        // Hint: check if self.tables already contains the name;
        // if so, return Err. Otherwise insert into both tables and table_data.
        todo!()
    }

    /// Look up a table's metadata by name.
    pub fn get_table(&self, name: &str) -> Option<&TableInfo> {
        // Hint: self.tables.get(name).
        todo!()
    }

    /// Remove a table and its data from the catalog.
    ///
    /// Returns an error if the table does not exist.
    pub fn drop_table(&mut self, name: &str) -> Result<(), String> {
        // Hint: remove from both self.tables and self.table_data.
        todo!()
    }

    /// Append a data chunk to the named table's storage.
    ///
    /// Returns an error if the table does not exist.
    pub fn insert_data(&mut self, table_name: &str, chunk: DataChunk) -> Result<(), String> {
        // Hint: look up the table_data entry and push the chunk.
        todo!()
    }

    /// Retrieve all stored data chunks for a table.
    ///
    /// Returns `None` if the table does not exist.
    pub fn get_table_data(&self, table_name: &str) -> Option<&[DataChunk]> {
        // Hint: self.table_data.get(table_name).map(|v| v.as_slice()).
        todo!()
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
