# R4 — Gerçek Claude Code ile başsız sınama (yayımlanmış 0.3.2; Q2-2 (iii)'ün R4 yarısı)

## Ortam
- Claude Code **2.1.319** (`claude --version`), macOS, 2026-11-09.
- rootmark: **yayımlanmış 0.3.2**. Kılavuzun §1.1#1–#2 bloklarıyla (`research/p2q/guide_run.py --mode release --only '1.1#1,1.1#2'`, iki blok ok) geçici bir `HOME`'a kuruldu: `rootmark 0.3.2`, GitHub sürümü `v0.3.2` (`c544a9e`, `release.yml` 42574633881).
- Vault: `research/data/d_notes/notes`'un depo dışı kopyası, R0/R1 ile aynı hazırlık. `ingest-dir` 399 belge (`events_hash` `35ba798a639429d33f3ce07005058e85`, R0 ile aynı), `index build` (`minilm-l6-v2/e73168f9+fixed1`). Kişisel içerik yok.

## Komut
R0 ve R1'inkiyle aynı (`research/p2q2/claude_probe.py --label r4_v0.3.2`). Claude Code: `claude -p "<4 adım>" --mcp-config <work>/mcp.json --strict-mcp-config --allowedTools mcp__rootmark__search,mcp__rootmark__propose,mcp__rootmark__status --output-format json --debug-file <work>/debug.log`. Sunucu kaydedicinin arkasında: `python3 research/p2q2/mcp_tap.py --log … -- <rootmark> serve --vault <kopya> --mcp stdio`.

## Sonuç (içeriksiz özet: `reports/rounds/P2-Q.2/claude_code/r4_v0.3.2.json`)
- **CC1 — araç listesi 11/11**, adlar 0.3.1'inkiyle aynı (`tools_match_0_3_1: true`).
- **CC2 — `search` başarılı**: oturumun yanıtı "2. SUCCEEDED: `search` returned 3 hits."; kaydedicide hata yok.
- **CC3 — `propose` başarılı**: `agent-probe/claude-code.md`, günlükte iki `BlockCreated`, `author_kind = Agent`, `source = claude-code/2.1.319`; `rootmark blame` iki bloğu da **`proposal`** gösterir. Yeni olaylar R1'dekiyle aynı türde: önce `CheckpointCreated`, sonra iki `BlockCreated`, sonra `SnapshotCommitted`.
- İletiler R1 ile aynı:
  - `server/discover` (tam 2026-07-28 `_meta`) → `-32022` (beklenen);
  - `initialize` 2025-11-25;
  - `notifications/initialized`;
  - `tools/list` → 11 araç;
  - iki `tools/call` (`_meta`: `claudecode/toolUseId`, `progressToken`).

  `-32602` yok. Oturum çıkışı 0.
- Token, içerik ve host değeri kayıtta yok (`grep` ile yol araması 0).

**CC0 (R0, 0.3.1) → CC1–CC3 (R1 yerel, R4 yayımlanmış 0.3.2): PQ1-1 yayımlanmış sürümde kapandı.**
