# P2-Q.4 — İnsan adımı 1 (2026-11-13): yeniden temellendirilmiş iki dalın zorla itilmesi

**Neden:** tur `b6f7193`'ten açıldıktan dakikalar sonra `main`'e PR #68 girdi (`8369fd3`, K-P2Q3.4: `.github/workflows/ci.yml`, `docs/DECISIONS.md`, `docs/DEV_PROCESS.md`). Yayım PR'ı #69 `main`'le `DECISIONS.md`'de çakıştı (`mergeStateStatus: DIRTY`) ve CI'ı başlamadı. İki dal yerelde `origin/main` üstüne yeniden temellendirildi; tek çakışma `DECISIONS.md`'deki iki yeni girdinin yeriydi (ikisi de korundu: K-P2Q3.4, sonra K-P2Q4.1). Yeniden temellendirmenin eklediği fark yalnız PR #68'in üç dosyası (`git diff backup/p2-q.4-round-pre-rebase p2-q.4/round --stat`); ürün kodu ve kayıtlar aynı. Önceki uç yerelde `backup/p2-q.4-round-pre-rebase` (`789c400`).

Ajanın `git push --force-with-lease`'i auto mod tarafından reddedildi; bu adım kullanıcınındır.

**Beklenen (depo kökünde):**
```
git push --force-with-lease=p2-q.4/round:789c400 origin p2-q.4/round
git -C /private/tmp/claude-501/-Users-arkns-Works-mdkernel/2e400508-7484-44d9-b907-96feef3d7e6c/scratchpad/wt_rel push --force-with-lease=release/0.3.4:e2c8c8d origin release/0.3.4
```
(ya da bu oturuma `--force-with-lease` iznini vermek). Sonra ajan sürer: #69'un CI'ı, yerel paket ve `release_check`, yayımın insan adımları (adım 2).

**Not:** kademe 2'nin sayılan tetiklemesi `43485696909` yeniden temellendirmeden önceki `b65ef1d`'nin ağacını ölçer; yeniden temellendirilmiş karşılığı `658e8f8` ile ürün ağacı aynıdır (fark yalnız PR #68'in CI/karar dosyaları). Mac kümesi (RUN `20261113T191922Z-789c400`) `789c400`'ın klonunda koşuyor.
