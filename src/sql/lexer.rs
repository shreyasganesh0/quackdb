//! Lesson 20: SQL Lexer
//!
//! Tokenize SQL strings into a stream of tokens.

use std::fmt;

/// SQL keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Select,
    From,
    Where,
    And,
    Or,
    Not,
    As,
    Join,
    Inner,
    Left,
    Right,
    Full,
    Outer,
    On,
    Group,
    By,
    Having,
    Order,
    Asc,
    Desc,
    Limit,
    Offset,
    Insert,
    Into,
    Values,
    Create,
    Table,
    Drop,
    Null,
    Is,
    In,
    Between,
    Like,
    Case,
    When,
    Then,
    Else,
    End,
    True,
    False,
    Distinct,
    Count,
    Sum,
    Avg,
    Min,
    Max,
    Int,
    Integer,
    Bigint,
    Float,
    Double,
    Varchar,
    Boolean,
    Date,
    Timestamp,
    Primary,
    Key,
    Nulls,
    First,
    Last,
    Over,
    Partition,
    Window,
    Row,
    Rows,
    Range,
    Unbounded,
    Preceding,
    Following,
    Current,
    Rank,
    DenseRank,
    RowNumber,
    Lag,
    Lead,
    Semi,
    Anti,
    Cross,
    Union,
    All,
    Exists,
    Cast,
}

/// A token produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Integer(i64),
    Float(f64),
    StringLiteral(String),
    // Identifiers and keywords
    Identifier(String),
    Keyword(Keyword),
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    NotEqual,      // != or <>
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    // Punctuation
    LeftParen,
    RightParen,
    Comma,
    Semicolon,
    Dot,
    // Special
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

/// Position in source text for error reporting.
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

/// A token with its position in the source.
#[derive(Debug, Clone)]
pub struct PositionedToken {
    pub token: Token,
    pub position: Position,
}

/// Lexer error.
#[derive(Debug, Clone)]
pub struct LexerError {
    pub message: String,
    pub position: Position,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Lexer error at line {}, column {}: {}", self.position.line, self.position.column, self.message)
    }
}

/// SQL Lexer: tokenizes SQL strings.
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    /// Create a new lexer for the given input.
    pub fn new(input: &str) -> Self {
        todo!()
    }

    /// Tokenize the entire input, returning all tokens.
    pub fn tokenize(&mut self) -> Result<Vec<PositionedToken>, LexerError> {
        todo!()
    }

    /// Get the next token.
    pub fn next_token(&mut self) -> Result<PositionedToken, LexerError> {
        todo!()
    }

    /// Try to match a keyword from an identifier string.
    pub fn match_keyword(s: &str) -> Option<Keyword> {
        todo!()
    }
}
