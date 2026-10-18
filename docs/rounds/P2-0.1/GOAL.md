# P2-0.1 — Tasdik Yolları: Yeniden Adlandırma, Çakışma, Obsidian Dışı Kanallar (Yama Turu)

Başlangıç: P2-0 "tanımlanmamış kalış" ile kaldı (K-P2-0.6); PR #11 `main`'e girdi. Bu tur P2-0'ın hipotezini **aynı eşiklerle** yeniden ölçer; yalnız ölçülen üç nedeni düzeltir ve kanalın Obsidian'a özgü olup olmadığını ölçer. Tur bitmeden P2-A başlamaz.

## Problem (P2-0 FINAL ve VERIFICATION'dan)
1. **0b kaldı (0.49), tamamı yeniden adlandırma yolundan.** Obsidian bir not yeniden adlandırılınca bağlantı veren notları eklentinin sarmaladığı `adapter.write` dışından yazar; bu güncellemeler tasdiksiz (`unverified`) alınır. Ardından canonical olmayan başlı notun kendi yeniden adlandırması da tasdik edilemez (`renamed_from` denetimi) ve yeni yolun tüm blokları `unverified` kalır. Yeniden adlandırma dışı: harness 4/627, bağımsız doğrulama 0/85.
2. **0d kaldı (0.979), tamamı `collide` tiklerinden.** İnsanın kaydı ile ajanın yazımı izleyicinin tek gözlemine düşünce dosya baytları tasdik edilen baytlar değildir; insanın paragrafı da `unverified` olur (güvenli taraf). Gerçek Obsidian'da 0–20 ms çakışmada 8/16.
3. **Kanal genelliği ölçülmedi.** İnsan kanalı yalnız Obsidian eklentisi, ajan yalnız `agent_fs_writer` ve Claude Code ile ölçüldü. Kalışın Obsidian'a özgü mü, tasarıma mı ait olduğu bilinmiyor (insan sorusu, 2026-10-18).
4. **Doğrulayıcı bulguları.** Terminalden `--remote-debugging-port` ile başlatılan Obsidian'ın CDP üzerinden yazdığı ajan metni eklentice tasdik edilir (3/3 `canonical`; token'a ve anahtar zincirine dokunmadan) — tehdit modelinde "açık soru" değil gösterilmiş yol. `make verify` özeti "counted FAILs 15" derken turun sınıflandırıcısı 0 sayar. Dosyada hiç olmamış `content_hash`'li tasdik 200 ile günlüğe yazılır.

## Hipotez
Yeniden adlandırma ve bağlantı güncellemesi aklama korumasıyla tasdik edilirse, çakışmada tasdik edilen baytların blokları blok düzeyinde eşleştirilirse ve insan kanalı Obsidian'a bağlı değilse, P2-0'ın hipotezi (dosya erişimli ajan canonical'a sızamaz **ve** gerçek insan düzenlemeleri pratikte `unverified`'a düşmez) P2-0'ın eşikleriyle tutar.

