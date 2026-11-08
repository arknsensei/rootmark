# R0 — PQ1-1'in 0.3.1'de yeniden üretimi (Q2-2 (i))

## Ortam
- Claude Code **2.1.319** (`claude --version`), macOS, 2026-11-08.
- rootmark **0.3.1**, yayımlanmış sürüm: `gh release download v0.3.1 -p '*aarch64-apple-darwin*' -p SHA256SUMS`, `shasum -a 256 -c` → OK; arşivin `rootmark` ikilisi. `ROOTMARK_MODELS` = deponun `models/` (`make models`; arşivde yalnız `MANIFEST.json` var).
- Vault: `research/data/d_notes/notes`'un depo dışı (scratchpad) kopyası; `rootmark ingest-dir` (399 belge, `events_hash` `35ba798a639429d33f3ce07005058e85`), `rootmark index build` (6063 kayıt, `minilm-l6-v2/e73168f9+fixed1`). Kişisel içerik yok.

## Komut
`research/p2q2/claude_probe.py --rootmark <0.3.1> --vault <kopya> --work <scratch> --out <scratch> --label r0_v0.3.1`. Betik tek bir başsız oturum açar:

```sh
claude -p "<4 adım: araçları say ve adlandır; search; propose; adım başına SUCCEEDED/FAILED>" \
  --mcp-config <work>/mcp.json --strict-mcp-config \
  --allowedTools mcp__rootmark__search,mcp__rootmark__propose,mcp__rootmark__status \
  --output-format json --debug-file <work>/debug.log
```

`mcp.json` tek sunucu tanımlar: `python3 research/p2q2/mcp_tap.py --log <work>/requests.jsonl -- <rootmark> serve --vault <kopya> --mcp stdio` (kaydedici stdio'yu bayt bayt kopyalar, her iletinin yalnız biçimini yazar).

## Sonuç — yeniden üretildi
- Araç listesi alınamadı: **0 araç**; `search` ve `propose` çağrılamadı ("No matching deferred tools found"); günlükte yeni olay 0.
- Claude Code'un hata ayıklama günlüğü (yollar maskeli): `Successfully connected (transport: stdio)`, `Connection established with capabilities: {…"serverVersion":{"name":"rootmark","version":"0.3.1"},"protocolEra":"legacy","negotiatedProtocolVersion":"2025-11-25"}`, `Failed to fetch tools: request _meta is missing or has malformed required fields: io.modelcontextprotocol/protocolVersion, io.modelcontextprotocol/clientCapabilities`.
- İstemcinin istekleri (`r0_requests.json`, 7 ileti): `server/discover` (tam 2026-07-28 `_meta`'sı) → `-32022`; `initialize` 2025-11-25 → sonuç; `notifications/initialized`; `tools/list` (meta'sız) → `-32602` (yukarıdaki ileti). Çözümleme `DESIGN.md` §1.
- Özet: `reports/rounds/P2-Q.2/claude_code/r0_v0.3.1.json`.

Ön sınama (tasarım dayanağı, Q2-2 kanıtı değil): aynı adımlar rmcp 3.6.0 ile derlenmiş deneme ikilisiyle 11 araç, `search` ve `propose` başarılı, `propose`'un blokları `proposal` (`blame`), aktör kaynağı `claude-code/2.1.319` — `reports/rounds/P2-Q.2/claude_code/pre_rmcp_3.6.0.json`.
