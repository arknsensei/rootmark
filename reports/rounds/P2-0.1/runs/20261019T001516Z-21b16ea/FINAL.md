# P2-0.1 — FINAL (tasdik yolları, yama turu)

- RUN: `20261019T001516Z-21b16ea`; commit `21b16ea7f643fa73051e2bc495ad7cb8d0f3b959`
- Kaynak: bu RUN dizinindeki dosyalar (her satırda adı geçer). Eşikler GOAL P2-0.1'inkiler (P2-0'ınkiler aynen); `W` = 0.5 s (P2-0 `Z5_tuning.json`, yeniden ayarlanmadı).

## Hipotezler

| Kod | Ölçüm | Değer | Eşik | Sonuç | Kaynak |
|---|---|---|---|---|---|
| H-P2-0.1a | Canonical sızıntısı (P2-0 0a) | 0 / 525 yeni blok, 200 senaryo | = 0 | geçti | `p2_0_1/d_a.json` |
| H-P2-0.1a′ | Canonical içerik korunumu (P2-0 0a′) | 200/200 senaryo; canonical soyu değiştiren senaryo 16, değişen canonical soy 52 | 200/200 | geçti | `p2_0_1/d_a.json` |
| H-P2-0.1b | Yanlış `unverified`, W sonunda (P2-0 0b; D-B test-2, gerçek Obsidian) | 0/1702 = 0.0000 (200 oturum, 278 kayıt, yan not 33) | ≤ 0.01 | geçti | `p2_0_1/d_b.json` |
| H-P2-0.1b′ | Yanlış `unverified`, yalnız yeniden adlandırma oturumları (bilgi) | 0/1017 (28 oturum, 61 kayıt, yan notlar dahil) | raporlanır | raporlandı (bilgi) | `p2_0_1/d_b.json` |
| H-P2-0.1c | Tasdik gecikmesi p95 (P2-0 0c; D-B test-2) | 289 ms (217 ölçüm; `canonical` olmayan 0; yazımsız kayıt 61) | < 500 ms | geçti | `p2_0_1/d_b.json` |
| H-P2-0.1d | Karışık katman doğruluğu (P2-0 0d; D-C, insan `content` gönderir) | 340/340 = 1.0000 (160 tik; ajan `canonical` 0; `partial` satırı 7) | ≥ 0.99 | geçti | `p2_0_1/d_c.json` |
| H-P2-0.1d′ | Gerçek Obsidian çakışma: insan bloğu doğruluğu; ajan bloğu `canonical` (D-C′) | 40/40 = 1.0000; ajan `canonical` 0/40 (40 tik) | ≥ 0.95; = 0 | geçti | `p2_0_1/d_c_prime.json` |
| H-P2-0.1e | Aklama: ajan bloğu `canonical`; sahte bağlantı güncellemesi reddi (D-A′) | ajan `canonical` 0/50 (50 senaryo; bağlantı yeniden yazıldı 47); sahte ret 10/10 | = 0; 10/10 | geçti | `p2_0_1/d_a_prime.json` |
| H-P2-0.1f | Obsidian dışı insan: yanlış `unverified`; gecikme p95 (D-B2) | 0/630 = 0.0000; p95 138.5 ms (150 oturum; `canonical` olmayan 0) | ≤ 0.01; < 500 ms | geçti | `p2_0_1/d_b2.json` |
| H-P2-0.1g | Obsidian dışı ajan: canonical sızıntısı (D-A2) | 0 / 210 yeni blok, 120 senaryo | = 0 | geçti | `p2_0_1/d_a2.json` |
| H-P2-0.1h | Eski vault'lar (3 `v0.1.0` + 2 `ce4cf04`): açılır, replay eşit, `events_hash` ve `attestations.jsonl` değişmez | 5/5 | 5/5 | geçti | `p2_0_1/d_e.json` |
| H-P2-0.1i | Determinizm: çift koşturma (D-A, D-A2, D-C, günlükler bayt eşit); bütün çift kümesi | günlükler eşit True; 108361 alan, fark 0; yapıtlar bayt eşit True | eşit | geçti | `pair_compare.json` |
| H-P2-0.1j | Uyumluluk: mevcut 37 + C2-13 | 38/38, seviye 3, spec 2.0.1 | 38/38 | geçti | `conformance_p1d1.json` |
| H-P2-0.1k | `make verify` özetinin sayılan FAIL'i = turun sınıflandırıcısı | sayılan 0 (özet = `eval.historical_p2_0_1`; tarihsel 14) | eşit (0) | geçti | `p2_0_1/historical.json` |

