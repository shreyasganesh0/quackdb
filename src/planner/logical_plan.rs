//! Lesson 22: Logical Query Plan
//!
//! Logical plan nodes and schema propagation. A logical plan is a tree of
//! relational algebra operators that describes *what* to compute without
//! specifying *how*. Each node carries its output [`Schema`] so that
//! downstream operators know the column names and types they will receive.
//!
//! **Key idea:** Build the plan bottom-up from the AST. Each SQL clause
//! maps to a plan node: FROM -> Scan, WHERE -> Filter, SELECT list ->
//! Projection, GROUP BY -> Aggregate, etc. Schema propagation ensures
//! type correctness at every level.

use crate::types::LogicalType;
use std::fmt;

/// Logical expression used inside logical plan nodes.
///
/// These are *bound* expressions: column references carry resolved indices
/// rather than raw names, and types have been checked.
#[derive(Debug, Clone)]
pub enum LogicalExpr {
    /// A resolved column reference (by index and name).
    ColumnRef { index: usize, name: String },
    /// A constant literal value.
    Literal(crate::types::ScalarValue),
    /// A binary operation on two sub-expressions.
    BinaryOp { op: crate::sql::ast::BinaryOpAst, left: Box<LogicalExpr>, right: Box<LogicalExpr> },
    /// A unary operation on a sub-expression.
    UnaryOp { op: crate::sql::ast::UnaryOpAst, expr: Box<LogicalExpr> },
    /// An aggregate function call (e.g., SUM, COUNT).
    AggregateFunction { func: String, args: Vec<LogicalExpr>, distinct: bool },
    /// An aliased expression (e.g., `expr AS name`).
    Alias { expr: Box<LogicalExpr>, name: String },
    /// A type cast.
    Cast { expr: Box<LogicalExpr>, target_type: LogicalType },
    /// An IS [NOT] NULL test.
    IsNull { expr: Box<LogicalExpr>, negated: bool },
    /// A CASE WHEN expression.
    Case { operand: Option<Box<LogicalExpr>>, when_clauses: Vec<(LogicalExpr, LogicalExpr)>, else_clause: Option<Box<LogicalExpr>> },
    /// A window function with partitioning and ordering.
    WindowFunction { func: Box<LogicalExpr>, partition_by: Vec<LogicalExpr>, order_by: Vec<LogicalSortKey>, frame: Option<crate::sql::ast::WindowFrame> },
    /// Wildcard (`*`) — expanded into concrete columns during binding.
    Wildcard,
}

/// A sort key in a logical plan (used for ORDER BY).
#[derive(Debug, Clone)]
pub struct LogicalSortKey {
    /// The expression to sort on.
    pub expr: LogicalExpr,
    /// `true` for ascending order, `false` for descending.
    pub ascending: bool,
    /// Explicit NULLS FIRST / NULLS LAST override, or `None` for default.
    pub nulls_first: Option<bool>,
}

/// Schema: an ordered list of (column_name, column_type) pairs.
///
/// Every plan node can produce its output schema, enabling type checking
/// and column resolution at plan construction time.
#[derive(Debug, Clone)]
pub struct Schema {
    pub columns: Vec<(String, LogicalType)>,
}

impl Schema {
    /// Create a schema from a list of (name, type) pairs.
    pub fn new(columns: Vec<(String, LogicalType)>) -> Self {
        Self { columns }
    }

    /// Return the number of columns.
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Find a column by name, returning its index if found.
    pub fn find_column(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|(n, _)| n == name)
    }

    /// Extract just the types, in column order.
    pub fn types(&self) -> Vec<LogicalType> {
        self.columns.iter().map(|(_, t)| t.clone()).collect()
    }

    /// Produce a new schema that concatenates this schema with `other`.
    ///
    /// Used for joins where the output contains columns from both sides.
    pub fn merge(&self, other: &Schema) -> Schema {
        let mut cols = self.columns.clone();
        cols.extend(other.columns.iter().cloned());
        Schema { columns: cols }
    }
}

/// Logical plan nodes — the relational algebra tree.
///
/// Each variant corresponds to a relational operator. The tree is built
/// bottom-up: leaf nodes are `Scan` or `Values`, and internal nodes wrap
/// their child plans in `Box<LogicalPlan>`.
#[derive(Debug, Clone)]
pub enum LogicalPlan {
    /// Scan a base table, optionally projecting a subset of columns.
    Scan { table_name: String, schema: Schema, projection: Option<Vec<usize>> },
    /// Filter rows by a boolean predicate.
    Filter { predicate: LogicalExpr, input: Box<LogicalPlan> },
    /// Project columns / compute new expressions.
    Projection { expressions: Vec<LogicalExpr>, input: Box<LogicalPlan> },
    /// Group-by aggregation.
    Aggregate { group_exprs: Vec<LogicalExpr>, agg_exprs: Vec<LogicalExpr>, input: Box<LogicalPlan> },
    /// Join two plan trees.
    Join { left: Box<LogicalPlan>, right: Box<LogicalPlan>, join_type: crate::sql::ast::JoinTypeAst, condition: Option<LogicalExpr> },
    /// Sort the input by the given keys.
    Sort { keys: Vec<LogicalSortKey>, input: Box<LogicalPlan> },
    /// Limit the number of output rows (with optional offset).
    Limit { count: usize, offset: usize, input: Box<LogicalPlan> },
    /// An empty result set with a known schema.
    Empty { schema: Schema },
    /// Literal row values (e.g., from `INSERT ... VALUES`).
    Values { rows: Vec<Vec<LogicalExpr>>, schema: Schema },
    /// DDL: create a new table.
    CreateTable { table_name: String, columns: Vec<(String, LogicalType, bool)> },
    /// DML: insert rows produced by `input` into a table.
    Insert { table_name: String, input: Box<LogicalPlan> },
    /// Window function evaluation over a partitioned/ordered input.
    Window { window_exprs: Vec<LogicalExpr>, input: Box<LogicalPlan> },
}

impl LogicalPlan {
    /// Compute the output schema of this plan node.
    ///
    /// Each node type derives its schema differently:
    /// - `Scan`: from the catalog table definition (with projection applied).
    /// - `Filter`: same as its input (filtering does not change columns).
    /// - `Projection`: one column per expression.
    /// - `Aggregate`: group columns followed by aggregate result columns.
    /// - `Join`: merge of left and right schemas.
    pub fn schema(&self) -> Schema {
        // Hint: match on self and compute the schema for each variant.
        // For Filter, Sort, Limit: return input.schema().
        // For Projection: derive names/types from the expression list.
        todo!()
    }

    /// Pretty-print the plan as an indented tree string.
    ///
    /// Useful for EXPLAIN output and debugging.
    pub fn pretty_print(&self) -> String {
        // Hint: recursively format each node with increasing indentation.
        // E.g., "Projection [col1, col2]\n  Filter (predicate)\n    Scan(table)"
        todo!()
    }

    /// Convert a parsed AST [`SelectStatement`] directly into a logical plan.
    ///
    /// This is a convenience shortcut; in production, use the [`Binder`] for
    /// proper name resolution and type checking.
    pub fn from_select(select: &crate::sql::ast::SelectStatement) -> Result<Self, String> {
        // Hint: build bottom-up: start with the FROM clause (Scan),
        // add Filter for WHERE, Aggregate for GROUP BY, Projection
        // for the SELECT list, Sort for ORDER BY, Limit for LIMIT.
        todo!()
    }
}

impl fmt::Display for LogicalPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.pretty_print())
    }
}
