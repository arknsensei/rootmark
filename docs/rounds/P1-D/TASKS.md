# P1-D — Görevler

Sıra: D0 → D1 → D2 → **[P1-C.1 doğrulandı mı?]** → D3 → D4 → D5 → D6 → D7 → D8. D2 bitince `p1c1-final` yoksa D3 PARK, `progress/WAITING.md` yazılır ve ajan durur; etiket gelince aynı istemle devam.

## D0 — Tur açılışı ve CLI taşıma
**Yapar:** `git tag p1c-final`; `ROUND_START`; `server/CLAUDE.md`; `server/Cargo.toml` (`rmcp`, `axum`, `tokio`, `notify`, sabit sürümler ≥ 14 gün); `mdk` ikilisi `server/src/bin/mdk.rs`'e taşınır, `core` ikili üretmez; tüm mevcut CLI testleri yeni yoldan geçer.
**Kabul:** `cargo build --workspace`; eski CLI e2e testleri 0 fark; `check_deps.sh` temiz (`server` → `core`,`index`; tersi yok).

## D1 — MCP sunucusu: yazmayan araçlar
**Yapar:** `rmcp` stdio; `status`, `history`, `blame`, `checkpoints`, `trace`; tüketici kimliği handshake'ten; JSON Schema'lar; `scripts/mcp_e2e.sh` iskeleti.
**Kabul:** Inspector 5/5 şema; e2e 5/5; çıktı deterministik (iki koşturma eşit).

## D2 — HTTP ve servis
**Yapar:** `axum` yerel sunucu, token, `X-Consumer`; `mdk serve` (`--mcp`, `--http`, `--watch`); izleyici → `index.watch`; compact zamanlayıcısı; kapanışta compact.
**Kabul:** D-7 5/5; D-10 (human düzenleme → canonical, sıcakta bulunuyor); `serve` SIGTERM'de compact yapıp çıkıyor (test).

## D3 — `search` ve `read_block` (P1-C.1 sonrası)
**Yapar:** `search` (`include_proposal` K-C1.3 anlamıyla), `read_block`; iz yazımı; boş sorgu → MCP hata yanıtı.
**Kabul:** D-5; sonuç `index` CLI'siyle bayt eşit; Inspector 7/7.

## D4 — `propose`
**Yapar:** yol doğrulama (vault içi, normalize, `..` yok); `append` / `replace_section <heading>`; md yazımı düz; ingest `author_kind=agent`; boyut sınırı (`policy.toml server.max_propose_bytes`, varsayılan 1 MB).
**Kabul:** D-3 4/4; D-11 4/4; `propose` sonrası `events.jsonl` yalnız beklenen olaylar; dosya diff'i yalnız eklenen metin.

## D5 — `promote`, `checkpoint`
**Yapar:** `promote` hook'u ile, `Deny` yanıtı yapılandırılmış; `checkpoint`; CLI `promote --actor human:<id>` mevcut.
**Kabul:** D-4 3/3; D-6 2/2; Inspector 9/9.

## D6 — Ürün sahnesi ajan tarafı
**Yapar:** `scripts/scene_agent.sh`: MCP üzerinden `status → propose → search(varsayılan: yok) → search(include_proposal: var) → promote(denied) → [CLI human promote] → search(var) → trace → checkpoint`; çıktı `reports/rounds/P1-D/scene.json`.
**Kabul:** sahne betiği temiz vault'ta geçer; iki koşturma (zaman damgası/token hariç) eşit.

## D7 — Doğrulama zinciri ve regresyon
**Yapar:** `verify.sh`: conformance → regress (P1-C.1 referansı) → mcp_e2e → http tests → scene → tamper (D-11, token, `X-Consumer`, Inspector şema bozma) → pair compare.
**Kabul:** D-8, D-9.

## D8 — Rapor ve hazır
**Yapar:** `report_p1d.py`; `FINAL.md`; `READY.md`.
**Kabul:** izlenebilirlik; karar kuralı.
