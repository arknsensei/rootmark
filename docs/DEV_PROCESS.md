# Geliştirme Süreci

Prototip turlarında işe yaramış disiplin, ürün geliştirmeye uyarlanmıştır. Her tur bir `/goal` koşturmasıdır ve bu belge o koşturmanın anayasasıdır.

## 1. Tur anatomisi
Her tur `docs/rounds/<id>/` altında dört dosya ile tanımlanır:
- `GOAL.md` — tur amacı, kapsam, **önceden kayıtlı çıkış kriterleri**, yapılmayacaklar.
- `TASKS.md` — sıralı görevler; her görevde Yapar / Kabul / Üretir.
- `VERIFY.md` — bağımsız doğrulayıcı yönergesi (temiz klon, çift koşturma, kasıtlı bozma).
- `PROMPT.md` — ana ajan ve doğrulayıcı için kopyala-yapıştır `/goal` istemleri.

Tur başladıktan sonra `GOAL.md` değişmez. Yeni ihtiyaç bir sonraki tura gider ve `docs/DECISIONS.md`'ye yazılır.

## 2. Ajan disiplini (kök `CLAUDE.md` bunu uygular)
- Okuma sırası: `NORTH_STAR.md` → `spec/SPEC.md` → `docs/ARCHITECTURE.md` → `docs/PACKAGES.md` → aktif tur dosyaları → ilgili paketin `CLAUDE.md`'si → `docs/rounds/<id>/progress/`.
- Görev başı/sonu kayıtları, gözden geçirici alt-ajan (yalnız kabul ölçütleri + değişen dosya listesi ile; kod yazmaz), görev başına commit (`<round>/<task>: ...`).
- Park kuralı: yalnız durma koşulunda; bağımlılar işaretlenir; park yerinden devam.
- Soru sorulmaz; varsayım etiketli (`[spec] [arch] [-]`) olarak `ASSUMPTIONS.md`'ye yazılır. Spesifikasyonla çelişen varsayım yapılmaz; çelişki görülürse görev park edilir ve `DECISIONS.md`'ye karar talebi yazılır.

## 3. Kalite kapıları (her tur, her commit)
- `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check`; Python için `pytest`, `ruff`; TypeScript için `tsc`, `eslint`, `vitest`.
- `make conformance` — claim edilen seviyeye kadar tüm uyumluluk testleri.
- `make verify` — temiz klon, iki bağımsız dizin, determinizm karşılaştırması; sıfır olmayan çıkış kodu başarısızlıktır.
- Kalite regresyonu: `research/` içindeki holdout sentetik set ve v2 `hybrid` sayıları her turda yeniden koşturulur; sayı değişiyorsa açıklanmak zorundadır.
- Sessiz başarısızlık yasak: çözülemeyen referans, eksik dosya, manifest uyuşmazlığı → hata.

## 4. Dondurma politikası
- `spec/SPEC.md` §1–6 ve uyumluluk fixture'ları yalnız karar kaydıyla ve spec sürüm artışıyla değişir.
- Ölçüm kodu (`research/eval/**`, `scripts/verify.sh`, dondurma denetleyicileri) tur içinde dondurulur; istisna: `FREEZE_EXCEPTIONS.md` girdisi + önce/sonra bit eşitliği + gözden geçirici onayı. Denetleyicilerin kendisi dondurma listesindedir ve derlenmiş ayarlanabilir sabitlere bağlı olamaz.
- Eşikler ve çıkış kriterleri tur başladıktan sonra değişmez.

## 5. Bağımsız doğrulama
- Ayrı oturum, temiz klon, ana çalışma dizinine okuma dahil erişim yok.
- Çift koşturma, kasıtlı bozma testleri, izlenebilirlik, karar kuralının bağımsız türetimi.
- Çıktı: `reports/rounds/<id>/VERIFICATION.md`; hüküm "güvenilir / şu koşullarla / değil" + en fazla 3 öneri. Öneriler bir sonraki turun `GOAL.md`'sinde ele alınır ya da `DECISIONS.md`'de reddedilme gerekçesiyle kapatılır.

## 6. Karar kayıtları
`docs/DECISIONS.md`: `K<n>` numaralı, tarihli, gerekçeli, etkilenen belgeler listeli. Mimari belgedeki K1–K8 ilk girdilerdir. "Sonuç görüldükten sonra alınan karar" olarak işaretleme zorunludur (K4 örneği).

## 7. Paket `CLAUDE.md` şablonu
```
# <paket> — ajan notları
Sorumluluk: <PACKAGES.md satırı>
Bağımlılık yönü: <core ← index ← server ← clients>; ters yön yasak.
Dil ve araçlar: <...>
Dondurulmuş yüzeyler: <spec bölümleri, public API>
Test komutları: <...>
Bu pakette yapılmayacaklar: <...>
```

## 8. Sürüm ve yayım
Faz 1 sonunda `mdk 0.1.0`: uyumluluk Seviye 3 raporu, `make verify` çıktısı, değişiklik günlüğü, spec sürümü. Sonrası SemVer; spec kırıcı değişiklik → majör.
