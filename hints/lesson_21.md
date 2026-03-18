# Lesson 21: SQL Parser

## What You're Building
A recursive descent parser that transforms tokens into an Abstract Syntax Tree (AST). The AST has Statement (Select, CreateTable, Insert, Drop), Expr (recursive with Box for binary ops, unary ops, CASE, BETWEEN), and TableRef (recursive with Box for joins and subqueries). Expression parsing uses Pratt parsing with binding power to handle precedence. This parser turns flat tokens into the tree structure the database engine operates on.

**Core concept count: 2** — the AST data structure and the Pratt parsing algorithm. Everything else (helpers, statement variants, special syntax) is scaffolding that supports these two.

> **Unified Concept:** The AST is a data structure, the parser is the algorithm. Together they are ONE concept: turning tokens into trees. The AST file defines *what* the tree looks like; the parser file defines *how* to build it. You cannot understand one without the other -- they are two halves of the same idea, split for code organization.

## Concept Recap
Building on Lesson 20: You'll consume the `Token` and `Keyword` types produced by the lexer. The parser reads from a `Vec<PositionedToken>` using peek/advance helpers, just like the lexer reads characters. The AST you produce here will be consumed by the binder (L23), which converts it into the `LogicalPlan` (L22) that the physical planner (L24) turns into the `Pipeline` operators from Part IV.

## Rust Concepts You'll Need
- [Box and Recursive Types](../concepts/box_and_recursive_types.md) -- Expr contains `Box<Expr>` (BinaryOp left/right, UnaryOp expr); TableRef::Join has `Box<TableRef>` for left/right; without Box these recursive types would be infinite size
- [Enums and Matching](../concepts/enums_and_matching.md) -- Statement, Expr, TableRef, BinaryOpAst are all enums; match on Token variants to decide which production to follow
- [Error Handling](../concepts/error_handling.md) -- `Result<T, ParseError>` with line/column; propagate with `?`

## Key Patterns

### Pratt Parsing (Binding Power)
Each infix operator has a left and right binding power. Parse a prefix (atom or unary), then loop: if the next operator's left bp exceeds the minimum, consume it and recurse with the operator's right bp. Think of it like a tug-of-war between operators -- each operator "pulls" on the expressions next to it, and the one with higher binding power wins.

```rust
// Analogy: a calculator parser (NOT the QuackDB solution)
#[derive(Debug)]
enum CalcExpr {
    Num(f64),
    BinOp { op: char, left: Box<CalcExpr>, right: Box<CalcExpr> },
}

fn infix_bp(op: char) -> (u8, u8) {
    match op {
        '+' | '-' => (1, 2),   // left-associative
        '*' | '/' => (3, 4),
        _ => panic!("unknown op"),
    }
}

fn parse_calc(tokens: &[char], pos: &mut usize, min_bp: u8) -> CalcExpr {
    let mut lhs = { *pos += 1; CalcExpr::Num(0.0) }; // simplified
    loop {
        if *pos >= tokens.len() { break; }
        let op = tokens[*pos];
        let (l_bp, r_bp) = infix_bp(op);
        if l_bp < min_bp { break; }
        *pos += 1;
        let rhs = parse_calc(tokens, pos, r_bp);
        lhs = CalcExpr::BinOp { op, left: Box::new(lhs), right: Box::new(rhs) };
    }
    lhs
}
```

### Box for Recursive AST Nodes
Rust enums must have known size. Recursive fields (Expr containing Expr) must be wrapped in Box. Think of it like a Russian nesting doll -- each doll contains a smaller one, but the outer doll needs to know its own size. Box is the fixed-size container that holds the inner doll on the heap.

```rust
// Analogy: a file system tree (NOT the QuackDB solution)
enum FsNode {
    File { name: String, size: u64 },
    Dir { name: String, children: Vec<FsNode> },
    Link { name: String, target: Box<FsNode> },
}
```

### Predictive Parsing by Token Peeking
The parser peeks at the next token to decide which grammar rule to follow. This is the "recursive descent" strategy -- each grammar rule becomes a function, and the function peeks to decide whether to recurse into sub-rules. Think of it like navigating a choose-your-own-adventure book based on the first word of each page.

