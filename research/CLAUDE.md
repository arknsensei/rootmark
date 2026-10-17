# research — ajan notları
Sorumluluk: ölçüm altyapısı (prototip v1–v3 mirası `eval/`, `rootmark/`), kalite regresyonu, araştırma kolları (docs/PACKAGES.md `research`). Çalışma zamanı dışı; yayımlanmaz.
Bağımlılık yönü: research yalnız `bindings/python` (`rootmark_core`) üzerinden çekirdeğe bakar; başka iç paket import edilmez.
Dil ve araçlar: Python 3.12, `uv` (`uv.lock` kilitli, `uv sync --frozen`), `pytest`, `ruff`. Komutlar `research/` içinden koşar (`python -m eval.<modül>`).
Dondurulmuş yüzeyler: `research/FROZEN.md` listesi (A4'ten itibaren): `eval/**` ölçüm kodu, dondurma denetleyicileri, fixture'lar, `scripts/verify.sh`; istisna prosedürü docs/DEV_PROCESS.md §4.
Test komutları: `make test-py`, `make lint-py`, `make regress RUN=<id>`.
Bu pakette yapılmayacaklar: çalışma zamanı kodu; parser/eşleyici/chunker kopyası; H2 koşturması; API çağrısı (P1-A); testleri geçmek için test değiştirmek.
