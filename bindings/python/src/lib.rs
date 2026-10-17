//! Python module `rootmark_core`: the SPEC §7 surface. Values cross the
//! boundary as JSON (serde → `json.loads`), so Python sees plain dict/list.
//! Core `Err` → `RuntimeError`.

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use rootmarkcore::chunk::{chunk, count_tokens as core_count_tokens, Strategy};
use rootmarkcore::{AuthorKind, MatchParams, MatcherVersion, Provenance};
use serde::Serialize;

fn err(e: anyhow::Error) -> PyErr {
    PyRuntimeError::new_err(format!("{e:#}"))
}

fn to_py<T: Serialize>(py: Python<'_>, v: &T) -> PyResult<Py<PyAny>> {
    let s = serde_json::to_string(v).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    Ok(py.import("json")?.call_method1("loads", (s,))?.unbind())
}

fn author(kind: &str) -> PyResult<AuthorKind> {
    Ok(match kind {
        "Human" => AuthorKind::Human,
        "Agent" => AuthorKind::Agent,
        "Transcript" => AuthorKind::Transcript,
        "Synthetic" => AuthorKind::Synthetic,
        _ => {
            return Err(PyRuntimeError::new_err(format!(
                "unknown author_kind {kind:?}"
            )))
        }
    })
}

#[pyfunction]
fn parse_document(py: Python<'_>, text: &str, doc_path: &str) -> PyResult<Py<PyAny>> {
    let r = rootmarkcore::parse_document(text, doc_path).map_err(err)?;
    to_py(py, &r)
}

#[pyfunction]
#[pyo3(signature = (text, is_code=false))]
fn normalize(text: &str, is_code: bool) -> String {
    rootmarkcore::normalize(text, is_code)
}

#[pyfunction]
fn extract_wikilinks(py: Python<'_>, text: &str) -> PyResult<Py<PyAny>> {
    to_py(py, &rootmarkcore::extract_wikilinks(text))
}

#[pyfunction]
fn count_tokens(text: &str) -> u32 {
    core_count_tokens(text)
}

#[pyfunction]
fn version_id(block_type: &str, normalized_text: &str) -> PyResult<String> {
    let bt: rootmarkcore::BlockType =
        serde_json::from_value(serde_json::Value::String(block_type.into()))
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    Ok(rootmarkcore::version_id(bt, normalized_text))
}

#[pyfunction]
fn lineage_formula(doc_path: &str, version_id: &str, k: usize) -> String {
    rootmarkcore::lineage::lineage_formula(doc_path, version_id, k)
}

/// Matcher version: explicit argument, else `$ROOTMARK_MATCHER`, else "v3"
/// (SPEC_DELTA §7). The environment variable lets frozen v2 code, which never
/// passes a matcher, run under v2 unchanged.
fn matcher_of(matcher: Option<&str>) -> PyResult<MatcherVersion> {
    let env = std::env::var("ROOTMARK_MATCHER").ok();
    let name = matcher.or(env.as_deref()).unwrap_or("v3");
    MatcherVersion::parse(name).map_err(err)
}

/// Compiled (THETA, THETA_SPLIT) of the matcher version.
#[pyfunction]
#[pyo3(signature = (matcher=None))]
fn matcher_defaults(matcher: Option<&str>) -> PyResult<(f64, f64)> {
    let p = matcher_of(matcher)?.defaults();
    Ok((p.theta, p.theta_split))
}

#[pyclass(unsendable)]
struct Vault {
    inner: rootmarkcore::Vault,
    matcher: MatcherVersion,
}

#[pymethods]
impl Vault {
    #[new]
    #[pyo3(signature = (root, theta=None, theta_split=None, matcher=None))]
    fn new(
        root: &str,
        theta: Option<f64>,
        theta_split: Option<f64>,
        matcher: Option<&str>,
    ) -> PyResult<Self> {
        let m = matcher_of(matcher)?;
        let d = m.defaults();
        let p = MatchParams {
            theta: theta.unwrap_or(d.theta),
            theta_split: theta_split.unwrap_or(d.theta_split),
        };
        Ok(Vault {
            inner: rootmarkcore::Vault::open_with(root, p).map_err(err)?,
            matcher: m,
        })
    }

