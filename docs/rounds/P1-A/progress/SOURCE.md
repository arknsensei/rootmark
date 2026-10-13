# P1-A — Prototip kaynağı

## İçe alma
- Kaynak: prototip deposu (`mdkernel-v2`, yerel yol `/Users/arkns/Works/BSv2`), çalışma ağacı temiz.
- Yöntem: `git subtree add --prefix=prototype <kaynak> v3-final` (squash yok). Prototipin tüm commit'leri orijinal sha'larıyla bu deponun geçmişinde; birleştirme commit'i `e561688` ("Add 'prototype/' from commit '71c244e…'").
- Etiketler (bu depoda, prototip commit'lerine işaret eder; prototipteki etiketlerle aynı):
  - `v3-final` = `71c244e551258601bfd5550b61d1aa7335a938fa` (T28: bağımsız doğrulama raporu; prototip HEAD)
  - `v2-final` = `5c53be42c04827ff33daf34bd69b402953179a41`
- Prototip ağacı `v3-final` commit'inin kökünde (`core/`, `eval/`, …); bu depoda `prototype/` altında. Örnek: `git show v3-final:core/src/lineage.rs`, `git archive v3-final core`.

## v3-final referans koşturmaları (prototipte git'te; bu depoda `prototype/reports/runs/<RUN>/`)
| RUN | Rol | `manifest.json` blake3[:32] |
|---|---|---|
| `20261013T154851Z-f133d0b` | v2 nihai RUN | `52eebad6f55743529d56e3acd7b700a9` |
| `20261013T182004Z-f82dabd` | v3 nihai RUN | `9163be3b5696ccc165ce9be54c8cde62` |

v3 RUN JSON'ları (blake3[:32]): `h1s_v3.json` `5f94c49d23bd61296faf1b86c7673769` · `h1s_v3_tuning.json` `1b7a1a898e83dcf21f2026e7d90a5ad1` · `h1r_v3.json` `5ad4dbd40df5b97be28ede251bd5946d` · `h3_v3.json` `3cd6811279282ec6cf73fd7d01cc4a2c` · `h4_v3.json` `6de774d4b26fe469d91eb9576380f6f2` · `h4_v2repro.json` `b4c6b554fb83d74c9051de88998fbea8` · `h1s.json` `1bf2f80d6a63fcfd759db23ff65279e8` · `h1s_tuning.json` `a0206ec83111bd0fef740849f1ad3743` · `h1r.json` `e0e1cb143daddb01545bda057a9d21b1`.

## Veri MANIFEST hash'leri (blake3[:32], `eval/runinfo.py` ile aynı yöntem)
| Dosya | Hash |
|---|---|
| `MANIFEST.d1.json` | `7516fda12ac6c0a6d910af9025b31f63` |
| `MANIFEST.d1h.json` | `657bb8d3f8df3757f0e8bcc45441fc7e` |
| `MANIFEST.d1k4.json` | `1710f3774c2276cc8545af4460557b80` |
| `MANIFEST.d2.json` | `c8b9283a028efd5bddbea5f29780f2e2` |
| `MANIFEST.d3.json` | `4a62699021ed8f4eca0cb3135ac5ee70` |

D3 `events_hash` (v2) `e6945bbc2da9c47376e017e815d7e51b`, `events_hash_v3` `7335baebefba377c84574928dd4cc05c`; mini_vault `events_hash` v2 `8914f9b418ae360d49c85c951050f432`, v3 `808a41f21a43e121d7d688fee9f5bc24` (GOAL A-5).

## Fixture üretimi (A3)
Uyumluluk fixture'larının beklenen değerleri `v3-final` çekirdeğinden (`git archive v3-final core Cargo.lock …`) derlenen üreteçle yazılır; ayrıntı ve fixture hash'leri A3'te bu dosyaya eklenir.
