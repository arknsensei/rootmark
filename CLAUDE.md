# mdkernel-v2 — Ajan Çalışma Kuralları

Bu depo, markdown-doğal versiyonlu blok çekirdeği için **ikinci hipotez doğrulama turu**dur. Çekirdek bu kez Rust ile yazılır; arama ve ölçüm Python'da kalır. Amaç ürün değil, bağımsız olarak **yeniden üretilebilir** ölçümdür.

v1'den çıkan dersler `docs/06_LESSONS.md` içindedir ve bu dosyadaki kurallara dönüştürülmüştür. Gerekçesini merak edersen oraya bak; kuralı uygula.

## Okuma sırası (her oturum başında)
1. `docs/00_GOAL.md` — hipotezler, önceden kayıtlı eşikler, karar kuralı
2. `docs/01_SPEC.md` — veri modeli ve Rust API; yorum alanı yok
3. `docs/02_TASKS.md` — görev grafı
4. `docs/03_EVAL.md` — ölçüm sözleşmesi
5. `docs/progress/` — nerede kaldığını buradan anla

Sadece çalıştığın görevin belgelerini bağlama al.

## Teknik standart
- **Rust:** stable toolchain, `rust-toolchain.toml` ile sabit sürüm. Crate'ler `Cargo.lock` ile kilitli. `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` her görevde temiz.
- **Python:** 3.11+, `uv`, `pyproject.toml` sabit sürümler. `pytest`, `ruff`.
- **Köprü:** `pyo3` + `maturin` (`maturin develop --release`). Python tarafı Rust çekirdeğine yalnızca `mdkernel_core` modülü üzerinden erişir; Python'da parser/eşleyici/chunker **yeniden yazılmaz**.
- Hash: `blake3`, hex ilk 32 karakter. Embedding: `sentence-transformers/all-MiniLM-L6-v2`, CPU, `normalize_embeddings=True`. Vektör: `lancedb`. BM25: `rank_bm25`. Graf: `networkx` (PPR için `scipy.sparse` ile kendi güç iterasyonu, bkz. SPEC §9). Token: `tiktoken` (Python) ve `tiktoken-rs` (Rust) — **aynı** `cl100k_base`; T7'de ikisinin aynı sayıyı verdiği test edilir.
- `SEED=20261013`. Tüm rastgelelik tohumlu. Zaman damgaları hiçbir kimliğe girmez.
- Ağ: PyPI, crates.io, GitHub (sabit rev klon), HuggingFace, Anthropic API (yalnız T12).
- Dizin:
  ```
  core/            # Rust crate: mdkernel-core (lib + mdk CLI)
  bindings/        # pyo3 crate: mdkernel_core (Python modülü)
  mdkernel/        # Python: embedding, indeks, arama
  eval/            # ölçüm (dondurma kurallarına tabi)
  data/            # git-ignored, üreteçlerle kurulur; MANIFEST dosyaları git'te
  reports/         # runs/<run_id>/... ve FINAL.md
  docs/progress/
  tests/           # Python testleri; Rust testleri core/ içinde
  ```

## Görev disiplini
- Görev = `02_TASKS.md` içindeki tek `T<nn>`. Sırayla. Bir görev yalnız **durma koşulu** nedeniyle `PARK` edilebilir; park edilen görev `docs/progress/T<nn>.md` içine `PARK: <neden>` ve hangi görevlerin ona bağımlı olduğu yazılarak atlanır; bağımlı görevler de park edilir. Park nedeni çözülünce sırası geldiği yerden değil, **park edildiği yerden** devam edilir ve sonrasındaki görevler yeniden koşturulur.
- Başlangıç: `docs/progress/T<nn>.md` → `BAŞLADI` + 5-10 satır plan.
- Bitiş: kabul ölçütleri tek tek işaretlenir → `TAMAMLANDI`/`KISMİ`.
- Commit: `T<nn>: <açıklama>`. Dondurma commit'i tam olarak `T9: freeze eval` mesajıyla.
- Gözden geçirici alt-ajan: her görev sonunda, yalnız kabul ölçütleri + değişen dosya listesiyle; `T<nn>.review.md` yazar; kod yazmaz. "Sağlanmadı" → göreve dön. 3 turda çözülmezse durma koşulu.
- Bir sonraki göreve geçiş: kendi kaydın **ve** gözden geçirici kaydı `TAMAMLANDI`.

## Ölçüm dürüstlüğü
1. `00_GOAL.md` eşikleri değiştirilemez.
2. **Dondurma:** `T9: freeze eval` sonrası `eval/metrics.py`, `eval/datasets/*.py` (o anda var olanlar) ve `eval/truth.py` değiştirilemez. T9 sonrası eklenen yeni dosyalar (`eval/datasets/questions.py`) eklendikleri commit'ten sonra dondurulmuş sayılır.
   **Tek istisna — uyumluluk düzeltmesi:** bir bağımlılığın API değişikliği yüzünden kod çalışmıyorsa, davranışı değiştirmeyen düzeltme yapılabilir; şartlar: (a) `docs/progress/FREEZE_EXCEPTIONS.md` içine girdi (ne, neden, diff), (b) `tests/fixtures/freeze_fixture/` üzerinde düzeltme öncesi ve sonrası metrik çıktısının **bit düzeyinde aynı** olduğunun kaydı, (c) gözden geçirici onayı aynı girdide. Bunlardan biri eksikse düzeltme yapılmaz, `EVAL_FREEZE_ISSUES.md` yazılır ve görev park edilir.
