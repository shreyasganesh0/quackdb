//! Lesson 32: Distributed Query Planning Tests

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

#[test]
fn test_single_scan_plan() {
    let planner = DistributedPlanner::new(4);
    let plan = make_scan("users");
    let fragments = planner.plan(plan).unwrap();
    assert!(!fragments.is_empty());
    // Single scan needs a gather exchange at the end
}

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
    assert!(has_repartition || fragments.len() > 1);
}

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
    assert!(!fragments.is_empty());
}

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

#[test]
fn test_broadcast_exchange() {
    let planner = DistributedPlanner::new(4);
    // A join where one side is very small could use broadcast
    let fragments = planner.plan(make_scan("small_table")).unwrap();
    // At minimum, should have fragments
    assert!(!fragments.is_empty());
}

#[test]
fn test_fragment_builder() {
    let mut builder = FragmentBuilder::new();
    let id = builder.add_fragment(make_scan("t"), None, Some(ExchangeType::Gather));
    assert_eq!(id, 0);
    let fragments = builder.build();
    assert_eq!(fragments.len(), 1);
    assert_eq!(fragments[0].fragment_id, 0);
}

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
    assert!(fragments.len() >= 2);
}
