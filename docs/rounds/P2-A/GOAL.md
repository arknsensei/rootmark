# P2-A — Göç ve Sürüm Uyumu, Kurulum (Altyapı Turu)

Başlangıç: P2-0 dizisi kapandı (K-P2-02.5): hipotez ön-kayıtlı kümelerde ajan tarafında kanıtlı, insan tarafında 0b dalı; varsayılan politika `canonical`, `unverified` isteğe bağlı. Bu tur bir **geliştirme turu**dur: D-set yok, hipotez yok; önceden kayıtlı çıkış kriterleri ve tek çift koşturma. Hedef süre yarım gün (K-P2.3).

## Kapsam (ROADMAP v2 P2-A + devralınanlar)
1. **Göç çerçevesi.** `rootmark migrate` komutu: `--dry-run` (ne değişecek, hangi biçimden hangi biçime, bayt etkisi), göç, `--rollback <id>`; her göç `.rootmark/migrations/<id>/` altına geri alma kaydı yazar (önceki dosyaların kopyası + manifest + SHA-256), geri alma bayt eşitliğini denetler; göçler kayıtlı ve sıralı (kayıt: mağaza adı `.mdkernel → .rootmark` (var), indeks biçimi `mdkernel-index/3 → güncel` (yeniden kurma; manifest kaydı), olay/tasdik günlüğü okuyucu sürümleri (2.0.0 → 2.0.2: bayt değişmez, yalnız doğrulama), kontrol noktası biçimi (değişmez; kayıt)). Göç `events_hash`'i değiştiremez; değiştirecek bir göç hata ile durur.
2. **Sürüm matrisi ve "göç gerekli" durumu.** `status.migrations`: yapıt başına (`store`, `events`, `attestations`, `index`, `checkpoints`) diskteki biçim ↔ beklenen biçim ↔ durum (`ok` / `migration_required` / `unknown`); indeks isteyen çağrılar `index_stale` yerine nedenini söyleyen `migration_required` hatası döner (sözleşme §3, CRITERION ile). Önceki turların kaydedicilerinin biçim sürümünden kaynaklanan tarihsel FAIL'leri sınıflandırıcıda "göç gerekli" kategorisiyle etiketlenir (kaydediciler değişmez, K-E1.4 listesi genişlemez).
3. **Kurulum ("Setup").** `release.yml`: Linux `x86_64` ve Windows `x86_64` işleri yeşil (Windows'ta `scripts/fetch_model.sh` düzeltmesi, GH-0 STOP 7); her artefakt `rootmark conformance` koşturur; `SHA256SUMS` + GitHub artefakt provenance tasdiki (`actions/attest-build-provenance`); Linux için `install.sh` (tar.gz + sağlama), Windows için zip + `install.ps1`; `cargo binstall` meta verisi doğru. Kod imzalama (Apple notarization, Windows Authenticode) insan adımı olarak RELEASE.md'ye yazılır, bu turda yapılmaz. Actions Node 20 uyarıları: eylemler ≥ 14 gün kuralına uyan sürümlere yükseltilir.
4. **Intel macOS yeniden değerlendirme.** Sabitlenmiş `ort`'un güncel sürümlerinde `x86_64-apple-darwin` için hazır onnxruntime var mı; varsa CI'da bilgi hedefi olarak derlenir ve `rootmark conformance` koşturulur, yoksa kayıt. Yayım hedefi değişmez (K-E1.1); karar talebiyle önerilir.
5. **K-GH.6.** `propose` dizin bileşenlerinin harf/Unicode varyantını her platformda `exists` ile reddeder; CLIENT_CONTRACT §3 metni; Linux CI işi yeşil olur ve **zorunlu** yapılır (ruleset, insan adımı; GOAL kriter A-6).
6. **A0 — P2-0 dizisinin kapanış işleri (K-P2-02.5).** (a) `rootmark serve`'ün ilk açılışta yazdığı tarif `[attestation] unattested = "canonical"` satırını açıkça yazar (yorumuyla: `unverified` isteğe bağlı, nasıl açılır); `status.attestation.unattested` bunu gösterir; spec §7.1 varsayılan cümlesi ve C2-11 **değişmez** (Draft; başvuru uygulamasının yazdığı tarif açık değer taşır). (b) 0.2b'nin (A) nedeni spec §7.1 metnine uydurulur: yeniden adlandırılan notun kendi geç bağlantı güncellemesi, yol günlük için yeniyken **eski yolun başına** karşı denetlenir; birim testi. (c) Sunucu tarafı tasdik tanı günlüğü (`--attest-log <dosya>`, vault dışı): kabul/ret nedeni, gözleme göre sıra uzaklığı; (B) "kabul edilmiş ama eşleşmemiş" vakalarını ayırır. (d) **Bilgi ölçümü (karar kriteri değil):** (b) sonrasında D-B test-3 tek zincir, gerçek Obsidian, 0.2b değeri raporlanır (eşik yok, karar yok; P2-0.3 hipotez turunun ön verisi). (e) Harness: gerçek-Obsidian kümelerinde asılı tek kayıt için küme düzeyinde sınırlı yeniden deneme (kayıtlı; P2-0.2 doğrulaması Ö3) — dondurma istisnasıyla.

## Önceden kayıtlı çıkış kriterleri
| Kod | Ölçüm | Eşik |
|---|---|---|
| A-1 | Her kayıtlı göç için ileri + geri alma testi; geri alma sonrası dosyalar bayt eşit; `--dry-run` hiçbir dosyayı değiştirmez | hepsi geçer |
| A-2 | `v0.1.0` ile yazılmış 3 vault (P2-0 D-E) + `ce4cf04` 2 vault + `21b16ea` 1 vault: `migrate --dry-run` raporu → göç → `events_hash` eşit, replay eşit, `blame` eşit, `rootmark conformance` yeni ikiliyle; geri alma → orijinal bayt eşit | 6/6 |
| A-3 | `status.migrations` matrisi ve `migration_required`; `contract_check.sh` yeni alanlar; eski biçimli indekste `index_stale` yerine `migration_required` | sözleşme testi + `contract_check` yeşil |
| A-4 | `release.yml` dispatch: Linux x86_64 ve Windows x86_64 artefaktları yeşil, her birinde `rootmark conformance` 39/39, `SHA256SUMS`, provenance tasdiki `gh attestation verify` ile doğrulanır | 2/2 |
| A-5 | Intel macOS: ölçüldü ve kaydedildi (derlenir/derlenmez, hangi `ort`) | rapor |
| A-6 | Linux CI işi `main`'de yeşil; `propose` dizin varyantı testi her platformda geçer | geçer |
| A-7 | A0: yeni vault'ta `status.attestation.unattested = "canonical"`, `policy.toml` satırı yazılı; `unverified` seçilince P2-0.2 davranışı; (A) birim testi; `--attest-log` kasıtlı vakalarda nedenleri yazıyor; bilgi ölçümü raporlandı | yazılı ve testli; bilgi |
| A-8 | `make verify` tek yüksüz çift fark 0; uyumluluk mevcut 39 + yeni fixture varsa hepsi; `check_frozen`, `check_deps` | eşit; hepsi |

## Karar kuralı
- **Hepsi geçti** → P2-B başlar; Linux CI işi zorunlu (insan ruleset adımı); kurulum paketleri RELEASE.md'de "yayımlanabilir" (etiket insan adımı).
- **A-4 veya A-5 kaldı** (diğerleri geçti) → platform kapsamı karar talebi (hangi hedefler yayım hedefi); P2-B başlar, kurulum maddesi açık kalır.
- **A-1, A-2, A-3, A-7 veya A-8 kaldı** → tur kaldı, karar talebi; P2-B başlamaz.
- **A-6 kaldı** → K-GH.6 yeniden açılır (seçenek (b): platforma bağlı davranış), P2-B başlar.
- **Tanımlanmamış kalış → tur kaldı, karar talebi.**

## Yapılmayacaklar
Yayım etiketi (insan adımı); kod imzalama/notarization (insan adımı, RELEASE.md'ye yazılır); sync; WASM; spec §1–6; eşik/W değişikliği; P2-0 test setlerinin yeniden ayarı; Obsidian community store; `unverified` varsayılanının geri gelmesi (P2-0.3 hipotez turu, ayrı GOAL).
