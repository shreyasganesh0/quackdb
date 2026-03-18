//! # Lesson 21: SQL Frontend — Abstract Syntax Tree (File 2 of 2)
//!
//! This file defines the AST node types for parsed SQL statements. These are
//! the data structures that the parser produces and the binder (Lesson 23)
//! consumes. Each node closely mirrors SQL syntax without any semantic
//! analysis (no type checking, no name resolution).
//!
//! It works together with:
//! - `parser.rs` — the recursive descent parser that constructs these AST nodes
//!   from a token stream.
//!
//! **Implementation order**: Read this file first to understand the target data
//! structures, then implement `parser.rs`. This file is mostly type definitions
//! with no `todo!()` calls (no logic to implement), but understanding the shape
//! of the AST is essential before writing the parser.

use crate::types::LogicalType;

/// A complete SQL statement.
#[derive(Debug, Clone)]
pub enum Statement {
    /// A SELECT query.
    Select(SelectStatement),
    /// A CREATE TABLE DDL statement.
    CreateTable(CreateTableStatement),
    /// An INSERT DML statement.
    Insert(InsertStatement),
    /// A DROP TABLE DDL statement.
    Drop(DropTableStatement),
}

/// A parsed SELECT statement with all optional clauses.
#[derive(Debug, Clone)]
pub struct SelectStatement {
    /// The expressions/columns in the SELECT list.
    pub select_list: Vec<SelectItem>,
    /// The FROM clause (table references and joins).
    pub from: Option<TableRef>,
    /// The WHERE predicate, if present.
    pub where_clause: Option<Expr>,
    /// GROUP BY expressions.
    pub group_by: Vec<Expr>,
    /// HAVING predicate (filter on grouped results).
    pub having: Option<Expr>,
    /// ORDER BY specifications.
    pub order_by: Vec<OrderByItem>,
    /// LIMIT expression.
    pub limit: Option<Expr>,
    /// OFFSET expression.
    pub offset: Option<Expr>,
    /// Whether SELECT DISTINCT was specified.
    pub distinct: bool,
}

/// An item in the SELECT list.
#[derive(Debug, Clone)]
pub enum SelectItem {
    /// A single expression, optionally aliased (e.g., `x + 1 AS total`).
    Expression { expr: Expr, alias: Option<String> },
    /// `SELECT *` — all columns from all tables.
    Wildcard,
    /// `SELECT table.*` — all columns from a specific table.
    QualifiedWildcard(String),
}

/// A table reference in the FROM clause.
#[derive(Debug, Clone)]
pub enum TableRef {
    /// A simple table name with an optional alias.
    Table { name: String, alias: Option<String> },
    /// A join between two table references.
    Join {
        left: Box<TableRef>,
        right: Box<TableRef>,
        join_type: JoinTypeAst,
        condition: Option<Expr>,
    },
    /// A subquery used as a table source, with a required alias.
    Subquery { query: Box<SelectStatement>, alias: String },
}

/// Join types as they appear in SQL syntax.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JoinTypeAst {
    Inner,
    Left,
    Right,
    Full,
    Cross,
    Semi,
    Anti,
}

