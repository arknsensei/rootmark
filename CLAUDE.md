# mdkernel — Kök Ajan Kuralları

Bu depo `mdkernel` monoreposudur. Tek gerçek kaynak bu depodaki belgelerdir; dış sohbet geçmişi yoktur.

## Okuma sırası (her oturum)
1. `NORTH_STAR.md`
2. `spec/SPEC.md` (çalıştığın bölümler) ve `spec/CONFORMANCE.md`
3. `docs/ARCHITECTURE.md`, `docs/PACKAGES.md`
4. `docs/ROADMAP.md` → aktif tur; `docs/rounds/<id>/GOAL.md`, `TASKS.md`
5. İlgili paketin `CLAUDE.md`'si
6. `docs/rounds/<id>/progress/`
Çelişkide üstteki kazanır. Yalnız görevinin gerektirdiğini bağlama al.

## Değişmez kurallar
- Çalışma zamanı kodu yalnız Rust (`core`, `index`, `server`, `bindings`). Python yalnız `research/`, TypeScript yalnız `clients/`. Başka dilde çalışma zamanı kodu yazılmaz.
- Bağımlılık yönü `core ← index ← server ← clients`; `research` yalnız `bindings/python`'a bakar. Ters yön yasak.
- `spec/SPEC.md` §1–6 ve uyumluluk fixture'ları değiştirilemez; çelişki görürsen görevi park et, `docs/DECISIONS.md`'ye karar talebi yaz.
- Markdown dosyalarına kimlik veya meta veri yazan kod yazılmaz.
- Kimliklere zaman damgası veya rastgelelik girmez.
- Sessiz başarısızlık yasak: hata ver.
- Aktif turun `GOAL.md` çıkış kriterleri ve eşikleri değiştirilemez.
- Dondurulmuş ölçüm kodu (`research/eval/**`, `scripts/verify.sh`, dondurma denetleyicileri) tur içinde değişmez; istisna prosedürü `docs/DEV_PROCESS.md` §4.
- Testleri geçmek için test değiştirilmez.

## Görev disiplini
`docs/DEV_PROCESS.md` §2 aynen: progress kaydı (BAŞLADI/TAMAMLANDI/KISMİ/PARK), kabul ölçütleri tek tek, gözden geçirici alt-ajan (`<task>.review.md`, kod yazmaz), görev başına commit `<round>/<task>: ...`, bir sonraki göreve iki kayıt da TAMAMLANDI ise.

## Kalite kapıları
`make test` (cargo+pytest+vitest), `make lint`, `make conformance`, `make verify`. Tur bitişinde dördü de temiz olmadan "hazır" yazılmaz.

## Durma koşulları
Toolchain/bağımlılık 2 alternatifle kurulamıyor; 3 gözden geçirici turu sonrası TAMAMLANDI yok; adım 2 saatten uzun; spec ile çelişki. → `docs/rounds/<id>/progress/STOP.md`, görevi PARK et, bağımsız görevlere devam et. Soru sorma; varsayımı etiketleyip `ASSUMPTIONS.md`'ye yaz.

## Tur bitişi
Son görev tamamlanınca `docs/rounds/<id>/progress/READY.md` içine "doğrulama için hazır" + commit sha + RUN kimliği yaz ve dur. Doğrulamayı sen yapma.
