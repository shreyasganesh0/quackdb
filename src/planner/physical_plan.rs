//! Lesson 24: Physical Plan & Execution
//!
//! Converts a [`LogicalPlan`] into physical execution pipelines. The physical
//! plan builder maps each logical operator to a concrete [`PhysicalOperator`]
//! and wires them into [`Pipeline`]s.
//!
//! **Key idea:** Walk the logical plan tree bottom-up. Leaf nodes (Scan)
//! become data sources. Streaming operators (Filter, Projection) are added
//! to the current pipeline. Pipeline breakers (Aggregate, Sort, Join) split
//! the plan into multiple pipelines connected by intermediate materialization.

use super::logical_plan::LogicalPlan;
use super::catalog::Catalog;
use crate::execution::pipeline::{Pipeline, PhysicalOperator, DataSource};
use crate::chunk::DataChunk;
use crate::types::LogicalType;

/// Builds a single physical execution pipeline from a logical plan.
///
/// Holds a reference to the catalog so it can look up table data for scans.
// Lifetime 'a: borrows the Catalog for the duration of plan building.
pub struct PhysicalPlanBuilder<'a> {
    catalog: &'a Catalog,
}

impl<'a> PhysicalPlanBuilder<'a> {
    /// Create a new physical plan builder.
    pub fn new(catalog: &'a Catalog) -> Self {
        Self { catalog }
    }

    /// Convert a logical plan into a physical execution pipeline.
    ///
    /// Recursively processes the plan tree:
    /// - `Scan` -> creates the pipeline's data source.
    /// - `Filter` / `Projection` -> appended as streaming operators.
    /// - `Aggregate` / `Sort` -> pipeline breakers (see [`PipelineBuilder`]).
    pub fn build(&self, plan: &LogicalPlan) -> Result<Pipeline, String> {
        // Hint: match on plan. For Scan, call build_scan and create a
        // Pipeline. For Filter/Projection, build the child pipeline
        // first, then add the operator. For Aggregate/Sort/Join, you
        // may need to split into multiple pipelines.
        todo!()
    }

    /// Create a data source that reads from a catalog table.
    ///
    /// Looks up the table's data chunks in the catalog and wraps them
    /// in an `InMemorySource`.
    fn build_scan(&self, table_name: &str, projection: &Option<Vec<usize>>) -> Result<Box<dyn DataSource>, String> {
        // Hint: call self.catalog.get_table_data(table_name) to get the
        // chunks. Wrap them in an InMemorySource. If projection is Some,
        // wrap in a TableScanOperator or apply projection in the source.
        todo!()
    }
}

/// Builds multiple pipelines from a logical plan, splitting at pipeline breakers.
///
/// Pipeline breakers (aggregates, sorts, joins) require all input before
/// producing output. The builder creates separate pipelines for each segment
/// and connects them through intermediate materialization.
pub struct PipelineBuilder;

impl PipelineBuilder {
    /// Analyze the logical plan and produce an ordered list of pipelines.
    ///
    /// Pipelines should be executed in order: earlier pipelines produce
    /// intermediate results consumed by later pipelines.
    pub fn build(plan: &LogicalPlan, catalog: &Catalog) -> Result<Vec<Pipeline>, String> {
        // Hint: recursively walk the plan. When you encounter a pipeline
        // breaker, create a new pipeline for the subtree below it,
        // execute it to get intermediate results, then continue building
        // the pipeline above the breaker using those results as a source.
        todo!()
    }
}

/// Execute a logical plan end-to-end, returning the result chunks.
///
/// This is the main entry point for query execution. It builds the physical
/// pipeline(s) and runs them to completion.
pub fn execute_plan(plan: &LogicalPlan, catalog: &Catalog) -> Result<Vec<DataChunk>, String> {
    // Hint: use PhysicalPlanBuilder (or PipelineBuilder for plans with
    // pipeline breakers) to create pipelines, then use PipelineExecutor
    // to run them and collect the results.
    todo!()
}
