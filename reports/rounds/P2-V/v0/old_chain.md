# P2-V/V0 — Eski zincirin referansı (`verify.yml`, `ROUND_START` `352f790`)

- **Koşu:** `40582139980` (`workflow_dispatch`, `main` = `352f790`, `ubuntu-latest`, eski `scripts/verify.sh pair`, değişmeden), 2026-10-27 20:54 → 2026-10-28 00:55 UTC, **kırmızı** (beklenen; K-P2R.5).
- **Yapıt:** `reports/rounds/P2-V/ci/40582139980/verify-pair.tar.xz` (yapıtın tamamı: `verify.log`, iki zincirin `*_checks.json`'ları ve `pair_compare.json`; açılmış hâli 234 MB, sıkıştırılmış 1.7 MB), `pair_compare.json` (dondurulmuş `compare_p2r pair`), `fail_summary.txt` (iki zincirin sınıflandırıcı özeti).
- **Zincirler:** A `20261027T210352Z-352f790` 6741 s, B `20261027T225551Z-352f790` 6237 s; çift karşılaştırması 788 033 alan.

## Farklar ve K-P2R.5 teşhisi

K-P2R.5'in iki nedeni: (1) **D-A yarışı** — `p2_0_2/d_a.json`'da bazı satırların `new_blocks`, `pass_0a`, `unverified` alanları ve onu özetleyen alanlar (dondurulmuş `Served.quiesce` 0.4 s); (2) **R17 host yolu** — `release_check.json` `items›17›detail` klon dizinini yazar.

| Dondurulmuş karşılaştırıcının farkı | Alan sayısı | Neden |
|---|---|---|
| `p2_0_2/d_a.json`: `new_blocks`, `rows›0›new_blocks`, `rows›0›pass_0a`, `rows›0›unverified` | 4 | (1) D-A, satır 0 |
| `p2_0_2_checks.json:H-P2-0·2a›d_a›new_blocks` | 1 | (1) D-A'nın özeti |
| `p2_0_2_checks.json:regress›p2_0_1_sets›d_a›differing›…` | 18 | (1) D-A'nın P2-0.1 RUN'ına karşı regress'i |
| `p2a_checks.json:regress›differing_fields›p2_0_2/d_a·json:…`, `…p2_0_2_checks·json:H-P2-0·2a…`, `regress›excluded_differences›p2_0_2_checks·json:regress›leaves` | 6 | (1) D-A'nın P2-A özeti |
| `p1e1/release/release_check.json:items›17›detail` | 1 | (2) R17: A'da `…/verify/A/repo/target/p1e1-dist/bin`, B'de `…/verify/B/…` |
| `p2_0_2_checks.json:regress›differing_fields›p1e1/release/release_check·json:items›17›detail›b` | 1 | (2) R17'nin P2-0.2 regress'indeki izi |
| bayt: `p2_0_2/d_a/events.jsonl` | 1 dosya | (1) D-A'nın günlüğü |

**Toplam 31 alan + 1 dosya; hepsi bu iki nedenden.** Başka hiçbir dosya ya da alan farklı değil: uyumluluk, P1-A…P2-R'nin bütün öteki kayıtları, `p2b/*`, `p2r/*` ve bayt karşılaştırılan öteki vault günlükleri eşit. Ürün kodundan gelen fark yok. **Teşhis tutuyor** (STOP gerekmedi).

Bu koşu, CI 40265181721 (2026-10-26, `ae7e692`) ve P2-R tam çiftinin 1. ve 3. denemesi (`reports/rounds/P2-R/full_attempt1/`, `runs/20261026T222501Z-5b235b0/`) birlikte: dört çiftin farklarının hepsi zamanlamaya duyarlı kümelerde (D-A, D-A2, D-C ve onları özetleyen alanlar) ya da R17'nin host yolundadır (envanter `observed_differences`).

## Sayılan FAIL (bilgi; eski zincirin hükmü değil)

Her zincirde `eval.historical_p2r` 16 sayılan FAIL (23 tarihsel, 2 göç gerekli). Mekanizmaları (V1 envanterinin sınıfları):
- **Z:** H-P2-0.2a, H-P2-0.3a, `logs_vs_p2a` (D-A ve günlükleri).
- **O — gerçek Obsidian:** E1-3, E1-9 (e2e adımı), H-P2-0.3b, b-yük, c, d, e (koşucuda Obsidian yok: `skipped`), p1e1 `tamper` (gerçek Obsidian vakaları koşmadı).
- **O — yayım Mac'ine bağlı referans:** C1-8 (P1-C RUN'ına karşı vektör dosyaları ve sıralama skorları), D-9 ve D1-12 (bildirdikleri her fark C1-8'in alanında), E-7 (yerel yayım hedefi `x86_64` paketi `aarch64-apple-darwin` etiketiyle), E1-4 (kontrol listesi: R4 tarihsel, R15 gerçek Obsidian, R17 host arşivi).

Hiçbiri D sınıfında bir determinizm kaybı değildir; kademe 2 bunları envanterle ayırır (`eval.tier2 fails`), Mac kümesi ölçer.
