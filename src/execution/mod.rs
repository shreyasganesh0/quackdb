//! Execution engine — expressions, pipelines, operators.

#[cfg(feature = "lesson13")]
pub mod expression;

#[cfg(feature = "lesson14")]
pub mod pipeline;

#[cfg(feature = "lesson15")]
pub mod scan;

#[cfg(feature = "lesson15")]
pub mod filter;

#[cfg(feature = "lesson15")]
pub mod projection;

#[cfg(feature = "lesson16")]
pub mod hash_aggregate;

#[cfg(feature = "lesson17")]
pub mod hash_join;

#[cfg(feature = "lesson18")]
pub mod sort_merge_join;

#[cfg(feature = "lesson19")]
pub mod sort;

#[cfg(feature = "lesson30")]
pub mod window;

#[cfg(feature = "lesson32")]
pub mod exchange;

#[cfg(feature = "lesson34")]
pub mod adaptive;
