# R4 (f) P3 — Gerçek Claude Code oturum sonu sınaması (TESTS P3)

- Ortam: Claude Code **2.1.325**, başsız (`claude -p … --mcp-config <dosya> --strict-mcp-config --allowedTools …`); rootmark **0.3.4** yayımlanmış sürüm, kılavuz §1.1 ile kurulmuş (`install.sh` ve `SHA256SUMS` GitHub sürümünden; `shasum -c` OK; `install.sh`, macOS ve Linux arşivleri için `gh attestation verify` 0; `install.sh --version 0.3.4 --prefix <scratch>`; `rootmark 0.3.4`); sunucu oturumun MCP sunucusu olarak `rootmark serve --vault <v> --mcp stdio --watch` (stderr oturum dizinine).
- Betik P2-Q.3'ünkü, aynen: `research/p2q3/claude_session_end.py` (kaydın `probe` alanı bu yüzden "P2-Q.3 R4 (f)" der). Vault: D-NOTES kopyası (399 not, açık korpus) + sentetik not `zz-probe/session-end.md` (sözcük `zqsessionend`); tarif varsayılan (`canonical`) + `window_s = 10.0` (kaldırma beklemesi 10,5 s: oturum biterken kaldırma beklemede kalsın diye). Üç bağımsız koşu, her biri taze vault kopyasıyla ve iki oturumla: oturum 1 `status` çağırıyor, son eylem olarak notu siliyor; oturum 2 sözcüğü arıyor.

| Koşu | Oturum 1 (s) | Kaldırma nerede alındı | Oturum 1 sunucusu | Oturum 2 başlangıcı | Oturum 2 `search` | `BlockDeleted` (önce → s1 → s2) | Sonuç |
|---|---|---|---|---|---|---|---|
| 1 | 9,4 | oturum 1'in durdurması (SIGINT) | `stopped` yok | `removed an index build left unfinished at .rootmark/index.build`; uzlaştırma 399 dosya, 0 değişiklik, 0 hata | çıkış 0, 5 isabet, sınama notu yok | 0 → 2 → 2 | geçti |
| 2 | 10,9 | oturum 1'in durdurması (SIGINT) | `stopped` yok | aynı satır; 399 / 0 / 0 | çıkış 0, 5 isabet, sınama notu yok | 0 → 2 → 2 | geçti |
| 3 | 9,0 | oturum 1'in durdurması (SIGINT) | `stopped` yok | aynı satır; 399 / 0 / 0 | çıkış 0, 5 isabet, sınama notu yok | 0 → 2 → 2 | geçti |

- **Sonuç: 3/3 geçti.** Oturumun son eylemiyle silinen not sonraki oturumun aramasında yok; arama yanıt verdi.
- Gözlem: üç koşuda da sunucu kapanış sıkıştırması bitmeden öldürüldü (Claude Code'un SIGTERM → ~0,5 s → SIGKILL'i; P2-Q.3'teki gibi). SIGKILL sıkıştırmanın kurulumuna düştü ve `.rootmark/index.build` yarım kaldı. 0.3.4'ün sonraki başlangıcı bunu açık bir satırla kaldırdı ve önceki tam dizinle sürdü. Bu, PQ3-7 düzeltmesinin (DESIGN §3) gerçek oturumdaki izi: 0.3.3'ün `remove_dir_all` + `rename`'i aynı anda kesilseydi dizin yarım kalabilirdi.
- Kayıtlar: `reports/rounds/P2-Q.4/claude_code/r4_v0.3.4_run{1,2,3}.json` (sürümler, sayılar, oturumların yanıt satırları; yalnız açık korpusun yolları, host değeri yok). Sunucu stderr'leri kayıt dışında, geçici alanda kaldı.
