//! mdkernel-index — derived indexes over `mdkernel-core` (spec §9).
//!
//! Round P1-A defines the traits only; implementations (tantivy, exact cosine
//! vector store, ONNX embedder, RRF) arrive in round P1-C. Every index is
//! derived: it can be dropped and rebuilt from the vault, and for the same
//! inputs it MUST return identical ranked results (spec §9, §10).

use anyhow::Result;

/// One retrieval unit handed to an index: a chunk (spec §6) identified by its
/// `chunk_id`, with the text the index sees.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexDoc {
    pub chunk_id: String,
    pub doc_path: String,
    pub text: String,
}

/// A ranked hit. Ties are broken by `chunk_id` (byte order) so rankings are total.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    pub chunk_id: String,
    pub score: f64,
}

/// Lexical index (BM25 or equivalent); part of the default retrieval path (K2).
pub trait LexicalIndex {
    fn upsert(&mut self, docs: &[IndexDoc]) -> Result<()>;
    fn remove(&mut self, chunk_ids: &[String]) -> Result<()>;
    fn search(&self, query: &str, k: usize) -> Result<Vec<Hit>>;
}

/// Text embedder. Implementations MUST be deterministic for the same model
/// and input (spec §10).
pub trait Embedder {
    /// Identifier of the model, recorded in index manifests.
    fn model_id(&self) -> &str;
    fn dim(&self) -> usize;
    fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
}

/// Vector store with exact (or provably equivalent) nearest-neighbour search.
pub trait VectorIndex {
    fn upsert(&mut self, items: &[(String, Vec<f32>)]) -> Result<()>;
    fn remove(&mut self, chunk_ids: &[String]) -> Result<()>;
    fn search(&self, query: &[f32], k: usize) -> Result<Vec<Hit>>;
}

/// Fusion of ranked lists (reference: reciprocal rank fusion).
pub trait Fusion {
    fn fuse(&self, lists: &[Vec<Hit>], k: usize) -> Vec<Hit>;
}
