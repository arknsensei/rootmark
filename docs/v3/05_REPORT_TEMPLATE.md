# mdkernel-v3 — Nihai Rapor

RUN: {{run_id}} · Commit: {{git_sha}} ({{dirty}}) · Rust {{rust_version}} · Python {{py_version}} · SEED {{seed}}
Veri: D2 {{d2.rev}} ({{d2.hash}}) · D3 {{d3.rev}} (v2 {{d3.events_hash}} / v3 {{d3.events_hash_v3}}) · D1 {{d1.hash}} · D1-easy(K4) {{d1k4.hash}} · D1-hard {{d1h.hash}}

> `eval/report_v3.py` tarafından yalnız `reports/runs/{{run_id}}/` içinden üretildi. Elle sayı yoktur. Her tablo altında kaynak alan yolları. Eşikler `docs/v3/00_GOAL.md`'den aynen kodlanmıştır.

> **K4 tanım değişikliği (sonuç görüldükten sonra alınmış tasarım kararı):** v3'te bölme/birleştirmede baskın parça soyu korur (`docs/ARCHITECTURE.md` §10 K4). Bu, v2 sonuçları görüldükten sonra alınmıştır ve gerçek tanımını değiştirir; H1-S2/H1-H2 yanlış eşleme paydası buna göre yeniden yazılmıştır (merge sonucu paydada değildir). v2 tanımlı S2 bilgi amaçlı ayrıca raporlanır; v2 davranışının bayrakla birebir korunduğu H4-V2 ile gösterilir.

## 1. Özet
| Kod | Ölçüm | Eşik | Sonuç | Durum |
|---|---|---|---|---|
{{summary_rows}}

## 2. Karar kuralı çıktısı
{{decision}}

## 3. H1-easy — D1-easy (K4 gerçek)
{{h1_easy.table}}
v2 tanımlı S2 (bilgi): {{h1_easy.s2_v2}} · Op bazında: {{h1_easy.per_op_table}}

## 4. H1-hard — D1-hard
### 4.1 THETA × THETA_SPLIT ızgarası (ayar, D1-hard ayar bölünmesi)
{{h1_hard.grid_table}}
Seçilen: THETA={{h1_hard.theta}}, THETA_SPLIT={{h1_hard.theta_split}} · H1-G (farklı amaç değeri) {{h1_hard.g}}
### 4.2 Test
{{h1_hard.table}}
v2 tanımlı S2 (bilgi): {{h1_hard.s2_v2}} · Op bazında: {{h1_hard.per_op_table}}

## 5. H1-real — D2 gerçek tarihçe (eşleyici v3)
Depo {{h1r.repo}}@{{h1r.rev}} · commit {{h1r.commits}} · çift {{h1r.pairs}} · blok {{h1r.blocks}} · dokunulmamış {{h1r.untouched}}
R1 {{h1r.r1}} · R2 {{h1r.r2}} · hunk içi (bilgi) {{h1r.touched}}
Kayıp kategorileri: {{h1r.loss_categories}}

## 6. H3 (eşleyici v3)
p50 {{h3.p50}} ms · p95 {{h3.p95}} ms (N={{h3.n}}) · yeniden-chunk kayıp {{h3.lost}}/{{h3.gained}} · olay hash eşit {{h3.log_eq}}

## 7. H4
H4-1 (v3): manifest `events_hash_v3` {{h4.manifest_hash}} vs yeniden kurulum {{h4.rebuilt_hash}} → {{h4.eq}}
### 7.1 H4-V2 — v2 davranışının yeniden üretimi
D3 `--matcher v2` `events_hash`: {{h4v2.d3_hash}} (beklenen {{h4v2.d3_expected}}) → {{h4v2.d3_eq}}
v2 D1, v2 metrikleri, git'teki `{{h4v2.ref_run}}/h1s.json` ile alan alan: {{h4v2.fields}} alan, farklı {{h4v2.diff}} → {{h4v2.h1s_eq}}
H4-2: doğrulama aşamasında ölçülür (`scripts/verify.sh pair`).

## 8. Varsayımlar, park, dondurma istisnaları
{{assumptions_by_hypothesis}}
{{parked}}
{{freeze_exceptions}}

## 9. Serbest yorum (ajan; sayı içermez)
{{commentary}}
