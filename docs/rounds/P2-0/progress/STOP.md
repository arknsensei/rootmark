# P2-0 — durma kayıtları

## Z6 — H-P2-0e ölçülemedi (PARK)
- **Ne durdu:** D-D (anahtar zinciri) ölçümü. Sonda, terminal sürecinden giriş anahtar zincirindeki "Obsidian Safe Storage" öğesinin gizini okumayı dener (eklentinin tek anahtar zinciri ilkeli Electron `safeStorage`'ın anahtarı).
- **Neden:** bu oturumun izin denetimi öğenin yalnız özniteliklerini sorgulamayı bile "Credential Exploration" diye reddetti. Ret aşılmadı: kullanıcının gerçek bir kimlik bilgisine erişim denemesi insanın kendi kararıdır.
- **Etki:** H-P2-0e alt-hipotezdir, karar kuralını etkilemez (GOAL P2-0). Etkin aşama 1 (`status.attestation.stage = 1`); aşama 1 tehdit modeli yazılı kalır.
- **Devam:** insan `scripts/keychain_probe.sh <out.json>`'u kendi Mac'inde koşturur (`! bash scripts/keychain_probe.sh …`; her istemde "Deny"). Gerekçeli rapor ve aşama 2'nin açık sorunları (sunucu tarafı saklama, açık anahtar kaydı, vault içi eklenti enjeksiyonu): `Z6.md`.
- Bağımsız görevlere (Z7, Z8) devam edildi.
