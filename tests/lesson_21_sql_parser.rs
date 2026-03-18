//! Lesson 21: SQL Parser Tests

use quackdb::sql::parser::Parser;
use quackdb::sql::ast::*;

#[test]
fn test_parse_simple_select() {
    let stmt = Parser::parse_sql("SELECT 1").unwrap();
    if let Statement::Select(s) = stmt {
        assert_eq!(s.select_list.len(), 1);
        assert!(s.from.is_none(), "SELECT without FROM should have no table reference");
    } else {
        panic!("Expected SELECT statement");
    }
}

#[test]
fn test_parse_select_from() {
    let stmt = Parser::parse_sql("SELECT * FROM users").unwrap();
    if let Statement::Select(s) = stmt {
        assert!(matches!(s.select_list[0], SelectItem::Wildcard), "* should parse as Wildcard, not an expression");
        assert!(s.from.is_some());
    } else {
        panic!("Expected SELECT statement");
    }
}

#[test]
fn test_parse_select_columns() {
    let stmt = Parser::parse_sql("SELECT id, name FROM users").unwrap();
    if let Statement::Select(s) = stmt {
        assert_eq!(s.select_list.len(), 2);
    } else {
        panic!("Expected SELECT statement");
    }
}

#[test]
fn test_parse_where() {
    let stmt = Parser::parse_sql("SELECT * FROM users WHERE age > 18").unwrap();
    if let Statement::Select(s) = stmt {
        assert!(s.where_clause.is_some());
    } else {
        panic!("Expected SELECT statement");
    }
}

#[test]
fn test_parse_join() {
    let stmt = Parser::parse_sql(
        "SELECT * FROM users INNER JOIN orders ON users.id = orders.user_id"
    ).unwrap();
    if let Statement::Select(s) = stmt {
        if let Some(TableRef::Join { join_type, .. }) = s.from {
            assert_eq!(join_type, JoinTypeAst::Inner);
        } else {
            panic!("Expected JOIN");
        }
    }
}

#[test]
fn test_parse_group_by() {
    let stmt = Parser::parse_sql(
        "SELECT department, COUNT(*) FROM employees GROUP BY department"
    ).unwrap();
    if let Statement::Select(s) = stmt {
        assert_eq!(s.group_by.len(), 1);
    } else {
        panic!("Expected SELECT");
    }
}

#[test]
fn test_parse_order_by() {
    let stmt = Parser::parse_sql(
        "SELECT * FROM users ORDER BY name ASC, age DESC"
    ).unwrap();
    if let Statement::Select(s) = stmt {
        assert_eq!(s.order_by.len(), 2);
        assert!(s.order_by[0].ascending);
        assert!(!s.order_by[1].ascending);
    } else {
        panic!("Expected SELECT");
    }
}

#[test]
fn test_parse_limit() {
    let stmt = Parser::parse_sql("SELECT * FROM users LIMIT 10").unwrap();
    if let Statement::Select(s) = stmt {
        assert!(s.limit.is_some());
    } else {
        panic!("Expected SELECT");
    }
}

#[test]
fn test_parse_expression_precedence() {
    let stmt = Parser::parse_sql("SELECT 1 + 2 * 3").unwrap();
    if let Statement::Select(s) = stmt {
        if let SelectItem::Expression { expr, .. } = &s.select_list[0] {
            // Should parse as 1 + (2 * 3), not (1 + 2) * 3
            if let Expr::BinaryOp { op, left, right } = expr {
                assert_eq!(*op, BinaryOpAst::Add, "top-level op should be Add since * binds tighter than +");
                // Right should be 2 * 3
                assert!(matches!(right.as_ref(), Expr::BinaryOp { op: BinaryOpAst::Multiply, .. }), "parser must respect operator precedence: 1 + (2 * 3), not (1 + 2) * 3");
            } else {
                panic!("Expected binary op");
            }
        }
    }
}

