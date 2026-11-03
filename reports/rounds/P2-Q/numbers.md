# P2-Q — Ölçülen sayılar (eşiksiz; kaynak: her satırın kaydı)

| Kayıt | Ortam | İşlemci | ingest-dir (s) | index build (s) | chunks embedded | chunks/s | MCP propose (ms) | propose → searchable (ms) | direct write → ingested (ms) | Obsidian save → layer (ms) | search (ms, HTTP, client side) | plugin proposals view open + refresh (ms) | serve peak RSS (KiB) | .rootmark/ (bytes) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `mac/dnotes/run1` | mac | Apple M4 Max | 5.998 | 297.428 | 6063 | 20.385 | 175.685/241.923 | 279.094/297.099 | 192.704/199.958 | — | 198.888/208.143 | — | 682992 | 47631419 |
| `mac/dnotes/run2` | mac | Apple M4 Max | 5.969 | 298.544 | 6063 | 20.309 | 176.864/245.246 | 279.837/300.002 | 189.628/200.405 | — | 198.533/209.72 | — | 678608 | 47631419 |
| `mac/obsidian` | mac | — | 9.189 | 549.022 | 12329 | — | — | — | — | 266.691/310.847 | 359.397/375.153 | — | 1941328 | 70107043 |
| `mac/qorbit/run1` | mac | Apple M4 Max | 1.34 | 116.051 | 1868 | 16.096 | 99.599/112.727 | 71.864/78.922 | 147.611/159.077 | — | 51.339/59.117 | — | 1390368 | 13429258 |
| `mac/qorbit/run2` | mac | Apple M4 Max | 1.406 | 116.092 | 1868 | 16.091 | 97.511/108.387 | 71.283/77.316 | 147.134/151.793 | — | 50.421/57.132 | — | 1391584 | 13429258 |

Gecikmeler p50/p95. Docker satırları emülasyondur (bilgi, hız sayısı değil).