## Tasarım (ölçülecek olan)
- **T1 Bağlantı güncellemesi ve yeniden adlandırma tasdiki.** Eklenti, Obsidian'ın yeniden adlandırma sırasında yan notlara yazdığı bağlantı güncellemelerini de tasdik eder (`attest` isteğine `link_update_of: <yeniden adlandırılan yol>`); sunucu bu tasdiki ancak **yeni baytlar ile canonical baş arasındaki farkın yalnız wikilink/markdown bağlantı hedef metinlerinde** olması koşuluyla kabul eder (aksi 403 `denied`, bloklar `unverified`). Böylece ajanın tasdiksiz yazdığı içerik bir yeniden adlandırmayla aklanamaz. Yeniden adlandırmanın `renamed_from` denetimi, bağlantı güncellemesiyle canonical kalan başla çalışır. Spec §7.1 (Draft) genişler; sürüm CRITERION ile.
- **T2 Gözlem başına blok eşleşmesi (çakışma).** `attest` isteğe bağlı `content` (kaydedilen baytlar) taşır; izleyicinin gözlediği baytlar `content_hash`'e eşit değilse ama gözlemin blokları tasdik edilen baytların bloklarını kapsıyorsa, tasdik edilen bloklar `canonical`, kalanlar `unverified` (`unattested_fs`). Sunucu `content`'i saklamaz; günlüğe yalnız `(doc_path, content_hash, at_seq, partial: true)` düşer. Spec §7.1/§8 (Draft); uyumluluk C2-13.
- **T3 Obsidian dışı kanallar.** `rootmark attest <vault> <doc> [--renamed-from <eski>]` ile vim/VS Code tarzı kullanıcı akışı (düzenleyici öykünmesi: geçici dosya + yeniden adlandırma ile atomik kayıt, yerinde yazma, ekleme, `mv`); ajan tarafında dosya sistemine giden başka yollar (Python, `sed -i`, `git apply`, `cp`/`rsync` üzerine yazma, `cat >>`, atomik yeniden adlandırma kalıbı). Ürün değişikliği yalnız CLI'nin `--renamed-from` eki.
- **T4 Tehdit modeli ve özet.** `docs/THREAT_MODEL.md` ve spec §7.1 bilgilendirici notu: insan kanalı = Obsidian sürecinden geçen yazım; bu süreci başlatabilen, hata ayıklayıcıyla (CDP) sürebilen ya da eklenti dosyalarını yazabilen her yerel süreç ve aşama 1'de token'ı okuyabilen her süreç insan kanalındadır; aşama 2 (anahtar zinciri ACL'si) bu yolları kapatmaz, OS kullanıcı ayrımı/kum havuzu bu turun ve P2-0'ın kapsamı dışındadır. `verify.sh` FAIL özeti turun sınıflandırıcısına bağlanır (dondurma istisnası). Hiç gözlenmemiş hash'li tasdik: günlüğe `matched: false` ile düşer ve `status.attestation.unmatched` sayar (reddedilmez: tasdik yazımdan önce gelebilir).
- **W** = 0.5 s, P2-0 ayarından sabit; **yeniden ayarlanmaz**.

## Veri setleri
- **D-A** P2-0 üreteci, aynı 200 senaryo (değişmez).
- **D-A′ (aklama, gerçek Obsidian):** 50 senaryo: ajan bir nota tasdiksiz içerik yazar; insan o notun bağlantı verdiği başka bir notu Obsidian'dan yeniden adlandırır (bağlantı güncellemesi ajanın notuna düşer); ayrıca 10 senaryo: ajan bağlantı satırı dışında da değişiklik içeren bir "sahte bağlantı güncellemesi" tasdiki dener (token'lı, `link_update_of` ile).
- **D-B test-2 (gerçek insan düzenleme, gerçek Obsidian):** P2-0'ın üreteci ve oturum türü dağılımı, **yeni tohum** (ön-kayıtlı: `seed = blake3("P2-0.1/D-B")[:8]`), 200 oturum; P2-0'ın test seti yeniden kullanılmaz. Ayar seti yok.
- **D-B2 (Obsidian dışı insan):** 150 oturum düzenleyici öykünmesi + `rootmark attest`: atomik kayıt 50, yerinde yazma 30, ekleme 20, `mv` ile yeniden adlandırma 30 (`--renamed-from`), hızlı art arda 20.
- **D-A2 (Obsidian dışı ajan):** 120 senaryo: Python 20, `sed -i` 20, `git apply` 20, `cp`/`rsync` üzerine yazma 20, `cat >>` 20, atomik yeniden adlandırma kalıbı 20.
- **D-C** P2-0 üreteci, aynı ağırlıklar (`apart` 80, `same_doc_after` 15, `collide` 5), 160 tik; artı **D-C′ gerçek Obsidian çakışma**: 40 tik, ajan kayıttan 0/20/50/100 ms sonra yazar.
- **D-E** P2-0'ın 3 vault'u (v0.1.0) + P2-0 `ce4cf04` ikilisiyle üretilmiş 2 vault (tasdik günlüklü).

