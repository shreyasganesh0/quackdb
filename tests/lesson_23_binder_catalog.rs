//! Lesson 23: Binder & Catalog Tests

use quackdb::types::LogicalType;
use quackdb::planner::catalog::*;
use quackdb::planner::binder::*;
use quackdb::sql::parser::Parser;
use quackdb::sql::ast::Statement;

fn setup_catalog() -> Catalog {
    let mut catalog = Catalog::new();
    catalog.create_table(TableInfo {
        name: "users".to_string(),
        columns: vec![
            ColumnInfo { name: "id".to_string(), data_type: LogicalType::Int32, nullable: false, column_index: 0 },
            ColumnInfo { name: "name".to_string(), data_type: LogicalType::Varchar, nullable: true, column_index: 1 },
            ColumnInfo { name: "age".to_string(), data_type: LogicalType::Int32, nullable: true, column_index: 2 },
        ],
    }).unwrap();
    catalog.create_table(TableInfo {
        name: "orders".to_string(),
        columns: vec![
            ColumnInfo { name: "id".to_string(), data_type: LogicalType::Int32, nullable: false, column_index: 0 },
            ColumnInfo { name: "user_id".to_string(), data_type: LogicalType::Int32, nullable: false, column_index: 1 },
            ColumnInfo { name: "amount".to_string(), data_type: LogicalType::Float64, nullable: false, column_index: 2 },
        ],
    }).unwrap();
    catalog
}

#[test]
fn test_catalog_create_get() {
    let catalog = setup_catalog();
    let users = catalog.get_table("users").unwrap();
    assert_eq!(users.columns.len(), 3);
    assert_eq!(users.columns[0].name, "id");
}

#[test]
fn test_catalog_table_not_found() {
    let catalog = setup_catalog();
    assert!(catalog.get_table("nonexistent").is_none(), "catalog lookup for unknown table should return None, not panic");
}

#[test]
fn test_catalog_drop_table() {
    let mut catalog = setup_catalog();
    assert!(catalog.get_table("users").is_some());
    catalog.drop_table("users").unwrap();
    assert!(catalog.get_table("users").is_none(), "dropped table should no longer be visible in the catalog");
}

#[test]
fn test_catalog_duplicate_create() {
    let mut catalog = setup_catalog();
    let result = catalog.create_table(TableInfo {
        name: "users".to_string(),
        columns: vec![],
    });
    assert!(result.is_err(), "catalog should reject duplicate table creation to maintain naming uniqueness");
}

#[test]
fn test_bind_simple_select() {
    let catalog = setup_catalog();
    let binder = Binder::new(&catalog);
    let stmt = Parser::parse_sql("SELECT id, name FROM users").unwrap();
    let plan = binder.bind(&stmt).unwrap();
    let schema = plan.schema();
    assert_eq!(schema.column_count(), 2);
}

#[test]
fn test_bind_select_star() {
    let catalog = setup_catalog();
    let binder = Binder::new(&catalog);
    let stmt = Parser::parse_sql("SELECT * FROM users").unwrap();
    let plan = binder.bind(&stmt).unwrap();
    let schema = plan.schema();
    assert_eq!(schema.column_count(), 3, "SELECT * should expand to all columns from the table schema");
}

#[test]
fn test_bind_unknown_table() {
    let catalog = setup_catalog();
    let binder = Binder::new(&catalog);
    let stmt = Parser::parse_sql("SELECT * FROM nonexistent").unwrap();
    let result = binder.bind(&stmt);
    assert!(result.is_err(), "binder should reject references to tables not in the catalog");
}

#[test]
fn test_bind_unknown_column() {
    let catalog = setup_catalog();
    let binder = Binder::new(&catalog);
    let stmt = Parser::parse_sql("SELECT nonexistent FROM users").unwrap();
    let result = binder.bind(&stmt);
    assert!(result.is_err(), "binder should reject references to columns not in the table schema");
}

#[test]
fn test_bind_where_clause() {
    let catalog = setup_catalog();
    let binder = Binder::new(&catalog);
    let stmt = Parser::parse_sql("SELECT * FROM users WHERE age > 18").unwrap();
    let plan = binder.bind(&stmt).unwrap();
    // Plan should have a Filter node
    let pp = plan.pretty_print();
    assert!(pp.contains("Filter") || pp.contains("filter"));
}

#[test]
fn test_bind_join() {
    let catalog = setup_catalog();
    let binder = Binder::new(&catalog);
    let stmt = Parser::parse_sql(
        "SELECT users.name, orders.amount FROM users INNER JOIN orders ON users.id = orders.user_id"
    ).unwrap();
    let plan = binder.bind(&stmt).unwrap();
    let schema = plan.schema();
    assert_eq!(schema.column_count(), 2);
}

#[test]
fn test_bind_alias() {
    let catalog = setup_catalog();
    let binder = Binder::new(&catalog);
    let stmt = Parser::parse_sql("SELECT id AS user_id FROM users").unwrap();
    let plan = binder.bind(&stmt).unwrap();
    // Should succeed without error
}

#[test]
fn test_bind_aggregate() {
    let catalog = setup_catalog();
    let binder = Binder::new(&catalog);
    let stmt = Parser::parse_sql(
        "SELECT age, COUNT(*) FROM users GROUP BY age"
    ).unwrap();
    let plan = binder.bind(&stmt).unwrap();
    let schema = plan.schema();
    assert_eq!(schema.column_count(), 2);
}

#[test]
fn test_bind_scope_resolution() {
    let scope = BindScope {
        columns: vec![
            (Some("users".to_string()), "id".to_string(), LogicalType::Int32, 0),
            (Some("users".to_string()), "name".to_string(), LogicalType::Varchar, 1),
            (Some("orders".to_string()), "id".to_string(), LogicalType::Int32, 2),
        ],
    };

    // Qualified reference should work
    let result = scope.resolve(Some("users"), "id");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().0, 0);

    // Ambiguous unqualified reference should error
    let result = scope.resolve(None, "id");
    assert!(result.is_err(), "ambiguous column reference must error: 'id' exists in both 'users' and 'orders'");

    // Unambiguous unqualified reference
    let result = scope.resolve(None, "name");
    assert!(result.is_ok());
}

#[test]
fn test_table_info_helpers() {
    let catalog = setup_catalog();
    let users = catalog.get_table("users").unwrap();
    assert!(users.find_column("id").is_some());
    assert!(users.find_column("xyz").is_none());
    assert_eq!(users.schema_types().len(), 3);
}
