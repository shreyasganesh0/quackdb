//! Lesson 22: Logical Query Plan Tests

use quackdb::types::LogicalType;
use quackdb::planner::logical_plan::*;
use quackdb::sql::ast::*;

#[test]
fn test_schema() {
    let schema = Schema::new(vec![
        ("id".to_string(), LogicalType::Int32),
        ("name".to_string(), LogicalType::Varchar),
    ]);
    assert_eq!(schema.column_count(), 2);
    assert_eq!(schema.find_column("id"), Some(0), "find_column should return the index of a known column");
    assert_eq!(schema.find_column("name"), Some(1));
    assert_eq!(schema.find_column("age"), None, "find_column should return None for columns not in the schema");
    assert_eq!(schema.types(), vec![LogicalType::Int32, LogicalType::Varchar]);
}

#[test]
fn test_schema_merge() {
    let s1 = Schema::new(vec![("a".to_string(), LogicalType::Int32)]);
    let s2 = Schema::new(vec![("b".to_string(), LogicalType::Float64)]);
    let merged = s1.merge(&s2);
    assert_eq!(merged.column_count(), 2, "schema merge concatenates columns from both schemas, as needed for joins");
    assert_eq!(merged.columns[0].0, "a");
    assert_eq!(merged.columns[1].0, "b");
}

#[test]
fn test_scan_plan_schema() {
    let schema = Schema::new(vec![
        ("id".to_string(), LogicalType::Int32),
        ("name".to_string(), LogicalType::Varchar),
    ]);
    let plan = LogicalPlan::Scan {
        table_name: "users".to_string(),
        schema: schema.clone(),
        projection: None,
    };
    let output = plan.schema();
    assert_eq!(output.column_count(), 2, "scan without projection exposes all table columns");
}

#[test]
fn test_scan_with_projection() {
    let schema = Schema::new(vec![
        ("id".to_string(), LogicalType::Int32),
        ("name".to_string(), LogicalType::Varchar),
        ("age".to_string(), LogicalType::Int32),
    ]);
    let plan = LogicalPlan::Scan {
        table_name: "users".to_string(),
        schema,
        projection: Some(vec![0, 2]),
    };
    let output = plan.schema();
    assert_eq!(output.column_count(), 2, "projection pushdown in scan reduces columns read from storage");
}

#[test]
fn test_filter_plan_schema() {
    let inner = LogicalPlan::Scan {
        table_name: "t".to_string(),
        schema: Schema::new(vec![("x".to_string(), LogicalType::Int32)]),
        projection: None,
    };
    let plan = LogicalPlan::Filter {
        predicate: LogicalExpr::Literal(quackdb::types::ScalarValue::Boolean(true)),
        input: Box::new(inner),
    };
    let output = plan.schema();
    assert_eq!(output.column_count(), 1, "filter does not change the schema; it only removes rows, not columns");
}

#[test]
fn test_join_plan_schema() {
    let left = LogicalPlan::Scan {
        table_name: "a".to_string(),
        schema: Schema::new(vec![("x".to_string(), LogicalType::Int32)]),
        projection: None,
    };
    let right = LogicalPlan::Scan {
        table_name: "b".to_string(),
        schema: Schema::new(vec![("y".to_string(), LogicalType::Float64)]),
        projection: None,
    };
    let plan = LogicalPlan::Join {
        left: Box::new(left),
        right: Box::new(right),
        join_type: JoinTypeAst::Inner,
        condition: None,
    };
    let output = plan.schema();
    assert_eq!(output.column_count(), 2, "join schema is the concatenation of left and right schemas");
}

#[test]
fn test_limit_plan_schema() {
    let inner = LogicalPlan::Scan {
        table_name: "t".to_string(),
        schema: Schema::new(vec![("a".to_string(), LogicalType::Int32)]),
        projection: None,
    };
    let plan = LogicalPlan::Limit {
        count: 10,
        offset: 0,
        input: Box::new(inner),
    };
    let output = plan.schema();
    assert_eq!(output.column_count(), 1);
}

#[test]
fn test_pretty_print() {
    let plan = LogicalPlan::Filter {
        predicate: LogicalExpr::Literal(quackdb::types::ScalarValue::Boolean(true)),
        input: Box::new(LogicalPlan::Scan {
            table_name: "users".to_string(),
            schema: Schema::new(vec![("id".to_string(), LogicalType::Int32)]),
            projection: None,
        }),
    };
    let output = plan.pretty_print();
    assert!(!output.is_empty());
    assert!(output.contains("Filter") || output.contains("filter"), "pretty print should show operator names for debugging query plans");
    assert!(output.contains("Scan") || output.contains("scan") || output.contains("users"));
}

#[test]
fn test_aggregate_plan_schema() {
    let inner = LogicalPlan::Scan {
        table_name: "t".to_string(),
        schema: Schema::new(vec![
            ("dept".to_string(), LogicalType::Varchar),
            ("salary".to_string(), LogicalType::Float64),
        ]),
        projection: None,
    };
    let plan = LogicalPlan::Aggregate {
        group_exprs: vec![LogicalExpr::ColumnRef { index: 0, name: "dept".to_string() }],
        agg_exprs: vec![LogicalExpr::AggregateFunction {
            func: "SUM".to_string(),
            args: vec![LogicalExpr::ColumnRef { index: 1, name: "salary".to_string() }],
            distinct: false,
        }],
        input: Box::new(inner),
    };
    let output = plan.schema();
    assert_eq!(output.column_count(), 2);
}

#[test]
fn test_display() {
    let plan = LogicalPlan::Scan {
        table_name: "test".to_string(),
        schema: Schema::new(vec![("id".to_string(), LogicalType::Int32)]),
        projection: None,
    };
    let display = format!("{}", plan);
    assert!(!display.is_empty());
}
