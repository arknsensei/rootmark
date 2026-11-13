# P2-Q.4 — Bağımsız Doğrulama (P2-Q.3 ile birlikte; K-P2Q3.2)

**Çalışma koşulları:**
- Ayrı oturum, temiz klon (kendi scratchpad'in; önce `make setup`, `make models`). Ana çalışma dizinine erişim yok; kod değiştirilmez.
- Hedef ≤ 2,5 saat.
- Kademe 2'yi bir kez kendin tetikle; Mac kümesini koşma, turun kaydını oku.
- Kişisel içerik okunmaz; vault olarak sentetik kasa ya da D-NOTES kopyası.
- Ajan oturumu kapanmış olmalı (aynı makinede eşzamanlı zamanlama testleri birbirini bozar).

- **VR1 — Tasarım, neden ve sınır.**
  - P2-Q.3 ve P2-Q.4'ün `DESIGN.md`/`TESTS.md`'leri kodlarından önce commit'li mi (`git log`)?
  - PQ3-9'un kök neden açıklaması kanıtla destekleniyor mu? Kendi okuman ve, mümkünse, kendi yeniden üretiminle karşılaştır.
  - Uzlaştırma spec §7.1 ile uyumlu mu (başlangıçta tasdik penceresi yok; katman ve aktör)?
  - `git diff`: ürün değişikliği her iki turda yalnız izin verilen dosyalarda mı; `docs/CLIENT_CONTRACT.md` ve dizinin biçimi değişmedi mi; enjeksiyon değişkeni yalnız test ortamında mı etkili?
- **VR2 — Hatayı kendin üret ve kapandığını gör.** `v0.3.2`, `v0.3.3`, `v0.3.4`'ü kur. Kendi betiğinle, kendi Mac'inde ve Docker'da (Linux):
  - SIGKILL kaldırma beklemesi içinde (silme ve taşıma): 0.3.2'de kalıcı; 0.3.4'te yeniden başlatmadan sonra kapalı — taşıma durumu yük altında ≥ 50 tekrar (PQ3-9);
  - gerçek istemci dizisi SIGINT → SIGTERM → ~0,5 s → SIGKILL;
  - sunucu kapalıyken düzenle, sil, ekle; değişiklik yokken yeniden başlatma (yeni olay 0);
  - `ingest-dir` silme ve kök sınırı;
  - `install` ortasında öldürme (enjeksiyonla): 0.3.4'te sonraki başlangıçta `search` çalışıyor;
  - izinsiz dizin (`chmod 000`): 0.3.3'te bloklar kapanıyor, 0.3.4'te kapanmıyor.
  Her durum en az 5 tekrar (taşıma ≥ 50). `tools/list` 0.3.2 ile bayt eşit mi?
- **VR3 — Karşılaştırma ve kademe 2.**
  - D-NOTES senaryosunu P2-Q'nün sürücüsüyle 0.3.4 ile kendi Mac'inde ve Docker'ında koş: turun kayıtlarıyla ve 0.3.3/0.3.2 kayıtlarıyla bayt eşit mi?
  - Kademe 2'yi kendin tetikle (`gh workflow run verify.yml --ref main -f sha=<doğrulanan commit>`): D farkı 0, sayılan FAIL 0; değişen kayıtlar R0'da adlı olanlar mı (E1-4/R14'ün etikete bağlılığı dahil)?
- **VR4 — Belgeler, sürüm, karar kuralı.**
  - Kılavuzun yeni metnini izle: sunucu kapalıyken bir notu sil, sunucuyu başlat, kılavuzun söylediği oluyor mu?
  - Açık Dependabot ve CodeQL uyarıları; 0.3.4'ün `SHA256SUMS`'ı ve köken tasdikleri.
  - İki turun karar kuralını GOAL'lerinden türet, FINAL'lerle karşılaştır. P2-Q.3 için: PQ3-9'un K-P2Q3.2'deki kaydı ve FINAL'in eksikliği doğru mu?
  - PR'ın son commit'inin CI'ı yeşil mi; READY'den sonraki commit'lerin CI'ı okunmuş mu?
  - Vaat sorusu (NORTH_STAR): "dosyalar gerçek kaynaktır" sözü sunucu kapalıyken, öldürülünce ve dizin kurulurken öldürülünce tutuyor mu?
- **Sonuç:** ihlal/fark/şüphe; Q3-1..Q3-10 ve Q4-1..Q4-11 için geçti/kaldı (P2-Q.3'ünkiler FINAL'in okumasının teyidi); her tur için tek hüküm ("güvenilir" / "şu koşullarla güvenilir" / "güvenilir değil"); ≤ 3 öneri.
