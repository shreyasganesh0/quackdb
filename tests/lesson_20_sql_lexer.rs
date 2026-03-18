//! # Lesson 20: SQL Lexer — Test Suite
//!
//! Tests are ordered from simple to complex:
//! 1. Keyword matching (`test_keyword_matching`)
//! 2. Simple SELECT tokenization (`test_lex_select`)
//! 3. Case insensitivity (`test_lex_case_insensitive`)
//! 4. Literal types — integers, floats, strings (`test_lex_integer`, `test_lex_float`, `test_lex_string`)
//! 5. Operators and punctuation (`test_lex_operators`, `test_lex_punctuation`)
//! 6. Edge cases (unterminated string, empty input, dot notation)
//! 7. Position tracking (`test_lex_position_tracking`)
//! 8. Complex queries — JOIN, CREATE TABLE (`test_lex_complex_query`, `test_lex_join`, `test_lex_create_table`)

use quackdb::sql::lexer::*;

/// Helper: tokenize SQL and return just the Token values (stripping position info).
/// Reduces boilerplate when tests only care about token types, not source positions.
fn tokenize(sql: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(sql);
    lexer.tokenize()
        .expect("tokenization should succeed")
        .into_iter()
        .map(|t| t.token)
        .collect()
}

/// Helper: tokenize SQL and extract only the Keyword tokens.
/// Useful for tests that verify keyword recognition without checking every token.
fn extract_keywords(sql: &str) -> Vec<Keyword> {
    tokenize(sql)
        .into_iter()
        .filter_map(|t| if let Token::Keyword(k) = t { Some(k) } else { None })
        .collect()
}

// ── 1. Keyword matching ─────────────────────────────────────────────

#[test]
fn test_keyword_matching() {
    assert_eq!(Lexer::match_keyword("SELECT"), Some(Keyword::Select));
    assert_eq!(Lexer::match_keyword("select"), Some(Keyword::Select));
    assert_eq!(Lexer::match_keyword("Select"), Some(Keyword::Select));
    assert_eq!(Lexer::match_keyword("not_a_keyword"), None);
}

// ── 2. Simple SELECT ────────────────────────────────────────────────

#[test]
fn test_lex_select() {
    let mut lexer = Lexer::new("SELECT * FROM users");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::Keyword(Keyword::Select), "SELECT should be recognized as a keyword, not an identifier");
    assert_eq!(tokens[1].token, Token::Star);
    assert_eq!(tokens[2].token, Token::Keyword(Keyword::From));
    assert_eq!(tokens[3].token, Token::Identifier("users".to_string()), "table names are identifiers, not keywords");
    assert_eq!(tokens[4].token, Token::Eof, "token stream must end with EOF to signal end of input");
}

// ── 3. Case insensitivity ───────────────────────────────────────────

#[test]
fn test_lex_case_insensitive() {
    let tokens = tokenize("select FROM Where");
    assert_eq!(tokens[0], Token::Keyword(Keyword::Select), "SQL keywords must be case-insensitive: 'select' == 'SELECT'");
    assert_eq!(tokens[1], Token::Keyword(Keyword::From));
    assert_eq!(tokens[2], Token::Keyword(Keyword::Where));
}

// ── 4. Literal types ────────────────────────────────────────────────

#[test]
fn test_lex_integer() {
    let mut lexer = Lexer::new("42 0 -100");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::Integer(42));
    assert_eq!(tokens[1].token, Token::Integer(0));
    // -100 could be Minus then Integer(100)
    assert_eq!(tokens[2].token, Token::Minus, "lexer treats minus as an operator, not part of a negative literal");
    assert_eq!(tokens[3].token, Token::Integer(100));
}

#[test]
fn test_lex_float() {
    let mut lexer = Lexer::new("3.14 0.5");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::Float(3.14));
    assert_eq!(tokens[1].token, Token::Float(0.5));
}

#[test]
fn test_lex_string() {
    let mut lexer = Lexer::new("'hello world' 'it''s'");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::StringLiteral("hello world".to_string()));
    // Escaped single quote
    assert_eq!(tokens[1].token, Token::StringLiteral("it's".to_string()), "SQL escapes single quotes by doubling them: '' becomes '");
}

