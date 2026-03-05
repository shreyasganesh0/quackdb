//! Storage engine — pages, buffer pool, and columnar file format.

#[cfg(feature = "lesson09")]
pub mod page;

#[cfg(feature = "lesson10")]
pub mod buffer_pool;

#[cfg(feature = "lesson11")]
pub mod columnar_file;

#[cfg(feature = "lesson12")]
pub mod reader;
