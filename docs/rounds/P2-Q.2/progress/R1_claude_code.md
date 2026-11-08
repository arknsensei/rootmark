# R1 — Gerçek Claude Code ile başsız sınama (yerel ikili; Q2-2 (iii)'ün R1 yarısı)

## Ortam
- Claude Code **2.1.319** (`claude --version`), macOS, 2026-11-08.
- rootmark: tur dalının `de58f87` ağacından derlenmiş yerel ikili (`cargo build --locked --release -p rootmark-server --bin rootmark`; `rmcp 3.6.0`; sürüm dizgisi henüz `0.3.1`). İkili commit'ten önce derlendi; derlenen kaynak `de58f87`'nin kaynağıyla aynı (aradaki fark yalnız bu kayıt ve `R1.md`).
- Vault: `research/data/d_notes/notes`'un depo dışı kopyası (R0 ile aynı hazırlık: `ingest-dir` 399 belge, `index build` 6063 kayıt). Kişisel içerik yok.

## Komut
R0'ınkiyle aynı (`research/p2q2/claude_probe.py --label r1_local`): `claude -p "<4 adım>" --mcp-config <work>/mcp.json --strict-mcp-config --allowedTools mcp__rootmark__search,mcp__rootmark__propose,mcp__rootmark__status --output-format json --debug-file <work>/debug.log`; sunucu `python3 research/p2q2/mcp_tap.py --log … -- <rootmark> serve --vault <kopya> --mcp stdio`.

## Sonuç (içeriksiz özet: `reports/rounds/P2-Q.2/claude_code/r1_local.json`)
- **CC1 — araç listesi 11/11**, adlar 0.3.1'inkiyle aynı (`blame, checkpoint, checkpoints, events, history, promote, propose, read_block, search, status, trace`; `tools_match_0_3_1: true`).
- **CC2 — `search` başarılı** (oturumun yanıtı "2. SUCCEEDED"; kaydedicide `tools/call search` → hata yok).
- **CC3 — `propose` başarılı**: `agent-probe/claude-code.md`, günlükte iki `BlockCreated` `author_kind = Agent`, `source = claude-code/2.1.319`, ardından `SnapshotCommitted`; `rootmark blame` blokların katmanını **`proposal`** gösterir (insan onayı bekliyor).
- İletiler: `server/discover` (tam 2026-07-28 `_meta`) → `-32022` (beklenen: sunucu 2026-07-28'i ilan etmez); `initialize` 2025-11-25; `notifications/initialized`; `tools/list` → 11 araç; `tools/call search`; `tools/call propose`. R0'daki `-32602` yok.
- Hata ayıklama günlüğünün rootmark satırları (yollar maskeli) özette; token, içerik ve host değeri kayıtta yok.
