# P2-Q.1 — Rapor (TASLAK: tur insan adımında bekliyor)

**Durum (2026-11-05):** bu bir ara rapordur, R5'te tamamlanır. Tur, 0.3.1'in yayımının insan adımlarını bekliyor (`docs/rounds/P2-Q.1/progress/HUMAN_STEP.md`: PR #51'in birleşmesi, `release.yml`, `v0.3.1` etiketi, GitHub sürümü). Yayıma bağlı ölçümler (R4 (d): D-NOTES Mac/Docker/CI ve Obsidian sahnesi O06/O10/O14 yayımlanmış 0.3.1 ile; Q1-8'in `release.yml` ve yayım kısmı) ve R5 (gözden geçirici, kapılar, READY, turun PR'ı) yapılmadı. **READY yazılmadı; doğrulama için hazır değil.** Karar kuralının dalı R5'te türetilir.

`ROUND_START` = `2a0e3a9`; dal `p2-q.1/round` (son itilen commit bu dosyanın commit'i); yayım dalı `release/0.3.1` (`f5002de`, PR #51).

## Ne yapıldı
- **PQ-22 (Linux'ta yeni dizin):** izleyici yeni oluşturulan ya da vault'a taşınan dizini tarar (`serve.rs`). `watch_newdir` N1–N6 (N1–N3 20'şer tekrar, 240 belge her biri tam bir kez): macOS 3/3; Linux (Docker arm64, inotify) **`ROUND_START` ağacında kaldı** (PQ-22 yeniden üretildi), R1'de 3/3; PR #51'in CI'ında `make test` macOS ve `ubuntu-latest`'te yeşil.
- **PQ-18 (silme ve taşıma):** kaldırılan md dosyası bloksuz belge olarak alınır — spec §5 adım 7 (`BlockDeleted`), aktör/kaynak §7, tasdiksiz `unattested_fs` §7.1; kaldırma `W + 500 ms` bekletilir ki tasdikli yeniden adlandırma eski yolun başına karşı denetlensin (çift işlem yok); durdurmada bekleyenler alınır (`serve.rs`, `tools.rs` izleme alımı). `watch_remove` D1–D10 macOS 9/9, Linux 9/9; `make test-rust` temiz. Tasarım: `docs/rounds/P2-Q.1/progress/DESIGN.md` (koddan önce, §1.1 geri-adlandırma eki dahil).
- **Belgeler:** kılavuz §5.1 VF-1 (PQ-24), PQ-25/26; §4.1 masaüstü ajan ayrımı (izin kuralları + sandbox) — sınandı: kurallı başsız Claude Code oturumu 5/5 reddedildi, kuralsız 5/5 yaptı (`R3_agent_isolation.md`); CHANGELOG 0.3.1; Known issues.
- **Güvenlik:** `release.yml` en az yetki; CodeQL harness uyarıları gerekçeyle "test kodu" (GitHub'da kapatma insan adımı); `minimatch` 14 gün kuralıyla gerekçeli açık (`R3_security.md`).
- **Kademe 2:** `41927841456` (`5dc015c`) D farkı 0, portable 0, **sayılan FAIL 1** (E1-5 `workflow_r13`: R3'ün `release.yml` değişikliği, R0'da adsız); düzeltme `29ea0b0`; `41944358184` (`110d243`) D farkı 0, sayılan FAIL 0, portable 0.
- **Mac kümesi** (`5dc015c`, 3 tekrar): Z kümeleri 3/3; O: 0.3c/0.3d/0.3e/E1-3/E1-9/tamper 3/3, C1-8 1/1; E-7 tarihsel eşit; **E1-4 kayıt farklı** (R1, R13: R3'ün CHANGELOG/`release.yml`'si, `29ea0b0` ile giderildi; yayım paketinde R1, R13 ok); `reported_only` 0.3b 0.247–0.250 (0.3.0 izleyicisiyle aynı harness 0.238 → PQ1-2, bu turun değişikliğinden değil).
- **0.3.1 yayım paketi:** `release_check` 21/21 (R15 gerçek Obsidian 11/11; R16 dondurma istisnası F1 ile); PR #51.
- **D-NOTES aday denemesi** (yerel paket, hükme girmez): Mac ve Docker 19/23, H09 20/20, Mac ↔ Docker ↔ P2-Q 0.3.0 Mac bayt eşit.

## Kriterlerin şimdiki durumu (R5'te kesinleşir)
| Kod | Durum |
|---|---|
| Q1-1 | DESIGN/TESTS R1'in kodundan önce commit'li (`4168bd3`, `2153791`); olay biçimi atıflı; spec'in tanımlamadığı durum yok → geçti beklenir |
| Q1-2 | test yarısı: macOS + Linux (Docker arm64; CI `ubuntu-latest` PR #51'de) yeşil; **yayım yarısı bekliyor** (D-NOTES CI iki koşu, Mac ↔ Docker ↔ CI 0.3.1 ile) |
| Q1-3 | test yarısı: macOS + Linux yeşil; **yayım yarısı bekliyor** (Obsidian O06/O10/O14) |
| Q1-4 | ilk tetikleme `41927841456`'da sayılan FAIL 1 ve adsız fark (turun kendi `release.yml` değişikliği) → GOAL "bir tetikleme" lafzıyla **kaldı**; düzeltmeden sonra `41944358184` temiz. Karar talebi R5'te. Mac kümesi kayıtlı; E1-4 farkı adıyla |
| Q1-5 | ürün farkı yalnız `server/src/serve.rs`, `server/src/tools.rs` ve yeni testler; dondurulmuş değişiklik yalnız F1 (`release_check.sh`); conformance 39/39 (yayım paketi R5) — R5'te `git diff` ile kesinleşir |
| Q1-6 | VF-1 (PQ-24), PQ-25/26, §4.1 sınandı, CHANGELOG ve Known issues → geçti beklenir |
| Q1-7 | `release.yml` en az yetki; harness uyarıları gerekçeli (kapatma insan adımı); Dependabot gerekçeli → R5 |
| Q1-8 | yerel R1–R20 21/21; **`release.yml` ve `v0.3.1` insan adımı bekliyor** |
| Q1-9 | R5'te (kapılar, kademe 1, gözden geçirici) |

## GOAL dışı bulgular
PQ1-1 (Claude Code 2.1.318 rootmark'ın MCP araçlarını listeleyemiyor; karar talebi K-P2Q1-T1), PQ1-2 (0.3b bu Mac'te 0.24; 0.3.0'da da) — `reports/rounds/P2-Q.1/FINDINGS.md`.
