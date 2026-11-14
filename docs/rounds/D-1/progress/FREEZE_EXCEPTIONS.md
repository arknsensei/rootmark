# D-1 — Dondurma istisnaları (docs/DEV_PROCESS.md §4)

Her girdi: ne değişti, neden, önce/sonra kanıtı, gözden geçirici onayı. Gözden geçirici bu turda tek seferdir, R7'de (K-P2.4).

## F1 · R0 · `scripts/release_check.sh` — R21: güncel Claude Code ile MCP sınaması (Ö3)
- **Ne:** yeni madde R21 (betiğin sonunda, özetten önce) ve başlık yorumunda `R1–R21`. R21 `<dist>/claude_probe/*.json` kayıtlarını (`research/p2q2/claude_probe.py --out <dist>/claude_probe`) okur; en az birinde `exit` 0, `rootmark_version` = `rootmark <sürüm>`, 11 araç ve 0.3.1 listesi, `tool_calls`'ta `propose`, ajan olayı ≥ 1, `recorded_at` (UTC; kaydın kendi alanı, dosya zamanı değil) şimdiden en çok 7 gün önce ve geleceğe değil, `claude_code_version` = bu makinedeki `claude --version`. Öteki 21 madde (R1–R20, R4a) aynen; `claude_probe.py` özete `recorded_at` ekler (dondurulmuş değil). `research/FROZEN.md`'ye yeniden dondurma satırı (`D-1/R0-f1:`). Fark: `reports/rounds/D-1/r0/release_check_F1.diff`.
- **Neden:** GOAL D-1 R6 ve D1-10 (yayım kapısı 22/22, R21 yeni), P2-Q.2 doğrulamasının Ö3'ü (CHANNELS §5): her yayımdan önce gerçek ve güncel Claude Code ile araç listesi ve `propose`. Sunucu hata listesi (`server_errors`) koşul değildir: P2-Q.2'nin geçen kaydında da Claude Code'un protokol sürümü müzakeresinin `-32022` yanıtı vardır.
- **Önce/sonra:** `reports/rounds/D-1/r0/r21_freeze.json` — aynı paketle (laboratuvarın eski `dist/`'i, 0.3.1) eski betik 15/21, yeni betik kayıt yokken 15/22 ve R21 dışındaki bütün maddelerin `{id, ok}` listesi eski betiğinkiyle birebir aynı. **Kasıtlı bozma:** kayıt yok, 8 gün önce, `recorded_at` yok, başka Claude Code sürümü, başka rootmark sürümü → R21 FAIL; şimdi ve 6 gün önce (kurulu Claude Code, bu sürüm) → R21 ok.
- **Gözden geçirici onayı:** R7.review.md.
