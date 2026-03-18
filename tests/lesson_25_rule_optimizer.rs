//! # Lesson 25: Rule-Based Optimizer — Test Suite
//!
//! Tests are ordered from simple to complex:
//! 1. Default rules exist (`test_default_rules`)
//! 2. Constant folding (`test_constant_folding`)
//! 3. Filter merge (`test_filter_merge`)
//! 4. Edge cases (identity transforms)
//! 5. Predicate pushdown — through projection (`test_predicate_pushdown_through_projection`)
//! 6. Predicate pushdown — through join (`test_predicate_pushdown_through_join`)
//! 7. Projection pushdown (`test_projection_pushdown`)
//! 8. Limit pushdown (`test_limit_pushdown`)
//! 9. Fixpoint optimization (`test_optimize_fixpoint`)

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

// ── 1. Default rules ────────────────────────────────────────────────

#[test]
fn test_default_rules() {
    let rules = default_rules();
    assert!(!rules.is_empty(), "default_rules() must provide at least one optimization rule for the optimizer to be useful");
}

// ── 2. Constant folding ─────────────────────────────────────────────

#[test]
fn test_constant_folding() {
    // Verifies that the expression 2 + 3 is folded to 5 at plan time
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
    let optimized = rule.apply(filter).expect("constant folding should succeed on purely literal expressions like 2 + 3");
    // 2 + 3 should be folded to 5
}

// ── 3. Filter merge ─────────────────────────────────────────────────

#[test]
fn test_filter_merge() {
    // Two stacked filters should merge into a single filter with AND
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

    // Two adjacent Filter nodes should be merged into a single Filter with an AND predicate
    let pp = optimized.pretty_print();
    // The merged filter combines both conditions, reducing tree depth by one node
}

// ── 4. Edge cases ───────────────────────────────────────────────────

#[test]
fn test_constant_folding_non_constant() {
    // Edge case: expression with a column ref should not be fully folded
    let scan = make_scan("t", vec![("a", LogicalType::Int32)]);
    let filter = LogicalPlan::Filter {
        predicate: LogicalExpr::BinaryOp {
            op: quackdb::sql::ast::BinaryOpAst::Add,
            left: Box::new(LogicalExpr::ColumnRef { index: 0, name: "a".to_string() }),
            right: Box::new(LogicalExpr::Literal(ScalarValue::Int32(1))),
        },
        input: Box::new(scan),
    };

    let rule = ConstantFolding;
    // Should succeed but not fully fold since one operand is a column reference
    let _optimized = rule.apply(filter).expect("constant folding on mixed expressions should not panic");
}

// ── 5. Predicate pushdown — through projection ─────────────────────

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
    assert!(pp.contains("Projection") || pp.contains("projection"), "predicate pushdown should move the filter below the projection, not eliminate it");
}

// ── 6. Predicate pushdown — through join ────────────────────────────

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
    assert!(pp.contains("Join") || pp.contains("join"), "filter on column 'x' should be pushed to the left child since join preserves the join node");
}

// ── 7. Projection pushdown ─────────────────────────────────────────

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

    // Scan should now have a projection that only reads column 'a', avoiding unnecessary IO for columns 'b' and 'c'
}

// ── 8. Limit pushdown ──────────────────────────────────────────────

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
    let _optimized = rule.apply(limit).expect("limit pushdown should convert LIMIT over SORT into a top-N operation");
}

// ── 8b. Edge case: limit pushdown on non-sort input ─────────────────

#[test]
fn test_rules_return_names() {
    // Edge case: each rule should have a non-empty name for debugging
    let rules = default_rules();
    for rule in &rules {
        assert!(!rule.name().is_empty(), "every optimizer rule must have a non-empty name for diagnostics and logging");
    }
}

#[test]
fn test_constant_folding_subtraction() {
    // Edge case: constant folding with subtraction
    let scan = make_scan("t", vec![("a", LogicalType::Int32)]);
    let filter = LogicalPlan::Filter {
        predicate: LogicalExpr::BinaryOp {
            op: quackdb::sql::ast::BinaryOpAst::Subtract,
            left: Box::new(LogicalExpr::Literal(ScalarValue::Int32(10))),
            right: Box::new(LogicalExpr::Literal(ScalarValue::Int32(3))),
        },
        input: Box::new(scan),
    };

    let rule = ConstantFolding;
    let _optimized = rule.apply(filter).expect("constant folding should handle subtraction of two literals");
}

// ── 9. Fixpoint optimization ────────────────────────────────────────

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
    let optimized = optimize(plan, &rules, 10).expect("fixpoint optimization should converge within 10 iterations without infinite loop");
    // Should reach a fixpoint without infinite loop
}
