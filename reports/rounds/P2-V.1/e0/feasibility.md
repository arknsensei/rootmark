# P2-V.1/E0 — Fizibilite ve seçim (ön-kayıt; E1'den önce commit'lendi)

GOAL P2-V.1 Kapsam 1 ve V1-1 (K-P2V.3, K-P2V.4); TASKS E0. Ham sayılar `feasibility.json` (`research/p2v1/e0_feasibility.py` ile üretildi).

## Girdiler
- **CI koşusu `41204929528`** (`.github/workflows/e0-feasibility.yml`, commit `3fdab2e`; dal `p2-v.1/round`): aynı sabit korpus — `research/data/d_notes/notes` (git'te, 401 not, **6 063 parça**, minilm'de 1 022 082 jeton; ASSUMPTIONS E0-3) — dört koşucuda, iki manifest modeliyle (`minilm-l6-v2`, `multilingual-minilm-l12-v2`), hem bugünkü ürünün ONNX Runtime embedder'ıyla (`index/src/embed.rs`, `ort.f32`) hem aday (r) ile (`index/src/detbert.rs`, `det.f32`). İşlemci modelleri yapıttan (`cpu.json`):

  | Koşucu | Mimari | İşlemci |
  |---|---|---|
  | `x86_64-a` (`ubuntu-latest`) | x86_64 | Intel Xeon Platinum 8370C |
  | `x86_64-b` (`ubuntu-latest`) | x86_64 | AMD EPYC 7763 |
  | `aarch64` (`ubuntu-24.04-arm`) | aarch64 | Neoverse-N2 |
  | `macos-arm64` (`macos-14`) | arm64 | Apple M1 (Virtual) |

