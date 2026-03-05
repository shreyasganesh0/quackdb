//! Query optimizer — rule-based and cost-based optimization.

#[cfg(feature = "lesson25")]
pub mod rules;

#[cfg(feature = "lesson26")]
pub mod statistics;

#[cfg(feature = "lesson26")]
pub mod cost_model;

#[cfg(feature = "lesson26")]
pub mod join_order;
