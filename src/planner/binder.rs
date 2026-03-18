//! # Lesson 23: Query Planning — Binder (File 2 of 2)
//!
//! This file implements name resolution and type checking against the catalog.
//! The binder walks the parsed AST, resolves table and column names using the
//! [`Catalog`], checks type compatibility, and produces a fully-resolved
//! [`LogicalPlan`].
//!
//! It works together with:
//! - `catalog.rs` — the database catalog that this binder queries to look up
//!   table schemas and resolve column names.
//!
//! **Implementation order**: Implement `catalog.rs` first, then this file.
//! The binder calls `Catalog::get_table()` and `TableInfo::find_column()` to
//! resolve names, so having the catalog ready simplifies testing.
//!
//! **Key idea:** Maintain a [`BindScope`] that tracks which columns are
//! currently visible (from the FROM clause). As each AST node is visited,
//! column references are resolved against this scope, and type errors are
//! reported early before execution begins.

use super::catalog::Catalog;
use super::logical_plan::{LogicalPlan, LogicalExpr, Schema};
use crate::sql::ast::*;
use crate::types::LogicalType;

/// Error produced during name resolution or type checking.
#[derive(Debug, Clone)]
pub struct BindError {
    /// Human-readable description of the binding failure.
    pub message: String,
}

impl std::fmt::Display for BindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bind error: {}", self.message)
    }
}

/// Tracks the columns that are visible at a given point during binding.
///
/// Each entry is `(optional_table_name, column_name, column_type, resolved_index)`.
/// The table name is `Some` when the column comes from a named table, allowing
/// qualified references like `t.col` to be resolved unambiguously.
#[derive(Debug, Clone)]
pub struct BindScope {
    /// (table_name_or_none, column_name, type, resolved_column_index).
    pub columns: Vec<(Option<String>, String, LogicalType, usize)>,
}

impl BindScope {
    /// Create an empty scope with no visible columns.
    pub fn new() -> Self {
        Self { columns: Vec::new() }
    }

    /// Resolve a column reference within this scope.
    ///
    /// If `table` is `Some`, only matches columns from that table.
    /// Returns the resolved column index and type, or an error if the
    /// column is not found or is ambiguous.
    pub fn resolve(&self, table: Option<&str>, column: &str) -> Result<(usize, LogicalType), BindError> {
        let matches: Vec<_> = self.columns.iter()
            .filter(|(tbl, col, _, _)| {
                col == column && match table {
                    Some(t) => tbl.as_deref() == Some(t),
                    None => true,
                }
            })
            .collect();
        match matches.len() {
            0 => Err(BindError {
                message: format!("Column '{}' not found", column),
            }),
            1 => {
                let (_, _, ty, idx) = matches[0];
                Ok((*idx, ty.clone()))
            }
            _ => Err(BindError {
                message: format!("Ambiguous column reference '{}'", column),
            }),
        }
    }
}

/// The binder: transforms an unresolved AST into a resolved [`LogicalPlan`].
///
/// Holds a reference to the catalog for looking up table schemas.
// Lifetime 'a: the binder borrows the Catalog for the duration of binding.
pub struct Binder<'a> {
    catalog: &'a Catalog,
}

// Lifetime annotation 'a: the impl block and all methods share the same
// lifetime as the borrowed Catalog.
impl<'a> Binder<'a> {
    /// Create a new binder that resolves names against the given catalog.
    pub fn new(catalog: &'a Catalog) -> Self {
        Self { catalog }
    }

    /// Bind a parsed SQL statement into a logical plan.
    ///
    /// Dispatches to statement-specific binding methods based on the
    /// AST variant.
    pub fn bind(&self, stmt: &Statement) -> Result<LogicalPlan, BindError> {
        // Hint: match on stmt — Statement::Select dispatches to
        // bind_select, Statement::CreateTable builds a CreateTable plan, etc.
        todo!()
    }

    /// Bind a SELECT statement, producing a logical plan.
    ///
    /// The binding order follows SQL's logical evaluation order:
    /// FROM -> WHERE -> GROUP BY -> HAVING -> SELECT -> ORDER BY -> LIMIT.
    pub fn bind_select(&self, select: &SelectStatement) -> Result<LogicalPlan, BindError> {
        // Hint: start by binding the FROM clause (if any) to get a base
        // plan and scope. Then layer on Filter (WHERE), Aggregate
        // (GROUP BY + HAVING), Projection (SELECT list), Sort (ORDER BY),
        // and Limit.
        todo!()
    }

    /// Bind a table reference, producing both a plan node and a scope
    /// containing the columns now visible.
    ///
    /// For a simple table, look it up in the catalog and build a Scan node.
    /// For a join, recursively bind both sides and merge their scopes.
    pub fn bind_table_ref(&self, table_ref: &TableRef) -> Result<(LogicalPlan, BindScope), BindError> {
        // Hint: match on table_ref. For TableRef::Table, look up the
        // catalog, build a Scan plan node, and populate a BindScope
        // with the table's columns. For TableRef::Join, recursively
        // bind both sides and combine.
        todo!()
    }

    /// Bind an expression within a scope, producing a resolved [`LogicalExpr`].
    ///
    /// Column references are resolved via the scope. Literals are converted
    /// to [`LogicalExpr::Literal`]. Operators and functions are recursively bound.
    pub fn bind_expression(&self, expr: &Expr, scope: &BindScope) -> Result<LogicalExpr, BindError> {
        // Hint: match on expr. For ColumnRef, call scope.resolve().
        // For BinaryOp/UnaryOp, recursively bind children.
        // For Function, check if it's an aggregate (COUNT, SUM, etc.).
        todo!()
    }

    /// Expand a wildcard (`SELECT *`) into explicit column references
    /// for all columns visible in the current scope.
    pub fn expand_wildcard(&self, scope: &BindScope) -> Vec<LogicalExpr> {
        scope.columns.iter().map(|(_, name, _, idx)| {
            LogicalExpr::ColumnRef {
                index: *idx,
                name: name.clone(),
            }
        }).collect()
    }
}