    #[pyo3(signature = (doc_path, text, author_kind="Human", source=""))]
    fn ingest(
        &mut self,
        py: Python<'_>,
        doc_path: &str,
        text: &str,
        author_kind: &str,
        source: &str,
    ) -> PyResult<Py<PyAny>> {
        let prov = Provenance {
            author_kind: author(author_kind)?,
            source: source.to_string(),
        };
        let r = self
            .inner
            .ingest(doc_path, text, prov, self.matcher)
            .map_err(err)?;
        to_py(py, &r)
    }

    fn ingest_dir(&mut self, py: Python<'_>, dir: &str) -> PyResult<Py<PyAny>> {
        let r = self
            .inner
            .ingest_dir(std::path::Path::new(dir), self.matcher)
            .map_err(err)?;
        to_py(py, &r)
    }

    fn snapshot(&self, py: Python<'_>, doc_path: &str) -> PyResult<Py<PyAny>> {
        to_py(py, &self.inner.snapshot(doc_path).map_err(err)?)
    }

    fn snapshot_blocks(&self, py: Python<'_>, doc_path: &str) -> PyResult<Py<PyAny>> {
        let s = self.inner.snapshot(doc_path).map_err(err)?;
        let s = s.ok_or_else(|| PyRuntimeError::new_err(format!("no snapshot for {doc_path}")))?;
        to_py(py, &self.inner.snapshot_blocks(&s).map_err(err)?)
    }

    fn block(&self, py: Python<'_>, version_id: &str) -> PyResult<Py<PyAny>> {
        to_py(py, &self.inner.block(version_id).map_err(err)?)
    }

    fn replay(&self, py: Python<'_>, doc_path: &str) -> PyResult<Py<PyAny>> {
        to_py(py, &self.inner.replay(doc_path).map_err(err)?)
    }

    fn events_hash(&self) -> PyResult<String> {
        self.inner.events_hash().map_err(err)
    }

    fn resolve_truth(&self, doc_path: &str, version_id: &str, k: usize) -> PyResult<String> {
        self.inner
            .resolve_truth(doc_path, version_id, k)
            .map_err(err)
    }

    #[pyo3(signature = (doc_path, strategy="structural"))]
    fn chunk(&self, py: Python<'_>, doc_path: &str, strategy: &str) -> PyResult<Py<PyAny>> {
        let s = self.inner.snapshot(doc_path).map_err(err)?;
        let s = s.ok_or_else(|| PyRuntimeError::new_err(format!("no snapshot for {doc_path}")))?;
        let st = Strategy::parse(strategy).map_err(err)?;
        to_py(py, &chunk(&s, &self.inner, st).map_err(err)?)
    }

    fn doc_paths(&self) -> Vec<String> {
        self.inner.doc_paths()
    }

    fn params(&self) -> (f64, f64) {
        let p = self.inner.params_for(self.matcher);
        (p.theta, p.theta_split)
    }

