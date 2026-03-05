//! Lesson 21: Abstract Syntax Tree
//!
//! AST nodes for parsed SQL statements.

use crate::types::LogicalType;

/// A complete SQL statement.
#[derive(Debug, Clone)]
pub enum Statement {
    Select(SelectStatement),
    CreateTable(CreateTableStatement),
    Insert(InsertStatement),
    Drop(DropTableStatement),
}

/// SELECT statement.
#[derive(Debug, Clone)]
pub struct SelectStatement {
    pub select_list: Vec<SelectItem>,
    pub from: Option<TableRef>,
    pub where_clause: Option<Expr>,
    pub group_by: Vec<Expr>,
    pub having: Option<Expr>,
    pub order_by: Vec<OrderByItem>,
    pub limit: Option<Expr>,
    pub offset: Option<Expr>,
    pub distinct: bool,
}

/// An item in the SELECT list.
#[derive(Debug, Clone)]
pub enum SelectItem {
    /// A single expression, optionally aliased.
    Expression { expr: Expr, alias: Option<String> },
    /// SELECT *
    Wildcard,
    /// SELECT table.*
    QualifiedWildcard(String),
}

/// Table reference in FROM clause.
#[derive(Debug, Clone)]
pub enum TableRef {
    /// A simple table name.
    Table { name: String, alias: Option<String> },
    /// A join between two table references.
    Join {
        left: Box<TableRef>,
        right: Box<TableRef>,
        join_type: JoinTypeAst,
        condition: Option<Expr>,
    },
    /// A subquery.
    Subquery { query: Box<SelectStatement>, alias: String },
}

/// Join types in SQL syntax.
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

/// Expression nodes.
#[derive(Debug, Clone)]
pub enum Expr {
    /// A column reference: optional_table.column
    ColumnRef { table: Option<String>, column: String },
    /// A literal value.
    Literal(LiteralValue),
    /// Binary operation.
    BinaryOp { left: Box<Expr>, op: BinaryOpAst, right: Box<Expr> },
    /// Unary operation.
    UnaryOp { op: UnaryOpAst, expr: Box<Expr> },
    /// Function call.
    Function { name: String, args: Vec<Expr>, distinct: bool },
    /// IS NULL / IS NOT NULL.
    IsNull { expr: Box<Expr>, negated: bool },
    /// BETWEEN.
    Between { expr: Box<Expr>, low: Box<Expr>, high: Box<Expr>, negated: bool },
    /// IN (list).
    InList { expr: Box<Expr>, list: Vec<Expr>, negated: bool },
    /// CASE WHEN.
    Case { operand: Option<Box<Expr>>, when_clauses: Vec<(Expr, Expr)>, else_clause: Option<Box<Expr>> },
    /// CAST(expr AS type).
    Cast { expr: Box<Expr>, data_type: LogicalType },
    /// Wildcard (*) — used in COUNT(*).
    Wildcard,
    /// Window function.
    WindowFunction {
        function: Box<Expr>,
        partition_by: Vec<Expr>,
        order_by: Vec<OrderByItem>,
        window_frame: Option<WindowFrame>,
    },
}

/// Literal values in SQL.
#[derive(Debug, Clone)]
pub enum LiteralValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
}

/// Binary operators in SQL.
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

/// Unary operators in SQL.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOpAst {
    Negate,
    Not,
}

/// ORDER BY item.
#[derive(Debug, Clone)]
pub struct OrderByItem {
    pub expr: Expr,
    pub ascending: bool,
    pub nulls_first: Option<bool>,
}

/// Window frame specification.
#[derive(Debug, Clone)]
pub struct WindowFrame {
    pub mode: WindowFrameMode,
    pub start: WindowFrameBound,
    pub end: WindowFrameBound,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowFrameMode {
    Rows,
    Range,
}

#[derive(Debug, Clone)]
pub enum WindowFrameBound {
    UnboundedPreceding,
    Preceding(Box<Expr>),
    CurrentRow,
    Following(Box<Expr>),
    UnboundedFollowing,
}

/// CREATE TABLE statement.
#[derive(Debug, Clone)]
pub struct CreateTableStatement {
    pub table_name: String,
    pub columns: Vec<ColumnDef>,
}

/// Column definition in CREATE TABLE.
#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: LogicalType,
    pub nullable: bool,
    pub primary_key: bool,
}

/// INSERT statement.
#[derive(Debug, Clone)]
pub struct InsertStatement {
    pub table_name: String,
    pub columns: Option<Vec<String>>,
    pub values: Vec<Vec<Expr>>,
}

/// DROP TABLE statement.
#[derive(Debug, Clone)]
pub struct DropTableStatement {
    pub table_name: String,
    pub if_exists: bool,
}
