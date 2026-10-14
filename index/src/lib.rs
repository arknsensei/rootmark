//! mdkernel-index — derived indexes over `mdkernel-core` (spec §9, GOAL P1-C).
//!
//! Every index is derived: it can be dropped and rebuilt from the vault, and
//! for the same checkpoint, query and index configuration it returns
//! identical ranked results (spec §9, §10). Search is exact (no ANN); one
//! embedder per index, recorded as `embedder_id` with every vector.

pub mod embed;
pub mod models;

pub use embed::{Embedder, OnnxMiniLm};
