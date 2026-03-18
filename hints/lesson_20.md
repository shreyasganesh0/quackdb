# Lesson 20: SQL Lexer

## What You're Building
A lexer that converts a raw SQL string into a stream of typed tokens. The Keyword enum
has 90+ variants for SQL reserved words. The Token enum represents literals, identifiers,
keywords, operators, and punctuation. The Lexer struct walks through the input character
by character, tracking line and column for error reporting. This is the first stage of
SQL compilation -- every query must be tokenized before the parser can process it.

## Concept Recap
Building on Lessons 13-19 (Part IV): In the previous 7 lessons, you built a complete execution engine that processes `DataChunk`s through operator pipelines. Now you'll build the SQL frontend that *generates* those pipelines from human-readable queries. The lexer is the first step: turning `"SELECT * FROM users"` into tokens that the parser (L21) can structure into an AST, which the planner (L22-L24) converts into the `Pipeline` and `PhysicalOperator` types you already know.

## Rust Concepts You'll Need
- [String Types](../concepts/string_types.md) -- input stored as `Vec<char>` for indexed access; identifiers produced as `String`
- [Enums and Matching](../concepts/enums_and_matching.md) -- Keyword and Token enums; match on characters to decide which token to emit
- [Error Handling](../concepts/error_handling.md) -- `Result<Vec<PositionedToken>, LexerError>` with position info for errors

## Key Patterns

### Character-by-Character Scanning
Read one character at a time. Whitespace is skipped. Multi-character tokens (identifiers, numbers, strings) are consumed in loops. Think of it like reading a sentence letter by letter and grouping letters into words -- you keep reading until you hit a space or punctuation.

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
After reading an identifier, check if it matches a known keyword with case-insensitive comparison. Think of it like a spellchecker that also detects special command words -- you read a word normally, then check it against a dictionary of reserved words.

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

### Two-Character Operator Lookahead
Some operators are two characters long (`<=`, `>=`, `!=`, `<>`). When you see `<`, you must peek at the next character to decide if it is `<`, `<=`, or `<>`. Think of it like reading road signs -- seeing "NO" you need to check if the next word is "PARKING" or "ENTRY" before you know the full meaning.

## Common Mistakes
- Treating negative numbers as a single token. The lexer should emit `Minus` then `Integer(100)` for `-100`, not `Integer(-100)`. The parser handles unary negation.
- Not handling the SQL-standard escaped quote `''` inside string literals. Two consecutive single quotes inside a string become one literal quote character.
- Forgetting to update line and column counters when advancing past newline characters. Error messages need accurate positions.

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
8. Make sure the Eof token is always the last token in the output
9. Return a `LexerError` with position info for unterminated strings or unexpected characters

## Reading the Tests
- **`test_lex_select`** tokenizes `"SELECT * FROM users"` and expects `[Keyword(Select), Star, Keyword(From), Identifier("users"), Eof]`. Note `*` is `Token::Star` (not Multiply) and `users` is an Identifier, not a Keyword. This is the basic happy-path test.
- **`test_lex_case_insensitive`** checks that `"select"`, `"FROM"`, `"Where"` all produce their Keyword tokens. Your `match_keyword` must uppercase before comparing.
- **`test_lex_integer`** tokenizes `"42 0 -100"` and checks that `-100` becomes `[Minus, Integer(100)]`, not `Integer(-100)`. This confirms the lexer does not handle unary negation.
- **`test_lex_float`** tokenizes `"3.14 0.5"` and expects Float tokens. This tests that your number reader detects the decimal point and switches to float parsing.
- **`test_lex_string`** tokenizes `"'hello world' 'it''s'"`. The second string tests SQL's escaped quote: `''` inside a string becomes a single `'`. Expects `StringLiteral("it's")`.
- **`test_lex_operators`** tokenizes all 12 operator symbols and checks each token type. Notably `!=` and `<>` both produce `NotEqual`. This is the comprehensive operator coverage test.
- **`test_lex_punctuation`** tokenizes `"(a, b);"` and checks `LeftParen, Identifier, Comma, Identifier, RightParen, Semicolon`. This validates all punctuation tokens.
- **`test_lex_complex_query`** tokenizes a full query with WHERE, ORDER BY, LIMIT and just checks it produces >10 tokens ending with Eof. This is a smoke test for realistic SQL.
- **`test_lex_join`** tokenizes a JOIN query and checks keywords include Select, From, Inner, Join, On. This confirms your keyword table includes join-related SQL words.
- **`test_lex_dot_notation`** tokenizes `"users.name"` as `[Identifier("users"), Dot, Identifier("name")]`. The dot is its own token, not part of the identifier.
- **`test_lex_error_unterminated_string`** expects `Err` for `'unterminated`. Your string reader must detect end-of-input before finding the closing quote and return an error.
- **`test_lex_position_tracking`** tokenizes `"SELECT\nFROM"` and checks that FROM is on line 2. This validates your newline tracking in `advance()`.
- **`test_lex_create_table`** checks that CREATE and TABLE are recognized as keywords. This confirms your keyword table is comprehensive.
- **`test_keyword_matching`** directly tests the `match_keyword` function with various cases. It confirms case-insensitive matching and that non-keywords return None.
