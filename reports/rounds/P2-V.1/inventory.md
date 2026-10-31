# P2-V.1 — Envanter (ön-kayıt; E3'ün araç değişikliklerinden önce commit'lendi, sonra değişmez)

Makine için kaynak `inventory.json` (`research/p2v1/inventory.py` üretir; girdileri: P2-V envanteri, bir yeni zincir RUN'ının **yapısı** — hangi yaprakların var olduğu, değerleri değil — ve dondurulmuş karşılaştırıcının dosya/bayt listesi). GOAL P2-V.1 İlkeler: "Yeni envanter … değişikliklerden önce commit'lenir ve sonra değişmez; P2-V'nin envanteri aynen kalır."

## Taban: P2-V'nin envanteri aynen
`reports/rounds/P2-V/inventory.json`'un her girdisi (26 adım, kayıtlar, 40 adlı Z/O alanı + 6 bayt artefaktı, 4 H alanı, `commit_bound` C1–C2, 9 Mac kümesi, 27 kenara alınan kriter, 134 kriter kimliği, kurallar, gözlemler) kimliği ve gerekçesiyle taşındı; o dosya değişmedi. Bu turun değişiklikleri yalnız aşağıdakilerdir (`p2v1_changes`).

## Değişiklikler ve ekler
| Bulgu | Envanterde | Araçta (E3, istisna prosedürüyle) |
|---|---|---|
| **R14** | `commit_bound` **C3**: `release_check.json` R14'ün `detail`'i (en yeni `ROUND_START`) | `tier2 equiv` commit'e bağlı sayar |
| **B6** | O13 → madde **R15**, H1 → **R17**, C2 → **R4a** (`item` + `field_in_item`; P2-V'nin dizi konumu `p2v_field`'da); C2'nin gerekçesi "R4a"; gerçek Obsidian kayıtları (`plugin_e2e_real.json`, `access_log_real.json`) tek O kaydı, baytları `byte_exclusions` **O14/O15** | `tier2` maddeyi kimliğiyle kayıttaki konumuna çözer (her kayıtta ayrı) |
| **B3** | `derived_criteria`: **`p1e1_checks.json:tamper·det`** = `p1e1/tamper_p1e1.json` `pass` | `tier2 fails` sayar; `tier2 criteria` kimliği listeler |
| **B2** | `criterion_fields`: C1-8 yalnız E1 alanlarını ve Z/O alanlarını kenara alır | `_criterion_fields` bütün kaydı değil bu adları kullanır |
| **B1** | kural (`rules` son satırlar) | `tier2 fails` sınıflandırıcının kırpılmış listesine değil kriterin tam fark listesine bakar |
| **E1** | `e1_fields` (E1F1…), `e1_bytes` (E1B1…), `e1_families` (sayılar) | `tier2 equiv` ve `fails` ile Mac kümesinin C1-8 okuması bu adları kullanır; `tier2 pair` kullanmaz |
| **CPU** | `cpu_fields`: `host.json` `cpu_model` (bilgi) | `verify.yml` zincirleri yazar, karşılaştırma işi ikisini ve aynı/farklı olduğunu `tier2.json`'a hükümle |
| **Mac (E4)** | `mac_reading`: tekrarsız deterministik O (C1-8, E-7, E1-4); tarihsel (E-7, E1-4, H-P2-0.3b) | `verify_mac` (E4) |

## E1 alanları (`e1_fields`, `e1_bytes`)
E1'in ön-kayıtlı aileleri (`reports/rounds/P2-V.1/e1/expected_families.md`) ad ad, **kayıt yapısından** sayıldı (fark gözlenmiş olsun olmasın bütün yapraklar):

| Aile | Alanlar | Sayı |
|---|---|---|
| E1-S | `p1c_index.json` `rankings_200›flat›q›r›3`, `rankings_after_compact›p1c›flat›q›r›3`, `…›p1c-hot›flat›q›r›3` (her isabetin skoru; kimlik, katman ve sıra yaprakları adlı değil) | 6000 |
| E1-V | `p1c_index.json` `index_files_{build,compact,final}` içinde `vectors/*`, `hot/vectors.f32`, `manifest.json` | 11 |
| E1-C10 | `p1c_checks.json:C-10›min_cosine` | 1 |
| E1-D13 | `p1d1/d1_3.json:{before,after}_hash` | 2 |
| E1-DTR | `p2l/d_tr.json:builds›<kurulum>›manifest›{vectors,embedders}_blake3` | 8 |
| E1-E | karşılaştırılan kayıtlarda `embedder_id` dizgesini taşıyan her dize yaprağı | 16 |
| E1-R | regress'lerin, E1 alanlarını taşıyan bir regress'i kenara alan `excluded_differences›<regress>›leaves` sayaçları (girdi regress'in kendisi; içindeki yollar değil) | 5 |
| bayt (`e1_bytes`) | `p1d/mcp_e2e/run1/transcript.jsonl`, `p1d/scene/scene.json`, `p1d1/d1_3.json` | 3 |

**Kullanım:** iki zincir arasında (kademe 2 `pair`, V1-2/V1-7) bu alanlar **D'dir, kenara alınmaz** — sayısal yol artık işlemciden bağımsız. Yalnız ONNX Runtime sayısalının kayıtlarına karşı karşılaştırmalarda adlıdırlar: eski zincir (`tier2 equiv`, V1-6), regress'lerin gömdüğü dondurulmuş referans RUN'lar (türev FAIL kuralı), Mac kümesinin C1-8 okuması.

## C1-8'in adları (B2, `criterion_fields`)
C1-8 artık bütün kaydıyla değil yalnız şu adlarla kenara alınır (yalnız kademe 2'nin türev FAIL kuralında; Mac kümesi C1-8'i yalnız `e1_fields` ile okur): `e1_fields` ve P2-V'nin Z/O alanlarının gömülü girdileri; C1-8'in kendi hükmü (`C1-8›pass`, `C1-8›regress›equal`); sıcak katmanın dört sözcüksel skoru (`rankings_after_compact›p1c-hot›lexical›{47›4, 47›5, 47›8, 120›9}›3`) — son ULP'de platform `libm`'ine bağlı (macOS ↔ glibc; Linux koşucuları arasında eşit). Bu dört yaprak **E1'den değil, P2-V'den beri** vardır: eski zincirin V0m koşusunda da aynı dört yaprak yayım Mac'inin referansından farklıdır; P2-V doğrulamasının "D-9 ve D1-12 doğru korumayla sayılırdı" bulgusunun (Ş4) kaynağının bir parçasıdır. P2-V envanterinin O sınıfı tanımı ("referans RUN'ları yayım Mac'inde kaydedilmiş, değeri o donanıma/platforma bağlı") kapsar; burada ad ad yazıldı.

## E1'in ailelerinde OLMAYAN bir gözlem (`observed_outside_e1`; ad değil, bilgi)
`p1e1/release/release_check.json` R7'nin `detail`'i (Python tekerleğinin adı): eski `…-linux_x86_64.whl`, yeni `…-manylinux_2_34_x86_64.whl`. Mekanizma: E1 `ort`'u (C++/libstdc++ bağlantısı) çıkardı, maturin uzantıyı manylinux 2.34 ile etiketleyebiliyor. İki zincir arasında aynı. **E1'in ön-kayıtlı ailelerinde yoktur; adlandırılmadı**: araçlar onu kenara almaz, V1-6 onu adı geçmeyen fark olarak okur (GOAL İlkeler: "adı geçmeyen fark tanımlanmamış farktır").

## Doğrulama (commit'ten önce; bilgi, ad üretmez)
Kaynak: kademe 2'nin E1 commit'indeki gözlem koşusu **41215091092** (P2-V V1-4 emsali: geliştirme koşusu envanterden önce; sınıflandırma mekanizmadan, gözlem sınamadır) ve P2-V'nin eski zincir koşusu V0m **40613392932** (`352f790`).
- **A ↔ B** (gözlem koşusu, donmuş `compare_p2r pair`): 788 906 alan, farklı 27 alan + `p2_0_2/d_a/events.jsonl` — hepsi D-A yarışı (Z4, Z7, Z11, Z20, Z28); `tier2 pair` D farkı **0**. Vektör türevli tek bir fark yok (V4b'nin 209 281 alanının karşılığı 0).
- **eski ↔ yeni** (V0m A ↔ gözlem A): 217 084 farklı alan; bu envanterin adlarıyla (P2-V'nin Z/O/H/commit adları + `e1_fields` + `e1_bytes`) adlandırılmamış **1**: R7 `detail` (yukarıda). Bayt farklarının hepsi adlı.
- **Sınıflandırıcının tam listeleri** (B1'in okuması, gözlem A/B ve eski V0m): D-9 18 056, D1-12 18 058 fark; `e1_fields` + Z/O + C1-8'in adlarıyla kenara alınmayan: D-9 **0**, D1-12 **1** (kaydın kendi `excluded_differences›p1d_checks·json:D-9›leaves` sayacı, E1-R'de adlı; B1 düzeltmesi kriterin kendi alanlarını da adlara bakarak okur). Eski zincirin V0m'unda aynı sonuç: aynı adlar eski ORT sayısallarında da yeterli.
- P2-V araçlarıyla (değişmeden) gözlem koşusunun hükmü: D farkı 0, sayılan FAIL 2 (D-9, D1-12; eski korumanın kırpılmış listesi E1 girdilerini gördü), süre 11 862 s (zincirler 10 633 s, donmuş karşılaştırıcı 1 090 s).
