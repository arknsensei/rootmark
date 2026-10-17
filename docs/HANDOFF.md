# HANDOFF — Yeni sohbet için başlangıç paketi

Bu dosya depoda `docs/HANDOFF.md` olarak durur. Yeni bir Claude sohbeti açıldığında aşağıdaki dosyalar **bu sırayla** yüklenir ve altındaki istem verilir. Başka hiçbir geçmiş gerekmez (K13).

## Yüklenecek dosyalar (sırayla)
1. `NORTH_STAR.md` — amaç, vaat, kim için, ilkeler
2. `docs/ROADMAP.md` — faz 1 retrospektifi, faz 2 sırası, araştırma kolları
3. `docs/DECISIONS.md` — tüm kararlar (K1'den bugüne), açık karar talepleri (`-T`)
4. `docs/DEV_PROCESS.md` — tur anayasası, `/goal` disiplini, dondurma, doğrulama
5. `docs/ARCHITECTURE.md` — katmanlar, kanıt tablosu
6. `spec/SPEC.md` — yalnız gerektiğinde (spec değişikliği içeren tur yazılacaksa)
7. Son turun `docs/rounds/<id>/GOAL.md`, `reports/rounds/<id>/FINAL.md`, `reports/rounds/<id>/VERIFICATION.md`
8. Varsa açık `STOP.md`, `ASSUMPTIONS.md` (son tur)

## Yeni sohbete verilecek istem
```
Bu proje (adı: rootmark), markdown dosyalarını tek gerçek kaynak kabul eden ve üzerlerinde blok düzeyinde soy, köken ve deterministik türev indeks sağlayan, istemciden bağımsız bir çekirdektir (yüklediğim NORTH_STAR.md). Çıkış noktası: Obsidian'ın md yönetiminde yaptığı şeyi vektörleme ve yönetişim tarafında yapmak; üç prototip turu ve faz 1 turlarıyla hipotezler ölçüldü, 0.1.0 yayımlandı; faz 2'deyiz (ROADMAP.md).

Senin rolün: mimar ve yönerge yazarı. Geliştirmeyi Claude Code `/goal` ile yürütür; her tur dört dosyadan oluşur (GOAL, TASKS, VERIFY, PROMPT; kısa yama turlarında GOAL+PROMPT+VERIFY) ve bağımsız bir doğrulayıcı ajanla kapanır. Sen: (1) gelen FINAL ve VERIFICATION'ı değerlendirir, karar kuralının hangi dalına düştüğünü söylersin; (2) doğrulayıcının GOAL dışı bulgularını ve önerilerini ya bir sonraki turun GOAL/ADDENDUM'una ya da DECISIONS'a ret gerekçesiyle koyarsın; (3) bir sonraki turun yönergelerini DEV_PROCESS şablonuyla yazarsın; (4) kararları K-numaralı, gerekçeli, "sonuç görüldükten sonra" işaretli yazarsın.

Değişmez kurallar: eşikler ve karar kuralları turdan önce yazılır ve değişmez; sonuç kötüyse eşik değil sonuç raporlanır; çalışma zamanı yalnız Rust, Python yalnız research, TypeScript yalnız istemci; md dosyasına kimlik yazılmaz; sessiz başarısızlık yok; yayım etiketi insan adımı; NORTH_STAR yalnız karar kaydıyla değişir.

Şu an: <son turun adı ve durumu, örn. "P2-0 bitti, FINAL ve VERIFICATION ekte; sonraki tur P2-A">. Önce değerlendirmeni ver, sonra bir sonraki turu yaz.
```

## Yeni sohbette ilk kontrol
Claude'dan şunu istemek yeterli: "NORTH_STAR'daki vaadi, ROADMAP'teki sıradaki turu ve DECISIONS'taki açık `-T` taleplerini üç cümleyle özetle." Özet doğruysa devam.
