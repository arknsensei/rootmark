# P2-Q.2 — Bağımsız Doğrulama (kısa; `docs/rounds/P2-Q.2/VERIFY.md` VR1–VR4)

**Doğrulanan commit:** `ff548727733431b44dc674c11861e479ed0f5a46` (`main`; PR #57, rebase ile birleşti 2026-11-11T07:46:52Z). Ağacı READY commit'i `f6e8d6e`'ninkiyle eşit (`d6c94118…`). `a11fd88..f6e8d6e` yalnız `docs/rounds/P2-Q.2/progress/` (READY). Yayımlanmış `v0.3.2` `c544a9e`'de; `c544a9e..ff54872` arasında `server/src`, `core`, `index`, `bindings` farkı yok.

Ayrı oturum, kendi scratchpad'imde GitHub'dan temiz klon (önce `make setup` ve `make models`, ikisi de çıkış 0), ana çalışma dizinine erişim yok, depo kodu değiştirilmedi (tek kasıtlı bozma atılan bir çalışma ağacında, VR3). Kişisel içerik okunmadı: bütün koşularım açık korpus D-NOTES'un kopyaları ya da kendi sentetik notlarım üzerinde. Mac kümesi koşulmadı (VERIFY; turun kaydı okundu). Ortam: Apple M4 Max, macOS 27.0; Docker 29.8.0 (Rosetta, `linux/amd64`); **Claude Code 2.1.322** (turun 2.1.319'undan yeni). Karşılaştırmalar kendi betiklerimle (`old_client.py`, `vr2_claude.sh`, `vr2_promote.py`, `vr3cmp.py`, `myscope.py`; turun `compare.py`'si, `claude_probe.py`'si ve dondurulmuş karşılaştırıcılar kullanılmadı; betikler scratchpad'te kalır). Süre: ~2 sa 50 dk (hedef ≤ 2 saat; fazlası kademe 2'nin 2 sa 29 dk'lık zincirleri, öteki işler onunla koşut koştu).

## VR1 — Tasarım ve sınır

- **Sıra (`git log`, birleşmiş `main`):** `b667326` R0 (`DESIGN.md` + `TESTS.md`, 22:01) → `9cd7a23` R1'in sabit dosyası ve yeni test (22:03; ürün kodu değil) → `11a1d97` R1'in ilk ürün değişikliği, `rmcp =3.6.0` (22:08). R2: `e67d2b4` TESTS R2 eki (22:14) → `0ce5dee` test ayarı (22:19) → `1f4aff2` düzeltme (22:24). CI yapılandırması: `99b3803` DESIGN §6 eki (02:04) → `c544a9e` `release.yml` (02:06). Tasarım ve ön-kayıtlı testler koddan önce.
- **Kök neden ↔ rmcp kaynağı (kendim okudum, `~/.cargo/registry`):**
  - `rmcp-3.5.0/src/service/server.rs`: `initialize` olmayan ilk istek 2026-07-28'in zorunlu meta anahtarlarını tam taşıyorsa oturum satır içi yaşam döngüsüne geçiyor ve `peer.require_request_metadata()` konuyor (satır 625). `server/discover` da bu yoldan geçiyor. Sonraki meta'sız `tools/list` `-32602` alıyor. DESIGN §1.2 ile aynı.
  - `rmcp-3.6.0`: aynı dosya, satır 556–561: "Select the lifecycle only after an initialize request or the first valid non-discover request with complete inline metadata. A discover request is a bootstrap probe: respond to it, but remain open to either lifecycle." DESIGN §1.3'ün alıntısı birebir.
  - `request_metadata_required` iki sürümde de `pub(crate)` alan (`service.rs`): "meta denetimini gevşetme" seçeneğinin olmadığı doğru.
- **Fork/yama yok:** `server/Cargo.toml` `rmcp = "=3.6.0"`; kilitte `rmcp` ve `rmcp-macros` 3.6.0, kaynak `registry+…crates.io-index`; çalışma alanında `[patch]` yok, `vendor/` yok.
- **Yaş:** kendi `scripts/check_dep_age.sh` koşum: 135 bağımlılık, hepsi `ok`; `cargo rmcp 3.6.0 — published 2026-10-20, adopted 19 days later`.
- **Ürün sınırı (`git diff 76977e0..ff54872`):**
  - `core`, `index`, `bindings`, `spec`, `clients/*/src`: yalnız üç `Cargo.toml` `version` satırı (0.3.1 → 0.3.2).
  - `server/`: `Cargo.toml` (sürüm + `rmcp`), `src/serve.rs` (izleyici, +84/−16), yeni `tests/mcp_compat.rs`, `tests/watch_hold.rs`, `tests/fixtures/p2q2/`. **`mcp.rs` değişmedi.** Dondurulmuş testler değişmedi.
  - Öteki: `Cargo.lock` (rmcp, rmcp-macros, rmcp→libc kenarı, sürüm satırları), `.github/workflows/release.yml` (tek satır `macos-14` → `macos-15` + açıklama; DESIGN §6 ekiyle önceden adlı), istemcilerin `package.json`/kilidi/`manifest.json` (sürüm + `minimatch` `overrides`), `research/p2q2/`, belgeler.
- **Kod okuması (`serve.rs`):** `due` artık belge döngüsünden **sonra** hesaplanıyor; belge alan bir toplu iş, başlangıcı bir kaldırmanın ilk `due`'sundan önceyse o kaldırmayı bir kez uzatıyor (sınırlı). Test ayarı `ROOTMARK_WATCH_TEST_INGEST_DELAY_MS` boşsa/yoksa davranış aynı, sayı değilse hata (sessiz değil), verildiğinde stderr'e yazılıyor. DESIGN §4.3 ile aynı.

## VR2 — Hatayı kendim ürettim, kapandığını gördüm

Yayımlanmış `v0.3.1` ve `v0.3.2` arşivleri (`gh release download`; `SHA256SUMS` OK, köken tasdiki VR4'te). İki ayrı D-NOTES kopyası (`research/data/d_notes/notes`), kılavuz §2.2'nin reçetesi (yalnız `http_port` 7457/7458 ve `human_id = "verifier"`), `ingest-dir` + `index build`: iki kasada da `events_hash` `35ba798a639429d33f3ce07005058e85` (turun R0/R4'üyle aynı). Model: `make models`'in dosyaları.

### Gerçek Claude Code 2.1.322, başsız (kılavuz §4.1'in adımları)
Her sürüm için ayrı proje dizininde: `claude mcp add rootmark --scope project -e ROOTMARK_MODELS=… -- <ikili> serve --vault <kasa> --mcp stdio --http 127.0.0.1 --watch` → `claude mcp list` (`⏸ Pending approval`, kılavuzdaki gibi) → kılavuzun başsız denetimi `claude -p "How many tools…"` → asıl sınama `claude -p "<liste, search, propose>" --mcp-config <proje>/.mcp.json --strict-mcp-config --allowedTools mcp__rootmark__search,mcp__rootmark__propose --output-format json --debug-file …`.

| | 0.3.1 | 0.3.2 |
|---|---|---|
| Bağlantı | `Successfully connected`, `protocolEra: "legacy"`, `serverVersion 0.3.1` | aynı, `0.3.2` |
| `tools/list` | **`[ERROR] … Failed to fetch tools: request _meta is missing or has malformed required fields: io.modelcontextprotocol/protocolVersion, io.modelcontextprotocol/clientCapabilities`** — 0 araç | **11 araç**: blame, checkpoint, checkpoints, events, history, promote, propose, read_block, search, status, trace |
| Kılavuzun başsız denetimi | "no rootmark tools are visible … 0 tools" | "The rootmark MCP server offers 11 tools: …" (kılavuzun beklediği çıktı) |
| `search` ("dataview", k=3) | FAILED (araç yok) | SUCCEEDED, 3 sonuç |
| `propose` (`verifier/claude-code-probe.md`, create) | FAILED (araç yok) | SUCCEEDED |
| Günlük | — | `BlockCreated` ×2 (seq 28494–28495) + `SnapshotCommitted`; `blame`: iki blok `layer = proposal`, **`author_kind = Agent`**, `source = claude-code/2.1.322` |

**PQ1-1, 0.3.1'de benim Claude Code'umla yeniden üretildi; 0.3.2'de kapalı.** Turun 2.1.319'undan üç sürüm yeni bir istemciyle de çalışıyor.

### Kendi JSON-RPC betiğim (`old_client.py`; stdio, `serve --mcp stdio`, sentetik tek notlu kasa)
| Biçim | 0.3.1 | 0.3.2 |
|---|---|---|
| Eski istemci: `initialize` **2025-06-18**, hiçbir istekte `_meta` yok | 11 araç, `search` ok, `propose` ok (`proposal`, `verifier-old/0.1`) | **aynı** |
| Claude Code'un kayıtlı biçimi: `server/discover` tam 2026-07-28 `_meta` → `initialize` 2025-11-25 → meta'sız `tools/list` | keşif `-32022`, `tools/list` **`-32602`** (PQ1-1) | keşif `-32022`, 11 araç, `search` ok, `propose` ok |

- **`tools/list` baytları:** eski istemcide 0.3.1 ve 0.3.2'nin ham yanıt satırı `cmp` ile **bayt eşit** (sha256 `9f4ca0c8…`); `result`'ın normalize JSON'u da eşit (`eeef6a12…`). 0.3.2'de Claude Code biçimiyle alınan satır da aynı (`9f4ca0c8…`).

## VR3 — PQ1-4, karşılaştırma ve kademe 2

### PQ1-4 (`server/tests/watch_hold.rs`, temiz klon, macOS)
- `cargo test --release -p rootmark-server --test watch_hold` **20 kez: 20/20 geçti** (her koşu H1–H5 5/5, ~157 s; H1 kendi içinde 20 geç tasdikli yeniden adlandırma → toplam 400). Ardından `watch_remove` 9/9, `watch_newdir` 3/3, `mcp_compat` 10/10.
- **Kasıtlı bozma (atılan çalışma ağacı):** düzeltmeyi geri alan tek değişiklik (`due` yine belge döngüsünden önce hesaplanır; test ayarı yerinde) → **H1 ve H5 kaldı** (`watch_hold.rs:246`). Test hatayı gerçekten yakalıyor; yalnız gecikme yokluğunda geçen bir test değil.
- Linux: PR CI'ı `42927226404` (`f6e8d6e`, ağacı `ff54872`'ninkiyle aynı) `ubuntu-latest`'te `watch_hold` 5/5 (154 s) ve `mcp_compat` 10/10; macOS'ta 5/5 (165 s) ve 10/10 (günlükten okudum).

### D-NOTES (P2-Q'nün sürücüsü `research/p2q/driver.py`, yayımlanmış 0.3.2)
- **Mac:** doğrulanmış arşivin ikilisi, `ROOTMARK_MODELS` = klonun `models/`. **Docker:** `research/p2q/docker_stage.sh` (ön-kayıtlı imaj, kılavuz §1.3#2 bloğu `VER=0.3.2` — bu kez GitHub'dan `curl` ile kuruldu, `guide_run` 1/1 ok). İkisi de **19/23**; kalanlar H02, H11, H15, H21 (P2-Q'den beri aynı, P2-C). **H09 geçti.** Sürücünün çıkışı 1 (kalan adımlar; turda da öyle).
- **Bayt eşitlik (`vr3cmp.py`; `hashes.json` ve `queries.json`, dosya ve yaprak yaprak):** benim Mac ↔ benim Docker ↔ turun Mac, Docker, CI `42575714093` ↔ P2-Q.1'in 0.3.1 Mac, Docker, CI `41996549106`, `41996563930`: `events_hash` `173c13df79d2d98dbff1df45453e193c` dokuzunda; `hashes.json` blake2b-128 `113d0184…`, `queries.json` `c42e42ca…` dokuzunda aynı; yaprak farkı **0**. İki düzeltme D-NOTES'un hiçbir baytını değiştirmemiş.

### Kademe 2 — kendi tetiklemem **`42982215036`**
`gh workflow run verify.yml --ref main -f sha=ff548727733431b44dc674c11861e479ed0f5a46`, 2026-11-11T07:48:38Z.
- İki zincir `success`, `exit` 0/0, `commit` dosyaları `ff54872…`. İşlemciler AMD EPYC 7763 ↔ AMD EPYC 9V74 (farklı). Zincirler 8 876 s (A 2 sa 29 dk, B 2 sa 5 dk), karşılaştırma 167 s.
- **Kendi D karşılaştırmam (`myscope.py`, yapıtlar `gh run download` ile):**
  - Kapsam: dondurulmuş çiftin dosya listesi veri olarak; karşılaştırmayı kendi kodum yapıyor (her JSON yaprağı tam eşitlik, bayt yolları bayt bayt). Envanterin adlı dışlamaları ve gömülü fark kuralı (kural 3), alan adlarındaki `·` = `.` kaçışı uygulandı.
  - **A ↔ B:** 87 dosya, 759 254 yaprak; bayt yolları 25 (28 dosya), **bayt farkı 0**, eksik dosya 0.
  - Zaman dışı yaprak farkı 5, hepsi dondurulmuş karşılaştırıcıların kaynakta adlı zaman/bilgi alanları (P2-Q.1 doğrulamasındakilerle aynı beşi): `h3_v3.json` `summary›p50/p95`, `p1d1/d1_6.json` ve `p1d1_checks.json` `D1-6›measured›ratio`, `p1e1/release/release.json` `targets›0›sha256`. Kalan 413 yaprak gecikme/süre alanı (10 ad).
  - **D farkı 0.** CI'ın `tier2_pair.json`'ı da `equal: true`, 788 892 alan, kenara alınan yalnız Z28.
  - **Kasıtlı bozma (karşılaştırıcımın sınaması, B'nin kopyasında):** `p1d/inspector/inspector.json`'a eklenen alan ve `p1d/mcp_e2e/run1/transcript.jsonl`'de tek bit yakalandı; envanterin dışladığı `p2_0_2/d_a/events.jsonl`'e (Z28) eklenen satır yakalanmadı (dışlama doğru uygulanıyor).
- **Karşılaştırma dışı liste = envanter:** `not_compared` kimlikleri `reports/rounds/P2-V.2/inventory.json`'ın 48 kimliğiyle birebir; desen (`*?[`) 0.
- **Sayılan FAIL (kendi sayımım, zincirlerin `fail_summary.json`'ından):** her zincirde 39 etiket — 23 tarihsel, 2 `migration_required` (K-P2A.2), 14 `fail`. 14'ün hepsi Mac kümesine ayrılan kriterler (C1-8, D-9, D1-12, E-7, E1-3, E1-4, E1-9, tamper, 0.3b, 0.3b-yük, 0.3c, 0.3d, 0.3e, `logs_vs_p2a`); küme A ve B'de ve P2-Q.1 doğrulamasında aynı. **Sayılan FAIL 0 / 0**; CI'ın `tier2_fails_{A,B}`'si de 0 (`tamper·det` tarihsel, `record_equal: true`).
- **`portable`:** A ve B **bulgu 0** (1 153 dosya, `skipped_by_name` 1).
- **Değişen kayıtlar ↔ R0'ın adları (DESIGN §6), aynı kapsamda kendi karşılaştırmamla:**
  - Benim A ↔ turun tek tetiklemesi `42515401135` (`d6eb311`, aynı ürün kodu, 0.3.1 dizgileri) A: zaman alanları dışında 23 yaprak + 2 bayt yolu. Hepsi adlı:
    - **Sürüm dizgileri (§6.1):** E-7 ve E1-5 `version`, `release_local.json`, `conformance_rootmark.json` `mdk_version`, `release.json` `version`/`targets›0›version`; `p1d/mcp_e2e/run1/transcript{,_b}.jsonl` bayt farkı — sürüm normalize edilince **eşit** (`serverInfo.version`, `instructions`, `mdk_version`).
    - **E1-4/R14 (§6.2):** `release_check` 18/21 → **17/21**, yeni kalan R14 (`v0.3.2` etiketi `c544a9e`'de, en yeni `ROUND_START` `76977e0`'ın atası değil) ve bunu taşıyan gömülü `regress` alanları.
    - Commit'e bağlı: `release.json` `commit` ve arşiv `sha256` (zaten A↔B bilgi alanı).
  - Benim A ↔ P2-Q.1'deki kendi tetiklemem `42471635466` (`08b2fa2`, 0.3.1) A: yalnız sürüm dizgileri, commit, arşiv sha'sı ve aynı iki transcript (sürüm). E1-4 ikisinde de 17/21 (o gün `v0.3.1` etiketi aynı durumdaydı).
  - R0'ın "değişmez" beklediği kayıtlar değişmedi: `p1d/inspector` (Inspector'ın `tools_list.json`'ı ve çağrıları) ve `p2_0_2/tamper_p2_0_2.json` iki karşılaştırmada da farksız; izleyiciye ya da MCP'ye bağlanabilen başka bir D kaydı farkı yok.
  - `release.yml` değişikliği (§6 eki) hiçbir kademe 2 kaydında fark üretmedi (R13 deseni aynı, E1-5 `workflow_r13` aynı). `minimatch` `overrides`'ı da fark üretmedi.
  - **Adsız fark: 0.**

### Kademe 1 (yerel)
`make verify FAST=1` (temiz klon, `ff54872`): çıkış 0, hızlı çift **eşit (18 550 alan)**, 1 273 s (RUN `20261111T090827Z-ff54872`). Turun PR'ında kademe 1 yeşil: `42921725577` (`a11fd88`) fast chain A 38 dk 14 s, B 35 dk 29 s (≤ 40 dk); READY commit'inin koşusu `42927226404` (`f6e8d6e`) de `success`.

### Mac kümesi (koşulmadı; turun kaydı okundu)
`reports/rounds/P2-Q.2/mac_set/mac_checks.json`: RUN `20261108T193444Z-d6eb311`, 3 tekrar (3 389, 1 729, 1 736 s), `pass: true`, okuma "P2-V.2 mac_reading". Koşudan önce `fseventsd` sağlığı R4'te kayıtlı. `d6eb311` ile `ff54872` arasında ürün kodu farkı yok (yalnız sürüm dizgileri, `release.yml`, `minimatch` kilitleri, belgeler).

## VR4 — Belgeler, güvenlik, karar kuralı

- **Kılavuz §4.1, Claude Code 2.1.322 ile izlendi** (VR2): `claude mcp add … --scope project` `.mcp.json`'ı kılavuzdaki biçimde yazıyor; `claude mcp list` "⏸ Pending approval (run `claude` to approve)" diyor; başsız denetim kılavuzun beklediği satırı veriyor (11 araç, aynı sırayla). Known issues'ta PQ1-1 yok; §4.1'deki "0.3.0 and 0.3.1 list no tools … fixed in 0.3.2" cümlesi benim 0.3.1 koşumla birebir (aynı hata dizgisi). Öneri 3a'nın maddesi (`unverified` altında tasdikli yeniden adlandırmanın eski yolu `unattested_fs`) kılavuzda yazılı; anahtar kuralı `docs/DEV_PROCESS.md` §2'de; `CHANGELOG.md` `## 0.3.2` var.
- **Güvenlik:** açık Dependabot **0** — #80/#81 (`minimatch`) birleşmeden 53 dk sonra, 2026-11-11T08:39:38Z `fixed` (ilk bakışımda, 08:25Z civarı, hâlâ açıktı; kilitler 11.0.1'i çözüyordu, bağımlılık grafiği 11.0.1). Açık CodeQL **0**.
- **0.3.2 sürümü:** `SHA256SUMS` 5/5 OK (üç arşiv, `install.sh`, `install.ps1`); `gh attestation verify` 5/5 ok. `SHA256SUMS.local`: eklenti zip'i ve wheel OK. `release.yml` `42574633881` (`c544a9e`): üç hedef ve `dist/release.json, SHA256SUMS, provenance` `success`; `v0.3.2` → `c544a9e`, 12 dosya, 2026-11-09T06:48:52Z. Yayımlanmış ikiliyle `rootmark conformance`: **level 3, 39/39**.
- **Karar kuralı (GOAL'den kendim):** Q2-1 geçti → park yok; Q2-2, Q2-3 geçti → PQ1-1 kapandı, sözleşme aynen; Q2-4 geçti → PQ1-4 kapandı; Q2-5..Q2-9 geçti → **"Hepsi geçti"**: sıradaki D-0, sonra D-1. FINAL'in dalıyla aynı.
- **Vaat sorusu — ajan yolu yeni bir kullanıcı için baştan sona çalışıyor mu? Evet.** Kılavuzu izleyerek: Claude Code 2.1.322 `propose` → bloklar `proposal`, `author_kind = Agent` → varsayılan `search`'te görünmüyor (0 isabet) → kişi HTTP'den (kılavuz §3 token, §6.4 `promote`, `reason`'lı) iki soyu onaylıyor → `Promoted` olayı `actor = human:verifier`, `policy = human_only`, `from proposal → to canonical` → bloklar `canonical`, yazarı hâlâ `Agent`/`claude-code/2.1.322` → varsayılan `search` artık buluyor (1 isabet).

## Sonuç

### İhlal / fark / şüphe
- **İhlal:** yok. Ürün sınırı (yalnız `serve.rs`, `Cargo.toml`/`Cargo.lock`, yeni testler; `mcp.rs` bile değişmedi), fork yok, dondurulmuş dosyalarda değişiklik yok, markdown'a kimlik yazılmıyor, sessiz başarısızlık yok (test ayarı geçersizse hata).
- **Fark 1 — `SHA256SUMS.local`'daki arşiv satırları yayımlanmış arşivlerle uyuşmuyor.** Dosya yerel `dist/`'in (yerelde derlenmiş `bin/rootmark-0.3.2-*.tar.gz`) sağlamalarını taşıyor; yayımlanan arşivler `release.yml`'in derlemesi, aynı adla. `shasum -c` ile bakan biri iki MISMATCH görür. 0.3.1'de de aynı (önceden var, bu turun değil). Kılavuz §5.1 bu dosyayı yalnız eklenti zip'i için kullandırıyor, o satır doğru; yanıltıcı ama ürün hatası değil (Öneri 2).
- **Fark 2 — Claude Code oturum sonunda rootmark'ı SIGKILL'le kapatıyor** (`SIGINT failed … SIGTERM failed … sending SIGKILL`; 2.1.322'de de, iki sürümde de). Bu PQ2-1 (bekleyen kaldırmalar kaybolur → P2-C); turun bulgusuyla aynı, yenisi değil.
- **Şüphe:** yok. Not: Claude Code sınaması 2.1.322 ile — turunkinden yeni; `server/discover` → `-32022` → eski model dizisi aynı kaldığı için sonuç aynı. Gelecekte istemci eski modele düşmeyi bırakırsa G6 sınırı (sunucu 2026-07-28'i ilan etmiyor) yeniden önem kazanır (Öneri 3).

### Kriter başına (benim ölçümüm ve okumam)
| Kod | Sonuç | Dayanak |
|---|---|---|
| Q2-1 | **geçti** | DESIGN/TESTS `b667326` < ilk ürün kodu `11a1d97` (R2 eki `e67d2b4` < `0ce5dee`); kök neden kaydı `r0_requests.json` + rmcp 3.5.0/3.6.0 kaynağıyla tutarlı (kendim okudum); yöntem ve reddedilen alternatif gerekçeli; fork 0 |
| Q2-2 | **geçti** | (i) 0.3.1 + Claude Code 2.1.322: 0 araç, aynı `-32602` (benim koşum). (ii) `mcp_compat` 10/10 benim Mac'imde; CI `ubuntu-latest` ve macOS 10/10. (iii) yayımlanmış 0.3.2 + Claude Code 2.1.322: 11/11, `search`, `propose` (`Agent`, `proposal`) — koştu 3 / geçti 3; R1'in yerel ikili yarısı turun kaydı (`claude_code/r1_local.json`), aynı ürün kodu |
| Q2-3 | **geçti** | `tools/list` ham satırı 0.3.1 ↔ 0.3.2 bayt eşit (kendi betiğim); 2025-06-18 `_meta`'sız istemci 0.3.2'de çalışıyor; `mcp_schemas`, `contract` vb. PR CI'ında geçti; `rootmark conformance` 39/39 |
| Q2-4 | **geçti** | `watch_hold` 20/20 koşu (macOS, benim), Linux CI 5/5; mutant H1/H5'i düşürüyor; `watch_remove` 9/9, `watch_newdir` 3/3 |
| Q2-5 | **geçti** | turun tek tetiklemesi `42515401135` (kaydı okundu) ve kendi tetiklemem `42982215036`: zincir çıkışları 0/0, D farkı 0 (kendi betiğim + CI), sayılan FAIL 0/0, `portable` 0/0; commit'ler arası farklar yalnız §6.1 (sürüm dizgileri) ve §6.2 (E1-4/R14) — adsız 0; `release.yml` değişikliği §6 ekiyle değişiklikten önce adlı. Mac kümesi bir kez, kayıtlı, `pass` (P2-V.2 okuması) |
| Q2-6 | **geçti** | sınır farkı yukarıdaki gibi; `mcp.rs` değişmedi; D-NOTES 0.3.2 benim Mac ↔ benim Docker ↔ turun Mac/Docker/CI ↔ 0.3.1 kayıtları bayt eşit; conformance 39/39 (göçsüz) |
| Q2-7 | **geçti** | §4.1 2.1.322 ile izlendi ve tutuyor; PQ1-1 Known issues'ta yok; Öneri 3a, anahtar kuralı, CHANGELOG yazılı; `rmcp` 19 günlük; açık Dependabot 0, CodeQL 0 |
| Q2-8 | **geçti** | `release.yml` `42574633881` zorunlu hedefler `success`; `v0.3.2` yayımlı; SHA256SUMS 5/5, köken 5/5; `release_check` 21/21 (turun kaydı, okundu) |
| Q2-9 | **geçti** | PR CI'ı `42921725577` ve `42927226404` yeşil, FAST A 38:14 / B 35:29 (≤ 40 dk); benim `make verify FAST=1` eşit; `check_dep_age` 0; gözden geçirici KABUL (`R5.review.md`) |

### Hüküm
**Hüküm: Güvenilir.**

Turun iddiaları bağımsız olarak yeniden üretildi:
- PQ1-1, yayımlanmış 0.3.1'de benim Claude Code'umla (2.1.322, turunkinden yeni) görünüyor, yayımlanmış 0.3.2'de kapalı: 11 araç, `search`, `propose` (`Agent`, `proposal`); eski istemci (2025-06-18, `_meta`'sız) çalışıyor ve `tools/list` 0.3.1 ile bayt eşit.
- PQ1-4 testi 20/20 koşu geçiyor ve düzeltme geri alınınca düşüyor.
- D-NOTES 0.3.2 Mac ↔ Docker ↔ turun kayıtları ↔ CI ↔ 0.3.1 bayt eşit.
- Kendi kademe 2 tetiklemem `42982215036` temiz; değişen kayıtların hepsi R0'da adlı.
- Ajan yolu (Claude Code `propose` → insan onayı → `canonical`) kılavuzla baştan sona çalışıyor.

Karar kuralının dalı ("Hepsi geçti" → D-0, sonra D-1) FINAL'inkiyle aynı; FINAL'deki sayılar okuduğum kayıtlarla tutuyor. Fark 1 ve 2 hükmü değiştirmiyor.

### Öneriler (≤ 3)
1. **`check_dep_age`'i dolaylı npm bağımlılıklarına genişletin (PQ2-6).** Bu turda `minimatch` 11.0.1'in getirdiği 5 günlük `@isaacs/nonrecursive-glob-match` ancak elle fark edilip `overrides` ile sabitlendi; araç yalnız doğrudan bağımlılıklara bakıyor. D-0/D-1'de depo bölünürken kilit dosyalarının tamamını (Cargo.lock zaten, npm kilitleri de) yaş kuralına sokan bir denetim kural ile araç arasındaki boşluğu kapatır.
2. **`SHA256SUMS.local`'ı yayımlanmış varlıklarla tutarlı yapın.** Ya yerel `bin/` satırlarını sürüm varlığından çıkarın ya da dosyayı yalnız yerelde üretilen varlıklarla (eklenti zip'i, wheel) sınırlayın; `RELEASE.md` adımı buna göre. Bugün `shasum -c SHA256SUMS.local` iki MISMATCH veriyor (0.3.1'de de).
3. **Gerçek Claude Code sınamasını tekrarlanabilir bir yerel kontrol olarak tutun.** PQ1-1 istemcinin yeni bir sürümüyle geldi, CI'da gerçek istemci yok. Turun `claude_probe.py`'si gibi başsız bir sınamayı her yayımdan önce (ve D-1'deki dağıtım işinde) güncel Claude Code ile koşmak, `server/discover`'dan sonra eski modele düşmeyi bırakan bir istemci sürümünü (G6 sınırı) erken yakalar.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
