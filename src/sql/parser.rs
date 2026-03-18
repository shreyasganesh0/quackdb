//! Lesson 21: SQL Parser (Pratt Parsing)
//!
//! Recursive descent parser that uses Pratt parsing (binding powers) for
//! expression precedence. Converts a token stream into an AST.
//!
//! **Key idea:** Each statement type (SELECT, CREATE TABLE, INSERT) has its
//! own parsing method. Expressions use Pratt parsing: each operator has a
//! left and right *binding power*; the parser only consumes an operator if
//! its left binding power exceeds the current minimum, naturally handling
//! precedence and associativity.

use super::ast::*;
use super::lexer::{Token, Keyword, PositionedToken, Lexer};
use crate::types::LogicalType;

/// Error produced by the parser.
#[derive(Debug, Clone)]
pub struct ParseError {
    /// Human-readable description of what went wrong.
    pub message: String,
    /// Source line where the error was detected.
    pub line: usize,
    /// Source column where the error was detected.
    pub column: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error at line {}, column {}: {}", self.line, self.column, self.message)
    }
}

/// SQL Parser: consumes tokens and produces AST nodes.
///
/// Holds the token stream and a cursor. Helper methods like `peek`,
/// `advance`, and `expect` simplify recursive descent logic.
pub struct Parser {
    /// The complete list of tokens from the lexer.
    tokens: Vec<PositionedToken>,
    /// Current index into the token list.
    position: usize,
}

impl Parser {
    /// Create a new parser from a list of positioned tokens.
    pub fn new(tokens: Vec<PositionedToken>) -> Self {
        Self { tokens, position: 0 }
    }

    /// Convenience: lex and parse a SQL string in one call.
    pub fn parse_sql(sql: &str) -> Result<Statement, ParseError> {
        // Hint: create a Lexer, call tokenize(), then create a Parser
        // and call parse_statement().
        todo!()
    }

    /// Parse a single SQL statement from the current position.
    ///
    /// Peeks at the first token to determine the statement type, then
    /// dispatches to the appropriate method.
    pub fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        // Hint: match on the current token — Keyword::Select dispatches
        // to parse_select, Keyword::Create to parse_create_table, etc.
        todo!()
    }

    /// Parse a SELECT statement.
    ///
    /// Grammar (simplified):
    /// ```text
    /// SELECT [DISTINCT] select_list
    /// [FROM table_ref]
    /// [WHERE expr]
    /// [GROUP BY expr_list]
    /// [HAVING expr]
    /// [ORDER BY order_list]
    /// [LIMIT expr [OFFSET expr]]
    /// ```
    pub fn parse_select(&mut self) -> Result<SelectStatement, ParseError> {
        // Hint: consume Keyword::Select, check for DISTINCT, then parse
        // the select list (comma-separated expressions/wildcards).
        // Parse each optional clause if the corresponding keyword appears.
        todo!()
    }

    /// Parse a CREATE TABLE statement.
    ///
    /// Grammar: `CREATE TABLE name (col_def, ...)`
    pub fn parse_create_table(&mut self) -> Result<CreateTableStatement, ParseError> {
        // Hint: consume CREATE TABLE, parse the table name, then parse
        // a parenthesized comma-separated list of column definitions
        // (name, data_type, optional NULL/NOT NULL, optional PRIMARY KEY).
        todo!()
    }

    /// Parse an INSERT statement.
    ///
    /// Grammar: `INSERT INTO name [(columns)] VALUES (expr, ...), ...`
    pub fn parse_insert(&mut self) -> Result<InsertStatement, ParseError> {
        // Hint: consume INSERT INTO, parse table name, optional column
        // list in parens, then VALUES followed by parenthesized rows.
        todo!()
    }

    /// Parse an expression using Pratt parsing.
    ///
    /// `min_bp` is the minimum binding power — the parser will only consume
    /// an infix operator if its left binding power is > `min_bp`.
    ///
    /// Start a top-level expression parse with `min_bp = 0`.
    pub fn parse_expression(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        // Hint: parse the "nud" (null denotation / prefix):
        //   - literal, identifier, unary op, parenthesized expr, function call
        // Then loop: peek at the next token; if it's an infix operator
        // with left BP > min_bp, consume it and recursively parse the
        // right side with the operator's right BP.
        todo!()
    }

    /// Parse a table reference (FROM clause), including joins.
    pub fn parse_table_ref(&mut self) -> Result<TableRef, ParseError> {
        // Hint: parse a base table (name with optional alias), then loop
        // checking for JOIN keywords. For each join, parse the join type,
        // the right table ref, and the ON condition.
        todo!()
    }

    /// Parse ORDER BY items (comma-separated expressions with ASC/DESC).
    pub fn parse_order_by(&mut self) -> Result<Vec<OrderByItem>, ParseError> {
        // Hint: consume ORDER BY, then parse comma-separated items:
        // expression, optional ASC/DESC (default ASC), optional NULLS FIRST/LAST.
        todo!()
    }

    // Pratt parsing helper: returns (left_binding_power, right_binding_power)
    // for infix binary operators. Higher BP = tighter binding.
    fn infix_binding_power(op: &BinaryOpAst) -> (u8, u8) {
        // Hint: OR=1,2  AND=3,4  comparisons=5,6  add/sub=7,8  mul/div=9,10
        // Left-associative ops have right BP = left BP + 1.
        todo!()
    }

    // Pratt parsing helper: returns the right binding power for prefix
    // unary operators.
    fn prefix_binding_power(op: &UnaryOpAst) -> u8 {
        // Hint: NOT and Negate typically have high binding power (e.g., 11).
        todo!()
    }
}
