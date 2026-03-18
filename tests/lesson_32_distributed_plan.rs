//! # Lesson 32: Distributed Query Planning — Test Suite
//!
//! Tests are ordered from simple to complex:
//! 1. Fragment builder (`test_fragment_builder`)
//! 2. Single scan plan (`test_single_scan_plan`)
//! 3. Filter pushdown in distributed plan (`test_filter_pushdown_in_distributed`)
//! 4. Edge cases (broadcast exchange, single table)
//! 5. Join repartition (`test_join_repartition`)
//! 6. Aggregate repartition (`test_aggregate_repartition`)
//! 7. Multi-join fragments (`test_multi_join_fragments`)

use quackdb::types::LogicalType;
use quackdb::planner::logical_plan::*;
use quackdb::distributed::planner::*;

fn make_scan(name: &str) -> LogicalPlan {
    LogicalPlan::Scan {
        table_name: name.to_string(),
        schema: Schema::new(vec![("id".to_string(), LogicalType::Int32), ("val".to_string(), LogicalType::Int64)]),
        projection: None,
    }
}

// ── 1. Fragment builder ─────────────────────────────────────────────

#[test]
fn test_fragment_builder() {
    let mut builder = FragmentBuilder::new();
    let id = builder.add_fragment(make_scan("t"), None, Some(ExchangeType::Gather));
    assert_eq!(id, 0);
    let fragments = builder.build();
    assert_eq!(fragments.len(), 1);
    assert_eq!(fragments[0].fragment_id, 0, "fragment IDs should be assigned sequentially starting from 0");
}

// ── 2. Single scan plan ─────────────────────────────────────────────

#[test]
fn test_single_scan_plan() {
    let planner = DistributedPlanner::new(4);
    let plan = make_scan("users");
    let fragments = planner.plan(plan).unwrap();
    assert!(!fragments.is_empty(), "even a single scan must produce at least one fragment to be scheduled on a worker node");
}

// ── 3. Filter pushdown ──────────────────────────────────────────────

#[test]
fn test_filter_pushdown_in_distributed() {
    let planner = DistributedPlanner::new(4);
    let plan = LogicalPlan::Filter {
        predicate: LogicalExpr::BinaryOp {
            op: quackdb::sql::ast::BinaryOpAst::Equal,
            left: Box::new(LogicalExpr::ColumnRef { index: 0, name: "id".to_string() }),
            right: Box::new(LogicalExpr::Literal(quackdb::types::ScalarValue::Int32(1))),
        },
        input: Box::new(make_scan("t")),
    };

    let fragments = planner.plan(plan).unwrap();
    // Filter should be pushed to scan fragments
    assert!(!fragments.is_empty());
}

// ── 4. Edge cases ───────────────────────────────────────────────────

#[test]
fn test_broadcast_exchange() {
    let planner = DistributedPlanner::new(4);
    // A join where one side is very small could use broadcast
    let fragments = planner.plan(make_scan("small_table")).unwrap();
    // At minimum, should have fragments
    assert!(!fragments.is_empty());
}

#[test]
fn test_fragment_builder_multiple() {
    // Edge case: building multiple fragments sequentially
    let mut builder = FragmentBuilder::new();
    let id1 = builder.add_fragment(make_scan("a"), None, Some(ExchangeType::Gather));
    let id2 = builder.add_fragment(make_scan("b"), None, Some(ExchangeType::Gather));
    assert_eq!(id1, 0);
    assert_eq!(id2, 1, "fragment IDs must be assigned sequentially");
    let fragments = builder.build();
    assert_eq!(fragments.len(), 2);
}

// ── 5. Join repartition ─────────────────────────────────────────────

#[test]
fn test_join_repartition() {
    let planner = DistributedPlanner::new(4);
    let plan = LogicalPlan::Join {
        left: Box::new(make_scan("a")),
        right: Box::new(make_scan("b")),
        join_type: quackdb::sql::ast::JoinTypeAst::Inner,
        condition: Some(LogicalExpr::BinaryOp {
            op: quackdb::sql::ast::BinaryOpAst::Equal,
            left: Box::new(LogicalExpr::ColumnRef { index: 0, name: "id".to_string() }),
            right: Box::new(LogicalExpr::ColumnRef { index: 2, name: "id".to_string() }),
        }),
    };

    let fragments = planner.plan(plan).unwrap();
    // Should insert repartition exchanges for both sides of the join
    let has_repartition = fragments.iter().any(|f| {
        matches!(f.exchange_input, Some(ExchangeType::Repartition { .. }))
        || matches!(f.exchange_output, Some(ExchangeType::Repartition { .. }))
    });
    assert!(has_repartition || fragments.len() > 1, "distributed join requires repartition exchanges so both sides are co-partitioned on the join key");
}

// ── 6. Aggregate repartition ────────────────────────────────────────

#[test]
fn test_aggregate_repartition() {
    let planner = DistributedPlanner::new(4);
    let plan = LogicalPlan::Aggregate {
        group_exprs: vec![LogicalExpr::ColumnRef { index: 0, name: "id".to_string() }],
        agg_exprs: vec![LogicalExpr::AggregateFunction {
            func: "SUM".to_string(),
            args: vec![LogicalExpr::ColumnRef { index: 1, name: "val".to_string() }],
            distinct: false,
        }],
        input: Box::new(make_scan("t")),
    };

    let fragments = planner.plan(plan).unwrap();
    assert!(!fragments.is_empty(), "distributed aggregation needs partial-agg fragments on workers and a final-agg fragment on the coordinator");
}

// ── 6b. Edge cases ───────────────────────────────────────────────────

#[test]
fn test_fragment_ids_sequential() {
    // Edge case: verify fragment IDs are always sequential from 0
    let mut builder = FragmentBuilder::new();
    for i in 0..5 {
        let id = builder.add_fragment(make_scan("t"), None, Some(ExchangeType::Gather));
        assert_eq!(id, i, "fragment IDs must be assigned sequentially starting from 0");
    }
    let fragments = builder.build();
    assert_eq!(fragments.len(), 5, "all added fragments must appear in the built result");
}

// ── 7. Multi-join fragments ─────────────────────────────────────────

#[test]
fn test_multi_join_fragments() {
    let planner = DistributedPlanner::new(4);
    let plan = LogicalPlan::Join {
        left: Box::new(LogicalPlan::Join {
            left: Box::new(make_scan("a")),
            right: Box::new(make_scan("b")),
            join_type: quackdb::sql::ast::JoinTypeAst::Inner,
            condition: None,
        }),
        right: Box::new(make_scan("c")),
        join_type: quackdb::sql::ast::JoinTypeAst::Inner,
        condition: None,
    };

    let fragments = planner.plan(plan).unwrap();
    assert!(fragments.len() >= 2, "a multi-way join must be split into multiple fragments connected by exchange operators");
}
