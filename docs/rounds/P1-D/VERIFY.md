# P1-D — Bağımsız Doğrulama

Bağımsız doğrulayıcısın; ana dizine erişme. docs/DEV_PROCESS.md §5 temel. Kod değiştirme. Tek çıktı reports/rounds/P1-D/VERIFICATION.md.

- **V1 Çift koşturma:** `verify.sh pair` çıkış 0; kalite alanları fark 0; sahne vault'larının `events.jsonl` ve `traces.jsonl` bayt eşit; MCP e2e çıktıları (zaman damgası/token hariç) eşit.
- **V2 MCP Inspector:** resmi Inspector'ı kendin kur; `tools/list` 9 araç; her şemayı doğrula; bir aracı geçersiz parametreyle çağır → yapılandırılmış hata.
- **V3 `propose` (D-3, D-11):** kendi vault'unda MCP ile 3 `propose`; dosyalarda `grep -E "lineage|version_id|mdkernel"` 0; `blame` katman `proposal`; `search` varsayılanda 0 hit; `include_proposal` etiketli; 4 kötü girdi → hata; dosya diff'i yalnız eklenen metin.
- **V4 Terfi (D-4):** MCP `promote` → `denied/human_only` (hata değil yanıt); CLI `promote --actor human:you` → `Promoted` olayı `actor/reason/policy_version` ile; `search` buluyor.
- **V5 İz (D-5):** 50 `search` + 50 `read_block` → 100 iz; `consumer` Inspector/istemci adı; 5 `trace_id` formülle bağımsız.
- **V6 Checkpoint (D-6):** `checkpoint` → değişiklik → `search --at` eski, `search` yeni.
- **V7 HTTP (D-7):** token'sız/yanlış/doğru; `X-Consumer` yok → 400; `0.0.0.0` bağlama denemesi reddediliyor; token dosyası izni 0600.
- **V8 İzleyici (D-10):** `serve --watch` altında dışarıdan md düzenle → `search` 1 s içinde; katman `canonical`; p95 durumu.
- **V9 Uyumluluk ve regresyon (D-9):** 31/31; P1-C.1 referansıyla fark 0; `check_frozen.sh`, `check_deps.sh`.
- **V10 Sahne:** `scripts/scene_agent.sh`'i kendin koştur; `scene.json` adımları sırayla ve beklenen sonuçlarla.
- **V11 Karar kuralı:** GOAL'den türet; FINAL ile karşılaştır.
- **Sonuç:** ihlal/fark/şüphe; hüküm; ≤ 3 öneri.
