# clients — ajan notları
Sorumluluk: TypeScript istemcileri (`obsidian` faz 1 / P1-E, `vscode` faz 2) (docs/PACKAGES.md).
Bağımlılık yönü: clients yalnız `server`'ın MCP/HTTP sözleşmesine bağlıdır; `core`/`index`/`server`/`bindings`/`research` import edilmez, yalnız `docs/CLIENT_CONTRACT.md` uçları çağrılır (`scripts/check_deps.sh`).
Dil ve araçlar: TypeScript, `tsc`, `eslint`, `vitest` (`make test-ts`, `make lint-ts`).
Dondurulmuş yüzeyler: yok.
Test komutları: `make test-ts`.
Bu pakette yapılmayacaklar: çalışma zamanı mantığı (TypeScript yalnız istemci); `core`/`index`/`server` kaynağına bağımlılık. Paket notları: `clients/obsidian/CLAUDE.md` (P1-E).
