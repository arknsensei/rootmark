# mdkernel-v2 — Nihai Rapor

RUN: {{run_id}} · Commit: {{git_sha}} ({{dirty}}) · Rust {{rust_version}} · Python {{py_version}} · SEED {{seed}}
Veri: D2 {{d2.rev}} ({{d2.hash}}) · D3 {{d3.rev}} ({{d3.events_hash}}) · D1 {{d1.hash}} · D4 single {{d4.single_n}} / multi {{d4.multi_n}}

> `eval/report.py` tarafından yalnız `reports/runs/{{run_id}}/` içinden üretildi. Elle sayı yoktur. Her tablo altında kaynak alan yolları.

## 1. Özet
| Kod | Ölçüm | Eşik | Sonuç | Durum |
|---|---|---|---|---|
{{summary_rows}}

## 2. Karar kuralı çıktısı
{{decision}}

## 3. H1
### 3.1 THETA × THETA_SPLIT ızgarası (ayar)
{{h1s.grid_table}} · Seçilen: THETA={{h1s.theta}}, THETA_SPLIT={{h1s.theta_split}} · Şekil `figs/h1s_grid.png`
### 3.2 Test, op bazında
{{h1s.per_op_table}}
### 3.3 Gerçek tarihçe
Depo {{h1r.repo}}@{{h1r.rev}} · commit {{h1r.commits}} · çift {{h1r.pairs}} · blok {{h1r.blocks}} · dokunulmamış {{h1r.untouched}}
R1 {{h1r.r1}} · R2 {{h1r.r2}} · hunk içi (bilgi) {{h1r.touched}}
Kayıp kategorileri: {{h1r.loss_categories}} · `other` örnekleri: {{h1r.other_examples}}

## 4. H2
ALPHA seçilen {{h2.alpha}} · Duyarlılık eğrisi: {{h2.alpha_curve_table}} · `figs/h2_alpha.png`
### 4.1 nDCG@10
{{h2.ndcg_table}}
### 4.2 Recall@5/10, MRR, full_cover@10 (structural)
{{h2.recall_table}}
### 4.3 Gecikme ms (structural, p50/p95)
{{h2.latency_table}}
### 4.4 Bootstrap %95 GA
graph_hybrid−hybrid: single {{h2.ci_gh_single}} · multi {{h2.ci_gh_multi}}
graph_flat−flat: single {{h2.ci_gf_single}} · multi {{h2.ci_gf_multi}}
`figs/h2_ndcg.png`

## 5. H3
p50 {{h3.p50}} ms · p95 {{h3.p95}} ms (N={{h3.n}}) · yeniden-chunk kayıp {{h3.lost}}/{{h3.gained}} · olay hash eşit {{h3.log_eq}}

## 6. H4
H4-1: manifest {{h4.manifest_hash}} vs yeniden kurulum {{h4.rebuilt_hash}} → {{h4.eq}}
H4-2: doğrulama aşamasında ölçülür (`VERIFICATION.md`).

## 7. Varsayımlar, park, dondurma istisnaları
{{assumptions_by_hypothesis}}
{{parked}}
{{freeze_exceptions}}

## 8. Serbest yorum (ajan; sayı içermez)
{{commentary}}