#[test]
fn test_parse_case_when() {
    let sql = "SELECT CASE WHEN x > 0 THEN 'positive' ELSE 'non-positive' END FROM t";
    let stmt = Parser::parse_sql(sql).unwrap();
    if let Statement::Select(s) = stmt {
        if let SelectItem::Expression { expr, .. } = &s.select_list[0] {
            assert!(matches!(expr, Expr::Case { .. }));
        }
    }
}

#[test]
fn test_parse_create_table() {
    let sql = "CREATE TABLE users (id INTEGER, name VARCHAR, active BOOLEAN)";
    let stmt = Parser::parse_sql(sql).unwrap();
    if let Statement::CreateTable(ct) = stmt {
        assert_eq!(ct.table_name, "users");
        assert_eq!(ct.columns.len(), 3, "parser should extract all column definitions from the CREATE TABLE statement");
        assert_eq!(ct.columns[0].name, "id");
        assert_eq!(ct.columns[1].name, "name");
    } else {
        panic!("Expected CREATE TABLE");
    }
}

#[test]
fn test_parse_insert() {
    let sql = "INSERT INTO users VALUES (1, 'alice'), (2, 'bob')";
    let stmt = Parser::parse_sql(sql).unwrap();
    if let Statement::Insert(ins) = stmt {
        assert_eq!(ins.table_name, "users");
        assert_eq!(ins.values.len(), 2, "INSERT should parse multiple value tuples separated by commas");
        assert_eq!(ins.values[0].len(), 2);
    } else {
        panic!("Expected INSERT");
    }
}

#[test]
fn test_parse_alias() {
    let stmt = Parser::parse_sql("SELECT id AS user_id FROM users u").unwrap();
    if let Statement::Select(s) = stmt {
        if let SelectItem::Expression { alias, .. } = &s.select_list[0] {
            assert_eq!(alias.as_deref(), Some("user_id"), "AS keyword should bind the alias to the expression");
        }
        if let Some(TableRef::Table { alias, .. }) = &s.from {
            assert_eq!(alias.as_deref(), Some("u"));
        }
    }
}

#[test]
fn test_parse_function() {
    let stmt = Parser::parse_sql("SELECT COUNT(DISTINCT id) FROM users").unwrap();
    if let Statement::Select(s) = stmt {
        if let SelectItem::Expression { expr, .. } = &s.select_list[0] {
            if let Expr::Function { name, distinct, .. } = expr {
                assert_eq!(name.to_uppercase(), "COUNT");
                assert!(*distinct, "DISTINCT modifier inside a function call must be captured by the parser");
            } else {
                panic!("Expected function");
            }
        }
    }
}

#[test]
fn test_parse_error() {
    let result = Parser::parse_sql("SELECT FROM");
    assert!(result.is_err(), "SELECT without expressions before FROM is a syntax error");
}

#[test]
fn test_parse_nested_expressions() {
    let sql = "SELECT (a + b) * (c - d) FROM t";
    let stmt = Parser::parse_sql(sql).unwrap();
    if let Statement::Select(s) = stmt {
        assert_eq!(s.select_list.len(), 1);
    }
}

#[test]
fn test_parse_is_null() {
    let stmt = Parser::parse_sql("SELECT * FROM t WHERE x IS NULL").unwrap();
    if let Statement::Select(s) = stmt {
        if let Some(Expr::IsNull { negated, .. }) = s.where_clause {
            assert!(!negated);
        } else {
            panic!("Expected IS NULL");
        }
    }
}

#[test]
fn test_parse_between() {
    let stmt = Parser::parse_sql("SELECT * FROM t WHERE x BETWEEN 1 AND 10").unwrap();
    if let Statement::Select(s) = stmt {
        assert!(matches!(s.where_clause, Some(Expr::Between { .. })));
    }
}

#[test]
fn test_parse_left_join() {
    let stmt = Parser::parse_sql(
        "SELECT * FROM a LEFT JOIN b ON a.id = b.id"
    ).unwrap();
    if let Statement::Select(s) = stmt {
        if let Some(TableRef::Join { join_type, .. }) = s.from {
            assert_eq!(join_type, JoinTypeAst::Left);
        }
    }
}
