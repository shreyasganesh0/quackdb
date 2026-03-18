//! # Lesson 24: Physical Plan & End-to-End Execution — Test Suite
//!
//! Tests are ordered from simple to complex:
//! 1. Database construction (`test_database_default`)
//! 2. CREATE TABLE + INSERT (`test_e2e_create_and_insert`)
//! 3. Simple SELECT (`test_e2e_select_all`, `test_e2e_select_columns`)
//! 4. Edge cases (empty table, expression in SELECT)
//! 5. WHERE clause (`test_e2e_where`)
//! 6. ORDER BY (`test_e2e_order_by`)
//! 7. LIMIT (`test_e2e_limit`)
//! 8. GROUP BY (`test_e2e_group_by`)
//! 9. JOIN — full integration (`test_e2e_join`)

use quackdb::types::{LogicalType, ScalarValue};
use quackdb::chunk::DataChunk;
use quackdb::db::Database;

fn setup_db() -> Database {
    let mut db = Database::new();
    db.execute_sql("CREATE TABLE users (id INTEGER, name VARCHAR, age INTEGER)").unwrap();
    db.execute_sql("INSERT INTO users VALUES (1, 'alice', 30)").unwrap();
    db.execute_sql("INSERT INTO users VALUES (2, 'bob', 25)").unwrap();
    db.execute_sql("INSERT INTO users VALUES (3, 'charlie', 35)").unwrap();
    db.execute_sql("INSERT INTO users VALUES (4, 'dave', 28)").unwrap();
    db
}

// ── 1. Database construction ────────────────────────────────────────

#[test]
fn test_database_default() {
    let db = Database::default();
    assert!(db.catalog().table_names().is_empty());
}

// ── 2. CREATE TABLE + INSERT ────────────────────────────────────────

#[test]
fn test_e2e_create_and_insert() {
    let mut db = Database::new();
    db.execute_sql("CREATE TABLE test (x INTEGER)").unwrap();
    db.execute_sql("INSERT INTO test VALUES (42)").unwrap();

    let results = db.execute_sql("SELECT x FROM test").unwrap();
    assert_eq!(results[0].column(0).get_value(0), ScalarValue::Int32(42), "CREATE TABLE + INSERT + SELECT round-trip should preserve inserted values");
}

// ── 3. Simple SELECT ────────────────────────────────────────────────

#[test]
fn test_e2e_select_all() {
    let db = setup_db();
    let results = db.execute_sql("SELECT * FROM users").unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 4, "SELECT * should return all rows inserted into the table");
}

#[test]
fn test_e2e_select_columns() {
    let db = setup_db();
    let results = db.execute_sql("SELECT name, age FROM users").unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 4);
    assert_eq!(results[0].column_count(), 2, "selecting specific columns should reduce output width to only those columns");
}

// ── 4. Edge cases ───────────────────────────────────────────────────

#[test]
fn test_e2e_empty_table() {
    // Edge case: SELECT from a table with no rows
    let mut db = Database::new();
    db.execute_sql("CREATE TABLE empty_t (x INTEGER)").unwrap();
    let results = db.execute_sql("SELECT * FROM empty_t").unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 0, "SELECT from an empty table must return zero rows");
}

#[test]
fn test_e2e_expression_in_select() {
    let db = setup_db();
    let results = db.execute_sql("SELECT age * 2 FROM users WHERE id = 1").unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 1);
    // age=30, so age*2=60
    assert_eq!(results[0].column(0).get_value(0), ScalarValue::Int32(60), "expressions in SELECT list are evaluated during projection");
}

// ── 5. WHERE clause ─────────────────────────────────────────────────

#[test]
fn test_e2e_where() {
    let db = setup_db();
    let results = db.execute_sql("SELECT * FROM users WHERE age > 28").unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 2, "WHERE clause filters at execution time: only alice (30) and charlie (35) have age > 28");
}

// ── 6. ORDER BY ─────────────────────────────────────────────────────

#[test]
fn test_e2e_order_by() {
    let db = setup_db();
    let results = db.execute_sql("SELECT name FROM users ORDER BY age ASC").unwrap();
    let chunk = &results[0];
    assert_eq!(chunk.column(0).get_value(0), ScalarValue::Varchar("bob".into()), "ORDER BY ASC should place the youngest (bob, 25) first");
}

// ── 7. LIMIT ────────────────────────────────────────────────────────

#[test]
fn test_e2e_limit() {
    let db = setup_db();
    let results = db.execute_sql("SELECT * FROM users LIMIT 2").unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 2, "LIMIT should cap the number of returned rows regardless of how many match");
}

// ── 8. GROUP BY ─────────────────────────────────────────────────────

#[test]
fn test_e2e_group_by() {
    let mut db = Database::new();
    db.execute_sql("CREATE TABLE sales (product VARCHAR, amount INTEGER)").unwrap();
    db.execute_sql("INSERT INTO sales VALUES ('a', 10)").unwrap();
    db.execute_sql("INSERT INTO sales VALUES ('b', 20)").unwrap();
    db.execute_sql("INSERT INTO sales VALUES ('a', 30)").unwrap();
    db.execute_sql("INSERT INTO sales VALUES ('b', 40)").unwrap();

    let results = db.execute_sql("SELECT product, SUM(amount) FROM sales GROUP BY product").unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 2, "GROUP BY should collapse rows into one per distinct group key");
}

// ── 9. JOIN — full integration ──────────────────────────────────────

#[test]
fn test_e2e_join() {
    let mut db = Database::new();
    db.execute_sql("CREATE TABLE t1 (id INTEGER, val VARCHAR)").unwrap();
    db.execute_sql("CREATE TABLE t2 (id INTEGER, score INTEGER)").unwrap();
    db.execute_sql("INSERT INTO t1 VALUES (1, 'a')").unwrap();
    db.execute_sql("INSERT INTO t1 VALUES (2, 'b')").unwrap();
    db.execute_sql("INSERT INTO t2 VALUES (1, 100)").unwrap();
    db.execute_sql("INSERT INTO t2 VALUES (3, 300)").unwrap();

    let results = db.execute_sql(
        "SELECT t1.val, t2.score FROM t1 INNER JOIN t2 ON t1.id = t2.id"
    ).unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 1, "end-to-end INNER JOIN: SQL text is parsed, bound, planned, and executed to produce correct join results");
}
