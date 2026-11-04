# P2-Q — Kılavuz koşusu (`docs/GUIDE.md`; GOAL Q-4, TASKS Q1 (b), Q2 (a))

Koşucu `research/p2q/guide_run.py`: kılavuzun `sh` bloklarını yazıldığı gibi, sırasıyla, tek bir bash oturumunda (`HOME` = turun çalışma dizininde bir sandık; `~` yolları oraya düşer) koşar; her blok bir ERR tuzağı altındadır, başarısız komut satırıyla kaydedilir. Terminalin yapamadığı yalnız şunlar düzenlenir: §3'ün ön plandaki `serve`'ü arka planda; kılavuzun sunucuyu durdurduğu yerlerde (§4.1, §6.1, §8) durdurma ve okuyucunun yeniden başlattığı yerde (§4.1, §6.1 sonrası) başlatma; §1.4 (kaldırma) en sona; Docker blokları Q3'e; Obsidian'daki tıklamalar (§5.1 adım 1–3) Obsidian harness'ine. Blokların metni değiştirilmez. Tam döküm (`guide_run.log`) depo dışında kalır.

Örnek vault: üç sentetik not (`coffee.md`, `tea.md`, `research/kahve.md`; kişisel içerik değil).

| Koşu | İkili | Kılavuz | Sonuç |
|---|---|---|---|
| R1 (Q2, yayımlanmış) | `v0.3.0` (`install.sh` ile kurulan; blake2b arşivdekiyle eşit `a8f25965…`) | `v0.3.0` etiketindeki `docs/GUIDE.md` (G1, G2 ve Demote düzeltmeleri dahil) | **23 blok `ok`, 0 başarısız**; atlanan yalnız `1.3#1`, `1.3#2` (Docker, Q3). §1.1 indirme + `SHA256SUMS` + `gh attestation verify` (çıkış 0) + kurulum; §1.2 çok dilli model; §5.1 eklenti zip'i `SHA256SUMS.local` ile `zip OK`; §7 `observe.py` etiketten indirildi ve koştu; §4.1 `claude mcp list` → `Pending approval` (kılavuzun söylediği gibi) |
| C1 (Q1, aday) | `release/0.3.0` `49fd5cd`'nin derlemesi (`rootmark 0.3.0`, yerel) | `49fd5cd`'den önceki taslak | 20 blok `ok`, 0 başarısız; atlanan 5: `1.1#1`, `1.2#1`, `5.1#1` (sürüm henüz yok: indirme), `1.3#1`, `1.3#2` (Docker, Q3) |

## Sapmalar (her biri bir belge bulgusu, kılavuz düzeltildi, yeniden koşulacak)
| # | Blok | Komut | Beklenen | Gerçek | Düzeltme |
|---|---|---|---|---|---|
| G1 | `4.1#1` | `claude mcp add rootmark --scope project … && claude mcp list` | sunucu bağlanır | çıkış 0, ama `⏸ Pending approval (run claude to approve)`: proje kapsamlı sunucu onay ister | §4.1'e onay adımı eklendi (`claude`'u o dizinde başlatıp `rootmark`'ı kabul etmek) |
| G2 | `4.3#1` | `jq … events.jsonl \| sort -u` | ajan adları | boş çıktı: henüz hiçbir ajan `propose` etmedi (kılavuz sırası) | "(after their first `propose`)" eklendi |

Belge bulguları ayrıca (koşudan önce, adayla denemede): en küçük tarif `missing field version_trigger` ile reddediliyor (§2.2 tam tarif verir ve tabloları "required" diye anar); `rootmark propose` (CLI) sunucu çalışırken kullanılmamalı (§6.1 sunucuyu durdurmayı söyler); `migrate --dry-run`'ın çıktısında `required`/`recommended` üst alanları yok (`.plan[]`).

Q2'de: yayımlanmış `v0.3.0` ile bütün bloklar (`--mode release`), düzeltilmiş kılavuzla.

## R1 notları (sapma değil)
- `gh attestation verify` TTY olmayan çıktıda hiçbir şey yazmaz; çıkış 0 doğrulamadır (koşucu ERR tuzağıyla denetler).
- `fetch_model.sh` kurulu kullanıcıya "run `make models` for the blake3 manifest check" der — kurulumda olmayan bir depo komutu (PQ-17, UX).
- Sahneler için kurulum ayrı bir önekte kılavuzun kurulum bloklarıyla (`--only 1.1#1,1.1#2,1.2#1`; 3/3 `ok`, 32 s), çünkü tam koşu §1.4'le kaldırarak biter.

## Kılavuz ↔ sahneler (gözden geçirici N-?: eşleme)
| Sahne adımı | Kılavuz bölümü | Nasıl |
|---|---|---|
| kurulum (Mac, Docker, CI) | §1.1, §1.2, §1.3 | blokların kendisi (`guide_run --only`) |
| H02 `ingest-dir`, H03 tarif + `policy lint`, H04 `index build` | §2.1, §2.2, §2.3 | aynı komutlar; tarif `scenario.json`'un ön-kayıtlı tarifi (§2.2'nin biçimi) |
| H05 `serve --mcp stdio --http … --watch` | §3, §4.2 | aynı bayraklar; port `127.0.0.1:0` (sahnenin kendi portu) ve `--attest-log` (§7) |
| ajan (MCP `propose`), insan (HTTP `promote`/`demote`/`search`/`blame`/`trace`/`checkpoint`/`prune`) | §4, §6 | aynı araçlar ve gövdeler (sürücünün HTTP istemcisi, §3'ün `rmk`'si yerine) |
| O05 eklenti | §5.1 | kabuk bloğu yazıldığı gibi; Obsidian içindeki iki tıklama Obsidian API'siyle (PQ-23) |
| `observe.py` | §7 | aynı betik ve bayraklar |
BRAT yolu (§5.3) koşulmadı (PQ-23).
