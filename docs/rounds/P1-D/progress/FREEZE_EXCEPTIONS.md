# P1-D — Dondurma istisnaları

Prosedür: docs/DEV_PROCESS.md §4 — (a) girdi (ne, neden, diff), (b) önce/sonra bit eşitliği kaydı, (c) gözden geçirici onayı. Her istisna `research/FROZEN.md` tablosuna aynı kalıpla **eklenen** bir yeniden dondurma satırıyla eşleşir (satırın notu `P1-D/F<n>` anar). Eski satırlar silinmez ve değişmez.

Ortak neden (F1–F3): TASKS D0 `mdk` ikilisini `core/src/bin/mdk.rs`'ten `server/src/bin/mdk.rs`'e taşır ve `core` artık ikili üretmez. `cargo build -p mdkernel-core --bin mdk` bu yüzden "no bin target named `mdk`" ile başarısız olur. Üç dondurulmuş betik güncel `mdk`'yı bu komutla derliyordu. Her betikte **yalnız derleme satırının paket adı** değişti (`mdkernel-core` → `mdkernel-server`). Betiğin geri kalanı, vakaları ve çıktı biçimi aynı kaldı.

## F1 · D0 · `scripts/check_v3_compat.sh` — güncel `mdk` `server`'dan
- **Ne:** satır 34 `(cd "$here" && cargo build --release -q -p mdkernel-core --bin mdk)` → `-p mdkernel-server`. Satır 32 (v3-final ağacının `mdk`'sı, `git archive v3-final core`) değişmedi; o ağaçta ikili hâlâ `core`'dadır. `git diff`: +1/−1.
- **Önce/sonra bit eşitliği:** `bash scripts/check_v3_compat.sh <out> <mixed>` çıkış 0; `<out>` ve `<mixed>` P1-C.1 resmi RUN'ının (`reports/rounds/P1-C.1/runs/20261015T094412Z-b9f7622/`) `v3_compat.json` ve `v3_compat_mixed.json` dosyalarıyla `cmp` ile **bayt eşit**.
- **Gözden geçirici onayı:** D0.review.md.

## F2 · D0 · `scripts/test_p1c1.sh` — `mdk` `server`'dan
- **Ne:** satır 31 `cargo build -q --release -p mdkernel-core --bin mdk -p mdkernel-index --bin mdk-index` → `-p mdkernel-server --bin mdk -p mdkernel-index --bin mdk-index`. +1/−1.
- **Önce/sonra bit eşitliği:** çıkış 0, 45/45; `p1c1_cli.json` resmi RUN'ın `p1c1_cli.json`'ıyla `cmp` ile **bayt eşit**.
- **Gözden geçirici onayı:** D0.review.md.

## F3 · D0 · `scripts/test_tamper_p1c.sh` — `mdk` `server`'dan
- **Ne:** satır 23, F2 ile aynı değişiklik. +1/−1. `mdk-index` (model doğrulama vakası) hâlâ `index` paketinden derlenir.
- **Önce/sonra bit eşitliği:** çıkış 0. Günlük, resmi RUN'ın `tamper_p1c.log`'uyla satır satır aynı. Tek fark `mktemp` dizin sonekleri (`mdk-tamper-p1c.XXXX`), bu sonekler her koşturmada değişir.
- **Gözden geçirici onayı:** D0.review.md.
