# mdkernel — Kuzey Yıldızı

Bu dosya projenin tek sayfalık sabit noktasıdır. Her `/goal` turu, her yeni sohbet, her belge önce bunu okur. Değişmesi için açık bir karar kaydı gerekir (`docs/DECISIONS.md`).

## Amaç
İnsanların ve ajanların birlikte yazdığı markdown bilgi korpusu için **blok düzeyinde soy, köken ve deterministik türev indeks** sağlayan, istemciden bağımsız bir çekirdek. Markdown dosyaları tek gerçek kaynaktır; her şey onlardan türetilir ve onlara dönülebilir.

## Tek cümlelik vaat
Bir ajan bilgi tabanına yazdığında, her bloğun kim tarafından, ne zaman, hangi bloktan türeyerek yazıldığı bilinir; onaylanmamış hiçbir şey bilgi sayılmaz; her cevap hangi bloklara dayandığını söyler; dünkü hale blok düzeyinde dönülebilir — ve dosyalar sıradan markdown olarak kalır.

## Biz neyiz, ne değiliz
- **Değiliz:** vektör veritabanı, ajan hafıza API'si, not uygulaması, ekip wiki'si, LLM çerçevesi.
- **Biziz:** markdown üzerinde çalışan, bağımsız dağıtılan bir çekirdek (`mdk`) ve onu konuşan istemciler. Obsidian, VS Code, CLI, MCP istemcileri eşit statüdedir; çekirdek hiçbirini bilmez. Tek uyumluluk sözü: düz markdown + wikilink.
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

## Kaynak sırası
`NORTH_STAR.md` → `spec/` → `docs/ARCHITECTURE.md` → `docs/PACKAGES.md` → `docs/ROADMAP.md` → `docs/DEV_PROCESS.md` → aktif turun yönergesi. Çelişkide üstteki kazanır.