/// Expression AST nodes.
///
/// These represent the syntactic structure of SQL expressions before
/// any semantic analysis (binding, type checking).
#[derive(Debug, Clone)]
pub enum Expr {
    /// A column reference: `[table.]column`.
    ColumnRef { table: Option<String>, column: String },
    /// A literal value (number, string, boolean, NULL).
    Literal(LiteralValue),
    /// A binary operation (e.g., `a + b`, `x = y`, `p AND q`).
    BinaryOp { left: Box<Expr>, op: BinaryOpAst, right: Box<Expr> },
    /// A unary operation (e.g., `-x`, `NOT p`).
    UnaryOp { op: UnaryOpAst, expr: Box<Expr> },
    /// A function call (e.g., `COUNT(x)`, `SUM(DISTINCT y)`).
    Function { name: String, args: Vec<Expr>, distinct: bool },
    /// `expr IS [NOT] NULL`.
    IsNull { expr: Box<Expr>, negated: bool },
    /// `expr [NOT] BETWEEN low AND high`.
    Between { expr: Box<Expr>, low: Box<Expr>, high: Box<Expr>, negated: bool },
    /// `expr [NOT] IN (list)`.
    InList { expr: Box<Expr>, list: Vec<Expr>, negated: bool },
    /// `CASE [operand] WHEN ... THEN ... [ELSE ...] END`.
    Case { operand: Option<Box<Expr>>, when_clauses: Vec<(Expr, Expr)>, else_clause: Option<Box<Expr>> },
    /// `CAST(expr AS type)`.
    Cast { expr: Box<Expr>, data_type: LogicalType },
    /// Wildcard (`*`) — used inside `COUNT(*)`.
    Wildcard,
    /// A window function invocation with OVER clause.
    WindowFunction {
        /// The function expression (e.g., `ROW_NUMBER()`).
        function: Box<Expr>,
        /// PARTITION BY expressions.
        partition_by: Vec<Expr>,
        /// ORDER BY within the window.
        order_by: Vec<OrderByItem>,
        /// Optional frame specification (ROWS/RANGE BETWEEN ...).
        window_frame: Option<WindowFrame>,
    },
}

/// SQL literal values.
#[derive(Debug, Clone)]
pub enum LiteralValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
}

/// Binary operators in SQL expressions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOpAst {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
    Like,
}

/// Unary operators in SQL expressions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOpAst {
    /// Arithmetic negation (`-expr`).
    Negate,
    /// Logical negation (`NOT expr`).
    Not,
}

/// An ORDER BY item: an expression with sort direction and null ordering.
#[derive(Debug, Clone)]
pub struct OrderByItem {
    /// The expression to sort on.
    pub expr: Expr,
    /// `true` for ASC (default), `false` for DESC.
    pub ascending: bool,
    /// Explicit NULLS FIRST / NULLS LAST, or `None` for default.
    pub nulls_first: Option<bool>,
}

/// Window frame specification for window functions.
#[derive(Debug, Clone)]
pub struct WindowFrame {
    /// Whether the frame is ROWS-based or RANGE-based.
    pub mode: WindowFrameMode,
    /// The start bound of the frame.
    pub start: WindowFrameBound,
    /// The end bound of the frame.
    pub end: WindowFrameBound,
}

/// Window frame mode: ROWS counts physical rows, RANGE uses logical values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowFrameMode {
    Rows,
    Range,
}

/// A bound in a window frame specification.
#[derive(Debug, Clone)]
pub enum WindowFrameBound {
    /// `UNBOUNDED PRECEDING` — the start of the partition.
    UnboundedPreceding,
    /// `N PRECEDING` — N rows/values before the current row.
    Preceding(Box<Expr>),
    /// `CURRENT ROW`.
    CurrentRow,
    /// `N FOLLOWING` — N rows/values after the current row.
    Following(Box<Expr>),
    /// `UNBOUNDED FOLLOWING` — the end of the partition.
    UnboundedFollowing,
}

/// A CREATE TABLE statement.
#[derive(Debug, Clone)]
pub struct CreateTableStatement {
    /// The name of the table to create.
    pub table_name: String,
    /// Column definitions.
    pub columns: Vec<ColumnDef>,
}

/// A column definition within CREATE TABLE.
#[derive(Debug, Clone)]
pub struct ColumnDef {
    /// Column name.
    pub name: String,
    /// Column data type.
    pub data_type: LogicalType,
    /// Whether the column allows NULLs.
    pub nullable: bool,
    /// Whether this column is part of the primary key.
    pub primary_key: bool,
}

/// An INSERT statement.
#[derive(Debug, Clone)]
pub struct InsertStatement {
    /// The target table name.
    pub table_name: String,
    /// Optional explicit column list; if `None`, values map to all columns in order.
    pub columns: Option<Vec<String>>,
    /// Rows of values to insert.
    pub values: Vec<Vec<Expr>>,
}

/// A DROP TABLE statement.
#[derive(Debug, Clone)]
pub struct DropTableStatement {
    /// The table to drop.
    pub table_name: String,
    /// Whether `IF EXISTS` was specified (suppresses error if table is missing).
    pub if_exists: bool,
}