- **P2-V V4b** (koşu `40613409396`, `tier2-A`/`tier2-B`): ürün zincirinin iki x86_64 koşucusundaki ORT vektörleri (`p1d/*` vault'ları; aynı içerikli kopyalar bir kez sayıldı, 140 vektör).
- **C-10 referansı** `research/data/embed_ref.json` (50 parça, `sentence-transformers`); aynı 50 metin her iki yolla gömüldü (`e0_embed` JSON kipi).
- Ölçek: 10⁵ doküman × E0 korpusunun doküman başına ortalama parçası (15,12) × 384 = **580 596 508 bileşen**. Beklenen sınır çarpışması = bileşen sayısı × ortalama |δ| / adım (K-P2V.4).

## Ham ORT vektörlerinde |δ| (iki işlemci, bileşen başına |a − b|)
| Çift | Model | Farklı bileşen | Farklı vektör | En büyük |δ| | Ortalama |δ| | Sıfır olmayan oran |
|---|---|---|---|---|---|---|
| Xeon ↔ EPYC | minilm | 2 052 461 / 2 328 192 | 5 711 / 6 063 | 1,64e-7 | 1,70e-8 | 0,88 |
| Neoverse ↔ Xeon | minilm | 2 179 492 / 2 328 192 | 6 059 / 6 063 | 1,56e-7 | 1,83e-8 | 0,94 |
| Neoverse ↔ M1 | minilm | **0** | 0 | 0 | 0 | 0 |
| Neoverse ↔ Xeon | multilingual | 2 144 810 / 2 328 192 | 6 053 / 6 063 | 1,42e-7 | 1,49e-8 | 0,92 |
| Xeon ↔ EPYC | multilingual | 2 043 812 / 2 328 192 | 5 768 / 6 063 | 1,62e-7 | 1,41e-8 | 0,88 |
| V4b (iki x86_64) | minilm | 46 276 / 53 760 | 129 / 140 | 9,69e-8 | 1,73e-8 | 0,86 |

(Bütün çiftler `feasibility.json` `models.*.pairs` ve `v4b`'de.) ORT'nin iki ARM koşucusu bu korpusta aynı baytı verdi; x86_64'ün iki modeli birbirinden ve ARM'dan farklı. Geliştiricinin Mac'i (M4 Max, SME2) ORT'de başka bir çekirdek kullanır (aşağıda).

## Adaylar
### (y) Normalize vektörü 2⁻ᵏ ızgarasına yuvarlamak (k = 10…20; en yakına, eşitlikte çifte)
Xeon ↔ EPYC (minilm; öteki çiftler aynı büyüklükte):

| k | Gözlenen farklı bileşen (E0) | Farklı vektör | Beklenen (E0) | **Beklenen (10⁵ doküman)** | C-10 `min_cosine` |
|---|---|---|---|---|---|
| 10 | 34 | 34 | 40 | **10 100** | 0,999984 |
| 11 | 89 | 88 | 81 | 20 200 | 0,999996 |
| 12 | 141 | 139 | 162 | 40 400 | 0,999999 |
| 14 | 684 | 643 | 648 | 161 600 | 0,99999994 |
| 16 | 2 585 | 2 053 | 2 592 | 646 401 | 0,999999996 |
| 18 | 10 305 | 4 805 | 10 368 | 2 585 604 | ≈1 |
| 20 | 41 642 | 5 703 | 41 473 | 10 342 416 | ≈1 |

Gözlenen sayı beklenen formülü her adımda tutuyor (34 ↔ 40, 684 ↔ 648, 41 642 ↔ 41 473): formül doğru, yuvarlama yalnız farkı ölçekler. **V4b'de k = 10'da gözlenen 0, beklenen 10⁵ dokümanda ~10 300** — K-P2V.4'ün uyardığı "küçük korpusta şans sıfırı" tam olarak bu (140 vektör). Hiçbir k gözlenen 0'ı E0 korpusunda vermiyor, hiçbiri beklenen < 1'e yaklaşmıyor (k ≤ 9 kaliteyi de bozar ve beklenen yine binlerdedir). **(y) V1-1'i geçmez.**

### (k) ORT/MLAS çekirdeklerini işlemciden bağımsız bir yola sabitlemek — denenemedi
`ort =2.0.0-rc.13` (`download-binaries`) ve onun ONNX Runtime'ı MLAS çekirdeğini çalışma zamanında işlemcinin özelliklerinden seçer (x86_64'te AVX2/AVX-512/AVX-VNNI, ARM'da NEON/KleidiAI/SME); oturum ya da ortam seçeneğiyle bu seçim kapatılamaz (`with_deterministic_compute` ve tek iş parçacığı zaten açık, `index/src/embed.rs`: bu ölçüm onlarla yapıldı). Sabitleme ONNX Runtime'ı kaynaktan özel bayraklarla derlemeyi gerektirir; yine de x86_64 ile aarch64 farklı komut kümeleridir ve tek bir çekirdek yolu yoktur. Ölçülebilir bir yapılandırma bulunmadı. Ölçülen olgu: aynı ORT ikilisi Xeon ↔ EPYC arasında bile farklıdır.

### (s) Havuzlama ve normalizasyonu sabit sıralı skaler Rust'ta yapmak
Ürün bunu zaten yapıyor (`embed.rs`: maskeli ortalama ve L2 Rust'ta, ORT yalnız `last_hidden_state`'i verir). Fark modelin içinden geliyor; (s) tek başına yukarıdaki ORT satırlarıyla **aynıdır** (gözlenen ~2 M farklı bileşen). V1-1'i geçmez.

### (r) Bütün ileri geçiş sabit sıralı Rust'ta (`index/src/detbert.rs`; GOAL "ya da başka yol")
Aynı `model.onnx` ağırlıkları (çizge düğüm adları ve şekilleriyle denetlenerek okunur), aynı tokenizer, aynı havuzlama. Her çıktı bileşeni tek bir IEEE 754 işlem dizisiyle tanımlıdır: toplamlar sabit sırayla (i = 0, 1, …), çarpımlar kaynaşık çarpma-toplamayla (FMA, tek yuvarlama; standart sonucunu tanımlar), vektör şeritleri yalnız bağımsız bileşenler arasında; `exp` ve `erf` platform `libm`'i olmadan binary64'te `+ − × ÷` ile; iş parçacığı yok. x86_64'te AVX2+FMA, aarch64'te NEON, başka yerde skaler `f32::mul_add` — aynı işlem.

| | minilm | multilingual |
|---|---|---|
| Farklı bileşen, **6 çiftin hepsi** (Xeon, EPYC, Neoverse-N2, Apple M1) ve yerel M4 Max | **0** | **0** |
| Ortalama |δ| | 0 | 0 |
| **Beklenen çarpışma (10⁵ doküman)** | **0** (yuvarlama yok; |δ| = 0) | **0** |
| ORT'ye karşı en küçük kosinüs (E0 korpusu, 6 063 parça) | 0,999999999998 | 0,999999999998 |
| C-10 `min_cosine` (≥ 0,999) | **0,9999999999992665** (ORT: 0,9999999999992164) | — |
| Süre, E0 korpusu: ORT → (r) | Xeon 249 → 828 s (×3,3); EPYC 320 → 555 s (×1,7); Neoverse 565 → 709 s (×1,25); M1 457 → 503 s (×1,1) | Xeon 288 → 942 s; EPYC 378 → 608 s; Neoverse 679 → 801 s; M1 432 → 550 s |

Yerel geliştirme makinesi (Apple M4 Max, SME2; beşinci işlemci, aynı commit'in `e0_embed`'i): (r)'nin `det.f32`'si iki modelde de CI'nınkiyle **bayt bayt aynı** (`cmp`); ORT'nin `ort.f32`'si ise Neoverse/M1'inkinden de farklı (minilm 2 233 403 / 2 328 192 bileşen, ortalama |δ| 2,84e-8; multilingual 2 208 511, 2,25e-8) — ORT bu işlemcide başka bir çekirdek (SME) seçer. Süre: minilm ORT 63,6 s → (r) 316,6 s; multilingual 70,5 → 334,4 s.

(r)+(y): (r)'nin |δ|'sı 0 olduğu için yuvarlama bir şey eklemez; ölçülmedi.

## Seçim: **(r)** — gömmenin bütün ileri geçişi sabit sıralı Rust'ta
- V1-1'in iki yarısı: E0 korpusunda gözlenen farklı bileşen **0** (dört işlemci, üç komut kümesi, iki model) **ve** 10⁵ doküman ölçeğine beklenen çarpışma **0 < 1** (|δ| tam olarak 0; ızgara yok). Öteki adayların hiçbiri gözlenen 0'ı vermiyor ((y) en iyi 29–44; (s) = ORT; (k) yapılandırılamıyor).
- Kalite: ORT'ye karşı kosinüs ≥ 0,999999999998, C-10 `min_cosine` ORT'ninkinden ayırt edilemez (|Δ| 5e-17). Sıralama ve P1-C/P1-C.1 kalite kriterleri E1'de yeniden ölçülür (V1-3).
- Bedel: gömme CI koşucularında ORT'nin 1,1–3,3 katı sürer (Xeon'da en kötü; AVX-512 ve ORT'nin daha iyi önbellek bloklaması yok). Zincir süresine etkisi E4'te kademe 2 ve FAST koşularında ölçülür; optimize edilmez, karar kuralı uygulanır. ORT bağımlılığı (`ort`, `ndarray`) E1'de ürünün gömme yolundan çıkar.
- E1'in sınırı: `index/src/embed.rs` `OnnxMiniLm`'in sayısal yolunu (r)'ye bağlar; model, tokenizer, havuzlama, `embedder_id` kuralı aynı kalır; vektör baytları değiştiği için indeks biçim sürümü artar ve eski indeks `migration_required` verir (GOAL Kapsam 2).
