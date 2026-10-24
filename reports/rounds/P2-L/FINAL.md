# P2-L — FINAL (Türkçe ve çok dilli indeks, yol öneki filtresi, kapanış kalıntıları, hızlı kip, 0.2.0; geliştirme turu, Blok 1)

Dal `p2-l/round`, `ROUND_START` `fc645ed` (PR #22'nin rebase birleşmesi, `v0.2.0`). Turun ölçüm RUN'ı (L-2..L-6, L-8) `20261024T170157Z-fa3072b` (ön-kayıt `cfb286f`'den sonra, `fa3072b`'de); tam çift (L-9) `20261024T184609Z-ced61f9` (A, resmi; `chain_B/` B'nin kayıtları) — iki zincir de turun ölçümünü aynı değerlerle yeniden üretti; hızlı kip `reports/rounds/P2-L/fast_l5/`. Bağımsız doğrulama bu turda yapılmaz: Blok 1 sonunda P2-P ile birlikte (`docs/rounds/P2-L/VERIFY.md`, K-P2.4).

## Kriterler (GOAL P2-L, önceden kayıtlı)
| Kod | Ölçüm | Eşik | Ölçülen | Sonuç |
|---|---|---|---|---|
| L-1 | `uni_fold_tr` tek token ve katlama birim testleri; `mdk_lower_alnum` 200 chunk'ta değişmedi | hepsi | `index/tests/p2_l_tokenize.rs` 6/6 (değerlendirme, Şengönül, İSTANBUL/istanbul, ILIK/ılık, NFC, ß/Rusça/Yunanca/emoji); `tok_ref.json` 200 + 8 kenar chunk bayt eşit | **geçti** |
| L-2 | D-TR lexical `uni_fold_tr` − `mdk_lower_alnum` nDCG@10 | ≥ +0.10 | **+0.0003** (0.9952 − 0.9949) | **kaldı** |
| L-3 | İngilizce referans (618 değer) `mdk_lower_alnum` Δ | 0 | 618 değer, fark 0 | **geçti** |
| L-4 | İngilizce referans `uni_fold_tr` Δ nDCG@10 | raporlanır; \|Δ\| ≤ 0.001 → varsayılan | hybrid single −0.0025, hybrid multi −0.0006, flat 0 / 0; en büyük \|Δ\| 0.0025 | **varsayılan `mdk_lower_alnum` kalır; `uni_fold_tr` opt-in** |
| L-5 | D-TR flat multilingual − minilm nDCG@10 | ≥ +0.05 | **+0.0759** (0.6475 − 0.5715) | **geçti** |
| L-6 | D-TR hybrid yeni en iyi (`uni_fold_tr` + multilingual) − eski | ≥ +0.10 | **+0.0772** (0.8801 − 0.8029) | **kaldı** |
| L-7 | Yol öneki: birim/sözleşme testleri; filtreli sorgu `trace`'te; filtresiz `query_hash` değişmez | hepsi | `server/tests/p2_l_l3.rs` 6/6 (güvenlik vakaları dahil); `contract_check` zincirde | **geçti** |
| L-8 | Eski indeks (`/3`, eski tokenizer) → `migration_required`; `migrate`; `events_hash` | 3 vault | 3/3 (`p2l/migrate.json`; P2-0'ın üç `v0.1.0` vault'u); birim `server/tests/p2_l_l1.rs` 3/3 | **geçti** |
| L-9 | Tam çift bir kez: iki varyant, iki embedder; indeks baytları ve kalite alanları | eşit | `make verify` (`scripts/verify.sh pair`) `ced61f9`'te çıkış 0: A `20261024T184609Z-ced61f9` (resmi), B `20261024T194908Z-ced61f9`; `compare_p2l pair` **189 215 alan, fark 0**, yapıtlar ve D-A/D-A2/D-C günlükleri bayt eşit; iki zincirde de sayılan FAIL **0** (`eval.historical_p2l`) | **geçti** |
| L-10 | C3 fixture'ları iki varyantla; mevcut 39 değişmez | hepsi | `index/tests/p2_l_conformance.rs`: 8 vaka iki varyantta kendi fixture'larına eşit (`spec/fixtures_variants/uni_fold_tr/`); `make conformance` 39/39 | **geçti** |
| L-11 | L0: Ö3 açıklandı (hata → düzeltme + test); FAST ≤ 20 dk; 0.2.0: etiket, 3 hedef, `--version` 0.2.0, provenance 5/5, `release_check` temiz | yazılı ve testli | Ö3 hata → düzeltildi + test; FAST 352 s (L0), 735 s (L5, turun ölçümüyle); `v0.2.0` `fc645ed`, `release.yml` 40113904549 üç hedef 39/39, provenance 5/5, `SHA256SUMS` 5/5, `release_check` 21/21 | **geçti** |

## 12 hücre (D-TR, nDCG@10; 400 belge, 800 sorgu, LLM yok)
| Kip | `mdk_lower_alnum` / minilm | `uni_fold_tr` / minilm | `mdk_lower_alnum` / multilingual | `uni_fold_tr` / multilingual |
|---|---|---|---|---|
| lexical | 0.9949 | 0.9952 | 0.9949 | 0.9952 |
| flat | 0.5715 | 0.5715 | 0.6475 | 0.6475 |
| hybrid | 0.8029 | 0.8097 | 0.8676 | 0.8801 |

recall@10 / MRR ve tür (cümle penceresi, başlık) ve dil (TR, EN) kırılımları `reports/rounds/P2-L/runs/20261024T170157Z-fa3072b/p2l/d_tr.json`; embedder'dan bağımsız lexical ve varyanttan bağımsız flat hücre çiftleri eşit. Hızlı kipin iki temiz zinciri ve geliştirme zinciri aynı değerleri bayt bayt üretti.

## Karar kuralı
L-2 ve L-6 kaldı, ötekiler (L-1, L-3, L-5, L-7…L-11; L-4 raporlandı) geçti → GOAL'ün **"L-2, L-5 veya L-6 kaldı (diğerleri geçti)"** dalı: yeni varyantlar **opt-in** kalır (`[index] tokenizer = "uni_fold_tr"`, `[index] embedder = "multilingual-minilm-l12-v2"`), Türkçe kazanç "**ölçüldü, eşik altı**"; karar talebi **K-P2L-T2** (model/tokenizer seçeneği); **P2-P başlar**. Varsayılan tokenizer L-4'e göre `mdk_lower_alnum`; varsayılan embedder `minilm-l6-v2` (GOAL gereği değişmez).

## 0.2.0 yayımı (L0 e; K-P2L.1)
`release: 0.2.0` PR #22 → `main` `fc645ed`; `v0.2.0` etiketi; `release.yml` koşusu 40113904549: `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu` (zorunlu, K-P2A.6), `x86_64-pc-windows-msvc` (bilgi) — her biri `rootmark 0.2.0`, spec 2.0.2, `rootmark conformance` 39/39, kurulum betiğiyle kurulduktan sonra 39/39; `SHA256SUMS` 5/5, `gh attestation verify` 5/5; GitHub release 12 varlık (https://github.com/arknsensei/rootmark/releases/tag/v0.2.0); `release_check.sh` etiketin commit'inde 21/21; imzalama yok. Kayıt `reports/rounds/P2-L/release_0_2_0.json`.

## Notlar (el ile; kaynakları adlı)
1. **Ö3 (P2-0.3 doğrulaması) bir hataydı.** Not önceden tuttuğu bir yola geri adlandırılınca (izleyici silmeleri almaz), notun kendi bağlantı güncellemesi log'un o yoldaki eski başına karşı denetlenip reddediliyordu; aradaki blok `unverified` oluyordu. Kök neden doğrulayıcının V3 harness'iyle ve `--attest-log` ile gösterildi, düzeltildi (spec 2.0.3 §7.1 E2), birim testi düzeltme olmadan düşer; gerçek Obsidian'da önce/sonra `reports/rounds/P2-L/l0_b/`. Kalan `unverified` yan not sınıfıdır (yarış, K-P2-03.1).
2. **L-2'nin bulgusu (ölçümden sonra; optimize edilmedi).** Ön-kayıtlı sorgular belgenin kendi sözcükleridir; lexical her hücrede tavanda (0.995), L-2'nin ölçebileceği pay yoktu. Katlamanın etkisi büyük harf ya da aksanı farklı yazılmış sorgularda ortaya çıkar; bu küme onları içermez. Multilingual'ın kazancı açık (flat +0.076, hybrid +0.065); `uni_fold_tr` hybrid'e +0.007/+0.012 ekler. K-P2L-T2 önerisi: Türkçe vault'lar için önerilen yapılandırmayı belgele; yeniden ölçüm R-1'in D-set'iyle.
3. **Spec 2.0.3** (SPEC_EDITS P2-L): §7.1 varsayılanı `canonical` (E1), eski yola geri yeniden adlandırma tabanı (E2), eskimiş atıf (E3), C2-11 başlığı (E4), §8 yol filtresi bilgilendirici notu (E6). Damga Draft kalır (DEV_PROCESS §5). 0.2.0 spec 2.0.2 ile yayımlandı; 2.0.3 bir sonraki yayımla.
4. **K-P2L-T1** (açık): 0.1.0'ın tarifi `[attestation]` taşımaz, 0.2.0 onu `unverified` okur; CHANGELOG yükseltme notu; öneri bir göç (P2-P).
5. **Geliştirme zincirinin bulduğu eksik**: arama şemasına `path_prefix` eklenince istemcinin üretilmiş sözleşme tipleri eskidi (P1-E'nin denetimi "untouched generated types"); `P2-L/L3-fix` (üreteç basit tipli dizileri alır). `make lint` bunu yakalardı; tur içinde `make lint` geç koşuldu.
6. **Dondurma istisnaları** F1–F8 (`docs/rounds/P2-L/progress/FREEZE_EXCEPTIONS.md`); gözden geçirici onayı `L5.review.md`.
7. **Kalite kapıları** (son commit): `make test` (cargo, pytest 265, vitest 142), `make lint` (check_frozen, check_compare_fields, gen_client_types), `make conformance` 39/39, `make verify FAST=1` 735 s eşit (82 789 alan), tam `make verify` çıkış 0 (189 215 alan eşit, sayılan FAIL 0; zincirler 3781 s ve 3820 s).
8. **Gözden geçirici** (tek tur, K-P2.4; `docs/rounds/P2-L/progress/L5.review.md`): **KABUL**, koşulu L-9'un eşit çıkması (karşılandı); F1–F8 onaylı; güvenlik incelemesi (L0 b, L3) bulgusuz. Bildirilen düşük bulgular, ölçülen commit'i değiştirmemek için bu turda kodlanmadı, P2-P'ye bırakıldı: (a) `ROOTMARK_DEFAULT_TOKENIZER` ortam düğmesi yayımlanan ikilide de okunur (yalnız tarif `tokenizer` adlandırmıyorsa; manifest varyantı kaydeder) — belgele ya da test yapısına taşı; (b) Ö3 düzeltmesinin ret yolları (kopya yol, yalnız ipucu, düşük katmanlı taban) ayrı testle sınanmalı; (c) `release_check` R16'nın gevşetilmesi bilinçli (K-P2L.1) ve öngörülebilir geçici dosya yolu (`$TMPDIR/rootmark-r16.$$`) düzeltilmeli; (d) iki sessizlik riski: filtre kaydının yazımı iz satırından sonra başarısız olursa arama hata döner ama iz filtresiz kalır; `status`'ta göç zinciri kurulamazsa `migrations.required` boş görünür (`unwrap_or_default`; önceki `applies(…).unwrap_or(false)` deseninin sürdürülmesi) — ikisi de hataya çevrilmeli.
