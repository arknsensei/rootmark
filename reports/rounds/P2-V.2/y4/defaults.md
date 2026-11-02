# Y4 (c) — Düğmesiz davranış aynen (V2-4 (c))

Tam koşu yapılmaz (GOAL: yeni ölçüm yok). Kanıt: varsayılanlar ve komut satırlarının farkı (`ROUND_START` `89c4b28` ↔ bu tur).

| | `ROUND_START` | P2-V.2 (düğme yokken) |
|---|---|---|
| `make verify-mac` | `bash scripts/verify_mac.sh` | aynı (`Makefile` farkı yok: `git diff 89c4b28 -- Makefile` boş) |
| tekrar sayısı | `reps=3` | `reps="${VERIFY_MAC_REPS:-3}"` → 3 |
| `summary --reps` | `"$reps"` | aynı |
| D-10 düzenleme başına koruma | `Duration::from_secs(10)` (`t0.elapsed() < 10 s` değilse panik) | `ROOTMARK_D10_DEADLINE_MS` yoksa `Duration::from_millis(10_000)` (`t0.elapsed() >= 10 000 ms` ise kayıt + panik): aynı sınır |
| SIGTERM başlama beklemesi | 60 s (`expect`) | 60 s (kayıt + panik) |
| SIGTERM çıkış beklemesi (ölçüm) | 15 s | 15 s (düğmeden bağımsız, ASSUMPTIONS Y2-1) |
| başarı yolunun `d10.json`/`sigterm.json` alanları | `edits, latencies_ms, p95_ms, max_ms, layers_canonical_human, sigterm_exit_0, sigterm_compacted, pass` / `trials, timer_compactions, pass` | aynı (yeni alanlar yalnız zaman aşımı yolunda) |
| okuma | `eval.verify_mac` varsayılanı P2-V.1 envanteri | P2-V.2 envanteri (F3; K-P2V1.2 T5) — kasıtlı değişiklik |

Komut satırlarının farkı (yorumlar dışında; `verify_mac_sh_commands.diff`): yalnız `reps` ataması (+ doğrulaması) ve dört kayıt denetimi. Her denetim **kayıt varken** önceki gibi hiçbir şey yapmaz (koşul doğru, `||` dalı koşmaz; sürüm kaydında `cp` aynı koşulun `then` dalında); yalnız kayıt **yokken** `exit 1` yerine `--error <harness>=<günlük>` kaydedip devam eder. Koşan komutlar ve sıraları aynı; `bash -n` temiz.
