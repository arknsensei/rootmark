# bindings/python — ajan notları
Sorumluluk: `research` ve Python kullanıcıları için `core` (+ ileride `index`) yüzeyi; pyo3 → `rootmark_core` modülü (docs/PACKAGES.md `bindings/python`).
Bağımlılık yönü: `core`, `index` → bindings/python ← research. Ters yön yasak.
Dil ve araçlar: Rust 1.92.0, `pyo3` (abi3-py311), `maturin` (research/pyproject.toml `[tool.maturin]`; `make setup` içinde `maturin develop --release`).
Dondurulmuş yüzeyler: prototip v3-final Python API'si (`Vault(root, theta, theta_split, matcher)`, `ingest`, `snapshot`, `events_hash`, `resolve_truth`, `chunk`, …) — dondurulmuş `research/eval/**` buna dayanır.
Test komutları: `make test-py` (research/tests).
Bu pakette yapılmayacaklar: Python'da parser/eşleyici/chunker kopyası; çekirdek mantığı.
