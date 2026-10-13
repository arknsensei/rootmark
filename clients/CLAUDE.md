# clients — ajan notları
Sorumluluk: TypeScript istemcileri (`obsidian` faz 1 / P1-E, `vscode` faz 2) (docs/PACKAGES.md).
Bağımlılık yönü: clients yalnız `server`'ın MCP/HTTP sözleşmesine bağlıdır; `core`/`index`/`bindings`/`research` import edilmez (`scripts/check_deps.sh`).
Dil ve araçlar: TypeScript, `tsc`, `eslint`, `vitest` (`make test-ts`, `make lint-ts`).
Dondurulmuş yüzeyler: yok.
Test komutları: `make test-ts`.
Bu pakette yapılmayacaklar: P1-A'da istemci kodu; çalışma zamanı mantığı (TypeScript yalnız istemci).