3. Parametre ayarı yalnız ayar bölünmesinde. Test bölünmesi, ayar parametresi sabitlenmeden **önce hiçbir koşturmada** okunmaz; `run_h2.py` ayar ve test aşamalarını ayrı komutlara böler (`--phase tune`, `--phase test`) ve `--phase test`, `ALPHA_DEFAULT`'ın `tuning.json` ile aynı olduğunu doğrulamadan çalışmaz.
4. Soru üretimi ve arama ayrı süreç; sorular diske yazılır; istem ve chunk metni aramaya sızmaz.
5. **Fail-fast:** değerlendirme kodu, gerçek (truth) çözülemediğinde, bir hücre için sorgu sayısı beklenenden azsa, veya `MANIFEST` hash'i tutmuyorsa **hata ile çıkar**. Sessizce sıfır yazmak yasak.
6. **Rapor tek koşturmadan:** `make report RUN=<run_id>` dışında rapor üretilmez. `report.py` dosya sisteminde arama yapmaz, "en son" seçmez, geri düşmez; `reports/runs/<run_id>/` içindeki dosyalar eksikse hata verir.
7. Her koşturma `reports/runs/<run_id>/manifest.json` yazar: git sha, çalışma ağacı temiz mi, veri MANIFEST hash'leri, Rust/Python sürümleri, seçilen parametreler. `run_id` = UTC zaman damgası + git sha'nın ilk 7 karakteri.
8. Hipotez tutmuyorsa optimize etme; raporla ve devam et. İstisna: `02_TASKS.md`'de açıkça yazılı ayar adımları.
9. Her rapor sayısı ham JSON'a izlenebilir; elle sayı yazmak yasak.

## Durma koşulları
`docs/progress/STOP.md` yaz, görevi `PARK` et, mümkünse bağımsız görevlere devam et:
- Rust toolchain/crate veya Python bağımlılığı 2 alternatifle kurulamıyor.
- `maturin develop` 3 denemede başarısız.
- Veri deposu klonlanamıyor (3 yeniden deneme, sonra yedek aday; o da olmazsa STOP).
- `ANTHROPIC_API_KEY` yok (yalnız T12 park edilir; T13–T16 beklemez, T17 bekler).
- 3 gözden geçirici turu sonrası `TAMAMLANDI` yok.
- Bir adım 2 saatten uzun.
Durma dışında soru sorma; varsayım yap, `ASSUMPTIONS.md`'ye yaz, devam et. Her varsayımın başına hangi hipotezi etkileyebileceğini `[H1]`, `[H2]`, `[H3]`, `[-]` etiketiyle yaz.

## Yapma
- UI, sync, CRDT, çok tenant, plugin API, sunucu, kimlik doğrulama.
- Rust'ta embedding/vektör arama (bu tur Python'da).
- Python'da parser/eşleyici/chunker kopyası.
- Model/vektör deposu/eşik değişikliği.
- Testleri geçmek için testi değiştirmek.
- `data/` altına rastgele kimlik içeren dosya commit'lemek (yalnız MANIFEST ve içerik-referanslı gerçekler commit'lenir, bkz. EVAL §Veri).

---

# Aktif tur: docs/v3 (v2-final etiketinden sonra)

Bu bölüm `CLAUDE.md`'nin üstündeki kuralları **değiştirmez**, daraltır ve üstüne ekler.

## Okuma sırası (v3)
1. `docs/v3/00_GOAL.md`
2. `docs/ARCHITECTURE.md` §4.1, §10 (K4 kararı)
3. `docs/v3/01_SPEC_DELTA.md` (üzerine yazdığı yerler dışında `docs/01_SPEC.md` geçerli)
4. `docs/v3/02_TASKS.md`
5. `docs/v3/03_EVAL_DELTA.md` (üzerine yazdığı yerler dışında `docs/03_EVAL.md` geçerli)
6. `docs/progress/v3/`

## v3'e özgü kurallar
- Görevler T20'den başlar; progress kayıtları `docs/progress/v3/T<nn>.md`.
- **v2 dondurması aynen geçerli:** `eval/metrics.py`, `eval/truth.py`, `eval/datasets/*.py` (T9'daki dosyalar), `eval/freeze_check.py`, `tests/fixtures/freeze_fixture/**` değişmez. v3 yalnız yeni dosya ekler.
- **İkinci dondurma:** `T23: freeze eval v3` sonrası `eval/metrics_v3.py`, `eval/datasets/synthetic_edits_v2.py`, `eval/run_*_v3.py`, `eval/run_h4_v2repro.py`, `tests/fixtures/freeze_fixture_v3/**` değişmez. İstisna prosedürü aynı (`FREEZE_EXCEPTIONS.md`, önce/sonra `freeze-check-v3`, gözden geçirici).
- **v2 davranışı kutsal:** `MatcherVersion::V2` yolu v2-final ile bayt bayt aynı çıktıyı vermeli. Bir v3 değişikliği bunu bozuyorsa değişiklik yanlıştır, test değil.
- Eşleyici sabitleri: `V2` sabitleri (0.60, 0.70) dokunulmaz; `V3` sabitleri yalnız T24 ayar fazında yazılır.
- `docs/v3/00_GOAL.md` eşikleri değiştirilemez. K4 ile değişen S2 tanımı zaten orada kayıtlıdır; başka tanım değişikliği yapılmaz.
- H2 koşturulmaz; `graph`, `hier`, varlık, akış kodu bu turda yazılmaz.
- `ANTHROPIC_API_KEY` bu turda **gerekmez**; API çağrısı yapan kod eklenmez.
- Varsayımlar `docs/progress/v3/ASSUMPTIONS.md`'ye etiketli (`[H1] [H3] [H4] [-]`) yazılır.
- T27 bittiğinde `docs/progress/v3/T27.md` sonuna "T28 için hazır" ve RUN kimliği yazılır; dur. T28'i sen yapma.
