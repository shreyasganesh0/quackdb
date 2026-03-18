//! # Lesson 21: SQL Frontend — Parser (File 1 of 2)
//!
//! This file implements the SQL parser, a recursive descent parser that uses
//! Pratt parsing (binding powers) for expression precedence. It consumes a
//! token stream and produces AST nodes defined in `ast.rs`.
//!
//! It works together with:
//! - `ast.rs` — defines all AST node types (`Statement`, `Expr`, `TableRef`,
//!   etc.) that this parser constructs.
//!
//! **Start here**: Read `ast.rs` first to understand the target data structures,
//! then implement `parser.rs`. The AST types are mostly data definitions with
//! no logic, so reading them is quick and gives you the "shape" of what the
//! parser must produce.
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
        let mut lexer = Lexer::new(sql);
        let tokens = lexer.tokenize().map_err(|e| ParseError {
            message: e.message,
            line: e.position.line,
            column: e.position.column,
        })?;
        let mut parser = Parser::new(tokens);
        parser.parse_statement()
    }

    // --- Helper methods for token navigation ---

    /// Peek at the current token without advancing.
    fn peek(&self) -> &Token {
        if self.position < self.tokens.len() {
            &self.tokens[self.position].token
        } else {
            &Token::Eof
        }
    }

    /// Get the current position info for error reporting.
    fn current_position(&self) -> (usize, usize) {
        if self.position < self.tokens.len() {
            let pos = &self.tokens[self.position].position;
            (pos.line, pos.column)
        } else {
            (0, 0)
        }
    }

    /// Advance the cursor by one token and return the consumed token.
    fn advance(&mut self) -> Token {
        if self.position < self.tokens.len() {
            let tok = self.tokens[self.position].token.clone();
            self.position += 1;
            tok
        } else {
            Token::Eof
        }
    }

    /// Expect a specific keyword; return error if not found.
    fn expect_keyword(&mut self, kw: Keyword) -> Result<(), ParseError> {
        let (line, column) = self.current_position();
        match self.advance() {
            Token::Keyword(k) if k == kw => Ok(()),
            other => Err(ParseError {
                message: format!("Expected keyword {:?}, found {:?}", kw, other),
                line,
                column,
            }),
        }
    }

    /// Expect a specific token; return error if not found.
    fn expect_token(&mut self, expected: &Token) -> Result<(), ParseError> {
        let (line, column) = self.current_position();
        let tok = self.advance();
        if std::mem::discriminant(&tok) == std::mem::discriminant(expected) {
            Ok(())
        } else {
            Err(ParseError {
                message: format!("Expected {:?}, found {:?}", expected, tok),
                line,
                column,
            })
        }
    }

    /// Parse an identifier and return its name.
    fn parse_identifier(&mut self) -> Result<String, ParseError> {
        let (line, column) = self.current_position();
        match self.advance() {
            Token::Identifier(name) => Ok(name),
            other => Err(ParseError {
                message: format!("Expected identifier, found {:?}", other),
                line,
                column,
            }),
        }
    }

    /// Check if the current token is a specific keyword; if so, consume it.
    fn match_keyword(&mut self, kw: Keyword) -> bool {
        if matches!(self.peek(), Token::Keyword(k) if *k == kw) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Parse a single SQL statement from the current position.
    ///
    /// Peeks at the first token to determine the statement type, then
    /// dispatches to the appropriate method.
    pub fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        let (line, column) = self.current_position();
        match self.peek() {
            Token::Keyword(Keyword::Select) => {
                Ok(Statement::Select(self.parse_select()?))
            }
            Token::Keyword(Keyword::Create) => {
                Ok(Statement::CreateTable(self.parse_create_table()?))
            }
            Token::Keyword(Keyword::Insert) => {
                Ok(Statement::Insert(self.parse_insert()?))
            }
            Token::Keyword(Keyword::Drop) => {
                self.advance(); // consume DROP
                self.expect_keyword(Keyword::Table)?;
                let if_exists = if matches!(self.peek(), Token::Keyword(Keyword::Exists)) {
                    // Check for IF EXISTS (simplified: just check for EXISTS after DROP TABLE)
                    false
                } else {
                    false
                };
                let table_name = self.parse_identifier()?;
                Ok(Statement::Drop(DropTableStatement { table_name, if_exists }))
            }
            _ => Err(ParseError {
                message: format!("Unexpected token: {:?}", self.peek()),
                line,
                column,
            }),
        }
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
        self.expect_keyword(Keyword::Create)?;
        self.expect_keyword(Keyword::Table)?;
        let table_name = self.parse_identifier()?;
        self.expect_token(&Token::LeftParen)?;

        let mut columns = Vec::new();
        loop {
            let col_name = self.parse_identifier()?;
            let data_type = self.parse_data_type()?;

            let mut nullable = true;
            let mut primary_key = false;

            // Check for optional constraints
            loop {
                if self.match_keyword(Keyword::Not) {
                    self.expect_keyword(Keyword::Null)?;
                    nullable = false;
                } else if self.match_keyword(Keyword::Null) {
                    nullable = true;
                } else if self.match_keyword(Keyword::Primary) {
                    self.expect_keyword(Keyword::Key)?;
                    primary_key = true;
                    nullable = false;
                } else {
                    break;
                }
            }

            columns.push(ColumnDef {
                name: col_name,
                data_type,
                nullable,
                primary_key,
            });

            if matches!(self.peek(), Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        self.expect_token(&Token::RightParen)?;
        // Consume optional semicolon
        if matches!(self.peek(), Token::Semicolon) {
            self.advance();
        }

        Ok(CreateTableStatement { table_name, columns })
    }

    /// Parse a SQL data type keyword into a LogicalType.
    fn parse_data_type(&mut self) -> Result<LogicalType, ParseError> {
        let (line, column) = self.current_position();
        match self.advance() {
            Token::Keyword(Keyword::Int) | Token::Keyword(Keyword::Integer) => Ok(LogicalType::Int32),
            Token::Keyword(Keyword::Bigint) => Ok(LogicalType::Int64),
            Token::Keyword(Keyword::Float) => Ok(LogicalType::Float32),
            Token::Keyword(Keyword::Double) => Ok(LogicalType::Float64),
            Token::Keyword(Keyword::Varchar) => {
                // Optionally consume (N)
                if matches!(self.peek(), Token::LeftParen) {
                    self.advance();
                    self.advance(); // consume the length
                    self.expect_token(&Token::RightParen)?;
                }
                Ok(LogicalType::Varchar)
            }
            Token::Keyword(Keyword::Boolean) => Ok(LogicalType::Boolean),
            Token::Keyword(Keyword::Date) => Ok(LogicalType::Date),
            Token::Keyword(Keyword::Timestamp) => Ok(LogicalType::Timestamp),
            other => Err(ParseError {
                message: format!("Expected data type, found {:?}", other),
                line,
                column,
            }),
        }
    }

    /// Parse an INSERT statement.
    ///
    /// Grammar: `INSERT INTO name [(columns)] VALUES (expr, ...), ...`
    pub fn parse_insert(&mut self) -> Result<InsertStatement, ParseError> {
        self.expect_keyword(Keyword::Insert)?;
        self.expect_keyword(Keyword::Into)?;
        let table_name = self.parse_identifier()?;

        // Optional column list
        let columns = if matches!(self.peek(), Token::LeftParen) {
            // Check if this is a column list (identifiers) or VALUES
            // Peek ahead: if next after '(' is an identifier, it's a column list
            let saved_pos = self.position;
            self.advance(); // consume '('
            if matches!(self.peek(), Token::Identifier(_)) {
                // Parse column list
                let mut cols = Vec::new();
                loop {
                    cols.push(self.parse_identifier()?);
                    if matches!(self.peek(), Token::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.expect_token(&Token::RightParen)?;
                Some(cols)
            } else {
                // Not a column list, rewind
                self.position = saved_pos;
                None
            }
        } else {
            None
        };

        self.expect_keyword(Keyword::Values)?;

        // Parse rows: (expr, ...), ...
        let mut values = Vec::new();
        loop {
            self.expect_token(&Token::LeftParen)?;
            let mut row = Vec::new();
            loop {
                row.push(self.parse_expression(0)?);
                if matches!(self.peek(), Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect_token(&Token::RightParen)?;
            values.push(row);

            if matches!(self.peek(), Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        // Consume optional semicolon
        if matches!(self.peek(), Token::Semicolon) {
            self.advance();
        }

        Ok(InsertStatement {
            table_name,
            columns,
            values,
        })
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
        match op {
            BinaryOpAst::Or => (1, 2),
            BinaryOpAst::And => (3, 4),
            BinaryOpAst::Equal
            | BinaryOpAst::NotEqual
            | BinaryOpAst::LessThan
            | BinaryOpAst::LessThanOrEqual
            | BinaryOpAst::GreaterThan
            | BinaryOpAst::GreaterThanOrEqual
            | BinaryOpAst::Like => (5, 6),
            BinaryOpAst::Add | BinaryOpAst::Subtract => (7, 8),
            BinaryOpAst::Multiply | BinaryOpAst::Divide | BinaryOpAst::Modulo => (9, 10),
        }
    }

    // Pratt parsing helper: returns the right binding power for prefix
    // unary operators.
    fn prefix_binding_power(op: &UnaryOpAst) -> u8 {
        match op {
            UnaryOpAst::Not => 11,
            UnaryOpAst::Negate => 11,
        }
    }
}
