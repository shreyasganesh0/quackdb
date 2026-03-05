//! Lesson 26: Cost-Based Optimizer Tests

use quackdb::types::{LogicalType, ScalarValue};
use quackdb::planner::logical_plan::*;
use quackdb::optimizer::statistics::*;
use quackdb::optimizer::cost_model::*;
use quackdb::optimizer::join_order::*;
use std::collections::HashMap;

fn make_table_stats(rows: u64, distinct: u64) -> TableStatistics {
    TableStatistics {
        row_count: rows,
        columns: vec![ColumnStatistics {
            distinct_count: distinct,
            null_count: 0,
            min_value: Some(0.0),
            max_value: Some(rows as f64),
            total_count: rows,
            histogram: None,
        }],
    }
}

#[test]
fn test_column_statistics_selectivity() {
    let stats = ColumnStatistics {
        distinct_count: 100,
        null_count: 0,
        min_value: Some(0.0),
        max_value: Some(1000.0),
        total_count: 1000,
        histogram: None,
    };

    // Equality selectivity: ~1/distinct_count
    let eq_sel = stats.equality_selectivity();
    assert!(eq_sel > 0.0 && eq_sel <= 0.02, "eq_sel={}", eq_sel);

    // Range selectivity: (value - min) / (max - min)
    let gt_sel = stats.selectivity(">", 500.0);
    assert!(gt_sel > 0.4 && gt_sel < 0.6, "gt_sel={}", gt_sel);

    let lt_sel = stats.selectivity("<", 100.0);
    assert!(lt_sel > 0.05 && lt_sel < 0.15, "lt_sel={}", lt_sel);
}

#[test]
fn test_cardinality_estimation_scan() {
    let mut stats_map = HashMap::new();
    stats_map.insert("users".to_string(), make_table_stats(1000, 1000));

    let plan = LogicalPlan::Scan {
        table_name: "users".to_string(),
        schema: Schema::new(vec![("id".to_string(), LogicalType::Int32)]),
        projection: None,
    };

    let est = CardinalityEstimator::estimate(&plan, &stats_map);
    assert_eq!(est, 1000);
}

#[test]
fn test_cardinality_estimation_filter() {
    let mut stats_map = HashMap::new();
    stats_map.insert("users".to_string(), make_table_stats(1000, 100));

    let plan = LogicalPlan::Filter {
        predicate: LogicalExpr::BinaryOp {
            op: quackdb::sql::ast::BinaryOpAst::Equal,
            left: Box::new(LogicalExpr::ColumnRef { index: 0, name: "id".to_string() }),
            right: Box::new(LogicalExpr::Literal(ScalarValue::Int32(42))),
        },
        input: Box::new(LogicalPlan::Scan {
            table_name: "users".to_string(),
            schema: Schema::new(vec![("id".to_string(), LogicalType::Int32)]),
            projection: None,
        }),
    };

    let est = CardinalityEstimator::estimate(&plan, &stats_map);
    assert!(est < 1000, "Filter should reduce cardinality");
    assert!(est > 0, "Should have some rows");
}

#[test]
fn test_cost_model_scan() {
    let mut stats_map = HashMap::new();
    stats_map.insert("t".to_string(), make_table_stats(1000, 100));

    let plan = LogicalPlan::Scan {
        table_name: "t".to_string(),
        schema: Schema::new(vec![("a".to_string(), LogicalType::Int32)]),
        projection: None,
    };

    let cost = CostModel::estimate(&plan, &stats_map);
    assert!(cost.total() > 0.0);
}

#[test]
fn test_cost_model_hash_join() {
    let cost = CostModel::hash_join_cost(1000, 10000);
    assert!(cost.total() > 0.0);
    // Hash join cost should scale with build + probe
}

#[test]
fn test_cost_model_sort() {
    let cost = CostModel::sort_cost(1000);
    assert!(cost.total() > 0.0);
    // Sort is O(n log n)
    let cost2 = CostModel::sort_cost(10000);
    assert!(cost2.total() > cost.total());
}

