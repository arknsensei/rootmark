# R3 — "git ile fark" sayfasının ve README'nin iddia → atıf tablosu

Her ürün iddiası bir laboratuvar kaydına ya da kararına atıf taşır (mutlak URL, `v0.4.0` etiketiyle: `https://github.com/arknsensei/rootmark/blob/v0.4.0/<yol>`). "Denetim" sütunu iddianın kayıttaki değerle aynı koşulda olduğunu gösteren satırdır (R3'te elle okundu; atıfların hedef dosyaları ağaçta var: 17/17 yol, bkz. R3.md). Etiket R6'da atılınca URL'ler çözülür; R6 her URL'yi `200` ile okur.

| # | İddia (sayfa) | Atıf | Kayıttaki değer (denetim) | Ölçülen koşul (sayfada yazılı) |
|---|---|---|---|---|
| G1 / E1 | blok soyu düzenleme, taşıma, bölme, birleştirmede (git.md, README) | `reports/rounds/P1-A/FINAL.md`; `NORTH_STAR.md` "Kanıtlanmış olanlar (prototip v1–v3)" | P1-A A-1: C1-01..C1-17 17/17; NORTH_STAR: kolay/zor sentetik, 56k bloklu gerçek tarihçe, bölme/birleştirme | soy içerikten çıkarılır (spec §5, matcher v3) |
| G2 / E2 | blok başına köken: yazar türü, ajan adı, kanal | `reports/rounds/P1-B/FINAL.md`, `reports/rounds/P1-D.1/FINAL.md` | P1-B B-1..B-10 geçti (spec §4.3, §7); P1-D.1 silme/değiştirme olayları aktör ve kaynak taşır | ajan adı beyandır (P2-P) |
| G3 / E2 | onaysız içerik aramaya girmez | `reports/rounds/P1-B/FINAL.md`, `P1-B.1/FINAL.md`, `P1-E/FINAL.md` | P1-B C2 8/8 + P1-D.1 C2-10 (C2 10/10, NORTH_STAR); P1-D D-3 "varsayılan search bulmuyor"; P1-E sahne 11/11 (adım 5) | varsayılan `human_only`; ya da insanın önceden yazdığı kural |
| G4 / E7 | kural listesiyle terfi; tasdiksiz yazım kuralla terfi etmez | `reports/rounds/P2-P/FINAL.md` | S1, S3, S4, S5, S7 güvenlik vakaları geçti | ajan adı kimlik doğrulamasız |
| G5 / E4 | ajan MCP üzerinden onaylı metni değiştiremez | `reports/rounds/P1-D/FINAL.md`, `P1-E/FINAL.md` | P1-D: `append_only` varsayılan, D-11 kasıtlı yazımlar reddedildi; P1-E sahne adım 10 `denied` | MCP kanalı; varsayılan `append_only` |
| G6 / E6 | doğrudan dosya yazımı `canonical` sayılmaz — **yalnız** koşulla | `reports/rounds/P2-0.3/FINAL.md`, K-P2-03.1 (`docs/DECISIONS.md`) | H-P2-0.3a: D-A 0/525, D-A2 0/210 (sızıntı 0); 0.3b 151/2228 = 0.0678 (eşik 0.01, kaldı) → varsayılan `canonical` | `unattested = "unverified"` **ve** tasdikli istemci; varsayılan `canonical`; yeniden adlandırmada insan içeriği `unverified` kalabilir |
| G7 / E2 | cevap izi (`trace`) | `reports/rounds/P1-B/FINAL.md`, `P1-E/FINAL.md` | spec §8, §11 Validated (P1-B); P1-D D-5 200 çağrı → 200 iz; sahne adım 8 | RootMark araçlarıyla verilen cevaplar |
| G8 / E2 | kontrol noktasında blok düzeyinde arama ve blame | `reports/rounds/P1-B/FINAL.md`, `P1-C/FINAL.md` | P1-D D-6 `search --at <cp>` eski; sahne adım 9 | — |
| G9 / E3 | aynı dosyalar her makinede aynı günlük ve indeks baytları | `reports/rounds/P2-V.1/FINAL.md`, `P1-A/FINAL.md` | P2-V.1 V1-1/V1-2 vektör baytları x86_64/aarch64 Linux ve macOS arm64'te eşit; P1-A A-5 `events_hash` eşit | sabitlenmiş varsayılan model; yayımın kendi ikilisi |
| G10 | git'in verdiği: tam geçmiş, dal, uzak eşitleme — RootMark vermez | `docs/ROADMAP.md` §2 (P2-C) | sync P2-C'de, sıra kararının arkasında | — |
| G11 | `.rootmark/` dosyalardan türetilir, kopya değildir | `spec/SPEC.md` §4 | spec §4 (nesne deposu, olay günlüğü) | — |
| E5 | 11 adımlık sahne uçtan uca | `reports/rounds/P1-E/FINAL.md`, `P1-E.1/FINAL.md` | 11/11; gerçek Obsidian arayüzünde 11/11 (P1-E doğrulaması) | — |

**Atıfsız iddia: 0.** README'nin "What it is" ve "Scope" maddeleri yukarıdaki E1–E7'ye atıf taşır; tasarım ilkeleri (md'ye yazılmaz, 127.0.0.1) iddia değil, kapsam tanımıdır ve spec/tehdit modeline bağlıdır. Vaat ölçülen koşulun ötesine genişletilmedi: README'nin alıntısı "That is the goal" ile hedef olarak işaretli ve hemen ardından "The measured condition" bölümüne bağlıdır.
