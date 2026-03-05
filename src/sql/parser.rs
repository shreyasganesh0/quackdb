//! Lesson 21: SQL Parser (Pratt Parsing)
//!
//! Recursive descent parser with Pratt parsing for expressions.

use super::ast::*;
use super::lexer::{Token, Keyword, PositionedToken, Lexer};
use crate::types::LogicalType;

/// Parser error.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error at line {}, column {}: {}", self.line, self.column, self.message)
    }
}

/// SQL Parser.
pub struct Parser {
    tokens: Vec<PositionedToken>,
    position: usize,
}

impl Parser {
    /// Create a new parser from a list of tokens.
    pub fn new(tokens: Vec<PositionedToken>) -> Self {
        Self { tokens, position: 0 }
    }

    /// Parse a SQL string into a statement.
    pub fn parse_sql(sql: &str) -> Result<Statement, ParseError> {
        todo!()
    }

    /// Parse a single statement.
    pub fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        todo!()
    }

    /// Parse a SELECT statement.
    pub fn parse_select(&mut self) -> Result<SelectStatement, ParseError> {
        todo!()
    }

    /// Parse a CREATE TABLE statement.
    pub fn parse_create_table(&mut self) -> Result<CreateTableStatement, ParseError> {
        todo!()
    }

    /// Parse an INSERT statement.
    pub fn parse_insert(&mut self) -> Result<InsertStatement, ParseError> {
        todo!()
    }

    /// Parse an expression using Pratt parsing.
    pub fn parse_expression(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        todo!()
    }

    /// Parse a table reference (FROM clause).
    pub fn parse_table_ref(&mut self) -> Result<TableRef, ParseError> {
        todo!()
    }

    /// Parse ORDER BY items.
    pub fn parse_order_by(&mut self) -> Result<Vec<OrderByItem>, ParseError> {
        todo!()
    }

    /// Get binding power for infix operators (for Pratt parsing).
    fn infix_binding_power(op: &BinaryOpAst) -> (u8, u8) {
        todo!()
    }

    /// Get binding power for prefix operators.
    fn prefix_binding_power(op: &UnaryOpAst) -> u8 {
        todo!()
    }
}