## Kümeler

**D-B test-2 (gerçek Obsidian), oturum türlerine göre** (`p2_0_1/d_b.json`):

| Oturum türü | kayıt | yeni blok | W sonunda `unverified` |
|---|---|---|---|
| char | 30 | 30 | 0 |
| focus | 28 | 28 | 0 |
| multi | 75 | 75 | 0 |
| paragraph | 27 | 27 | 0 |
| paste | 31 | 437 | 0 |
| rapid | 26 | 88 | 0 |
| rename | 61 | 1017 | 0 |

**D-B2 (Obsidian dışı insan), oturum türlerine göre** (`p2_0_1/d_b2.json`):

| Tür | oturum | yeni blok | W sonunda `unverified` |
|---|---|---|---|
| append | 20 | 20 | 0 |
| atomic | 50 | 50 | 0 |
| inplace | 30 | 30 | 0 |
| mv | 30 | 459 | 0 |
| rapid | 20 | 71 | 0 |

**D-A2 (Obsidian dışı ajan), yollara göre** (`p2_0_1/d_a2.json`):

| Yol | senaryo | yeni blok | `canonical` sızıntısı |
|---|---|---|---|
| atomic_rename | 20 | 35 | 0 |
| cat | 20 | 35 | 0 |
| cp_rsync | 20 | 35 | 0 |
| git_apply | 20 | 35 | 0 |
| python | 20 | 35 | 0 |
| sed | 20 | 35 | 0 |

**D-C′ (gerçek Obsidian çakışma), gecikmeye göre** (`p2_0_1/d_c_prime.json`):

| Gecikme (ms) | tik | insan bloğu | insan `canonical` | ajan `canonical` |
|---|---|---|---|---|
| 0 | 10 | 10 | 10 | 0 |
| 100 | 10 | 10 | 10 | 0 |
| 20 | 10 | 10 | 10 | 0 |
| 50 | 10 | 10 | 10 | 0 |

**D-A′ (aklama, gerçek Obsidian), türlere göre** (`p2_0_1/d_a_prime.json`):

| Tür | senaryo | ajan `canonical` |
|---|---|---|
| in_agent_block | 25 | 0 |
| in_human_block | 15 | 0 |
| unobserved | 10 | 0 |

## Regresyon ve kasıtlı vakalar

- P2-0 resmi RUN'ına karşı (`regress_vs_p2_0.json`, referans `20261018T161850Z-e378be2`): 102433 alan, hariç dışı fark 0, karar gereği 72 alt ağaç, kullanılmayan hariç 3; uyumluluk eski testler eşit True, yeni test {'C2-13': True}; P2-0'ın yeniden koşan kümeleri (D-A, D-E `v0.1.0`) eşit True → geçti.
- Kasıtlı vakalar (`p2_0_1/tamper_p2_0_1.json`): 9/9 reddedildi, kontroller 3/3 → geçti.

## Önceki turların kaydedicileri

