# P2-Q.4 — İnsan adımı 2 (2026-11-14): K-P2Q4-T1'in kararı ve 0.3.4'ün yayımı

**Adım 1 yapıldı** (zorla itmeler; `origin/p2-q.4/round`, `origin/release/0.3.4` yeniden temellendirilmiş uçlarda). Aşağıda yalnız adım 2 beklenir.

## 2a. Karar: K-P2Q4-T1 (`docs/DECISIONS.md`, açık talepler)
Kademe 2'nin tek tetiklemesi `43485696909` çift karşılaştırmasında kaldı (4 alan, PQ4-5); kaynağı CI Linux'ta SIGTERM kaydedicisinin 15 s'de çıkmayan denemeleri (PQ4-6; A 7/10, B 1/10). Aynı makinede 0.3.3 ve 0.3.4'ün aynı davrandığı gösterildi (`reports/rounds/P2-Q.4/r4/sigterm_diag.json`); bu turun gerilemesi gösterilemedi. Seçenekler (a) okuma + sonraki tura, (b) ikinci tetikleme, (c) yayım bekler. **Öneri (a).** Karar `docs/DECISIONS.md`'ye K-P2Q4.x olarak yazılır (ya da kararı oturuma bildirin, ajan yazar).

## 2b. 0.3.4'ün yayımı (2a (a) ya da (b) ise; `docs/RELEASE.md` "İnsan adımları" 1–5)
Durum: yayım PR'ı **#69** (`release/0.3.4` → `main`, uç `05a4c57`) — CI `43511347182` **bütün işler yeşil**; yerel paket + `release_check` **21/21** (gerçek Obsidian e2e 11/11; R4 aarch64-apple-darwin ve x86_64-unknown-linux-gnu `ok`). #69, PQ4-4'ün düzeltmesini de taşır.
1. #69'u birleştirin (`main` korumalı; birleştirme insanındır).
2. `gh workflow run release.yml --ref main` → `release-json`'da iki zorunlu hedef `ok`.
3. Birleşmiş `main` commit'inde `git tag -a v0.3.4 -m "rootmark 0.3.4"` ve `git push origin v0.3.4`; GitHub release'e varlıklar (`docs/RELEASE.md` adım 4; yerel `dist/` `…/scratchpad/wt_rel/dist/`).
4. (İsteğe bağlı) Homebrew tap'i (adım 5).

Ya da bu oturuma açıkça izin verin: ajan (1)–(3)'ü sizin adınıza yapar ve her adımı kaydeder.

**Sonra ajan sürer:** yayımlanmış 0.3.4 ile P1/P2 durumları (macOS + Docker), ≥ 3 gerçek Claude Code oturum sonu sınaması (P3), D-NOTES Mac ↔ Docker ↔ CI (`pilot.yml`) ↔ 0.3.3 (P4); R4'ün kapanışı; R5 (kalite kapıları, gözden geçirici, FINAL, READY, tek PR).

---

# (Geçmiş) İnsan adımı 1 (2026-11-13): yeniden temellendirilmiş iki dalın zorla itilmesi

**Neden:** tur `b6f7193`'ten açıldıktan dakikalar sonra `main`'e PR #68 girdi (`8369fd3`, K-P2Q3.4: `.github/workflows/ci.yml`, `docs/DECISIONS.md`, `docs/DEV_PROCESS.md`). Yayım PR'ı #69 `main`'le `DECISIONS.md`'de çakıştı (`mergeStateStatus: DIRTY`) ve CI'ı başlamadı. İki dal yerelde `origin/main` üstüne yeniden temellendirildi; tek çakışma `DECISIONS.md`'deki iki yeni girdinin yeriydi (ikisi de korundu: K-P2Q3.4, sonra K-P2Q4.1). Yeniden temellendirmenin eklediği fark yalnız PR #68'in üç dosyası (`git diff backup/p2-q.4-round-pre-rebase p2-q.4/round --stat`); ürün kodu ve kayıtlar aynı. Önceki uç yerelde `backup/p2-q.4-round-pre-rebase` (`789c400`).

Ajanın `git push --force-with-lease`'i auto mod tarafından reddedildi; bu adım kullanıcınındır.

**Beklenen (depo kökünde):**
```
git push --force-with-lease=p2-q.4/round:789c400 origin p2-q.4/round
git -C /private/tmp/claude-501/-Users-arkns-Works-mdkernel/2e400508-7484-44d9-b907-96feef3d7e6c/scratchpad/wt_rel push --force-with-lease=release/0.3.4:e2c8c8d origin release/0.3.4
```
(ya da bu oturuma `--force-with-lease` iznini vermek). Sonra ajan sürer: #69'un CI'ı, yerel paket ve `release_check`, yayımın insan adımları (adım 2).

**Not:** kademe 2'nin sayılan tetiklemesi `43485696909` yeniden temellendirmeden önceki `b65ef1d`'nin ağacını ölçer; yeniden temellendirilmiş karşılığı `658e8f8` ile ürün ağacı aynıdır (fark yalnız PR #68'in CI/karar dosyaları). Mac kümesi (RUN `20261113T191922Z-789c400`) `789c400`'ın klonunda koşuyor.
