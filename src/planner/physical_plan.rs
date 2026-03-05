//! Lesson 24: Physical Plan & Execution
//!
//! Convert logical plans to physical operators and build pipelines.

use super::logical_plan::LogicalPlan;
use super::catalog::Catalog;
use crate::execution::pipeline::{Pipeline, PhysicalOperator, DataSource};
use crate::chunk::DataChunk;
use crate::types::LogicalType;

/// Build physical operators from a logical plan.
pub struct PhysicalPlanBuilder<'a> {
    catalog: &'a Catalog,
}

impl<'a> PhysicalPlanBuilder<'a> {
    pub fn new(catalog: &'a Catalog) -> Self {
        Self { catalog }
    }

    /// Convert a logical plan to a physical execution pipeline.
    pub fn build(&self, plan: &LogicalPlan) -> Result<Pipeline, String> {
        todo!()
    }

    /// Create a data source for a table scan.
    fn build_scan(&self, table_name: &str, projection: &Option<Vec<usize>>) -> Result<Box<dyn DataSource>, String> {
        todo!()
    }
}

/// Pipeline builder that handles pipeline breakers (joins, aggregates, sorts).
pub struct PipelineBuilder;

impl PipelineBuilder {
    /// Build pipelines from a logical plan, splitting at pipeline breakers.
    pub fn build(plan: &LogicalPlan, catalog: &Catalog) -> Result<Vec<Pipeline>, String> {
        todo!()
    }
}

/// Execute a logical plan end-to-end, returning result chunks.
pub fn execute_plan(plan: &LogicalPlan, catalog: &Catalog) -> Result<Vec<DataChunk>, String> {
    todo!()
}
