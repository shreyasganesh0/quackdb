//! Lesson 20: SQL Lexer Tests

use quackdb::sql::lexer::*;

#[test]
fn test_lex_select() {
    let mut lexer = Lexer::new("SELECT * FROM users");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::Keyword(Keyword::Select));
    assert_eq!(tokens[1].token, Token::Star);
    assert_eq!(tokens[2].token, Token::Keyword(Keyword::From));
    assert_eq!(tokens[3].token, Token::Identifier("users".to_string()));
    assert_eq!(tokens[4].token, Token::Eof);
}

#[test]
fn test_lex_case_insensitive() {
    let mut lexer = Lexer::new("select FROM Where");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::Keyword(Keyword::Select));
    assert_eq!(tokens[1].token, Token::Keyword(Keyword::From));
    assert_eq!(tokens[2].token, Token::Keyword(Keyword::Where));
}

#[test]
fn test_lex_integer() {
    let mut lexer = Lexer::new("42 0 -100");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].token, Token::Integer(42));
    assert_eq!(tokens[1].token, Token::Integer(0));
    // -100 could be Minus then Integer(100)
    assert_eq!(tokens[2].token, Token::Minus);
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
    assert_eq!(tokens[1].token, Token::StringLiteral("it's".to_string()));
}

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
    assert_eq!(tokens[11].token, Token::NotEqual); // <> is also NotEqual
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
    let sql = "SELECT a.id FROM a INNER JOIN b ON a.id = b.id";
    let mut lexer = Lexer::new(sql);
    let tokens = lexer.tokenize().unwrap();
    // Check some key tokens
    let keywords: Vec<_> = tokens.iter().filter_map(|t| {
        if let Token::Keyword(k) = &t.token { Some(*k) } else { None }
    }).collect();
    assert!(keywords.contains(&Keyword::Select));
    assert!(keywords.contains(&Keyword::From));
    assert!(keywords.contains(&Keyword::Inner));
    assert!(keywords.contains(&Keyword::Join));
    assert!(keywords.contains(&Keyword::On));
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
fn test_lex_error_unterminated_string() {
    let mut lexer = Lexer::new("'unterminated");
    let result = lexer.tokenize();
    assert!(result.is_err());
}

#[test]
fn test_lex_position_tracking() {
    let mut lexer = Lexer::new("SELECT\nFROM");
    let tokens = lexer.tokenize().unwrap();
    assert_eq!(tokens[0].position.line, 1);
    assert_eq!(tokens[1].position.line, 2);
}

#[test]
fn test_lex_create_table() {
    let sql = "CREATE TABLE users (id INTEGER, name VARCHAR)";
    let mut lexer = Lexer::new(sql);
    let tokens = lexer.tokenize().unwrap();
    let keywords: Vec<_> = tokens.iter().filter_map(|t| {
        if let Token::Keyword(k) = &t.token { Some(*k) } else { None }
    }).collect();
    assert!(keywords.contains(&Keyword::Create));
    assert!(keywords.contains(&Keyword::Table));
}

#[test]
fn test_keyword_matching() {
    assert_eq!(Lexer::match_keyword("SELECT"), Some(Keyword::Select));
    assert_eq!(Lexer::match_keyword("select"), Some(Keyword::Select));
    assert_eq!(Lexer::match_keyword("Select"), Some(Keyword::Select));
    assert_eq!(Lexer::match_keyword("not_a_keyword"), None);
}
