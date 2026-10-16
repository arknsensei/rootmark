# P1-D.1 — Bağımsız Doğrulama

Bağımsız doğrulayıcısın; ana dizine erişme. docs/DEV_PROCESS.md §5 temel. Kod değiştirme. Tek çıktı reports/rounds/P1-D.1/VERIFICATION.md.

- **V1 Çift koşturma:** `verify.sh pair` çıkış 0; kalite alanları fark 0; sahne ve boru hattı vault'larının `events.jsonl`/`traces.jsonl`/md dosyaları bayt eşit.
- **V2 Canonical korunumu (D1-1/2/3):** kendi vault'unda (insan içeriği + birkaç proposal) MCP ile canonical bloklara dokunan en az 8 `replace_section` denemesi (GOAL'ün 6'sı + senin 2'n) → hepsi `denied`, kod `denied`/`policy_violation`, dosya ve `events.jsonl` sha256 değişmemiş; yalnız-proposal bölümde kabul. Sonra 100 rastgele ajan yazımı ve öncesi/sonrası varsayılan `search` canonical sonuçları (50 sorgu × 3 mod) bayt eşit.
- **V3 Aktör izi (D1-4):** `policy.toml`'da `Unrestricted` aç, bir canonical bloğu `replace_section` ile sil; `events.jsonl`'de `BlockDeleted.meta.actor.kind=agent`, `id` istemci adı; `history` gösteriyor. Sonra `AppendOnly`'ye dön.
- **V4 Boru hattı (D1-5/6):** kendi istemcinle (yanıt beklemeden 100 çağrı, 40 `propose` dahil) iki özdeş vault'a → `events.jsonl`, `traces.jsonl`, md dosyaları `cmp` eşit. 100 salt okunur çağrının paralel işlendiğini (toplam süre) not et.
- **V5 Kanal/aktör (D1-7):** HTTP `promote` → `Promoted` actor `human:<human_id>`; MCP `promote` → `denied`; `human_id` kaldır → HTTP yazma 403; `X-Consumer: human:x` MCP'de aktörü değiştirmiyor.
- **V6 Hata kodları (D1-8):** CLIENT_CONTRACT'taki her kodu en az bir vakayla tetikle (MCP ve HTTP); HTTP durum eşlemesi tabloyla aynı.
- **V7 Checkpoint adlandırma (D1-9):** `search` → adsız checkpoint → `checkpoint {name}` → `CheckpointNamed`, kimlik aynı; aynı ad tekrar → `exists`.
- **V8 Uyumluluk ve eski vault'lar (D1-10/11):** 34/34; C2-10/C3-06/C3-07 beklenenlerini yeniden üret, `git status` temiz; spec sürümünü CRITERION kuralıyla kendin türet ve `SPEC_EDITS.md` ile karşılaştır; P1-D sahne vault'unu ve v3-final karışık vault'ları yeni `mdk` ile aç → replay eşit, `events_hash` aynı, `blame` çalışıyor.
- **V9 Sözleşme (D1-13):** `CLIENT_CONTRACT.md`'deki her araç, uç ve kod için bir test var (`contract_check.sh` çıkış 0); sözleşmede K-D1.3 kanal kuralı ve K-D1.8 uzunluk kuralı yazılı; `SECURITY.md` tehdit modeli token/insan varsayımını açıkça söylüyor.
- **V10 Dondurma, bağımlılık, karar kuralı:** `check_frozen.sh`, `check_deps.sh`; kuralı GOAL'den türet (varsayılan dal dahil); FINAL ile karşılaştır.
- **Sonuç:** ihlal/fark/şüphe; hüküm; ≤ 3 öneri.
