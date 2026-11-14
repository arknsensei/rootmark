# bindings/python — agent notes
Responsibility: the `core` surface for Python users (pyo3 → the `rootmark_core` module; distribution `rootmark-bindings`, not on PyPI).
Dependency direction: `core`, `index` → bindings/python. Never the reverse.
Language and tools: Rust 1.92.0, `pyo3` (abi3-py311), `maturin` (`maturin build --release -m bindings/python/Cargo.toml`).
Frozen surfaces: the Python API (`Vault(root, theta, theta_split, matcher)`, `ingest`, `snapshot`, `events_hash`, `resolve_truth`, `chunk`, …) — the development repository's measurement code depends on it.
Test commands: the release workflow builds the wheel; the development repository runs its Python tests.
Not in this package: a Python copy of the parser, matcher or chunker; core logic.