    /// Matcher version this handle ingests with ("v2" / "v3").
    fn matcher(&self) -> &'static str {
        self.matcher.as_str()
    }

    /// Matcher version the vault's log is locked to, or None while empty.
    fn matcher_lock(&self) -> Option<&'static str> {
        self.inner.matcher_lock().map(|m| m.as_str())
    }

    /// `index watch` (P1-C): ingests `text` as `doc_path` with this handle's
    /// matcher and adds the new head's unindexed chunks to the hot tier.
    #[pyo3(signature = (doc_path, text, author_kind="Human", source=""))]
    fn index_watch(
        &mut self,
        py: Python<'_>,
        doc_path: &str,
        text: &str,
        author_kind: &str,
        source: &str,
    ) -> PyResult<Py<PyAny>> {
        let e = rootmarkindex::OnnxMiniLm::load_default().map_err(err)?;
        let mut store = rootmarkindex::store::Store::open(&self.inner, &e).map_err(err)?;
        let prov = Provenance {
            author_kind: author(author_kind)?,
            source: source.to_string(),
        };
        let r = store
            .watch_with(&mut self.inner, &e, doc_path, text, prov, self.matcher)
            .map_err(err)?;
        to_py(py, &r)
    }

    /// `index compact` (P1-C): the hot tier moved into a new cold index.
    fn index_compact(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let e = rootmarkindex::OnnxMiniLm::load_default().map_err(err)?;
        to_py(
            py,
            &rootmarkindex::store::compact(&self.inner, &e).map_err(err)?,
        )
    }

    /// `index status` (P1-C).
    fn index_status(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let e = rootmarkindex::OnnxMiniLm::load_default().map_err(err)?;
        let store = rootmarkindex::store::Store::open(&self.inner, &e).map_err(err)?;
        to_py(py, &store.status(&self.inner).map_err(err)?)
    }

    /// Checkpoint of the current state (spec §8): `{checkpoint_id, name,
    /// event_seq, docs}`; same state → same id.
    #[pyo3(signature = (name=None))]
    fn checkpoint(&mut self, py: Python<'_>, name: Option<&str>) -> PyResult<Py<PyAny>> {
        to_py(py, &self.inner.checkpoint(name).map_err(err)?)
    }

    /// `index build` (P1-C): `<root>/.rootmark/index/` from every head and
    /// checkpoint snapshot, with the manifest-verified ONNX embedder.
    fn index_build(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let e = rootmarkindex::OnnxMiniLm::load_default().map_err(err)?;
        to_py(
            py,
            &rootmarkindex::store::build(&self.inner, &e).map_err(err)?,
        )
    }

    /// `search` (P1-C) for every query, in order, as of checkpoint `at`
    /// (id or name): one `SearchResult` dict per query
    /// (`checkpoint_id, mode, include_proposal, query_hash, trace_id, hits`);
    /// every search records one consumption trace. The index and the
    /// embedder are opened once.
    #[pyo3(signature = (queries, at, k=10, mode="hybrid", include_proposal=false, consumer="research"))]
    #[allow(clippy::too_many_arguments)] // the Python keyword signature
    fn search_many(
        &mut self,
        py: Python<'_>,
        queries: Vec<String>,
        at: &str,
        k: usize,
        mode: &str,
        include_proposal: bool,
        consumer: &str,
    ) -> PyResult<Py<PyAny>> {
        // every query first: an empty one is refused before anything is
        // opened or written (spec §9, K-C1.5)
        for q in &queries {
            rootmarkindex::search::check_query(q).map_err(err)?;
        }
        let e = rootmarkindex::OnnxMiniLm::load_default().map_err(err)?;
        let store = rootmarkindex::store::Store::open(&self.inner, &e).map_err(err)?;
        let mode = rootmarkindex::search::Mode::parse(mode).map_err(err)?;
        let mut se = rootmarkindex::search::Searcher::new(&store, &e, consumer).map_err(err)?;
        let mut out = Vec::with_capacity(queries.len());
        for q in &queries {
            out.push(
                se.search(&mut self.inner, mode, q, k, Some(at), include_proposal)
                    .map_err(err)?,
            );
        }
        to_py(py, &out)
    }
}

#[pymodule]
fn rootmark_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_document, m)?)?;
    m.add_function(wrap_pyfunction!(normalize, m)?)?;
    m.add_function(wrap_pyfunction!(extract_wikilinks, m)?)?;
    m.add_function(wrap_pyfunction!(count_tokens, m)?)?;
    m.add_function(wrap_pyfunction!(version_id, m)?)?;
    m.add_function(wrap_pyfunction!(lineage_formula, m)?)?;
    m.add_function(wrap_pyfunction!(matcher_defaults, m)?)?;
    m.add_class::<Vault>()?;
    Ok(())
}
