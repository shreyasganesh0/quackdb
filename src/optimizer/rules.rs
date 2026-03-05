//! Lesson 25: Rule-Based Optimizer
//!
//! Transformation rules for logical plan optimization.

use crate::planner::logical_plan::LogicalPlan;

/// A rule that transforms a logical plan.
pub trait OptimizerRule {
    /// Name of this rule.
    fn name(&self) -> &str;
    /// Apply the rule to a logical plan, returning the optimized plan.
    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan, String>;
}

/// Pushes filter predicates closer to their data sources.
pub struct PredicatePushdown;

impl OptimizerRule for PredicatePushdown {
    fn name(&self) -> &str { "PredicatePushdown" }
    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan, String> { todo!() }
}

/// Pushes column projections down to reduce data flowing through the plan.
pub struct ProjectionPushdown;

impl OptimizerRule for ProjectionPushdown {
    fn name(&self) -> &str { "ProjectionPushdown" }
    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan, String> { todo!() }
}

/// Folds constant expressions at compile time.
pub struct ConstantFolding;

impl OptimizerRule for ConstantFolding {
    fn name(&self) -> &str { "ConstantFolding" }
    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan, String> { todo!() }
}

/// Merges adjacent filter nodes.
pub struct FilterMerge;

impl OptimizerRule for FilterMerge {
    fn name(&self) -> &str { "FilterMerge" }
    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan, String> { todo!() }
}

/// Pushes LIMIT through certain operators.
pub struct LimitPushdown;

impl OptimizerRule for LimitPushdown {
    fn name(&self) -> &str { "LimitPushdown" }
    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan, String> { todo!() }
}

/// Apply a set of rules iteratively until a fixpoint is reached.
pub fn optimize(plan: LogicalPlan, rules: &[Box<dyn OptimizerRule>], max_iterations: usize) -> Result<LogicalPlan, String> {
    todo!()
}

/// Get the default set of optimization rules.
pub fn default_rules() -> Vec<Box<dyn OptimizerRule>> {
    todo!()
}
