//! rootmark-index — derived indexes over `rootmark-core` (spec §9, GOAL P1-C).
//!
//! Every index is derived: it can be dropped and rebuilt from the vault, and
//! for the same checkpoint, query and index configuration it returns
//! identical ranked results (spec §9, §10). Search is exact (no ANN); one
//! embedder per index, recorded as `embedder_id` with every vector.

pub mod cli;
pub mod config;
pub mod date;
pub mod embed;
pub mod lexical;
pub mod models;
pub mod record;
pub mod search;
pub mod store;
pub mod tokenize;
pub mod vectors;

pub use embed::{Embedder, OnnxMiniLm};
