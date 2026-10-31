# P2-V.1 — Varsayımlar

Etiketler: `[spec]` spesifikasyonu ilgilendirir, `[arch]` mimariyi, `[-]` yalnız tur içi uygulama. Hiçbiri spec §1–6 ya da GOAL ile çelişmez.

## E0
- **E0-1 [-]** `ROUND_START` = `749a4a5`: P2-V.1'in GOAL'üne K-P2V.4'ü ekleyen PR #38'in birleşmiş `main` HEAD'i; turu açan son `main` commit'i (K-P2.2). Uygulama orkestratörün oturumunda koşar (`/goal` bu oturuma verildi).
- **E0-2 [arch]** Aday (r) "başka yol"dur (GOAL İlkeler: "yuvarlama adımı ya da başka yol"): aynı `model.onnx` ağırlıkları ve aynı çizge, sabit sıralı Rust'ta. Ölçülebilmesi için prototipi (`index/src/detbert.rs`) ve gömme aracı (`index/examples/e0_embed.rs`) E0'da depoya girer; ürünün embedder'ı (`index/src/embed.rs`) E1'e kadar ona bağlanmaz, ürün davranışı E0'da değişmez. Seçim ölçümden sonra, E1'den önce commit'lenir.
- **E0-3 [-]** E0 korpusu `research/data/d_notes/notes` (P2-B'nin açık lisanslı gerçek not korpusu, git'te; 401 not, 6 063 parça, ~1,02 M jeton): D3 (`make d3`) git'te değil ve CI'da yeniden üretilmesi gerekir; d_notes uzun parçaları (256 jetona kadar) ve Türkçe metni (çok dilli model için) içerir. E2'nin korpusu ayrıca E2'de adlandırılır.
- **E0-4 [-]** (r)'de çarpımlar kaynaşık çarpma-toplamayla (FMA) birikir: IEEE 754 FMA'nın sonucunu tanımlar (tek yuvarlama), x86_64 AVX2+FMA, aarch64 NEON ve yazılım `f32::mul_add` aynı sonucu verir; vektör şeritleri bileşenler arasındadır, hiçbir toplamın içinde değil. Yerelde (Apple M4 Max) skaler FMA yolu ile NEON yolu bayt bayt aynı çıktı verdi (geliştirme sınaması).
