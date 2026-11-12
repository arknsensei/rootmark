# R0 — Hataların 0.3.2'de yeniden üretimi

Yayımlanmış `v0.3.2` (GitHub sürümü; arşivler `SHA256SUMS`'la denetlendi: `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`), sabitlenmiş model (`models/MANIFEST.json` ile denetlendi), D-0'ın sondaları değiştirilmeden: `research/d0/pq2_1_mac.sh`, `pq2_1_docker.sh` (`debian:trixie-slim`, `linux/amd64`), `offline_probe.py` (macOS: `python3 -E -s -m d0.offline_probe`), `offline_docker.sh`. Sentetik vault, kişisel içerik yok. Kayıtlar `reports/rounds/P2-Q.3/r0/` (sayılar, sürümler, komutlar; içerik, mutlak yol, host adı yok).

| Durum | macOS arm64 | Linux x86_64 (Docker) |
|---|---|---|
| SIGKILL, kaldırma beklemesi içinde, silme (0,5 s) | kalıcı 3/3 | kalıcı 3/3 |
| SIGKILL, kaldırma beklemesi içinde, taşıma | kalıcı 3/3 | kalıcı 3/3 |
| SIGTERM kontrolü (5 s), silme | alındı 3/3 | alındı 3/3 |
| SIGTERM kontrolü, taşıma | alındı 3/3 | alındı 3/3 |
| kapalıyken düzenle → yeniden başlatmada alındı | 0/3 | 0/3 |
| kapalıyken sil → alındı | 0/3 | 0/3 |
| kapalıyken ekle → alındı | 0/3 | 0/3 |
| `ingest-dir` (kapalı): düzenleme / ekleme / **silme** | 3/3 / 3/3 / **0/3** | 3/3 / 3/3 / **0/3** |

Dosyalar: `pq2_1_macos.json`, `pq2_1_linux.json`, `offline_macos.json`, `offline_linux.json`. Sonuç D-0'ın ölçümüyle aynı (D0F-1: SIGKILL 20/20 kalıcı; D0F-5: 0/3, `ingest-dir` silme 0/3). İki hata 0.3.2'de iki platformda yeniden üretildi (Q3-1'in ikinci yarısı).