## Önceden kayıtlı eşikler
| Kod | Ölçüm | Küme | Eşik |
|---|---|---|---|
| H-P2-0.1a | Canonical sızıntısı (P2-0 0a) | D-A | = 0 |
| H-P2-0.1a′ | Canonical içerik korunumu (P2-0 0a′) | D-A | 200/200 |
| H-P2-0.1b | Yanlış `unverified` oranı, W sonunda (P2-0 0b, aynı eşik) | D-B test-2 | ≤ 0.01 |
| H-P2-0.1b′ | Yanlış `unverified`, yalnız yeniden adlandırma oturumları (bilgi) | D-B test-2 | raporlanır |
| H-P2-0.1c | Tasdik gecikmesi p95 (P2-0 0c) | D-B test-2 | < 500 ms |
| H-P2-0.1d | Karışık katman doğruluğu (P2-0 0d, aynı eşik) | D-C | ≥ 0.99 |
| H-P2-0.1d′ | Gerçek Obsidian çakışma: insan bloğu doğruluğu; ajan bloğu `canonical` | D-C′ | ≥ 0.95; = 0 |
| H-P2-0.1e | Aklama: ajan bloğu `canonical`; sahte bağlantı güncellemesi reddi | D-A′ | = 0; 10/10 |
| H-P2-0.1f | Obsidian dışı insan: yanlış `unverified`; gecikme p95 | D-B2 | ≤ 0.01; < 500 ms |
| H-P2-0.1g | Obsidian dışı ajan: canonical sızıntısı | D-A2 | = 0 |
| H-P2-0.1h | Eski vault'lar: açılır, replay eşit, `events_hash` ve `attestations.jsonl` değişmez | D-E | 5/5 |
| H-P2-0.1i | Determinizm: çift koşturma (D-A, D-A2, D-C, günlükler bayt eşit) | — | eşit |
| H-P2-0.1j | Uyumluluk: mevcut 37 + C2-13 | — | 38/38 |
| H-P2-0.1k | `make verify` özetinin sayılan FAIL'i = turun sınıflandırıcısı | — | eşit (0) |

## Karar kuralı
- **Hepsi geçti** → P2-0 hipotezi kanıtlandı; spec §7.1 "Validated"; `unverified` varsayılan kalır; tehdit modeli T4 metniyle; NORTH_STAR önerisi FINAL'de; **P2-A başlar**.
- **Yalnız 0.1d ve/veya 0.1d′ kaldı**, yanlışların tamamı çakışma tiklerinden ve ajan bloğu `canonical` = 0 → hipotez **belgelenmiş sınırla** kanıtlandı ("eşzamanlı yazımda insan içeriği güvenli tarafta `unverified` kalabilir; insan terfi eder"); diğer sonuçlar ilk dal gibi; sınır THREAT_MODEL ve spec §7.1 notuna yazılır; P2-A başlar.
- **0.1b veya 0.1f kaldı** (diğerleri geçti ya da yukarıdaki sınır) → GOAL P2-0'ın 0b dalı uygulanır: `unverified` varsayılan **olmaz**, politika `canonical` ile yayımlanır, NORTH_STAR vaadi "tasdikli istemci ile" ibaresiyle daraltılır (karar kaydıyla); P2-A başlar.
- **0.1a, 0.1a′, 0.1e veya 0.1g kaldı** → tasarım kusuru; tur kaldı, karar talebi; P2-A başlamaz.
- **0.1h, 0.1i, 0.1j veya 0.1k kaldı** → altyapı kusuru; tur kaldı, karar talebi.
- **Tanımlanmamış kalış → tur kaldı, karar talebi.**

## Yapılmayacaklar
W'nin yeniden ayarlanması; P2-0 test setinin yeniden okunması; eşik değişikliği; MCP'de `attest`; eklentinin hata ayıklayıcı altında tasdik etmemesi (harness CDP kullanır; ayrı karar); OS kullanıcı ayrımı/kum havuzu; sync; WASM; yayım (paket sürümü değişmez).
