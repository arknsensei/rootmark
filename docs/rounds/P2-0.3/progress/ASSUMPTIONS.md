# P2-0.3 — Varsayımlar

Etiketler: `[spec]` spesifikasyon yorumu, `[arch]` mimari/süreç yorumu, `[-]` yerel uygulama kararı, `[ön-kayıt]` ölçümden önce sabitlenen tanım.

## R0 — Ön-kayıt (D-B test-4'ün hiçbir oturumu koşmadan önce yazıldı)

- **R0-1 [arch] Dal tabanı ve `ROUND_START`.** PR #15 (P2-A) 2026-10-23T23:19Z'de rebase ile `main`'e girdi (`gh pr view 15`: `MERGED`); `origin/main` = `c50ee2f` (P2-A/close, K-P2A.5, K-P2A.6, K-Q.1 ve bu turun dosyaları dahil); `main`'deki CI koşusu (`ci`, 40038986381) üç işte yeşil. Dal `p2-0.3/round` bu commit'ten açıldı; `ROUND_START` = `c50ee2f04f8962a436a113440d1abfbcb363ebff` (K-P2.2). Ajan PR'ı birleştirmez.
- **R0-2 [ön-kayıt] D-B test-4'ün tohumu.** `seed = int(blake3("P2-0.3/D-B").hexdigest()[:8], 16)` = **`2671050228`** (onaltılık `9f34fdf4`); P2-0.2 X0-4'ün türetimi, yalnız etiket değişir. Yeniden kullanılmayan: D-B test-3'ün tohumu `1793744460` (= `blake3("P2-0.2/D-B")[:8]`; P2-0.2 resmi ölçümü ve P2-A bilgi ölçümü `reports/rounds/P2-A/info_d_b_test3.json` bunu kullandı). Sabit `research/eval/run_p2_0_3.py` `SEED`; `research/tests/test_p2_0_3_tools.py` değeri sınar.
- **R0-3 [ön-kayıt] Üreteç ve dağılım P2-0.2'ninkiyle birebir.** D-B test-4, P2-0.2'nin test-3 üretecidir (`eval.datasets.p2_0_2_sets`: `vault3` — X0-5, X0-17 —, `d_b3` — X0-4, X0-6 —, `coverage` — X0-16 —, `classify` — X0-7, X0-18 —; bağlantı ayarları X0-15 harness'te), tohum dışında hiçbir şey değişmez: tohum hem vault'u hem oturumları belirler (ikisi de `d_b_seed()`'den türer), dolayısıyla test-4 yeni bir vault ve yeni 200 oturumdur; biçim karışımı, oturum türü oranları (yeniden adlandırma %35) ve kurallar aynıdır. `ROUND_START`'taki dosyalar (BLAKE3, ilk 32 onaltılık; çalışma ağacında eşit):

  | Dosya | BLAKE3 | P2-0.2 ölçülen ağaca (`4513203`, `72fd78f`'in rebase karşılığı) göre |
  |---|---|---|
  | `research/eval/datasets/p2_0_2_sets.py` | `a4b4426f6418cc92a889d4dcac7aa6e8` | aynı |
  | `research/eval/datasets/p2_0_1_sets.py` | `732f74c4e20a7199d7d6d19e5677622f` | aynı |
  | `research/eval/datasets/p2_0_sets.py` | `eb115cf56503ceb1b26c1fdebd615e1b` | aynı |
  | `research/eval/run_p2_0_2.py` | `e168f9ac02357e77f4779578e98a562e` | aynı |
  | `research/eval/run_p2_0_2_checks.py` | `b9473b4a1f3630a17e499d057e06d870` | aynı |
  | `research/eval/run_p2_0_1.py` | `ba5ba107d0c2101d8253f111c0b207ce` | aynı |
  | `research/eval/run_p2_0.py` | `ed42f9543f78ca8cbc9d22aa0fb0b730` | +3 satır (P2-A/F1: tarifin varsayılan `[attestation]` tablosu kaldırılır; politika önce/sonra eşit) |
  | `research/eval/agent_fs_writer.py` | `9fae2f79e132f1666a3d4e1d68805442` | aynı |
  | `research/eval/real_retry.py` | `bef6b2cee2a11ebe93f6953b4a7bc9d2` | yeni (P2-A/A0 e) |
  | `clients/obsidian/e2e/p2_0_2/sessions.ts` | `2d722746ca1294056828706527e0d49d` | +2 satır (P2-A/F1) |
  | `clients/obsidian/e2e/p2_0_2/real.ts` | `a4afaa617c7b464e084acf98f743cf89` | +2 satır (P2-A/F1) |
  | `docs/rounds/P2-0/progress/Z5_tuning.json` (W) | `6fa12ec6477042efba96d3aa08474613` | aynı |

  Kapsam denetimi (X0-16; yalnız üreteç, hiçbir oturum koşmadan): tohum `2671050228`, 200 oturum; bağlantı sayıları `wiki 8, wiki_alias 7, wiki_heading 8, embed 7, md_name 8, md_label 7, self_wiki 2, self_wiki_heading 2, self_md 2, md_path 4`; oturum türleri `char 22, paragraph 22, multi 22, rapid 22, paste 21, focus 21, rename 40, rename_again 15, move_folder 15`; yeniden adlandırma 70 (kendine bağlantılı 35, klasörde 39, gelen Markdown bağlantısı 45); sıfır sayı yok (`ok: true`).
- **R0-4 [-] Kümelerin kaynağı.** Zincir (`scripts/verify.sh chain`) P2-A'daki gibi `make p2a-checks`'i koşar; bu, P2-0.2'nin denetimlerini gerçek-Obsidian kümeleri olmadan aynı RUN'ın `p2_0_2/` dizinine yazar (rules, D-E, D-A, D-A2, D-C, D-B2; P2-0.2'nin kodu). P2-0.3'ün yeni adımı (`make p2-0-3-checks`) bunlardan sonra yalnız gerçek-Obsidian kümelerini koşar — D-B test-4, D-A′, D-C′ — ve `p2_0_3/`'e yazar; Obsidian'sız kümeler ikinci kez koşmaz (aynı kod, aynı RUN; çift sürede bir saat kazanır). H-P2-0.3a, a′, d'nin D-C yarısı, f, h, j bu yüzden `p2_0_2/*.json`'dan okunur; b, b′, c, b-yük, d′, e `p2_0_3/*.json`'dan.
- **R0-5 [ön-kayıt] Eşiklerin uygulanışı.** `eval.run_p2_0_3 checks`, P2-0.2'nin `run_p2_0_2_checks.checks`'ini (P2-0'ın eşikleri; GOAL P2-0.3'ün tablosuyla aynı) RUN'ın bir görünümüne uygular: görünümde `p2_0_2/{d_b,d_a_prime,d_c_prime}.json` bu turun kayıtlarıdır; kodlar `H-P2-0.2*` → `H-P2-0.3*`. P2-0.2'nin `k`'si (yazılı kurallar) P2-0.3'ün hipotezi değildir, `rules` adıyla yalnız kaydedilir. **H-P2-0.3b-yük**: aynı oran ≤ 0.02 ve 200 oturum, **yüklü çiftin iki zincirinin ikisinde de** (rapor iki zincirin kaydını okur). **H-P2-0.3i**: iki çiftin `pair_compare.json`'u (`eval.compare_p2a pair`: P2-0.2 çift kümesi + P2-A dosyaları; D-A, D-A2, D-C `events.jsonl`/`attestations.jsonl` bayt eşit) — `equal` ve `h_p2_0_2i.equal` ikisi de. `p2_0_3/` dosyaları zamanlamaya bağlı kayıtlardır, çift karşılaştırmasına girmez (P2-0.2'de de girmiyordu). Resmi değer yüksüz çiftin A zinciridir (resmi RUN); dört zincirin değeri raporlanır.
- **R0-6 [arch] "Ürün değişmedi" kanıtı ve tur geçersizliği.** GOAL "değişirse tur geçersizdir": kanıt R2'de `git diff main -- core index server clients spec` ve üreteç/harness/`run_p2_0_2.py` yollarının boş olmasıdır (FINAL). Ayrıca her zincirde D-A, D-A2, D-C günlüklerinin P2-A resmi RUN'ıyla (`reports/rounds/P2-A/runs/20261020T230453Z-1a775d3`) bayt eşitliği `logs_vs_p2a` olarak kaydedilir ve raporlanır; bu bir karar dalı **değildir** — P2-A'nın geliştirme RUN'ında D-A harness'inin bir zamanlama yarışı görülmüştü (P2-A FINAL not 4), tek zincirdeki yarış H-P2-0.3i'nin (çift) konusudur. Rapordaki "tur geçersiz" dalı yalnız ön-kayıtlı olmayan bir D-B tohumu içindir.
- **R0-7 [-] Önceki tohumlar okunmaz.** Ölçüm sürecinde `p2_0_2_sets.d_b3` bir bekçiyle sarılır: `SEED` dışındaki tohumla çağrılırsa hata verir (`run_p2_0_3._guard`); D-B test-3'ün oturumları üretilmez. D-A′ P2-0.2'nin kümesidir ve tanımı gereği (X0-8) test-3 **vault'u** üzerindedir; bu, D-B test-3'ün oturum kümesinin yeniden kullanımı değildir ve GOAL'ün "D-A′ … aynen"ini uygular. `checks` kayıtlı tohumu `SEED`'e karşı denetler (`seed`), P2-A bilgi ölçümünün dosyası yalnız raporun karşılaştırma sütununda okunur.
- **R0-8 [-] Yeniden deneme.** Üç gerçek-Obsidian kümesi `eval.real_retry` (P2-A/A0 e) altında koşar: yalnız asılı kayıt iletilerinde, küme başına en çok bir yeniden deneme, taze çalışma dizininde; ölçülen sonuç son denemenin tamamıdır ve `retries` RUN'da kayıtlıdır (P2-0.2'nin resmi zincirinde yeniden deneme yoktu; P2-A bilgi ölçümünde vardı, kullanılmadı).
- **R0-9 [-] Rapor üreteci.** TASKS "`report_p2_0_3.py` aynı biçimde": `report_p2_0_2`'nin tablo ve yardımcıları (`_table`, `_ok`, `_f`, `_load`) değişmeden içe aktarılır; karar kuralı GOAL P2-0.3'ünküdür (P2-0.2'ninki farklı olduğu için yeniden kullanılamaz) ve tam: adı geçmeyen bir kalış (ör. yalnız 0.3c) → "tanımlanmamış kalış, tur kaldı, karar talebi"; ölçülmemiş bir hipotez → karar bekler. Dalların birim testi `test_p2_0_3_tools.py`.
- **R0-10 [arch] Ortamlar ve sıra.** Resmi: yüksüz çift (`make verify`); ikinci ortam: `VERIFY_LOAD=1 make verify` (mantıksal CPU sayısı kadar `yes`), aynı commit'te, yüksüz çiftten sonra. Çiftler sırasında bu makinede başka bir Obsidian deneyi yapılmaz; harness'ler kendi Obsidian örneklerini ayrı veri dizinleriyle açar.
- **R0-11 [-] Geliştirme denemesi.** Sarmalayıcı ve rapor, P2-A resmi RUN'ının bir kopyası üstünde (`research/reports/runs/20261024T00000{1..4}Z-c50ee2f`, git dışı; gözden geçirmeden sonra silindi, O1) denendi: Obsidian'sız yol (üç küme `skipped`, hipotezler ölçülmedi), gerçek Obsidian'da yalnız **D-C′** (regresyon kümesi; 40 tik, insan 40/40 `canonical`, ajan 0, yeniden deneme yok) ve raporun dört-zincir biçimi (P2-A bilgi ölçümünün kaydından uydurulmuş bir D-B ve P2-0.2 resmi RUN'ının `d_a_prime.json` kopyasıyla; yalnız biçim denemesi, hiçbir değeri raporlanmaz). **D-B test-4'ün hiçbir oturumu resmi çiftten önce koşmadı.**
- **R0-12a [-] Sarmalayıcının özü (TASKS R0 "kod gösterilir"; `research/eval/run_p2_0_3.py`).** Tohum yalnız D-B çağrısında değişir; P2-0.2'nin kodu `d_b_seed()`'i her çağrıda modül üzerinden okur:

  ```python
  SEED = int(blake3.blake3(b"P2-0.3/D-B").hexdigest()[:8], 16)

  def run_d_b4(rootmark, obsidian, plugin, window_s, work):
      with _seed(SEED):                       # sets2.d_b_seed = lambda: SEED, yalnız bu blokta
          r = r2.run_d_b(rootmark, obsidian, plugin, window_s, work)   # P2-0.2, aynen
      ...                                     # kayıtlı tohum SEED değilse hata
      r["set"] = "D-B test-4"

  # test: her küme real_retry altında; D-A′/D-C′ = r2.run_real(s, …) aynen; süreç boyunca _guard()
  # checks: run_p2_0_2_checks.checks(görünüm) — p2_0_2/{d_b,d_a_prime,d_c_prime}.json → p2_0_3/'ün kayıtları
  ```

  `report_p2_0_3.py`: `from eval.report_p2_0_2 import _f, _load, _ok, _table` (aynen) + GOAL P2-0.3'ün karar kuralı (`decide`).
- **R0-12b [ön-kayıt] H-P2-0.3c'nin koşulu.** P2-0.2'nin kodu 0.2c'yi `p95 < 500 ms` **ve** `latency_p95_over_window = false` ile değerlendirir (P2-0'ın tanımı; P2-0.2'de de böyleydi). GOAL P2-0.3'ün tablosu yalnız `< 500 ms` yazar; W = 0.5 s olduğu için `p95 < 500 ms` ikinci koşulu zaten gerektirir, değerlendirme aynen kalır (R0 gözden geçirme O3).
- **R0-12c [-] Ölçülmemiş hipotez.** Bir hipotez ölçülmemişse (Obsidian ikilisi yok, eksik zincir) rapor başka kalışlara bakmadan "karar bekler" der (R0 gözden geçirme O2): kural eksik ölçümle uygulanmaz. Resmi zincirlerin gerçek Obsidian ikilisiyle (`/Applications/Obsidian.app`) koştuğu FINAL'de yazılır (O5).
- **R0-12 [-] Karar numaraları.** Bu turun kararları `K-P2-03.<n>`, karar talepleri `K-P2-03.<n>-T`.
