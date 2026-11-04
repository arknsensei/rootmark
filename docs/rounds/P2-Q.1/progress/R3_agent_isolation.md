# R3 (b) — Masaüstü ajan ayrımı: sınama

Kılavuz §4.1 "Keep the agent out of `.rootmark/` and the vault's files" yapılandırmasının sınaması (P2-Q doğrulaması öneri 2; Q1-6). Token değeri hiçbir yere yazılmadı: betik çıktıyı token'a karşı denetler ve değeri `<TOKEN>` ile maskeler.

## Ortam
- Claude Code 2.1.318, macOS (Seatbelt sandbox), 2026-11-05.
- Karalama vault'u (depo dışı, scratchpad): bir not; `rootmark serve --http 127.0.0.1:<port> --watch` (0.3.0 ikilisi, `target/release/rootmark`) `.rootmark/server.token`'ı yarattı (64 karakter + satır sonu, `-rw-------`). Sunucu sınamadan önce durduruldu (dosya yerinde).
- Kurallar (`rules.json`, `--settings` ile; kılavuzdaki örnekle aynı, vault yolu karalama dizini):

```json
{
  "permissions": { "deny": ["Read(//<vault>/.rootmark/**)", "Edit(//<vault>/**)"] },
  "sandbox": {
    "enabled": true, "failIfUnavailable": true, "allowUnsandboxedCommands": false,
    "filesystem": { "denyRead": ["/<vault>/.rootmark"], "denyWrite": ["/<vault>"] }
  }
}
```

## Komut
`probe.sh <dizin> <etiket> [settings]`: proje dizininden tek bir başsız oturum, araçlar önceden izinli (`--allowedTools "Bash Read Write Edit"` — red kuralı izni yener, sınanan budur):

```sh
claude -p "<beş adım>" --allowedTools "Bash Read Write Edit" --output-format text [--settings rules.json]
```

Beş adım: (1) Read aracı ile `<vault>/.rootmark/server.token`; (2) Bash `cat … | wc -c`; (3) Bash `python3 -c "print(len(open('…server.token').read()))"`; (4) Bash `echo agent > <vault>/probe-bash.md`; (5) Write aracı ile `<vault>/probe-write.md`. Oturuma içerik değil yalnız uzunluk yazdırması söylendi; ayrıca betik çıktıda token değerini aradı.

## Çıktı
```text
== control (exit 0, token value in output: no)
step 1: SUCCEEDED
step 2: SUCCEEDED
step 3: SUCCEEDED
step 4: SUCCEEDED
step 5: SUCCEEDED
probe-bash.md: written
probe-write.md: written
== rules (exit 0, token value in output: no)
step 1: REFUSED - File is in a directory that is denied by your permission settings
step 2: REFUSED - cat: Operation not permitted (wc printed 0 because cat could not read the file)
step 3: REFUSED - PermissionError: [Errno 1] Operation not permitted
step 4: REFUSED - operation not permitted: notes/probe-bash.md
step 5: REFUSED - File is in a directory that is denied by your permission settings
probe-bash.md: absent
probe-write.md: absent
```

**Sonuç:** kurallı oturum token'ı okuyamadı (Read aracı, `cat`, alt süreç `python3`) ve vault'a doğrudan yazamadı (Bash yönlendirmesi, Write aracı); kuralsız aynı oturum beşini de yaptı. Yalnız `permissions.deny` yetmez: (3) adımını işletim sistemi düzeyindeki sandbox durdurdu (Claude Code belgesi: Read reddi "doesn't apply to … arbitrary subprocesses … like a Python … script"; https://code.claude.com/docs/en/permissions).

## Yan bulgu: Claude Code 2.1.318 rootmark'ın MCP araçlarını listeleyemiyor (GOAL dışı)
Aynı kurallarla ajanın MCP üzerinden `propose` ile yazabildiğini (doğru kanalın açık kaldığını) sınarken: sunucu bağlanıyor, `tools/list` başarısız — kurallar **olmadan** da aynı (`--mcp-config`, `--strict-mcp-config`). Claude Code'un hata ayıklama günlüğü (yol ve zaman kısaltıldı):

```text
MCP server "rootmark": Successfully connected (transport: stdio) in 45ms
MCP server "rootmark": Connection established with capabilities: {"hasTools":true,…,"serverVersion":{"name":"rootmark","version":"0.3.0"},"protocolEra":"legacy","negotiatedProtocolVersion":"2025-11-25"…
MCP server "rootmark" Failed to fetch tools: request _meta is missing or has malformed required fields: io.modelcontextprotocol/protocolVersion, io.modelcontextprotocol/clientCapabilities
```

Hata metni `rmcp 3.5.0`'ındır (`src/handler/server.rs:94`): bir isteğin `_meta`'sı 2026-07-28 ya da sonrası bir protokol sürümü bildirdiğinde sunucu o sürümün bütün istek meta alanlarını ister. Elle `initialize` (`2025-06-18` ve `2025-11-25`) + `tools/list` (meta'sız) rootmark'tan 11 aracı alıyor; yani sunucu çalışıyor, uyumsuzluk Claude Code'un yeni istemcisinin istek meta'sıyla `rmcp 3.5.0`'ın doğrulaması arasında. Etki: kılavuz §4.1'in Claude Code bağlantısı bu sürümle araç göstermez — NORTH_STAR'ın ilk ürün sahnesinin (Claude Code `propose`) yolu. Düzeltme `server/src/mcp.rs` ya da `rmcp` sürümüdür (bu turun ürün sınırı dışında, GOAL "Ürün değişikliği yalnız izleyicide"): bulgu `reports/rounds/P2-Q.1/FINDINGS.md` PQ1-1, karar talebi `docs/DECISIONS.md` K-P2Q1-T1. Kılavuzun "Known issues"una yazıldı. MCP sunucuları sandbox dışında koştuğundan (belge) yapılandırma `propose`'u engellemez; bunun sınaması uyumluluk düzelince yapılır.
