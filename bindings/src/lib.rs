use pyo3::prelude::*;

#[pymodule]
fn mdkernel_core(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
