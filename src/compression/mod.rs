//! Compression module — RLE, Dictionary, Bitpacking, Delta, and auto-selection framework.

#[cfg(feature = "lesson05")]
pub mod rle;

#[cfg(feature = "lesson06")]
pub mod dictionary;

#[cfg(feature = "lesson07")]
pub mod bitpack;

#[cfg(feature = "lesson07")]
pub mod delta;

#[cfg(feature = "lesson08")]
pub mod frame;

/// Compression algorithm identifiers.
#[cfg(feature = "lesson08")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    None,
    Rle,
    Dictionary,
    Bitpack,
    Delta,
    DeltaBitpack,
}
