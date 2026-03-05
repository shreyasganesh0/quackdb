//! Lesson 23: Binder
//!
//! Name resolution and type checking against the catalog.

use super::catalog::Catalog;
use super::logical_plan::{LogicalPlan, LogicalExpr, Schema};
use crate::sql::ast::*;
use crate::types::LogicalType;

/// Binder error.
#[derive(Debug, Clone)]
pub struct BindError {
    pub message: String,
}

impl std::fmt::Display for BindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bind error: {}", self.message)
    }
}

/// Scope for tracking available columns during binding.
#[derive(Debug, Clone)]
pub struct BindScope {
    pub columns: Vec<(Option<String>, String, LogicalType, usize)>,
}

impl BindScope {
    pub fn new() -> Self {
        Self { columns: Vec::new() }
    }

    /// Resolve a column reference within this scope.
    pub fn resolve(&self, table: Option<&str>, column: &str) -> Result<(usize, LogicalType), BindError> {
        todo!()
    }
}

/// The binder: resolves names, checks types, and produces logical plans.
pub struct Binder<'a> {
    catalog: &'a Catalog,
}

impl<'a> Binder<'a> {
    pub fn new(catalog: &'a Catalog) -> Self {
        Self { catalog }
    }

    /// Bind a parsed SQL statement into a logical plan.
    pub fn bind(&self, stmt: &Statement) -> Result<LogicalPlan, BindError> {
        todo!()
    }

    /// Bind a SELECT statement.
    pub fn bind_select(&self, select: &SelectStatement) -> Result<LogicalPlan, BindError> {
        todo!()
    }

    /// Bind a table reference, producing a plan and a scope.
    pub fn bind_table_ref(&self, table_ref: &TableRef) -> Result<(LogicalPlan, BindScope), BindError> {
        todo!()
    }

    /// Bind an expression within a scope.
    pub fn bind_expression(&self, expr: &Expr, scope: &BindScope) -> Result<LogicalExpr, BindError> {
        todo!()
    }

    /// Expand SELECT * into all columns from scope.
    pub fn expand_wildcard(&self, scope: &BindScope) -> Vec<LogicalExpr> {
        todo!()
    }
}
