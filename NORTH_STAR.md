# rootmark — Kuzey Yıldızı

Bu dosya projenin tek sayfalık sabit noktasıdır. Her `/goal` turu, her yeni sohbet, her belge önce bunu okur. Değişmesi için açık bir karar kaydı gerekir (`docs/DECISIONS.md`).

## Amaç
İnsanların ve ajanların birlikte yazdığı markdown bilgi korpusu için **blok düzeyinde soy, köken ve deterministik türev indeks** sağlayan, istemciden bağımsız bir çekirdek. Markdown dosyaları tek gerçek kaynaktır; her şey onlardan türetilir ve onlara dönülebilir.

## Tek cümlelik vaat
Bir ajan bilgi tabanına yazdığında, her bloğun kim tarafından, ne zaman, hangi bloktan türeyerek yazıldığı bilinir; onaylanmamış hiçbir şey bilgi sayılmaz; her cevap hangi bloklara dayandığını söyler; dünkü hale blok düzeyinde dönülebilir — ve dosyalar sıradan markdown olarak kalır.

**Ölçülen koşul (P2-0 dizisi, 2026-10-20; K-P2-02.5):** dosya erişimli bir ajanın tasdiksiz doğrudan yazımının `canonical` sayılmaması, `[attestation] unattested = "unverified"` seçildiğinde ve tasdikli bir istemciyle (Obsidian eklentisi ya da `rootmark attest`) geçerlidir; varsayılan politika `canonical`'dır. Ölçülen sınır: yeniden adlandırmalarda insan içeriği güvenli tarafta `unverified` kalabilir — yarışa bağlı (P2-0.2 ve P2-0.3; P2-0.3'te resmi tohumda 151/2228 = 0.0678, olaysız zincirde 0.0112: büyük bir notun yeniden adlandırılmasında Obsidian'ın gelen bağlantılarını yeniden yazdığı yan notlar); bu yüzden varsayılan `canonical` kesindir (K-P2-03.1). Token'ı okuyabilen, insan istemcisini başlatıp sürebilen ya da eklenti dosyalarını yazabilen bir süreç insan kanalındadır (THREAT_MODEL).

## Biz neyiz, ne değiliz
- **Değiliz:** vektör veritabanı, ajan hafıza API'si, not uygulaması, ekip wiki'si, LLM çerçevesi.
- **Biziz:** markdown üzerinde çalışan, bağımsız dağıtılan bir çekirdek (`rootmark`) ve onu konuşan istemciler. Obsidian, VS Code, CLI, MCP istemcileri eşit statüdedir; çekirdek hiçbirini bilmez. Tek uyumluluk sözü: düz markdown + wikilink.
- **Spesifikasyon yayımlanır** (`spec/`), ContextNest'in kavram setiyle uyumlu ve onun doküman düzeyinde bıraktığı yönetişimi blok düzeyine indirir.

## Kim için (öncelik sırasıyla)
1. Ajan hafızasını markdown'da tutan geliştiriciler.
2. Ajan yazımlı içerik için köken ve denetim isteyen ekipler.
3. Bilgi tabanı güç kullanıcıları (Obsidian vb.).

## İlk ürün sahnesi (faz 1 çıkış kriteri)
Claude Code bir vault'a `propose` ile yazar → kullanıcı dosyayı herhangi bir markdown editöründe görür → `blame` kökeni blok düzeyinde gösterir → onaylanmamış bloklar aramaya girmez → `promote` ile onaylanır → `trace` bir cevabın dayandığı blokları listeler → `checkpoint` ile dünkü hale dönülür. Bağımsız doğrulayıcı bu sahneyi betikle koşturabilir.

## İlkeler
1. Dosyalar gerçek, indeksler türev; md'ye kimlik veya meta veri yazılmaz.
2. Determinizm: aynı girdi, her makinede aynı kimlik, aynı günlük, aynı sonuç.
3. Köken birinci sınıf ve değiştirilemez.
4. Çekirdek karar vermez, karar verilebilir kılar; politikalar plugin'dedir.
5. Öğrenme türev katmanda, geri alınabilir sürümlerde yaşar.
6. Altyapı dürüst olmayan sonucu üretemez: sessiz başarısızlık yok, manifest, çift koşturma doğrulaması.
7. Çalışma zamanı tamamen Rust; Python yalnız araştırma; TypeScript yalnız istemci. Fazlı dil geçişi yok.
8. Ölçek sınırı dürüst: on binlerce–yüz binlerce doküman, anlam-yoğun korpus.

