//! Python module `mdkernel_core`: the SPEC §7 surface. Values cross the
//! boundary as JSON (serde → `json.loads`), so Python sees plain dict/list.
//! Core `Err` → `RuntimeError`.

use mdkcore::chunk::{chunk, count_tokens as core_count_tokens, Strategy};
use mdkcore::{AuthorKind, MatchParams, Provenance};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
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
    let r = mdkcore::parse_document(text, doc_path).map_err(err)?;
    to_py(py, &r)
}

#[pyfunction]
#[pyo3(signature = (text, is_code=false))]
fn normalize(text: &str, is_code: bool) -> String {
    mdkcore::normalize(text, is_code)
}

#[pyfunction]
fn extract_wikilinks(py: Python<'_>, text: &str) -> PyResult<Py<PyAny>> {
    to_py(py, &mdkcore::extract_wikilinks(text))
}

#[pyfunction]
fn count_tokens(text: &str) -> u32 {
    core_count_tokens(text)
}

#[pyfunction]
fn version_id(block_type: &str, normalized_text: &str) -> PyResult<String> {
    let bt: mdkcore::BlockType =
        serde_json::from_value(serde_json::Value::String(block_type.into()))
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    Ok(mdkcore::version_id(bt, normalized_text))
}

#[pyfunction]
fn lineage_formula(doc_path: &str, version_id: &str, k: usize) -> String {
    mdkcore::lineage::lineage_formula(doc_path, version_id, k)
}

#[pyfunction]
fn matcher_defaults() -> (f64, f64) {
    (mdkcore::lineage::THETA, mdkcore::lineage::THETA_SPLIT)
}

#[pyclass(unsendable)]
struct Vault {
    inner: mdkcore::Vault,
}

#[pymethods]
impl Vault {
    #[new]
    #[pyo3(signature = (root, theta=None, theta_split=None))]
    fn new(root: &str, theta: Option<f64>, theta_split: Option<f64>) -> PyResult<Self> {
        let d = MatchParams::default();
        let p = MatchParams {
            theta: theta.unwrap_or(d.theta),
            theta_split: theta_split.unwrap_or(d.theta_split),
        };
        Ok(Vault {
            inner: mdkcore::Vault::open_with(root, p).map_err(err)?,
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
        let r = self.inner.ingest(doc_path, text, prov).map_err(err)?;
        to_py(py, &r)
    }

    fn ingest_dir(&mut self, py: Python<'_>, dir: &str) -> PyResult<Py<PyAny>> {
        let r = self
            .inner
            .ingest_dir(std::path::Path::new(dir))
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
        (self.inner.params.theta, self.inner.params.theta_split)
    }
}

#[pymodule]
fn mdkernel_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
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