## Common Mistakes
- Getting operator precedence wrong in `infix_binding_power`. If `+` and `*` have the same binding power, `1 + 2 * 3` will be parsed as `(1 + 2) * 3` instead of `1 + (2 * 3)`. Use ascending power pairs: OR < AND < comparisons < add/sub < mul/div.
- Forgetting that left-associative operators need `(left_bp, left_bp + 1)` while right-associative need `(left_bp + 1, left_bp)`. Getting this backward changes how `a - b - c` is grouped.
- Not handling all prefix expressions in the Pratt parser. Unary NOT, unary minus, parenthesized expressions, function calls, CASE WHEN, and literal values are all prefix forms that must be recognized before the infix loop begins.

## Where to Start
Start with `ast.rs` — read the data structures to understand what the parser must produce. Then implement `parse_expression` using Pratt parsing (it has the clearest test). Build outward: expressions → SELECT → FROM → WHERE → JOIN.

## Step-by-Step Implementation Order
1. Implement helpers: `peek()`, `advance()`, `expect_keyword()`, `expect_token()`
2. Implement `parse_sql()` -- create Lexer, tokenize, create Parser, call `parse_statement()`
3. Implement `parse_statement()` -- peek first token; SELECT calls `parse_select()`, CREATE calls `parse_create_table()`, INSERT calls `parse_insert()`
4. Implement `parse_select()` -- parse SELECT list, optional FROM, WHERE, GROUP BY, HAVING, ORDER BY, LIMIT/OFFSET
5. Implement `parse_expression(min_bp)` with Pratt parsing -- handle prefix atoms (literals, identifiers, dot notation, NOT/negate, parenthesized exprs, functions, CASE WHEN); loop on infix using `infix_binding_power`
6. Implement `infix_binding_power()` -- OR: (1,2), AND: (3,4), comparisons: (5,6), add/sub: (7,8), mul/div/mod: (9,10)
7. Implement `parse_table_ref()` -- parse table name, check for JOIN keywords, wrap in `TableRef::Join { left: Box::new(...), right: Box::new(...) }`
8. Implement `parse_create_table()` and `parse_insert()`
9. Handle special syntax: IS NULL, BETWEEN, aliases (AS keyword or implicit)

## Reading the Tests
- **`test_parse_simple_select`** parses `"SELECT 1"` and checks for one select item and no FROM clause. This is the minimal valid SELECT statement and tests your parser's base case.
- **`test_parse_select_from`** parses `"SELECT * FROM users"`. It checks that `*` becomes `SelectItem::Wildcard` and that FROM is present. This validates wildcard handling and basic FROM parsing.
- **`test_parse_select_columns`** parses `"SELECT id, name FROM users"` and checks `select_list.len() == 2`. This confirms your parser handles comma-separated select items.
- **`test_parse_where`** parses a query with `WHERE age > 18` and checks `where_clause.is_some()`. This validates that the WHERE clause is detected and parsed.
- **`test_parse_join`** checks that `INNER JOIN` produces `TableRef::Join { join_type: JoinTypeAst::Inner, .. }`. This confirms your table ref parser detects join keywords and wraps the result in the recursive Box-containing variant.
- **`test_parse_expression_precedence`** parses `"SELECT 1 + 2 * 3"` and asserts the outer op is Add with Multiply as the right child. This is the critical test for Pratt parsing -- `*` must bind tighter than `+`.
- **`test_parse_case_when`** parses a CASE WHEN expression and checks it produces `Expr::Case`. This tests your prefix parsing of the CASE keyword.
- **`test_parse_create_table`** parses `"CREATE TABLE users (id INTEGER, name VARCHAR, active BOOLEAN)"` and checks 3 column definitions. This validates DDL parsing.
- **`test_parse_insert`** parses `"INSERT INTO users VALUES (1, 'alice'), (2, 'bob')"` and checks 2 value tuples of 2 elements each. This validates multi-row INSERT parsing.
- **`test_parse_alias`** checks that `"SELECT id AS user_id FROM users u"` captures both column alias ("user_id") and table alias ("u"). This tests AS and implicit alias handling.
- **`test_parse_function`** parses `"SELECT COUNT(DISTINCT id)"` and checks the function name and distinct flag. This validates function call parsing with modifiers.
- **`test_parse_error`** expects `"SELECT FROM"` to fail. This confirms your parser rejects malformed SQL with an appropriate error.
- **`test_parse_is_null`** and **`test_parse_between`** check that IS NULL and BETWEEN are parsed into their respective AST variants. These are special SQL syntax forms that need dedicated handling.
- **`test_parse_left_join`** checks that LEFT JOIN produces `JoinTypeAst::Left`. This validates your parser recognizes all join type keywords.
