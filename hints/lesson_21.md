# Lesson 21: SQL Parser

## What You're Building
A recursive descent parser that transforms tokens into an Abstract Syntax Tree (AST). The AST has Statement (Select, CreateTable, Insert, Drop), Expr (recursive with Box for binary ops, unary ops, CASE, BETWEEN), and TableRef (recursive with Box for joins and subqueries). Expression parsing uses Pratt parsing with binding power to handle precedence. This parser turns flat tokens into the tree structure the database engine operates on.

## Rust Concepts You'll Need
- [Box and Recursive Types](../concepts/box_and_recursive_types.md) -- Expr contains `Box<Expr>` (BinaryOp left/right, UnaryOp expr); TableRef::Join has `Box<TableRef>` for left/right; without Box these recursive types would be infinite size
- [Enums and Matching](../concepts/enums_and_matching.md) -- Statement, Expr, TableRef, BinaryOpAst are all enums; match on Token variants to decide which production to follow
- [Error Handling](../concepts/error_handling.md) -- `Result<T, ParseError>` with line/column; propagate with `?`

## Key Patterns

### Pratt Parsing (Binding Power)
Each infix operator has a left and right binding power. Parse a prefix (atom or unary), then loop: if the next operator's left bp exceeds the minimum, consume it and recurse with the operator's right bp.

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
Rust enums must have known size. Recursive fields (Expr containing Expr) must be wrapped in Box.

```rust
// Analogy: a file system tree (NOT the QuackDB solution)
enum FsNode {
    File { name: String, size: u64 },
    Dir { name: String, children: Vec<FsNode> },
    Link { name: String, target: Box<FsNode> },
}
```

## Step-by-Step Implementation Order
1. Implement helpers: `peek()`, `advance()`, `expect_keyword()`, `expect_token()`
2. Implement `parse_sql()` -- create Lexer, tokenize, create Parser, call `parse_statement()`
3. Implement `parse_statement()` -- peek first token; SELECT calls `parse_select()`, CREATE calls `parse_create_table()`, INSERT calls `parse_insert()`
4. Implement `parse_select()` -- parse SELECT list, optional FROM, WHERE, GROUP BY, HAVING, ORDER BY, LIMIT/OFFSET
5. Implement `parse_expression(min_bp)` with Pratt parsing -- handle prefix atoms (literals, identifiers, dot notation, NOT/negate, parenthesized exprs, functions, CASE WHEN); loop on infix using `infix_binding_power`
6. Implement `infix_binding_power()` -- OR: (1,2), AND: (3,4), comparisons: (5,6), add/sub: (7,8), mul/div/mod: (9,10)
7. Implement `parse_table_ref()` -- parse table name, check for JOIN keywords, wrap in `TableRef::Join { left: Box::new(...), right: Box::new(...) }`
8. Implement `parse_create_table()` and `parse_insert()`
9. Watch out: prefix binding power (unary NOT, negate) differs from infix binding power

## Reading the Tests
- **`test_parse_expression_precedence`** parses `"SELECT 1 + 2 * 3"` and asserts the outer op is Add with Multiply as the right child. This validates that your binding power gives `*` higher precedence than `+`.
- **`test_parse_join`** checks that INNER JOIN produces `TableRef::Join { join_type: JoinTypeAst::Inner, .. }`. This confirms your table ref parser detects join keywords and wraps the result in the recursive Box-containing variant.
