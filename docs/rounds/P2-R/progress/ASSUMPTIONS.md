# P2-R — Varsayımlar

Etiketler: `[spec]` spesifikasyonu ilgilendirir, `[arch]` mimariyi, `[-]` yalnız tur içi uygulama. Hiçbiri spec §1–6 ya da GOAL ile çelişmez; çelişki görülseydi görev park edilirdi.

## R0
- **R0-1 [-]** K-P2B-T1'in kararı K-P2B.5'te "(a) + (c), insan onayına açık" diye yazılmıştı. İnsan, R0 (a)'yı "yalnız K-P2B-T1 kararı (a) ise" koşuluyla içeren P2-R istemini K-P2B.5 ve K-P2R.1 `main`'e birleştikten sonra verdi; bu kararın kesinleşmesi olarak okundu (K-P2R.2). R0 (a) uygulanır, R-3 ölçülür ve kriter sayılır.
- **R0-2 [-]** Uygulama orkestratörün oturumunda koşar (istem bu oturuma verildi); DEV_PROCESS §2 disiplini aynen (K-P2B.2 emsali).
- **R0-3 [-]** (a) "girintili kod bloğu" pulldown-cmark'ın `CodeBlockKind::Indented` üst düzey bloğudur; `byte_start` ilk satırın başına yalnız aradaki baytlar boşluk ya da sekmeyse çekilir. Çitli kod ve liste içindeki kod (üst düzey blok değil) değişmez. `byte_end`, satırlar ve blok ham metni aynen.
- **R0-4 [-]** (c9) "seçici filtrede bütün günlük ayrıştırılmaz": `since_seq`'ten önceki ve sayfa dolduktan sonraki satırlar yalnız sayılır (`head_seq` için); aradaki satırlar yalnız `seq`, `kind`, `doc_path` için okunur (`EventHead`), dönen satır bütün olay olarak doğrulanır. Sayfa dolmadıkça filtreye uymayan satırlar okunmaya devam eder (yanıt baytları aynen kalsın diye tarama sınırı konmadı: sınır `next_seq`'i değiştirirdi, §2.8).
- **R0-5 [-]** (c8) `search` konumu `rootmarkindex::Searcher::search_filtered_then` kancasıyla, isabetler sıralandıktan sonra ve iz yazılmadan önce hesaplar; `search_filtered` aynen (kancasız).