`p2_0_1/historical.json` (`eval.historical_p2_0_1`, referans `reports/rounds/P2-0/runs/20261018T161850Z-e378be2`; `make verify`'ın FAIL özeti bu sınıflandırıcının çıktısıdır): 14 tarihsel FAIL sayılmadı; sayılan FAIL 0.

- `p1b1_checks.json` B1-3: tarihsel, karar K-P2-01.3 ile kapandı — a P2-0.1 decision (compare_p2_0_1.EXCLUDED)
- `p1c1_checks.json` C1-7: tarihsel, karar K-P2-01.3 ile kapandı — a P2-0.1 decision (compare_p2_0_1.EXCLUDED)
- `p1c_checks.json` C-11: tarihsel, karar K-D1.2, K-D1.4-T, K-E0.1 ile kapandı — P1-C's regression against P1-B.1 sees B1-3's spec-version difference (historical in P1-E.1 too: research/eval/historical_fails.json)
- `p1c_checks.json` C-3: tarihsel, karar K-C8-T, K-C1.1, K-C1.2 ile kapandı — P1-C's hybrid multi-question nDCG@10 was -0.0136 off v2; closed in P1-C.1 by the declared lexical variant and the Rust quality reference (historical in P1-E.1 too: research/eval/historical_fails.json)
- `p1d1_checks.json` D1-10: tarihsel, karar K-P2-01.3 ile kapandı — a P2-0.1 decision (compare_p2_0_1.EXCLUDED)
- `p1d1_checks.json` D1-12: tarihsel, karar K-P2-01.3 ile kapandı — a P2-0.1 decision (compare_p2_0_1.EXCLUDED)
- `p1d_checks.json` D-2: tarihsel, karar K-D1.1, K-D1.5, K-E0.4, K-D1.4-T ile kapandı — P1-D's MCP e2e expects agent edits of human blocks to pass (append_only since K-D1.1), the old error codes (K-D1.5, K-E0.4) and spec 1.0.2 in status (historical in P1-E.1 too: research/eval/historical_fails.json)
- `p1d_checks.json` D-7: tarihsel, karar K-D1.4-T, K-D1.5 ile kapandı — P1-D's HTTP e2e expects spec 1.0.2 (D7-3) and the old status codes of two cases (historical in P1-E.1 too: research/eval/historical_fails.json)
- `p1d_checks.json` D-9: tarihsel, karar K-P2-01.3 ile kapandı — a P2-0.1 decision (compare_p2_0_1.EXCLUDED)
- `p1e1_checks.json` E1-6: tarihsel, karar K-P2-01.3 ile kapandı — a P2-0.1 decision (compare_p2_0_1.EXCLUDED)
- `p1e1_checks.json` E1-8: tarihsel, karar K-P2-0.5 ile kapandı — the p1e-final server case of P1-E.1's contract tamper expects only status gaps; it also lacks §2.6 and status.attestation since P2-0 (P2-0/F14); historical in P2-0 too, its record equal to the P2-0 official RUN's
- `p1e1_checks.json` tamper: tarihsel, karar K-P2-0.5 ile kapandı — the p1e-final server case of P1-E.1's contract tamper expects only status gaps; it also lacks §2.6 and status.attestation since P2-0 (P2-0/F14); historical in P2-0 too, its record equal to the P2-0 official RUN's
- `p1e_checks.json` E-7: tarihsel, karar K-P2-01.3 ile kapandı — a P2-0.1 decision (compare_p2_0_1.EXCLUDED)
- `p1e_checks.json` E-9: tarihsel, karar K-P2-01.3 ile kapandı — a P2-0.1 decision (compare_p2_0_1.EXCLUDED)

## Karar kuralı

Kalan hipotezler: yok; bağımsız doğrulama açık.

**Hepsi geçti → P2-0 hipotezi kanıtlandı**: spec §7.1 "Validated"; `unverified` varsayılan kalır; tehdit modeli T4 metniyle; NORTH_STAR önerisi aşağıda (insan kararı); **P2-A başlar**. Bağımsız doğrulama bu RUN'ın parçası değildir.

## Tehdit modeli

`docs/THREAT_MODEL.md` / `docs/SECURITY.md` "Tasdikli yazma ve insan kanalı" ve spec §7.1 bilgilendirici notu (K-P2-01.4): insan kanalı Obsidian sürecinden geçen yazım ve `rootmark attest`'i çalıştıranın beyanıdır; bu süreci başlatabilen, hata ayıklayıcıyla (CDP) sürebilen ya da eklenti dosyalarını yazabilen her yerel süreç ve aşama 1'de token'ı okuyabilen her süreç insan kanalındadır (CDP yolu P2-0 doğrulamasında gösterildi); aşama 2 bu yolları kapatmaz; kasıtlı ajana karşı ayrım OS düzeyindedir ve kapsam dışıdır.

## NORTH_STAR önerisi

NORTH_STAR değiştirilmedi (insan kararı). Öneri, karar kuralının dalına göre:

- Hipotez kanıtlandıysa (belgelenmiş sınırla ya da sınırsız): vaade "Bir ajanın dosyaya doğrudan yazdığı içerik, insan onaylayana dek `canonical` değildir (tasdikli istemciyle; insan kanalına giremeyen ajana karşı — token'ı okuyan, insan istemcisini başlatıp süren ya da eklenti dosyalarını yazan süreç kapsam dışı)." eklenir; belgelenmiş sınır dalında: "eşzamanlı yazımda insan içeriği güvenli tarafta `unverified` kalabilir".
- 0b dalında: vaat "tasdikli istemci ile" ibaresiyle daraltılır; varsayılan politika `canonical`, `unverified` isteğe bağlı (`[attestation] unattested = "unverified"`).
- Kaldı dallarında: vaat değişmez; öneri insanın karar talebine verdiği cevaba bağlıdır.