// ── 5. Operators and punctuation ────────────────────────────────────

#[test]
fn test_lex_operators() {
    let mut lexer = Lexer::new("+ - * / % = != < <= > >= <>");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::Plus);
    assert_eq!(tokens[1].token, Token::Minus);
    assert_eq!(tokens[2].token, Token::Star);
    assert_eq!(tokens[3].token, Token::Slash);
    assert_eq!(tokens[4].token, Token::Percent);
    assert_eq!(tokens[5].token, Token::Equal);
    assert_eq!(tokens[6].token, Token::NotEqual);
    assert_eq!(tokens[7].token, Token::LessThan);
    assert_eq!(tokens[8].token, Token::LessThanOrEqual);
    assert_eq!(tokens[9].token, Token::GreaterThan);
    assert_eq!(tokens[10].token, Token::GreaterThanOrEqual);
    assert_eq!(tokens[11].token, Token::NotEqual, "<> is the SQL-standard not-equal operator, equivalent to !=");
}

#[test]
fn test_lex_punctuation() {
    let mut lexer = Lexer::new("(a, b);");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::LeftParen);
    assert_eq!(tokens[1].token, Token::Identifier("a".to_string()));
    assert_eq!(tokens[2].token, Token::Comma);
    assert_eq!(tokens[3].token, Token::Identifier("b".to_string()));
    assert_eq!(tokens[4].token, Token::RightParen);
    assert_eq!(tokens[5].token, Token::Semicolon);
}

// ── 6. Edge cases ───────────────────────────────────────────────────

#[test]
fn test_lex_error_unterminated_string() {
    let mut lexer = Lexer::new("'unterminated");
    let result = lexer.tokenize();
    assert!(result.is_err(), "unterminated string literal must be a lexer error, not silently accepted");
}

#[test]
fn test_lex_dot_notation() {
    let mut lexer = Lexer::new("users.name");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::Identifier("users".to_string()));
    assert_eq!(tokens[1].token, Token::Dot);
    assert_eq!(tokens[2].token, Token::Identifier("name".to_string()));
}

#[test]
fn test_lex_empty_string_literal() {
    // Edge case: empty string literal
    let mut lexer = Lexer::new("''");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::StringLiteral("".to_string()), "empty string literal '' must tokenize to an empty string, not an error");
}

#[test]
fn test_lex_integer_zero() {
    // Edge case: the integer literal 0
    let tokens = tokenize("0");
    assert_eq!(tokens[0], Token::Integer(0), "integer literal 0 must be recognized");
}

// ── 7. Position tracking ────────────────────────────────────────────

#[test]
fn test_lex_position_tracking() {
    let mut lexer = Lexer::new("SELECT\nFROM");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].position.line, 1);
    assert_eq!(tokens[1].position.line, 2, "lexer must track line numbers across newlines for error reporting");
}

// ── 8. Complex queries ──────────────────────────────────────────────

#[test]
fn test_lex_complex_query() {
    let sql = "SELECT id, name FROM users WHERE age > 18 ORDER BY name ASC LIMIT 10";
    let mut lexer = Lexer::new(sql);
    let tokens = lexer.tokenize().unwrap();
    // Just verify it parses without error and produces reasonable tokens
    assert!(tokens.len() > 10);
    assert_eq!(tokens.last().unwrap().token, Token::Eof);
}

#[test]
fn test_lex_join() {
    let keywords = extract_keywords("SELECT a.id FROM a INNER JOIN b ON a.id = b.id");
    assert!(keywords.contains(&Keyword::Select));
    assert!(keywords.contains(&Keyword::From));
    assert!(keywords.contains(&Keyword::Inner));
    assert!(keywords.contains(&Keyword::Join));
    assert!(keywords.contains(&Keyword::On));
}

#[test]
fn test_lex_create_table() {
    let keywords = extract_keywords("CREATE TABLE users (id INTEGER, name VARCHAR)");
    assert!(keywords.contains(&Keyword::Create));
    assert!(keywords.contains(&Keyword::Table));
}
