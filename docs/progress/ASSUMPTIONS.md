# Varsayımlar

Her satır etkileyebileceği hipotezle etiketlenir: `[H1]`, `[H2]`, `[H3]`, `[H4]`, `[-]`.

- [-] T0: `ASSUMPTIONS.md`, `FREEZE_EXCEPTIONS.md`, `STOP.md`, `EVAL_FREEZE_ISSUES.md` dosyaları `docs/progress/` altında tutulur (CLAUDE.md yol vermiyor).
- [-] T0: Python sürümü 3.12'ye sabitlendi (`requires-python = ">=3.12,<3.13"`); `networkx==3.7` 3.11'i desteklemiyor. CLAUDE.md "3.11+" koşulu sağlanır.
- [-] T0: Rust çekirdek crate'inin kütüphane adı `mdkcore`; Python modül adı `mdkernel_core` bağlama crate'ine ayrıldığı için çakışmayı önler.
- [-] T0: `uv` projesi `package = false`; uzantı modülü yalnız `maturin develop --release` ile kurulur (`make build`). `uv sync` uzantıyı kaldırabildiği için `make setup` sırayla ikisini koşturur.
