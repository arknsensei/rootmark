# 06 — v1'den Çıkan Dersler ve v2'deki Karşılıkları

Bu belge gerekçe içindir; kurallar `CLAUDE.md`, `01_SPEC.md`, `03_EVAL.md` içindedir.

| # | v1'de olan | Kök neden | v2'de karşılığı |
|---|---|---|---|
| 1 | H2-1 temiz ortamda 0.061 → 0 | `lineage_id` ULID (rastgele); gerçekler soy kimliğine bağlıydı; D3 git'te değildi | Deterministik `lineage_id` (SPEC §5.1); gerçekler `(doc_path, version_id, k)`; `events_hash` manifest'te; H4 ayrı hipotez |
| 2 | `report.py` eski JSON'a sessizce düştü | "En son uygun dosyayı bul" mantığı | Rapor yalnız `RUN=` ile; dosya sisteminde arama yok; eksik → hata |
| 3 | `run_h2.py` `min_cover=0` ile sessizce sıfır yazdı | Fail-fast yok | Çözülemeyen tek gerçek → hata; hücre sorgu sayısı kontrolü; V6 kasıtlı bozma testi |
| 4 | H1-S3 = 0 | SPEC §5 sıralaması: benzerlik ataması split tespitinden önce | Split/merge önce; artık kontrolü (§5.3 adım 5); T6 kabulünde "ızgara boyunca sonuç değişmeli" |
| 5 | H1-R1 0.9997, tekrar bloklar | k'ıncı→k'ıncı eşleme, bağlam yok | Bağlam çapası `ctx` (§5.2); `dup_delete` sentetik op ve H1-S4; eşik ≥0.999 + kayıp kategorizasyonu |
| 6 | `hybrid` her yerde en iyi, graf hiç BM25 ile birleşmedi | Taban çizgisi `flat` seçilmişti | Taban `hybrid`; `graph_hybrid` ana strateji; `graph_flat` v1 karşılaştırması için tutuldu |
| 7 | H2-3 gecikme 6.5× | Tam graf PPR, networkx | Ego alt graf (2 sıçrama, ≤2000 düğüm) + sparse güç iterasyonu |
| 8 | H2-2 ALPHA'ya duyarlı (0.6 kalıyor, 0.8 geçiyor); test seti ayar sabitlenmeden bir kez görülmüş | `run_h2.py` ayar+test tek komut | `--phase tune` / `--phase test` ayrı; test sabit uyuşmazlığında çalışmaz; duyarlılık eğrisi zorunlu |
| 9 | nDCG multi-hop'ta yalnız A'dan iki chunk ile 1.0 verebiliyor | Gain tanımı kapsamadan bağımsız | `full_cover@10` ayrı metrik ve eşik (H2-4); 1.0 kırpması kaldırıldı, >1 hata |
| 10 | `hier` top-5 doküman, kapsamı öldürdü | Keyfi sayı | Top-20 ve `hybrid` ile |
| 11 | Tekrar `chunk_id` ilk dokümanda tutuldu, bazı gerçekler indeks dışı kaldı | Anahtar `chunk_id` | Anahtar `(doc_path, chunk_id)` |
| 12 | `questions.py` dondurma sonrası SDK uyumu için değişti; kural belirsizdi | İstisna tanımsız | "Uyumluluk düzeltmesi" istisnası: `FREEZE_EXCEPTIONS.md` + `freeze-check` önce/sonra eşitliği + gözden geçirici |
| 13 | T11 STOP'tayken T13/T14 önce yürütüldü; kural "sırayla" diyordu | Park durumu tanımsız | `PARK` durumu, bağımlılık işaretleme, park yerinden devam |
| 14 | İstemler Türkçe yazılmış, ajan İngilizceye çevirmiş | Vault İngilizce | İstemler baştan İngilizce ve kodda sabit |
| 15 | Hunk içi koruma için iki farklı tanım (0.87 / 0.82) | EVAL tanımı ile JSON alanı farklı | Tek tanım `touched_retention` |
| 16 | "Dokunulmamış" blok hunk sınırındaki satırı kapsayabiliyordu | Tanım dar | Hunk'a bitişik satır da "dokunulmuş" |
| 17 | Doğrulayıcı, ana ajanın yerel D3'ünü okumak zorunda kaldı | Yeniden üretilebilirlik yoktu | V0 kuralı: ana depoya okuma dahil erişim yok; `make verify` iki bağımsız dizinde |
| 18 | `data/d4` git'e zorla eklenmişti, belirsiz | Politika yoktu | Açık politika: manifest + içerik-referanslı gerçekler git'te; rastgele kimlik içeren hiçbir veri git'te değil |

Genel ilke: **ajanın dürüst olması yetmez, altyapının dürüst olmayan sonucu üretmesi imkânsız olmalı.** Her sessiz başarısızlık noktası v2'de ya hata ya manifest kontrolü ya da kasıtlı bozma testiyle kapatıldı.
