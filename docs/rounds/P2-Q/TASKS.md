# P2-Q — Görevler

Genel kurallar:
- Sıra: Q0 → Q1 → (insan adımları: yayım) → Q2 → Q3 → Q4. Q0'ın ön-kaydı commit'lenmeden Q2 başlamaz; `v0.3.0` sürümü yayımlanmadan Q2 ve Q3 başlamaz.
- Progress `docs/rounds/P2-Q/progress/`. Commit öneki `P2-Q/Q<n>:`. Dal `p2-q/round` (yalnız birleşmiş `main`'den, K-P2.2); yayım hazırlığı (sürüm dizgileri, CHANGELOG, README, kılavuz) ayrı dal `release/0.3.0` (P2-L/L0 emsali).
- Geliştirme turu, araç dozu (K-P2.4): gözden geçirici yalnız Q4'te (tek tur).
- Pilot sürücüsü `research/p2q/`: **yalnız Python standart kütüphanesi** (Mac'te, Docker konteynerinde ve CI'da aynı dosya); kurulu `rootmark` ile yalnız komut satırı, MCP stdio ve HTTP üzerinden konuşur (`rootmark_core` bağlaması kullanılmaz — ölçülen yayımlanmış ikilidir). Dondurulmuş ölçüm kodu değişmez. Testleri `research/tests/test_p2q_tools.py` (senaryo yükleyici, sorgu değerlendirici, içerik sızıntısı denetimi; kendi küçük fixture'ıyla).
- Çalışma dizini depo dışında (scratchpad): kurulum önekleri, vault kopyaları, Docker bağlamaları, günlükler. Depoya yalnız sayılar, sonuçlar, blake2b'ler, içeriksiz günlük özetleri girer. Kişisel vault'un ve qorbit'in **metni hiçbir dosyaya, commit'e, PR'a, CI'a girmez**.
- Durma koşulları kök `CLAUDE.md`'deki gibi; insan adımı beklemek durma değildir (`HUMAN_STEP.md`; beklerken bağımsız işler sürer: kılavuz, sürücü, testler, Docker imajı — **ölçüm yapılmaz**).

## Q0 — Açılış ve ön-kayıt
**Yapar:**
- `ROUND_START`; DECISIONS K-P2Q.1 (açılış: `ROUND_START`; kopyaların kaynağı ve büyüklüğü — dosya sayısı, bayt, blake2b'lerin özeti; içerik değil).
- `docs/rounds/P2-Q/progress/SCENARIO.md` + `research/p2q/scenario.json`: üç korpusun (qorbit kopyası, Obsidian kopyası, D-NOTES) adımları sırasıyla ve her adımın beklenen sonucu (makinece denetlenebilir: katman, sayı, eşitlik); ajan yazım şablonları (deterministik; tohum sabit); korpus başına ≥ 40 sorgu ve beklenen belge kümeleri (kişisel korpuslarda kayıtta yolların blake2b'si; D-NOTES'ta yollar açık); Obsidian düzenlemelerinin listesi (not seçimi tohumla, kayıtta blake2b); ölçülecek sayılar; Docker taban imajı (etiket + özet).
- `research/p2q/` iskeleti ve testleri.

**Kabul:** Q-1'in ilk yarısı (ön-kayıt commit'li, Q2'den önce).

## Q1 — Kılavuz, README, 0.3.0 yayımı
**Yapar:**
- (a) `progress/Q1_verifications.md`: `v0.2.0`'dan beri birleşen her turun doğrulaması, hükmü (R16'nın okuma kuralıyla), koşulları ve her koşulu kapatan karar (DECISIONS'tan alıntı satırıyla). Kapatan kararı olmayan koşul → **karar talebi** (`K-P2Q-T<n>`) ve `HUMAN_STEP.md`; yayım bekler, öteki işler sürer.
- (b) `docs/GUIDE.md` (İngilizce, GOAL Kapsam 2 (b)'nin bölümleri): kurulum (macOS, Linux, Docker'da Linux), modeller, vault hazırlama ve tarif (her alanın anlamı), `serve`, ajan bağlama (Claude Code için `claude mcp add` ya da `.mcp.json` örneği, genel MCP istemcisi), **Obsidian eklentisi** (sürüm zip'inden elle kurulum: dizin, dosyalar, topluluk eklentilerini açma, ayarlar, bağlanma, durum çubuğu, tasdik ve izleyicinin birlikte çalışması; BRAT; kaldırma), günlük kullanım (HTTP ve CLI örnekleri), **değerlendirme için veri toplama** (kaydedilen/kaydedilmeyen veri, `--attest-log`, stderr yönlendirmesi, saklama, `observe.py`), yükseltme ve göç, başsız/Linux kullanım (ayrı OS kullanıcısı, systemd birimi örneği — "sınanmadı" notuyla), sorun giderme. Her komut kopyala-çalıştır; çıktı örnekleri yerelde 0.3.0 adayının ikilisiyle alınır, Q2'de yayımlanmış ikiliyle yeniden doğrulanır.
- (c) `README.md`: 0.3.0, spec 2.0.3 (rozet dahil), kısa kurulum + kılavuz bağlantısı, Obsidian bölümü kılavuza bağlı, eskimiş iddiaların düzeltilmesi (`rootmark status` → HTTP/MCP `status`; her README komutu adayın ikilisiyle denetlenir, liste `progress/Q1_readme_check.md`'de).
- (d) `release/0.3.0` dalı (`main`'den): sürüm dizgileri 0.3.0 (R1), `CHANGELOG.md` `## 0.3.0`, `README.md`, `docs/GUIDE.md`, `docs/RELEASE.md` (başlık, insan adımı 1'in 0.3.0 satırı: (a)'nın doğrulamaları ve kapatan kararları), `Cargo.lock`; `make test`, `make lint`, `make conformance`; `scripts/release.sh` + `scripts/release_check.sh` → R1–R20 hepsi `ok` (R14: etiket yok; R15 gerçek Obsidian e2e bu makinede); PR (`gh pr create --base main --head release/0.3.0 --title "rootmark 0.3.0"`), CI yeşil.
- (e) `HUMAN_STEP.md`: insan adımları sırasıyla (PR birleşmesi; `gh workflow run release.yml --ref main`; `release-json`'da iki zorunlu hedef `ok`; `git tag -a v0.3.0` birleşmiş `main` commit'inde ve itme; GitHub sürümü `release-files` artefaktıyla). Ajan sürümü bekler (`gh release view v0.3.0`), sonra `release.json`'u ve `SHA256SUMS`'ı `reports/rounds/P2-Q/release/`'e alır.

**Kabul:** Q-2, Q-3; Q-4'ün yazım yarısı (kılavuzun bölümleri, README güncel).

## Q2 — Mac, kılavuzla
**Yapar:**
- (a) Kurulum kılavuzun macOS bölümüyle (scratchpad öneki); `progress/Q2.md`'ye süre, `--version`, `--spec-version`, conformance, kurulu ikilinin ve arşivin blake2b'si; kılavuzun her adımının sonucu (`progress/GUIDE_RUN.md`: adım, komut, beklenen, gerçek, sapma → bulgu + kılavuz düzeltmesi + yeniden koşu).
- (b) qorbit kopyası ve (d) D-NOTES: başsız senaryo sürücüyle; korpus başına iki taze kopya (determinizm). Kayıtlar `reports/rounds/P2-Q/mac/{qorbit,dnotes}/` (adım sonuçları, sayılar, `events_hash`, indeks dosyalarının ve sorgu sonuçlarının blake2b'leri, beklenenle karşılaştırma).
- (c) Obsidian kopyası: eklenti 0.3.0'ın zip'inden kılavuzun elle kurulum adımlarıyla; gerçek Obsidian ayrı profille; ön-kayıtlı düzenlemeler; kayıtlar `reports/rounds/P2-Q/mac/obsidian/` (her düzenlemenin katmanı, `attest` sonuçları ve hata gerekçeleri, gecikmeler, md bayt denetimi). Harness durması o adımın kayıtlı başarısızlığıdır.
- (e) `research/p2q/observe.py` (stdlib, salt okunur) ve testleri (`test_p2q_tools.py`: kendi küçük olay günlüğü fixture'ıyla sayılar; vault'a yazmadığı; çıktıda içerik olmadığı); her Mac vault'unda senaryodan sonra koşar; çıktısı `reports/rounds/P2-Q/mac/<korpus>/observe.json`, sürücünün saydıklarıyla karşılaştırması yanında; öncesi/sonrası blake2b (salt okunur kanıtı).
- (f) İnsan denemesi isteğe bağlı: `HUMAN_STEP.md`'de kopyanın yolu ve kılavuzun ilgili bölümü; `progress/DIARY.md`.
- Bulgular geldikçe `reports/rounds/P2-Q/FINDINGS.md`'ye (`PQ-<n>`); belge bulguları `p2-q/round`'da düzeltilir (kılavuz/README), yeniden koşulur.

**Kabul:** Q-5, Q-6, Q-7; Q-4'ün koşu yarısı (Mac); Q-11'in Mac yarısı.

## Q3 — Linux: Docker ve CI, karşılaştırma
**Yapar:**
- (a) Docker: ön-kayıtlı taban imaj, `--platform linux/amd64`; konteynerde kılavuzun Linux bölümüyle `install.sh` kurulumu, conformance; qorbit ve D-NOTES senaryoları (kopyalar salt okunur bağlanır, konteyner içinde kopyalanır; konteyner ve bağlamalar sonunda silinir); Mac ↔ Docker karşılaştırması. Kayıtlar `reports/rounds/P2-Q/docker/` (imaj özeti, sayılar "emülasyon, bilgi", eşitlik sonuçları, adlı skor farkları); `observe.py` konteynerde de koşar.
- (b) CI: `.github/workflows/pilot.yml` (yalnız `workflow_dispatch`; tam sürüme sabit eylemler, R20 gibi): `ubuntu-latest`'te kılavuzun Linux bölümüyle `v0.3.0` kurulumu, D-NOTES senaryosu ve `observe.py`, yapıtta sayılar, işlemci modeli, `events_hash` ve blake2b'ler; tetikleme `gh workflow run pilot.yml --ref p2-q/round` (bu bir ölçüm koşusudur, kademe 2 değildir). Mac ↔ Docker ↔ CI karşılaştırması `reports/rounds/P2-Q/ci/<koşu>/`.

**Kabul:** Q-8; Q-4'ün koşu yarısı (Linux); Q-11'in Linux yarısı.

## Q4 — Bulgular, gözden geçirici, rapor, hazır
**Yapar:**
- `FINDINGS.md` tamam (her bulgu sınıflı, hedefli); `numbers.json` + `numbers.md` (Mac üç korpus, CI D-NOTES, Docker bilgi; her sayının kaynak kaydı); P2-C için hız/ölçek eşiği önerisi, P2-H için gömme hızı hedefi önerisi ve P2-O (gözlemlenebilirlik) için kapsam önerisi — `FINDINGS.md`'deki "işletim telemetrisi yok" bulgusuyla (öneriler, karar değil).
- Gizlilik denetimi (Q-9): `eval.portable --all` turun kayıtlarında; kopyaların her belgesinden tohumla seçilen 32 baytlık parçaların depoda (`git grep -F`), PR metninde ve CI yapıtlarında hiç geçmediği.
- `make test`, `make lint`, `make conformance`, `make verify FAST=1`, `check_frozen`, `check_deps`, `check_dep_age`.
- Gözden geçirici tek tur; başlık: "kişisel içerik sızdı mı; senaryo ön-kayıtla aynı mı; kılavuz izlenerek mi koşuldu, her komutu çalışıyor mu; bulgular sınıflı ve hedefli mi; sayılar kayıtlardan mı".
- `reports/rounds/P2-Q/FINAL.md` (Q-1..Q-10, karar kuralının dalı, koşu kimlikleri, süreler); READY; tek PR (`gh pr create --base main --head p2-q/round --title "P2-Q: pilot, kılavuz ve 0.3.0" --body-file reports/rounds/P2-Q/FINAL.md`). Birleşmeden sonra yerel dallar, Docker konteyner/imajları ve çalışma dizinleri temizlenir (K-P2V2.2).

**Kabul:** Q-9, Q-10, Q-11; READY "doğrulama için hazır" + commit sha + PR'ın CI koşu kimliği.