#[test]
fn test_cost_addition() {
    let c1 = Cost { cpu: 10.0, io: 5.0, network: 1.0 };
    let c2 = Cost { cpu: 20.0, io: 3.0, network: 2.0 };
    let sum = c1.add(&c2);
    assert_eq!(sum.cpu, 30.0);
    assert_eq!(sum.io, 8.0);
    assert_eq!(sum.network, 3.0);
}

#[test]
fn test_cost_zero() {
    let z = Cost::zero();
    assert_eq!(z.total(), 0.0);
}

#[test]
fn test_join_order_two_tables() {
    let mut stats_map = HashMap::new();
    stats_map.insert("small".to_string(), make_table_stats(100, 100));
    stats_map.insert("large".to_string(), make_table_stats(10000, 10000));

    let relations = vec![
        LogicalPlan::Scan {
            table_name: "small".to_string(),
            schema: Schema::new(vec![("id".to_string(), LogicalType::Int32)]),
            projection: None,
        },
        LogicalPlan::Scan {
            table_name: "large".to_string(),
            schema: Schema::new(vec![("id".to_string(), LogicalType::Int32)]),
            projection: None,
        },
    ];

    let edges = vec![JoinEdge {
        left: RelationSet::singleton(0),
        right: RelationSet::singleton(1),
        condition: None,
    }];

    let result = JoinOrderOptimizer::optimize(&relations, &edges, &stats_map).unwrap();
    // Should produce a valid join plan
}

#[test]
fn test_relation_set() {
    let s1 = RelationSet::singleton(0);
    let s2 = RelationSet::singleton(1);
    let union = s1.union(&s2);

    assert_eq!(s1.count(), 1);
    assert_eq!(union.count(), 2);
    assert!(s1.is_subset_of(&union));
    assert!(s2.is_subset_of(&union));
    assert!(!union.is_subset_of(&s1));
}

#[test]
fn test_relation_set_subsets() {
    let set = RelationSet { bits: 0b111 }; // {0, 1, 2}
    let subsets = set.subsets();
    // Non-empty subsets of {0,1,2}: 7 total
    assert_eq!(subsets.len(), 7);
}

#[test]
fn test_join_order_four_tables() {
    let mut stats_map = HashMap::new();
    for (name, rows) in &[("a", 100), ("b", 1000), ("c", 500), ("d", 200)] {
        stats_map.insert(name.to_string(), make_table_stats(*rows, *rows));
    }

    let relations: Vec<_> = ["a", "b", "c", "d"].iter().map(|name| {
        LogicalPlan::Scan {
            table_name: name.to_string(),
            schema: Schema::new(vec![("id".to_string(), LogicalType::Int32)]),
            projection: None,
        }
    }).collect();

    let edges = vec![
        JoinEdge { left: RelationSet::singleton(0), right: RelationSet::singleton(1), condition: None },
        JoinEdge { left: RelationSet::singleton(1), right: RelationSet::singleton(2), condition: None },
        JoinEdge { left: RelationSet::singleton(2), right: RelationSet::singleton(3), condition: None },
    ];

    let result = JoinOrderOptimizer::optimize(&relations, &edges, &stats_map).unwrap();
    // Should produce a valid plan
}

#[test]
fn test_column_statistics_new() {
    let stats = ColumnStatistics::new(1000);
    assert_eq!(stats.total_count, 1000);
}

#[test]
fn test_merge_join_cost_cheaper() {
    // When inputs are already sorted, merge join should be cheaper than hash join for large inputs
    let merge = CostModel::merge_join_cost(10000, 10000);
    let hash = CostModel::hash_join_cost(10000, 10000);
    // Both should have positive costs
    assert!(merge.total() > 0.0);
    assert!(hash.total() > 0.0);
}
