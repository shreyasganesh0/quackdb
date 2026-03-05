//! Lesson 22: Logical Query Plan
//!
//! Logical plan nodes and schema propagation.

use crate::types::LogicalType;
use std::fmt;

/// Logical expression (used in logical plans).
#[derive(Debug, Clone)]
pub enum LogicalExpr {
    ColumnRef { index: usize, name: String },
    Literal(crate::types::ScalarValue),
    BinaryOp { op: crate::sql::ast::BinaryOpAst, left: Box<LogicalExpr>, right: Box<LogicalExpr> },
    UnaryOp { op: crate::sql::ast::UnaryOpAst, expr: Box<LogicalExpr> },
    AggregateFunction { func: String, args: Vec<LogicalExpr>, distinct: bool },
    Alias { expr: Box<LogicalExpr>, name: String },
    Cast { expr: Box<LogicalExpr>, target_type: LogicalType },
    IsNull { expr: Box<LogicalExpr>, negated: bool },
    Case { operand: Option<Box<LogicalExpr>>, when_clauses: Vec<(LogicalExpr, LogicalExpr)>, else_clause: Option<Box<LogicalExpr>> },
    WindowFunction { func: Box<LogicalExpr>, partition_by: Vec<LogicalExpr>, order_by: Vec<LogicalSortKey>, frame: Option<crate::sql::ast::WindowFrame> },
    Wildcard,
}

/// Sort key for ORDER BY in logical plans.
#[derive(Debug, Clone)]
pub struct LogicalSortKey {
    pub expr: LogicalExpr,
    pub ascending: bool,
    pub nulls_first: Option<bool>,
}

/// Schema: list of (name, type) pairs.
#[derive(Debug, Clone)]
pub struct Schema {
    pub columns: Vec<(String, LogicalType)>,
}

impl Schema {
    pub fn new(columns: Vec<(String, LogicalType)>) -> Self {
        Self { columns }
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn find_column(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|(n, _)| n == name)
    }

    pub fn types(&self) -> Vec<LogicalType> {
        self.columns.iter().map(|(_, t)| t.clone()).collect()
    }

    pub fn merge(&self, other: &Schema) -> Schema {
        let mut cols = self.columns.clone();
        cols.extend(other.columns.iter().cloned());
        Schema { columns: cols }
    }
}

/// Logical plan nodes.
#[derive(Debug, Clone)]
pub enum LogicalPlan {
    /// Scan a table.
    Scan { table_name: String, schema: Schema, projection: Option<Vec<usize>> },
    /// Filter rows.
    Filter { predicate: LogicalExpr, input: Box<LogicalPlan> },
    /// Project columns/expressions.
    Projection { expressions: Vec<LogicalExpr>, input: Box<LogicalPlan> },
    /// Aggregate with GROUP BY.
    Aggregate { group_exprs: Vec<LogicalExpr>, agg_exprs: Vec<LogicalExpr>, input: Box<LogicalPlan> },
    /// Join two plans.
    Join { left: Box<LogicalPlan>, right: Box<LogicalPlan>, join_type: crate::sql::ast::JoinTypeAst, condition: Option<LogicalExpr> },
    /// Sort.
    Sort { keys: Vec<LogicalSortKey>, input: Box<LogicalPlan> },
    /// Limit.
    Limit { count: usize, offset: usize, input: Box<LogicalPlan> },
    /// Empty result.
    Empty { schema: Schema },
    /// Values (for INSERT ... VALUES).
    Values { rows: Vec<Vec<LogicalExpr>>, schema: Schema },
    /// Create table.
    CreateTable { table_name: String, columns: Vec<(String, LogicalType, bool)> },
    /// Insert into table.
    Insert { table_name: String, input: Box<LogicalPlan> },
    /// Window.
    Window { window_exprs: Vec<LogicalExpr>, input: Box<LogicalPlan> },
}

impl LogicalPlan {
    /// Get the output schema of this plan node.
    pub fn schema(&self) -> Schema {
        todo!()
    }

    /// Pretty-print the plan tree.
    pub fn pretty_print(&self) -> String {
        todo!()
    }

    /// Convert an AST SelectStatement to a LogicalPlan.
    pub fn from_select(select: &crate::sql::ast::SelectStatement) -> Result<Self, String> {
        todo!()
    }
}

impl fmt::Display for LogicalPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pretty_print())
    }
}
