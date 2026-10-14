# P1-B — Bağımsız Doğrulama

Ayrı oturum, temiz klon, ana çalışma dizinine erişim yok. Kod değiştirme. Çıktı: `reports/rounds/P1-B/VERIFICATION.md`. Etiketler `[OK] [FARK] [ŞÜPHE] [İHLAL]`.

- **V1 Çift koşturma:** `scripts/verify.sh pair` çıkış 0; A↔B kalite alanları fark 0; `A/.../traces.jsonl` ↔ B bayt eşit; `checkpoints/` dizinleri `diff -r` eşit; A↔resmi RUN fark 0.
- **V2 Uyumluluk:** `conformance.json` `level_claimed 2`, 25/25 (C1 17 + C2 8); `fixtures_root` mutlak yol ve `fixtures_manifest_hash` var; C2 beklenen değerlerini mevcut çekirdekten kendin yeniden üret, `git status` temiz.
- **V3 Geriye uyumluluk (B-5):** `v3-final` ikilisiyle üretilmiş bir vault'u (kendin üret: `git archive v3-final` ile derle, `mini_vault` ingest) mevcut `mdk` ile aç; `replay` `snapshot_id` eşit; `events_hash` değişmemiş; `blame` çalışıyor ve `layer` türetilmiş.
- **V4 Regresyon (B-3):** `make regress` → v3-final 9 dosya + P1-A holdout/split_grid alan alan 0 fark.
- **V5 Yönetişim davranışı (kasıtlı, kopyada):** (a) hook'suz terfi derlenmiyor ya da `Err`; (b) `HumanOnly` ile actor `agent` → `Err`; (c) `Promoted` olayında `reason` eksik → `Err`; (d) `prune_inactive` sonrası `objects/` dosya sayısı aynı; (e) checkpoint sonrası ingest → `snapshot_at` eski; (f) `checkpoints/<cid>.json` üzerine farklı içerik yazma → `Err`; (g) policy.toml 6 sınır vakası → her biri `Err`, hiçbiri varsayılana düşmüyor (log'da "default" kelimesi aranır).
- **V6 Determinizm:** B-10 fuzz testini iki farklı SEED ile koştur; her SEED kendi içinde replay = durum; `traces.jsonl` A↔B bayt eşit (V1).
- **V7 Dondurma:** `check_frozen.sh` temizde 0; FROZEN.md'den satır silme → ≠ 0 (B-8); `ROUND_START` sha tur yönerge commit'i; spec §7/8/11 değişiklikleri `SPEC_EDITS.md` ile birebir ve yalnız açıklık (kendin oku, anlam değişikliği varsa `[İHLAL]`).
- **V8 Ö2 (B-7):** `target/` ile birlikte kopyalanmış ağaçta fixture boz → `make conformance` hata; `conformance.json` `fixtures_root` kopyanın yolu.
- **V9 Holdout-split (B-9):** `d1hs` kesişimleri boş; gerçekleşen split oranları GOAL Ö3 alt sınırlarının üstünde; eşikler.
- **V10 Bağımlılık yönü:** `check_deps.sh` temiz; `core` hâlâ `index`/`server`'a bağımlı değil.
- **V11 Karar kuralı:** GOAL'den kendin türet; FINAL ile karşılaştır.
- **Sonuç:** ihlal/fark/şüphe; hüküm; en fazla 3 öneri.