## Kanıtlanmış olanlar (prototip v1–v3, 2026-10)
Diff ile soy takibi (kolay/zor sentetik, 56k bloklu gerçek tarihçe); bölme/birleştirme tespiti; tekrar-blok çapalama; blok/chunk ayrımı; 25 ms altı artımlı indeks; bayt düzeyinde determinizm ve bağımsız çift koşturma doğrulaması. **Desteklenmeyen:** wikilink grafının sözcüksel+vektör arama üstüne sıralama katkısı (araştırma kolunda).

## Kanıtlanmış olanlar (faz 1, P1-A…P1-E, 2026-10)
Her madde bir turun önceden kayıtlı kriteriyle ölçüldü (rapor: `reports/rounds/<tur>/`). P1-A ve P1-B'nin ölçümleri bağımsız olarak yeniden üretildi (P1-B doğrulamasının farkı F1, P1-B.1'de kapandı); P1-B.1, P1-C, P1-C.1 bağımsız doğrulamada "güvenilir"; P1-D ve P1-D.1 "şu koşullarla güvenilir" — P1-D'nin koşulları P1-D.1'de kapandı, P1-D.1'in iki koşulu (D1-12 çift koşturma, K-D1.4-T) P1-E/E0'da kapatıldı (`reports/rounds/P1-D.1/FINAL_ADDENDUM.md`) ve P1-E doğrulamasında bağımsız olarak teyit edildi (V0); P1-E "şu koşullarla güvenilir" — koşulu (`status` sözleşme ihlali) P1-E.1 kapatır.
- **Spec §1–6 donmuş, başvuru çekirdeği uyumlu:** C1 17/17, holdout eşikleri; çift koşturmada kalite alanlarının en büyük göreli farkı 0 (1 790 alan) ve `events_hash` eşit (P1-A).
- **Yönetişim primitifleri:** katman (`canonical`/`proposal`/`derived`), politika kancasıyla terfi (varsayılan yalnız insan), adlandırılabilir kontrol noktası, tüketim izi, ham günlük/sürüm tetikleyici sınırları; spec §7, §8, §11 Validated, C2 10/10 (P1-B, P1-B.1, P1-D.1).
- **Deterministik türev indeks:** Rust BM25 (manifest'te bildirilen varyant) + kesin kosinüs + hibrit RRF, kontrol noktasında sorgu, yazma→aranabilir p95 32 ms; kalite referansı 618 değerde Δ 0; spec §9 Validated, C3 7/7 (P1-C, P1-C.1, P1-D.1).
- **Ajan yazımı canonical içeriği değiştiremez:** `append_only` varsayılan; silme/değiştirme olayları aktör taşır; MCP ajan, HTTP insan kanalı; tek yazar kuyruğuyla boru hatlı istemci aynı günlüğü üretir (P1-D, P1-D.1).
- **Ürün sahnesi uçtan uca (P1-E):** ajan MCP ile önerir, insan HTTP ile (Obsidian istemcisinin komut katmanı) blame'de görür, terfi ettirir, arama bulur, iz cevabın bloklarını gösterir, kontrol noktasına dönülür, ajanın canonical değiştirme denemesi reddedilir; 11/11, iki koşturmada bayt eşit. Obsidian'ın başsız kipi yok; bağımsız doğrulama aynı 11 adımı gerçek Obsidian arayüzünde, ayrı bir örnekte (kendi `--user-data-dir`'i, CDP) koşturdu: 11/11. P1-E.1'den itibaren "plugin e2e" bu gerçek arayüz harness'idir (K-E1.3).
- **Desteklenmeyen / açık:** wikilink grafının sıralama katkısı (araştırma kolu); `rootmark 0.1.0` yayım ikilileri **E-7 2/4**: macOS arm64 ve Linux x86_64 34/34, Intel macOS derlenemedi (sabitlenmiş `ort`'ta hazır onnxruntime yok), Windows ölçülmedi (CI tetiklenmedi) — karar kuralı gereği yayım ertelendi (K-E5-T); K-E1.1 ile 0.1.0'ın hedefi yalnız `aarch64-apple-darwin`, Linux/Windows CI bilgi; istemcide Demote (sözleşmede yok, K-E2-T).

## Kaynak sırası
`NORTH_STAR.md` → `spec/` → `docs/ARCHITECTURE.md` → `docs/PACKAGES.md` → `docs/ROADMAP.md` → `docs/DEV_PROCESS.md` → aktif turun yönergesi. Çelişkide üstteki kazanır.
