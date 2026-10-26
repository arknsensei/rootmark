# Kalite kapıları `866dad9` (P2-R/R5)

Sırayla, aynı ağaçta, son FAST çiftinden sonra; dördü de çıkış 0.
- `make test`: cargo 73 test ikilisi `ok` (`make test-rust`'ın bilinen iki K-P2A.2 atlaması dışında), pytest 268 geçti / 1 atlandı, vitest Obsidian 164/164, VS Code 12/12 (`test_summary.log`).
- `make lint`: `fmt`, `clippy -D warnings`, ruff, eslint, `check_deps`, `test-check-deps`, `check_frozen` (F1–F8), `check_compare_fields` (18 karşılaştırıcı, 413 alan), `gen_client_types --check` (`lint_tail.log`).
- `make conformance`: seviye 3, 39/39 (`reports/conformance.json` değişmedi).
- `scripts/check_dep_age.sh`: 111 bağımlılık, en az 14 gün (`dep_age.json`).
