# P2-Q.1 — Bulgular (GOAL dışı)

Sınıflar `reports/rounds/P2-Q/FINDINGS.md`'deki gibi. P2-Q'nün bulgularının (PQ-1..PQ-26) durumu orada.

| Kod | Sınıf | Bulgu | Yeniden üretim | Etki | Hedef |
|---|---|---|---|---|---|
| PQ1-1 | ürün hatası (MCP uyumluluğu) | Claude Code 2.1.318, `rootmark serve --mcp stdio`'ya bağlanıyor ama araç listesini alamıyor: `Failed to fetch tools: request _meta is missing or has malformed required fields: io.modelcontextprotocol/protocolVersion, io.modelcontextprotocol/clientCapabilities` (hata `rmcp 3.5.0`'ın `handler/server.rs`'inden: isteğin `_meta`'sı 2026-07-28+ bir sürüm bildirince o sürümün istek meta alanları zorunlu). Sunucu elle `initialize` + `tools/list` ile 11 aracı veriyor | `docs/rounds/P2-Q.1/progress/R3_agent_isolation.md` "Yan bulgu" (başsız `claude -p --mcp-config … --debug mcp`, kurallı ve kuralsız) | kılavuz §4.1'in Claude Code bağlantısı güncel Claude Code ile araç göstermez; NORTH_STAR'ın ilk ürün sahnesinin ajan yolu (Claude Code `propose`) bu sürümde çalışmaz. 0.3.0'da da aynı (bu turun değişikliği değil) | karar talebi K-P2Q1-T1 (bu turun ürün sınırı dışında: `server/src/mcp.rs` ya da `rmcp` sürümü) |
