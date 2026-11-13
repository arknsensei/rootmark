# P2-Q.3 — İnsan adımı bekleniyor

**Durum (2026-11-13):** tur bitti. READY.md "doğrulama için hazır" yazıldı. Turun tek PR'ı **#66** (`p2-q.3/round` → `main`) açık; içerik commit'i `2084ecb` ve CI `43407138158`'in bütün işleri yeşil. Karar kuralının dalı: **tur kaldı** (Q3-2 ve Q3-10 lafzen). Kalışın nedenleri R5'in ilk `make test` koşusunda açığa çıkan, bu turdan önce de var olan dizin hatası PQ3-7 ve düzenek yarışı PQ3-8; ikinci koşu temiz. 0.3.3 yayımlı (`v0.3.3` `8f0bcb8`).

## Beklenen (sırasıyla; ajan yapmaz)
1. **K-P2Q3-T1'in kararı** (`docs/DECISIONS.md`, "Karar talepleri (açık)"):
   - (a) D-1 açılır; PQ3-7 ve PQ3-8 P2-C'ye gider.
   - (b) D-1'den önce kısa yama turu P2-Q.4 (0.3.4): dizinin atomik kurulumu (`index/src/store.rs` `install`) ve başlangıçta eksik dosyalı dizinin yeniden kurulacak sayılması; `restartkit`/`watchkit` port yarışı; C1'in zorunlu SIGKILL yolu (PQ3-4); ayrıca PQ3-6 (`EACCES`'in yokluk sayılması). **Ajanın önerisi budur.**
   - (c) (a) ile birlikte kılavuza geçici not ("arama 500 `internal` dönerse `rootmark index build`").
2. **PR #66'nın incelenmesi ve birleşmesi** (orkestratör ya da kullanıcı; ajan birleştirmez). Birleştirmeden önce `gh pr view 66 --json mergeStateStatus` okunmalı: `main` `8f0bcb8`'de; dal onun üstünde, PR güncel.
3. Birleşmeden sonra **bağımsız doğrulama** (`docs/rounds/P2-Q.3/VERIFY.md`, `PROMPT.md`'deki doğrulayıcı istemi) ve temizlik: yerel `release/0.3.3` dalı ve çalışma ağacı (oturumun geçici dizininde), `backup/p2-q.3-round-pre-rebase`.

Ajan bu adımlarda bekler; yeni iş başlatmaz.
