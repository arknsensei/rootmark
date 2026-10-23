# P2-0.3 — Bağımsız Doğrulama (dar kapsam; hükmü varsayılanı belirler)

Bağımsız doğrulayıcısın; ana dizine erişme (yalnız bu yönergeyi ve DEV_PROCESS §5'i oku). Temiz klon, kendi araçların. Kod değiştirme. Tek çıktı `reports/rounds/P2-0.3/VERIFICATION.md`. Bu turun "güvenilir" hükmü `unverified` varsayılanını geri getirir ve spec damgasını koydurur; o yüzden V1 ve V3 tam, gerisi dar.

- **V0 Değişmezlik:** `git diff <ROUND_START> <PR başı> -- core index server clients spec NORTH_STAR.md research/eval/datasets research/eval/run_p2_0_2.py` boş; sarmalayıcı yalnız tohum/RUN adı geçiriyor; test-4 tohumu R0 commit'inde, veri sonra; test-3 ve bilgi ölçümünün tohumları okunmamış (git sırası ve kod).
- **V1 Çift koşturma, iki ortam:** `verify.sh pair` yüksüz ve `VERIFY_LOAD=1`; ikisinde de çıkış 0, D-A/D-A2/D-C günlükleri bayt eşit; dört zincirin D-B test-4 değerlerini kendin oku ve FINAL'le karşılaştır (bu değerler çift karşılaştırmasının dışındadır, yön ve eşik dalı aynı olmalı). Çift sırasında başka Obsidian deneyi yapma.
- **V3 İnsan düzenleme, kendi vault'un:** gerçek Obsidian (ayrı örnek, CDP), karışık bağlantılı kendi vault'un, **büyük notlar dahil** (≥ 60 blok, kendine bağlantılı, P2-0.2 doğrulamasının (A) senaryosu), 50 düzenleme, en az 20 yeniden adlandırma; W sonunda `unverified` ≤ 1; biçim başına tablo; p95. Ayrıca aynı 50 düzenlemeyi makine yük altındayken (`nproc` kadar `yes >/dev/null`) bir kez daha koştur ve raporla.
- **V7 Uyumluluk:** 39/39.
- **V9 Ön-kayıt, dondurma, karar kuralı:** `check_frozen.sh`, `check_deps.sh`; karar kuralını GOAL'den türet, FINAL ile karşılaştır; NORTH_STAR değişmemiş; damga konmamış.
- **V10 Vaat sorusu (zorunlu):** vaat bu turdan sonra dosya erişimli ajanla ve yaygın Obsidian bağlantı biçimleriyle hangi koşulda tutuyor; NORTH_STAR "ölçülen koşul" notu için önerdiğin yeni metin; damga görüşü ("Validated konabilir / konamaz, neden").
- **Sonuç:** ihlal/fark/şüphe; hipotez başına geçti/kaldı; hüküm; ≤ 3 öneri.
