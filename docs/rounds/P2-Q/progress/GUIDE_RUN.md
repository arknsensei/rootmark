# P2-Q — Kılavuz koşusu (`docs/GUIDE.md`; GOAL Q-4, TASKS Q1 (b), Q2 (a))

Koşucu `research/p2q/guide_run.py`: kılavuzun `sh` bloklarını yazıldığı gibi, sırasıyla, tek bir bash oturumunda (`HOME` = turun çalışma dizininde bir sandık; `~` yolları oraya düşer) koşar; her blok bir ERR tuzağı altındadır, başarısız komut satırıyla kaydedilir. Terminalin yapamadığı yalnız şunlar düzenlenir: §3'ün ön plandaki `serve`'ü arka planda; kılavuzun sunucuyu durdurduğu yerlerde (§4.1, §6.1, §8) durdurma ve okuyucunun yeniden başlattığı yerde (§4.1, §6.1 sonrası) başlatma; §1.4 (kaldırma) en sona; Docker blokları Q3'e; Obsidian'daki tıklamalar (§5.1 adım 1–3) Obsidian harness'ine. Blokların metni değiştirilmez. Tam döküm (`guide_run.log`) depo dışında kalır.

Örnek vault: üç sentetik not (`coffee.md`, `tea.md`, `research/kahve.md`; kişisel içerik değil).

| Koşu | İkili | Kılavuz | Sonuç |
|---|---|---|---|
| C1 (Q1, aday) | `release/0.3.0` `49fd5cd`'nin derlemesi (`rootmark 0.3.0`, yerel) | `49fd5cd`'den önceki taslak | 20 blok `ok`, 0 başarısız; atlanan 5: `1.1#1`, `1.2#1`, `5.1#1` (sürüm henüz yok: indirme), `1.3#1`, `1.3#2` (Docker, Q3) |

## Sapmalar (her biri bir belge bulgusu, kılavuz düzeltildi, yeniden koşulacak)
| # | Blok | Komut | Beklenen | Gerçek | Düzeltme |
|---|---|---|---|---|---|
| G1 | `4.1#1` | `claude mcp add rootmark --scope project … && claude mcp list` | sunucu bağlanır | çıkış 0, ama `⏸ Pending approval (run claude to approve)`: proje kapsamlı sunucu onay ister | §4.1'e onay adımı eklendi (`claude`'u o dizinde başlatıp `rootmark`'ı kabul etmek) |
| G2 | `4.3#1` | `jq … events.jsonl \| sort -u` | ajan adları | boş çıktı: henüz hiçbir ajan `propose` etmedi (kılavuz sırası) | "(after their first `propose`)" eklendi |

Belge bulguları ayrıca (koşudan önce, adayla denemede): en küçük tarif `missing field version_trigger` ile reddediliyor (§2.2 tam tarif verir ve tabloları "required" diye anar); `rootmark propose` (CLI) sunucu çalışırken kullanılmamalı (§6.1 sunucuyu durdurmayı söyler); `migrate --dry-run`'ın çıktısında `required`/`recommended` üst alanları yok (`.plan[]`).

Q2'de: yayımlanmış `v0.3.0` ile bütün bloklar (`--mode release`), düzeltilmiş kılavuzla.
