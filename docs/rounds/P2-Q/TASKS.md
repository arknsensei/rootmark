# P2-Q — Görevler

Genel kurallar:
- Sıra: Q0 → Q1 → (insan adımları: yayım) → Q2 → Q3 → Q4. Q0'ın ön-kaydı commit'lenmeden Q2 başlamaz; `v0.3.0` sürümü yayımlanmadan Q2 başlamaz.
- Progress `docs/rounds/P2-Q/progress/`. Commit öneki `P2-Q/Q<n>:`. Dal `p2-q/round` (yalnız birleşmiş `main`'den, K-P2.2); yayım hazırlığı ayrı dal `release/0.3.0` (P2-L/L0 emsali).
- Geliştirme turu, araç dozu (K-P2.4): gözden geçirici yalnız Q4'te (tek tur).
- Pilot sürücüsü `research/p2q/` (Python; kurulu `rootmark` ikilisiyle yalnız komut satırı, MCP stdio ve HTTP üzerinden konuşur; `rootmark_core` bağlaması kullanılmaz — ölçülen şey yayımlanmış ikilidir). Dondurulmuş ölçüm kodu değişmez.
- Çalışma dizini depo dışında (scratchpad): kurulum öneki, vault kopyaları, sunucu günlükleri. Depoya yalnız sayılar, sonuçlar, blake2b'ler, içeriksiz günlük özetleri girer. Kişisel vault'un ve qorbit'in **metni hiçbir dosyaya, commit'e, PR'a girmez**.
- Yeni ölçüm aracının kendi testleri (`research/tests/test_p2q_tools.py`): senaryo yükleyici, sorgu değerlendirici, içerik sızıntısı denetimi (kendi küçük fixture'ıyla).
- Durma koşulları kök `CLAUDE.md`'deki gibi; insan adımı beklemek durma değildir (`HUMAN_STEP.md`, beklerken bağımsız işler: Q2'nin sürücüsü ve testleri yayımdan önce yazılabilir, **ölçülemez**).

## Q0 — Açılış ve ön-kayıt
**Yapar:**
- `ROUND_START`; DECISIONS K-P2Q.1 (açılış: `ROUND_START`; kopyaların kaynağı ve büyüklüğü — dosya sayısı, bayt, blake2b'lerin özeti; içerik değil; sunucu erişimi var mı).
- `docs/rounds/P2-Q/progress/SCENARIO.md` + `research/p2q/scenario.json`: Q2 (b) ve (c)'nin adımları sırasıyla, her adımın beklenen sonucu (makinece denetlenebilir: katman, sayı, eşitlik); ajan yazım şablonları (deterministik; tohum sabit); ≥ 40 sorgu ve her birinin beklenen belge kümesi (kopyaların **yolları** üzerinden; yol adları kayda girerse kişisel bilgi olabileceği için kayıtta yolun blake2b'si); Obsidian düzenlemelerinin listesi (hangi notlara ne tür düzenleme — not seçimi tohumla, kayıtta blake2b); ölçülecek sayılar.
- `research/p2q/` iskeleti ve testleri.

**Kabul:** Q-1'in ilk yarısı (ön-kayıt commit'li, Q2'den önce).

## Q1 — 0.3.0 yayımı
**Yapar:**
- (a) `progress/Q1_verifications.md`: 0.2.0'dan (`v0.2.0`) beri birleşen her turun doğrulaması, hükmü (R16'nın okuma kuralıyla), koşulları ve her koşulu kapatan karar (DECISIONS'tan alıntı satırıyla). Kapatan kararı olmayan koşul → `STOP.md` değil, **karar talebi** (`K-P2Q-T<n>`) ve `HUMAN_STEP.md`; yayım bekler, Q0'ın ve sürücünün işleri sürer.
- (b) `release/0.3.0` dalı (`main`'den): sürüm dizgileri 0.3.0 (R1), `CHANGELOG.md` `## 0.3.0`, `docs/RELEASE.md` (başlık, insan adımı 1'in 0.3.0 satırı: (a)'nın doğrulamaları ve kapatan kararları), `Cargo.lock`; `make test`, `make lint`, `make conformance`; `scripts/release.sh` + `scripts/release_check.sh` → R1–R20 hepsi `ok` (R14: etiket yok; R15 gerçek Obsidian e2e bu makinede); PR (`gh pr create --base main --head release/0.3.0 --title "rootmark 0.3.0"`), CI yeşil.
- (c) `HUMAN_STEP.md`: insan adımları sırasıyla (PR birleşmesi; `gh workflow run release.yml --ref main`; `release-json`'da iki zorunlu hedef `ok`; `git tag -a v0.3.0` birleşmiş `main` commit'inde ve itme; GitHub sürümü `release-files` artefaktıyla). Ajan sürümü bekler (`gh release view v0.3.0`), sonra `release.yml` koşusunun `release.json`'unu ve sürümün `SHA256SUMS`'ını `reports/rounds/P2-Q/release/`'e alır.

**Kabul:** Q-2, Q-3.

## Q2 — Aşama 1: Mac
**Yapar:**
- (a) Kurulum (`install.sh`, sürümden, scratchpad öneki); `progress/Q2.md`'ye süre, `--version`, `--spec-version`, conformance, kurulu ikilinin ve arşivin blake2b'si.
- (b) qorbit kopyası: senaryo (ön-kayıt) sürücüyle; iki taze kopya (determinizm). Kayıtlar `reports/rounds/P2-Q/mac/qorbit/` (adım sonuçları, sayılar, `events_hash`, indeks dosyalarının blake2b'leri, sorgu sonuçlarının blake2b'leri ve beklenenle karşılaştırması).
- (c) Obsidian kopyası: gerçek Obsidian ayrı profille, 0.3.0 eklentisiyle; ön-kayıtlı düzenlemeler; kayıtlar `reports/rounds/P2-Q/mac/obsidian/` (her düzenlemenin katmanı, `attest` sonuçları ve hata gerekçeleri, gecikmeler, md bayt denetimi). Harness durması o adımın kayıtlı başarısızlığıdır (Ş1 ilkesi).
- (d) İnsan denemesi isteğe bağlı: `HUMAN_STEP.md`'de kullanıcıya kopyanın yolu ve nasıl açılacağı; `progress/DIARY.md`.
- Bulgular geldikçe `reports/rounds/P2-Q/FINDINGS.md`'ye (`PQ-<n>`).

**Kabul:** Q-4, Q-5, Q-6.

## Q3 — Aşama 2: Linux sunucu
**Yapar:**
- Önkoşul `HUMAN_STEP.md`: kullanıcı erişimi verir (SSH; mümkünse ayrı OS kullanıcısı; ajan sunucu adresini kayda yazmaz). Erişim yoksa Q3 **PARK** (GOAL karar kuralı), Q4'e geçilir.
- Önce: qorbit servis listesi/durumları ve dinlenen portların blake2b'si. Kurulum (Linux arşivi, ayrı dizin); conformance; qorbit md'sinin sunucudaki kopyasıyla Q2 (b)'nin senaryosu (Obsidian'sız); Mac ↔ Linux karşılaştırması (`events_hash`, vektör dosyaları, sıralamalar). Sonra: aynı blake2b'ler. Kayıtlar `reports/rounds/P2-Q/linux/`.
- Kurulumu ve kopyaları sunucudan kaldırır (kullanıcı tutmak istemedikçe); kaldırma `progress/Q3.md`'de.

**Kabul:** Q-7.

## Q4 — Bulgular, gözden geçirici, rapor, hazır
**Yapar:**
- `FINDINGS.md` tamam (her bulgu sınıflı, hedefli); `numbers.json` + `numbers.md` (Mac ve Linux; her sayının kaynak kaydı); P2-C için hız/ölçek eşiği önerisi ve hız turu (P2-H) için hedef önerisi (öneri, karar değil).
- İçerik sızıntısı denetimi (Q-8): `eval.portable --all` turun kayıtlarında; kopyaların her belgesinden tohumla seçilen 32 baytlık parçaların depoda (`git grep -F`) ve PR metninde hiç geçmediği.
- `make test`, `make lint`, `make conformance`, `make verify FAST=1`, `check_frozen`, `check_deps`, `check_dep_age`.
- Gözden geçirici tek tur; başlık: "gerçek içerik sızdı mı; senaryo ön-kayıtla aynı mı; bulgular sınıflı ve hedefli mi; sayılar kayıtlardan mı; canlı sisteme dokunuldu mu".
- `reports/rounds/P2-Q/FINAL.md` (Q-1..Q-9, karar kuralının dalı, koşu kimlikleri, süreler); READY; tek PR (`gh pr create --base main --head p2-q/round --title "P2-Q: pilot ve 0.3.0" --body-file reports/rounds/P2-Q/FINAL.md`). Birleşmeden sonra yerel dallar ve çalışma dizinleri temizlenir (K-P2V2.2).

**Kabul:** Q-8, Q-9; READY "doğrulama için hazır" + commit sha + PR'ın CI koşu kimliği.
