//! Lesson 20: SQL Lexer
//!
//! Tokenizes a SQL string into a stream of [`Token`]s with source positions.
//! The lexer handles keywords, identifiers, numeric and string literals,
//! operators, and punctuation.
//!
//! **Key idea:** Consume the input one character at a time. Whitespace is
//! skipped, multi-character tokens (identifiers, numbers, strings) are
//! accumulated, and each token records its line/column for error reporting.

use std::fmt;

/// SQL keywords recognized by the lexer.
///
/// When an identifier matches one of these (case-insensitively), the lexer
/// emits `Token::Keyword(kw)` instead of `Token::Identifier`.
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
///
/// Variants cover literals, identifiers/keywords, operators, punctuation,
/// and the end-of-file sentinel.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // --- Literals ---
    /// An integer literal (e.g., `42`).
    Integer(i64),
    /// A floating-point literal (e.g., `3.14`).
    Float(f64),
    /// A string literal enclosed in single quotes (e.g., `'hello'`).
    StringLiteral(String),

    // --- Identifiers and keywords ---
    /// A non-keyword identifier (table name, column name, alias).
    Identifier(String),
    /// A recognized SQL keyword.
    Keyword(Keyword),

    // --- Operators ---
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    /// `!=` or `<>`.
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,

    // --- Punctuation ---
    LeftParen,
    RightParen,
    Comma,
    Semicolon,
    Dot,

    // --- Special ---
    /// Marks the end of the input.
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Integer(n) => write!(f, "{}", n),
            Token::Float(n) => write!(f, "{}", n),
            Token::StringLiteral(s) => write!(f, "'{}'", s),
            Token::Identifier(s) => write!(f, "{}", s),
            Token::Keyword(kw) => write!(f, "{:?}", kw),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Equal => write!(f, "="),
            Token::NotEqual => write!(f, "!="),
            Token::LessThan => write!(f, "<"),
            Token::LessThanOrEqual => write!(f, "<="),
            Token::GreaterThan => write!(f, ">"),
            Token::GreaterThanOrEqual => write!(f, ">="),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"),
            Token::Dot => write!(f, "."),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

/// Position in the source text, used for error reporting.
#[derive(Debug, Clone, Copy)]
pub struct Position {
    /// 1-based line number.
    pub line: usize,
    /// 1-based column number.
    pub column: usize,
    /// 0-based byte offset from the start of input.
    pub offset: usize,
}

/// A token paired with its source position.
#[derive(Debug, Clone)]
pub struct PositionedToken {
    /// The token value.
    pub token: Token,
    /// Where in the source this token starts.
    pub position: Position,
}

/// Error produced by the lexer when it encounters invalid input.
#[derive(Debug, Clone)]
pub struct LexerError {
    /// Human-readable error description.
    pub message: String,
    /// Where in the source the error occurred.
    pub position: Position,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Lexer error at line {}, column {}: {}", self.position.line, self.position.column, self.message)
    }
}

/// SQL Lexer: consumes a string and produces a sequence of tokens.
///
/// The lexer maintains a cursor (`position`) into a `Vec<char>` and
/// tracks the current line and column for error messages.
pub struct Lexer {
    /// The input as a vector of characters (for easy indexing).
    input: Vec<char>,
    /// Current character index into `input`.
    position: usize,
    /// Current line number (1-based).
    line: usize,
    /// Current column number (1-based).
    column: usize,
}

impl Lexer {
    /// Create a new lexer for the given SQL string.
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    /// Tokenize the entire input, returning all tokens (including a trailing `Eof`).
    pub fn tokenize(&mut self) -> Result<Vec<PositionedToken>, LexerError> {
        let mut tokens = Vec::new();
        loop {
            let positioned = self.next_token()?;
            let is_eof = positioned.token == Token::Eof;
            tokens.push(positioned);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    /// Advance the cursor and return the next token.
    ///
    /// Skips whitespace, then examines the current character to determine
    /// the token type:
    /// - Digit -> number literal (integer or float)
    /// - Letter/underscore -> identifier or keyword
    /// - Single quote -> string literal
    /// - Operator or punctuation character -> corresponding token
    pub fn next_token(&mut self) -> Result<PositionedToken, LexerError> {
        // Hint: skip whitespace first. If position >= input.len(), return Eof.
        // Then match on the current character to dispatch to the appropriate
        // scanning logic (scan_number, scan_identifier, scan_string, etc.).
        todo!()
    }

    /// Try to match an identifier string (case-insensitively) to a SQL keyword.
    ///
    /// Returns `Some(keyword)` if the string is a keyword, `None` otherwise.
    pub fn match_keyword(s: &str) -> Option<Keyword> {
        // Hint: convert s to uppercase and match against known keywords.
        // A match statement or a HashMap lookup both work.
        todo!()
    }
}
