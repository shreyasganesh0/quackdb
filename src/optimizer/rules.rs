//! Lesson 25: Rule-Based Optimizer
//!
//! Transformation rules for logical plan optimization. Each rule rewrites a
//! [`LogicalPlan`] tree to an equivalent but more efficient form. Rules are
//! applied iteratively until a fixpoint (no further changes) or a maximum
//! iteration count is reached.

use crate::planner::logical_plan::LogicalPlan;

/// A rule that transforms a logical plan into an equivalent, optimized plan.
///
/// Implement this trait for each rewrite rule. The optimizer calls [`apply`]
/// repeatedly until the plan stops changing.
pub trait OptimizerRule {
    /// Human-readable name used for logging and debugging.
    fn name(&self) -> &str;

    /// Apply the rule to `plan`, returning the rewritten plan.
    ///
    /// Return `Ok(plan)` unchanged if the rule does not apply.
    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan, String>;
}

/// Pushes filter predicates closer to their data sources.
///
/// This reduces the number of rows flowing through the plan tree early,
/// which is typically the single most impactful optimization.
pub struct PredicatePushdown;

// Trait impl for PredicatePushdown -- walk the plan tree recursively,
// moving Filter nodes below Joins and Projections where safe.
impl OptimizerRule for PredicatePushdown {
    fn name(&self) -> &str { "PredicatePushdown" }
    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan, String> { todo!() }
}

/// Pushes column projections down to reduce data flowing through the plan.
///
/// Only the columns actually needed by upstream operators are kept, allowing
/// storage layers to skip reading unused columns.
pub struct ProjectionPushdown;

// Trait impl for ProjectionPushdown -- collect required columns from
// parent operators and push a narrower projection into scans.
impl OptimizerRule for ProjectionPushdown {
    fn name(&self) -> &str { "ProjectionPushdown" }
    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan, String> { todo!() }
}

/// Folds constant expressions at compile time.
///
/// For example, `1 + 2` becomes `3` and `true AND x` becomes `x`.
pub struct ConstantFolding;

// Trait impl for ConstantFolding -- visit every LogicalExpr and evaluate
// sub-expressions that contain only literal values.
impl OptimizerRule for ConstantFolding {
    fn name(&self) -> &str { "ConstantFolding" }
    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan, String> { todo!() }
}

/// Merges adjacent filter nodes into a single filter with a combined predicate.
///
/// `Filter(a, Filter(b, child))` becomes `Filter(a AND b, child)`.
pub struct FilterMerge;

// Trait impl for FilterMerge -- pattern-match consecutive Filter nodes.
impl OptimizerRule for FilterMerge {
    fn name(&self) -> &str { "FilterMerge" }
    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan, String> { todo!() }
}

/// Pushes LIMIT through certain operators (projections, filters) to reduce
/// intermediate result sizes.
pub struct LimitPushdown;

// Trait impl for LimitPushdown -- only push through operators that
// preserve or reduce row count (never through aggregations).
impl OptimizerRule for LimitPushdown {
    fn name(&self) -> &str { "LimitPushdown" }
    fn apply(&self, plan: LogicalPlan) -> Result<LogicalPlan, String> { todo!() }
}

/// Apply a set of rules iteratively until a fixpoint is reached or
/// `max_iterations` rounds have been executed.
///
/// Each iteration applies every rule in order. If no rule changes the plan
/// during an iteration, the loop terminates early.
pub fn optimize(plan: LogicalPlan, rules: &[Box<dyn OptimizerRule>], max_iterations: usize) -> Result<LogicalPlan, String> {
    // Hint: compare the plan before and after each iteration (e.g., via Debug)
    // to detect fixpoint convergence.
    todo!()
}

/// Returns the default set of optimization rules in recommended application order.
///
/// Typical ordering: constant folding, filter merge, predicate pushdown,
/// projection pushdown, limit pushdown.
pub fn default_rules() -> Vec<Box<dyn OptimizerRule>> {
    todo!()
}
