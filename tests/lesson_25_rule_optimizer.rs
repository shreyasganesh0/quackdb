//! Lesson 25: Rule-Based Optimizer Tests

use quackdb::types::{LogicalType, ScalarValue};
use quackdb::planner::logical_plan::*;
use quackdb::optimizer::rules::*;

fn make_scan(name: &str, cols: Vec<(&str, LogicalType)>) -> LogicalPlan {
    LogicalPlan::Scan {
        table_name: name.to_string(),
        schema: Schema::new(cols.into_iter().map(|(n, t)| (n.to_string(), t)).collect()),
        projection: None,
    }
}

#[test]
fn test_predicate_pushdown_through_projection() {
    let scan = make_scan("t", vec![("a", LogicalType::Int32), ("b", LogicalType::Int32)]);
    let proj = LogicalPlan::Projection {
        expressions: vec![
            LogicalExpr::ColumnRef { index: 0, name: "a".to_string() },
        ],
        input: Box::new(scan),
    };
    let filter = LogicalPlan::Filter {
        predicate: LogicalExpr::BinaryOp {
            op: quackdb::sql::ast::BinaryOpAst::Equal,
            left: Box::new(LogicalExpr::ColumnRef { index: 0, name: "a".to_string() }),
            right: Box::new(LogicalExpr::Literal(ScalarValue::Int32(5))),
        },
        input: Box::new(proj),
    };

    let rule = PredicatePushdown;
    let optimized = rule.apply(filter).unwrap();

    // Filter should be pushed below projection
    let pp = optimized.pretty_print();
    assert!(pp.contains("Projection") || pp.contains("projection"));
}

#[test]
fn test_predicate_pushdown_through_join() {
    let left = make_scan("a", vec![("x", LogicalType::Int32)]);
    let right = make_scan("b", vec![("y", LogicalType::Int32)]);

    let join = LogicalPlan::Join {
        left: Box::new(left),
        right: Box::new(right),
        join_type: quackdb::sql::ast::JoinTypeAst::Inner,
        condition: None,
    };

    let filter = LogicalPlan::Filter {
        predicate: LogicalExpr::BinaryOp {
            op: quackdb::sql::ast::BinaryOpAst::Equal,
            left: Box::new(LogicalExpr::ColumnRef { index: 0, name: "x".to_string() }),
            right: Box::new(LogicalExpr::Literal(ScalarValue::Int32(5))),
        },
        input: Box::new(join),
    };

    let rule = PredicatePushdown;
    let optimized = rule.apply(filter).unwrap();

    // Filter on 'x' should be pushed to the left side of the join
    let pp = optimized.pretty_print();
    assert!(pp.contains("Join") || pp.contains("join"));
}

#[test]
fn test_constant_folding() {
    let scan = make_scan("t", vec![("a", LogicalType::Int32)]);
    let filter = LogicalPlan::Filter {
        predicate: LogicalExpr::BinaryOp {
            op: quackdb::sql::ast::BinaryOpAst::Add,
            left: Box::new(LogicalExpr::Literal(ScalarValue::Int32(2))),
            right: Box::new(LogicalExpr::Literal(ScalarValue::Int32(3))),
        },
        input: Box::new(scan),
    };

    let rule = ConstantFolding;
    let optimized = rule.apply(filter).unwrap();
    // 2 + 3 should be folded to 5
}

#[test]
fn test_filter_merge() {
    let scan = make_scan("t", vec![("a", LogicalType::Int32)]);
    let filter1 = LogicalPlan::Filter {
        predicate: LogicalExpr::BinaryOp {
            op: quackdb::sql::ast::BinaryOpAst::GreaterThan,
            left: Box::new(LogicalExpr::ColumnRef { index: 0, name: "a".to_string() }),
            right: Box::new(LogicalExpr::Literal(ScalarValue::Int32(5))),
        },
        input: Box::new(scan),
    };
    let filter2 = LogicalPlan::Filter {
        predicate: LogicalExpr::BinaryOp {
            op: quackdb::sql::ast::BinaryOpAst::LessThan,
            left: Box::new(LogicalExpr::ColumnRef { index: 0, name: "a".to_string() }),
            right: Box::new(LogicalExpr::Literal(ScalarValue::Int32(10))),
        },
        input: Box::new(filter1),
    };

    let rule = FilterMerge;
    let optimized = rule.apply(filter2).unwrap();

    // Two filters should be merged into one with AND
    let pp = optimized.pretty_print();
    // Should have only one filter node
}

#[test]
fn test_projection_pushdown() {
    let scan = make_scan("t", vec![
        ("a", LogicalType::Int32),
        ("b", LogicalType::Int32),
        ("c", LogicalType::Int32),
    ]);
    let proj = LogicalPlan::Projection {
        expressions: vec![
            LogicalExpr::ColumnRef { index: 0, name: "a".to_string() },
        ],
        input: Box::new(scan),
    };

    let rule = ProjectionPushdown;
    let optimized = rule.apply(proj).unwrap();

    // Scan should now have a projection that only reads column 'a'
}

#[test]
fn test_limit_pushdown() {
    let scan = make_scan("t", vec![("a", LogicalType::Int32)]);
    let sort = LogicalPlan::Sort {
        keys: vec![LogicalSortKey {
            expr: LogicalExpr::ColumnRef { index: 0, name: "a".to_string() },
            ascending: true,
            nulls_first: None,
        }],
        input: Box::new(scan),
    };
    let limit = LogicalPlan::Limit {
        count: 10,
        offset: 0,
        input: Box::new(sort),
    };

    let rule = LimitPushdown;
    let _optimized = rule.apply(limit).unwrap();
}

#[test]
fn test_optimize_fixpoint() {
    let scan = make_scan("t", vec![("a", LogicalType::Int32), ("b", LogicalType::Int32)]);
    let plan = LogicalPlan::Filter {
        predicate: LogicalExpr::BinaryOp {
            op: quackdb::sql::ast::BinaryOpAst::Equal,
            left: Box::new(LogicalExpr::ColumnRef { index: 0, name: "a".to_string() }),
            right: Box::new(LogicalExpr::Literal(ScalarValue::Int32(5))),
        },
        input: Box::new(LogicalPlan::Projection {
            expressions: vec![
                LogicalExpr::ColumnRef { index: 0, name: "a".to_string() },
            ],
            input: Box::new(scan),
        }),
    };

    let rules = default_rules();
    let optimized = optimize(plan, &rules, 10).unwrap();
    // Should reach a fixpoint without infinite loop
}

#[test]
fn test_default_rules() {
    let rules = default_rules();
    assert!(!rules.is_empty());
}
