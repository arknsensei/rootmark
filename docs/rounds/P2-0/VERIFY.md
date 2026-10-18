# P2-0 — Bağımsız Doğrulama

Bağımsız doğrulayıcısın; ana dizine erişme. docs/DEV_PROCESS.md §5 temel. Kod değiştirme. Tek çıktı reports/rounds/P2-0/VERIFICATION.md. Bu bir hipotez turu: sayıları kendin üret, hükmü hipotez bazında ver.

- **V1 Çift koşturma:** `verify.sh pair` çıkış 0; `attestations.jsonl`, `events.jsonl` bayt eşit; kalite alanları fark 0.
- **V2 Ajan sızıntısı (0a, 0a′) — asıl test:** kendi vault'unda **gerçek Claude Code** (ya da herhangi bir dosya erişimli ajan) ile 30 doğrudan md yazımı yaptır (yeni dosya, ekleme, bölüm değiştirme, tam üzerine yazma); her birinde `blame`: değişen bloklar `unverified`, aktör `unattested_fs`; canonical blok sayısı ve içeriği değişmemiş; üzerine yazılan canonical `history`/checkpoint'ten okunabiliyor. Ayrıca token'ı okuyan ajanın HTTP `attest` çağrısı: aşama 1'de **başarılı olur** (bilinen sınır, raporla), aşama 2 etkinse başarısız olmalı.
- **V3 İnsan düzenleme (0b, 0c):** gerçek Obsidian'da (CDP) 50 düzenleme, otomatik kayıt dahil; `unverified` kalan 0 ya da ≤ 1; gecikme p95; geç tasdik vakaları. Harness'in D-B sonuçlarını kendi oturumlarınla karşılaştır.
- **V4 Karışık (0d):** aynı anda ajan betiği + senin Obsidian düzenlemen; katmanlar doğru kaynağa göre.
- **V5 Anahtar zinciri (0e):** macOS'ta `keychain_probe.sh`; terminalden gizi okumayı kendin dene (`security find-generic-password`, kendi Python'un); Obsidian tasdik üretirken terminal üretemiyor mu.
- **V6 Eski vault'lar (0f):** v0.1.0 ikilisini kendin derle (`git archive v0.1.0`), 3 vault üret, yeni ikili ile aç; replay, `events_hash`, tüm bloklar canonical.
- **V7 Uyumluluk (0h):** 37/37; yeni fixture'ları yeniden üret; spec sürümü CRITERION kuralıyla.
- **V8 Yönetişim sınırları:** MCP'de `attest` yok; sahte tasdik (yanlış hash) reddediliyor ve bloklar `unverified` kalıyor; `[attestation] unattested = canonical` ile eski davranış ve `status` bunu gösteriyor.
- **V9 Ayar/test ayrımı, dondurma, bağımlılık yönü, karar kuralı:** W yalnız ayar setinde; D-B test seti `W` sabitlenmeden okunmamış (kod ve git sırası); `check_frozen.sh`, `check_deps.sh`; kuralı GOAL'den türet; FINAL ile karşılaştır; NORTH_STAR değişmemiş olmalı.
- **V10 Vaat sorusu (zorunlu):** NORTH_STAR'ın tek cümlelik vaadini oku; bu turdan sonra, dosya erişimli ajanla hangi koşulda tutuyor, hangi koşulda tutmuyor; tek paragraf.
- **Sonuç:** ihlal/fark/şüphe; hipotez başına geçti/kaldı; hüküm; ≤ 3 öneri; tehdit modeli için önerdiğin ifade.

---

# P2-0 — `/goal` İstemi (ana ajan)

```
Aktif tur P2-0 (hipotez turu). Yönergeler: docs/rounds/P2-0/GOAL.md, TASKS.md; VERIFY.md yalnız doğrulayıcı içindir.
Başlangıç koşulu: `v0.1.0` etiketi ve reports/rounds/P1-E.1/VERIFICATION.md "Güvenilir". Değilse dur, STOP.md yaz.
Önce CLAUDE.md, NORTH_STAR.md, docs/ROADMAP.md §1–§2 (P2-0 gerekçesi), spec/SPEC.md §7–§8, docs/CLIENT_CONTRACT.md, docs/SECURITY.md ve THREAT_MODEL.md, docs/DEV_PROCESS.md, docs/SPEC_EDITS_CRITERION.md, reports/rounds/P1-E/VERIFICATION.md (faz 2 risk 1), reports/rounds/P1-E.1/VERIFICATION.md (öneriler), core/ index/ server/ clients/obsidian CLAUDE.md'leri oku; sonra GOAL.md, TASKS.md.
Z0'dan Z8'e sırayla; DEV_PROCESS §2 disiplini; commit öneki P2-0/Z<n>; Z0'da ROUND_START ve DECISIONS ekleri.
Kırmızı çizgiler: GOAL eşikleri ve karar kuralı değişmez; W yalnız D-B ayar setinde seçilir, test seti sabitlenmeden okunmaz; MCP'de attest aracı yoktur; eski vault'larda tüm bloklar canonical kalır ve events_hash değişmez; istemci md yazmaz; spec §1–6 değişmez, §7–§8 yalnız GOAL'deki genişleme ve CRITERION ile; çalışma zamanı yalnız Rust (plugin TS); sessiz başarısızlık yok; NORTH_STAR'ı değiştirme, öneri yaz.
Hipotez düşerse optimize etme, raporla, karar kuralını uygula. Durma koşulunda STOP.md, PARK; soru sorma, varsayımı etiketleyip ASSUMPTIONS.md'ye yaz. Gerçek Obsidian yoksa D-B'yi PARK et ve 0b/0c "ölçülmedi" raporla.
Z8 bitince docs/rounds/P2-0/progress/READY.md, dur. Doğrulamayı sen yapma.
```

# P2-0 — `/goal` İstemi (doğrulayıcı, ayrı oturum, temiz klon)

```
Bağımsız doğrulayıcısın; ana dizine erişme. docs/DEV_PROCESS.md §5 ve docs/rounds/P2-0/VERIFY.md V1–V10'u uygula; V2'yi gerçek bir dosya erişimli ajanla yap; v0.1.0'ı kendin derle. Kod değiştirme. Tek çıktı reports/rounds/P2-0/VERIFICATION.md.
```
