//! # Lesson 24: End-to-End Execution — Database Facade (File 2 of 2)
//!
//! This file provides the top-level `Database` struct, the user-facing interface
//! for executing SQL queries against QuackDB. It ties together the full query
//! pipeline: lexing, parsing, binding, planning, and execution.
//!
//! It works together with:
//! - `planner/physical_plan.rs` — the physical plan builder and `execute_plan()`
//!   function that this facade calls to run queries after parsing and binding.
//!
//! **Implementation order**: Implement `planner/physical_plan.rs` first, then
//! this file. `Database::execute_sql` is a thin orchestration layer that calls
//! `Parser::parse_sql`, `Binder::bind`, and `execute_plan` in sequence.
//!
//! **Usage:** Create a [`Database`], then call [`Database::execute_sql`] with
//! a SQL string. The method returns the result as a `Vec<DataChunk>`.

use crate::planner::catalog::Catalog;
use crate::chunk::DataChunk;

/// The top-level QuackDB database instance.
///
/// Owns the [`Catalog`] and orchestrates the full SQL execution pipeline:
/// SQL string -> tokens -> AST -> logical plan -> physical plan -> results.
pub struct Database {
    /// The database catalog storing table schemas and data.
    catalog: Catalog,
}

impl Database {
    /// Create a new empty database with no tables.
    pub fn new() -> Self {
        Self {
            catalog: Catalog::new(),
        }
    }

    /// Execute a SQL query string and return the result chunks.
    ///
    /// Internally performs the full pipeline:
    /// 1. **Lex** the SQL string into tokens.
    /// 2. **Parse** the tokens into an AST.
    /// 3. **Bind** the AST against the catalog (name resolution, type checking).
    /// 4. **Plan** the bound logical plan into physical pipelines.
    /// 5. **Execute** the pipelines and collect the result.
    pub fn execute_sql(&mut self, sql: &str) -> Result<Vec<DataChunk>, String> {
        // Hint: call Parser::parse_sql(sql), then Binder::new(&self.catalog).bind(&stmt),
        // then execute_plan(&plan, &self.catalog). Map BindError/ParseError to String.
        todo!()
    }

    /// Get a shared reference to the catalog (for inspection / read-only queries).
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    /// Get a mutable reference to the catalog (for DDL operations, data loading).
    pub fn catalog_mut(&mut self) -> &mut Catalog {
        &mut self.catalog
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}
