# P2-Q — İnsan adımı bekleniyor

**Durum (2026-11-03): karar bekleniyor — K-P2Q-T1** (`docs/DECISIONS.md`, "Karar talepleri (açık)").

Q1 (a)'nın yayım önkoşulu (R16, GOAL Q-2) tablosu `docs/rounds/P2-Q/progress/Q1_verifications.md`'de. Blok 2 doğrulamasının (`reports/rounds/P2-R/VERIFICATION.md`, "Şu koşullarla güvenilir") iki koşulu — **R-10** (tam çiftin temiz koşusu) ve **F1** (budanmış bir öneriyle parça paylaşan canlı `canonical` bloğun aramadan düşmesi) — bir kararla kapanmamış. GOAL'ün karar kuralı gereği 0.3.0 bu karar olmadan yayımlanmaz ve pilot (Q2/Q3) koşmaz.

**Beklenen:** K-P2Q-T1 için seçim — (a) iki koşul kararla kapanır (R-10 kademelerle yerine geçti; F1 0.3.0'da bilinen sınır olarak CHANGELOG ve kılavuzda, düzeltmesi P2-C'de) **[öneri]**, (b) koşullar kabul edilir, bilinen sorun olarak yayım, ya da (c) 0.3.0 ertelenir. Kararı `docs/DECISIONS.md`'ye yazmak (ya da bana söylemek) yeterli; karar kaydı `reports/rounds/P2-R/VERIFICATION.md`'yi anmalıdır (R16).

**Beklerken sürenler (ölçüm yok):** `docs/GUIDE.md`, README, `release/0.3.0` dalının hazırlığı (sürüm dizgileri, CHANGELOG, RELEASE.md; R1–R20'nin yerel denemesi; PR taslak), pilot sürücüsü, `observe.py`, Docker imajı ve `pilot.yml`.

**Karardan sonraki insan adımları (sırasıyla; Q1 (e)):**
1. `release/0.3.0` PR'ının (`rootmark 0.3.0`) birleşmesi.
2. `gh workflow run release.yml --ref main`; `release-json` artefaktında `aarch64-apple-darwin` ve `x86_64-unknown-linux-gnu` `ok`.
3. `git tag -a v0.3.0 -m "rootmark 0.3.0"` birleşmiş `main` commit'inde ve `git push origin v0.3.0`.
4. GitHub sürümü `v0.3.0`: `release-files` artefaktının dosyaları + eklentinin `main.js`, `manifest.json`, `styles.css` ayrıca (BRAT).
Ajan sürümü `gh release view v0.3.0` ile bekler, sonra Q2'ye geçer.
