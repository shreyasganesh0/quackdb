//! QuackDB: A distributed analytical database built from scratch in Rust.

// Part I: Foundations
#[cfg(feature = "lesson01")]
pub mod arena;

#[cfg(feature = "lesson02")]
pub mod types;

#[cfg(feature = "lesson03")]
pub mod vector;

#[cfg(feature = "lesson04")]
pub mod chunk;

// Part II: Compression
#[cfg(feature = "lesson05")]
pub mod compression;

// Part III: Storage Engine
#[cfg(feature = "lesson09")]
pub mod storage;

// Part IV: Vectorized Execution Engine
#[cfg(feature = "lesson13")]
pub mod execution;

// Part V: SQL Frontend
#[cfg(feature = "lesson20")]
pub mod sql;

#[cfg(feature = "lesson22")]
pub mod planner;

// Part VI: Query Optimization
#[cfg(feature = "lesson25")]
pub mod optimizer;

// Part VII: Transactions & Durability
#[cfg(feature = "lesson27")]
pub mod transaction;

// Part VIII: Parallelism & Distribution
#[cfg(feature = "lesson29")]
pub mod parallel;

#[cfg(feature = "lesson31")]
pub mod distributed;

// Part IX: Advanced
#[cfg(feature = "lesson35")]
pub mod simd;

// Top-level database facade
#[cfg(feature = "lesson24")]
pub mod db;
