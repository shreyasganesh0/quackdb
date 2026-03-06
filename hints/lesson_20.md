# Lesson 20: SQL Lexer

## What You're Building
A lexer that converts a raw SQL string into a stream of typed tokens. The Keyword enum has 90+ variants for SQL reserved words. The Token enum represents literals, identifiers, keywords, operators, and punctuation. The Lexer struct walks through the input character by character, tracking line and column for error reporting. This is the first stage of SQL compilation -- every query must be tokenized before the parser can process it.

## Rust Concepts You'll Need
- [String Types](../concepts/string_types.md) -- input stored as `Vec<char>` for indexed access; identifiers produced as `String`
- [Enums and Matching](../concepts/enums_and_matching.md) -- Keyword and Token enums; match on characters to decide which token to emit
- [Error Handling](../concepts/error_handling.md) -- `Result<Vec<PositionedToken>, LexerError>` with position info for errors

## Key Patterns

### Character-by-Character Scanning
Read one character at a time. Whitespace is skipped. Multi-character tokens (identifiers, numbers, strings) are consumed in loops.

```rust
// Analogy: a config file tokenizer (NOT the QuackDB solution)
struct Scanner { chars: Vec<char>, pos: usize }

impl Scanner {
    fn peek(&self) -> Option<char> { self.chars.get(self.pos).copied() }
    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied();
        self.pos += 1;
        ch
    }
    fn read_word(&mut self) -> String {
        let start = self.pos;
        while self.peek().map_or(false, |c| c.is_alphanumeric() || c == '_') {
            self.advance();
        }
        self.chars[start..self.pos].iter().collect()
    }
}
```

### Keyword Lookup from String
After reading an identifier, check if it matches a known keyword with case-insensitive comparison.

```rust
// Analogy: a command dispatcher (NOT the QuackDB solution)
enum Command { Help, Quit, Run }

fn match_command(word: &str) -> Option<Command> {
    match word.to_uppercase().as_str() {
        "HELP" => Some(Command::Help),
        "QUIT" | "EXIT" => Some(Command::Quit),
        "RUN" => Some(Command::Run),
        _ => None,
    }
}
```

## Step-by-Step Implementation Order
1. Start with `Lexer::new()` -- convert input `&str` to `Vec<char>`, initialize position to 0, line to 1, column to 1
2. Implement helpers: `peek()`, `advance()` (update line/column on newlines), `skip_whitespace()`
3. Implement `match_keyword()` -- uppercase the string, match against all Keyword variants
4. Implement `next_token()` -- skip whitespace, then match on current character:
   - Digit: read number (integer or float if dot appears)
   - Single quote: read string literal (handle escaped `''`)
   - Letter/underscore: read identifier, check match_keyword
   - Operators: handle single-char (+, -, *, /) and two-char (<=, >=, !=, <>)
   - Punctuation: (, ), comma, semicolon, dot
   - End of input: emit Eof
5. Implement `tokenize()` -- loop calling `next_token()` until Eof
6. Implement `Display for Token`
7. Watch out for two-character operators: `<=`, `>=`, `!=`, `<>` require peeking ahead

## Reading the Tests
- **`test_lex_select`** tokenizes `"SELECT * FROM users"` and expects `[Keyword(Select), Star, Keyword(From), Identifier("users"), Eof]`. Note `*` is `Token::Star` and `users` is an Identifier, not a Keyword.
- **`test_lex_case_insensitive`** checks that `"select"`, `"FROM"`, `"Where"` all produce their Keyword tokens. Your `match_keyword` must do case-insensitive comparison.
- **`test_lex_error_unterminated_string`** expects `Err` for `'unterminated`. Your string reader must detect end-of-input before finding the closing quote.
