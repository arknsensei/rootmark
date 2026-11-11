# D-0 — Bağımsız Doğrulama (kısa; karar turu)

**Çalışma koşulları:**
- Ayrı oturum, temiz klon (kendi scratchpad'in). Ana çalışma dizinine erişim yok; kod değiştirilmez.
- Hedef ≤ 1,5 saat. Kademe 2 ve Mac kümesi koşulmaz (ürün kodu değişmedi; VR1'de göster).
- Kişisel içerik okunmaz. Dışarıya yazan eylem yok: kayıt defterlerine yalnız herkese açık `GET`.

- **VR1 — Sınır.** `git diff ROUND_START..<sha> --stat`: değişiklik yalnız `docs/`, `reports/rounds/D-0/`, `research/d0/` ve `server/CLAUDE.md`'nin `rmcp` satırında mı? `DESIGN.md` ilk tarama kaydından önce mi (`git log`)? Raporlarda token, parola, kişisel e-posta adresi ya da kişisel yol var mı?
- **VR2 — Tarama.** `registry_scan.json`'dan rastgele en az 10 hücreyi kendi sorgunla (kendi betiğin ya da `curl`) yeniden sor: sonuç (dolu/boş, sahibi) raporla aynı mı? Farklıysa zaman farkıyla açıklanabilir mi? Marka taramasının dört kaynağından en az birini kendin sorgula.
- **VR3 — Aktarım planı.** İzin listesini kendi betiğinle bugünkü `main`'e uygula: sınıflandırılmamış dosya 0 mı, giden kümede `research/`, `reports/`, `docs/rounds/`, kişisel içerik ya da anahtar var mı? Giden küme tek başına derlenebilir mi (izin listesindeki yolları ayrı bir dizine kopyala, `cargo build --release` ve istemcilerin derlemesi; dönüşen dosyalar yoksa neyin eksik kaldığını yaz)?
- **VR4 — PQ2-1 ve öneriler.** PQ2-1 sınamasını yayımlanmış 0.3.2 ile kendi Mac'inde en az 5 tekrar koş: sonuç raporla aynı mı? Karar listesindeki her önerinin gerekçesi raporun kanıtına dayanıyor mu? P2-Q.2 doğrulamasının üç önerisi D-1 planında yerli yerinde mi?
- **Sonuç:** ihlal/fark/şüphe; D0-1..D0-7 için geçti/kaldı; tek hüküm ("güvenilir" / "şu koşullarla güvenilir" / "güvenilir değil"); ≤ 3 öneri.
