# P2-0.2 — READY

doğrulama için hazır

- Ölçülen commit: `72fd78f96a593949303ea9eedde61bbd15cf223d` (dal `p2-0.2/round`'un commit'i; PR yeniden tabanlanarak birleşirse `main`'de bulunmaz — K-P2.2; doğrulama için dal ya da bir etiket korunmalı). `make verify` yüksüz ve `VERIFY_LOAD=1` çıkış 0, iki çift eşit.
- Dalın başı (kayıtlarla): `9de02600354987b2153ade884aeae35abe39d169` (`P2-0.2/X6-rec`); `72fd78f..9de0260` arasında ürün, ölçüm kodu, spec ve NORTH_STAR değişmedi (`git diff --quiet 72fd78f HEAD -- core index server clients bindings research/eval scripts Makefile spec NORTH_STAR.md` → eşit).
- RUN: `20261019T212951Z-72fd78f` (resmi, yüksüz çiftin A zinciri); yüksüz B `20261019T222411Z-72fd78f`; yüklü çift A `20261019T232111Z-72fd78f`, B `20261020T002304Z-72fd78f`.
- Rapor: `reports/rounds/P2-0.2/FINAL.md`; karar kuralı: H-P2-0.2b kaldı (116/2743), geri kalanı geçti → GOAL P2-0'ın 0b dalı (K-P2-02.4; uygulanışı karar talebi K-P2-02.4-T). Spec §7.1/§8 Draft (damga yok); NORTH_STAR değişmedi (öneri FINAL'de).
- Kalite kapıları: `make test`, `make lint`, `make conformance` (39/39), `make verify` (iki çift) temiz (FINAL not 1, 6).
- Yan kayıtların SHA-256'ları (`artifacts.sha256` kapsamı dışında; RUN dizinine göre):

```
99d18bdde27fc08a92c713b24ad5ea77a9762f54ab943c2dca7f6f5a4a5d7193  chain_B/d_a_prime.json
c97395c7239e7b1d313e8cc53113ef3d4f68e5e7a7c8e7dd7672879203ce0527  chain_B/d_b.json
053dbb1b91521ff834d17f97d86f7440f08dc44644105e0a52b98e4ccacdbfa1  chain_B/d_b2.json
4af8c8ac7ace285e8692d6cc505ef33e807c3d96c4b548846388d5dc9b8b371d  chain_B/d_c_prime.json
04f87bd363ffaf7720e5d5b6c2142b2999a1a206db2451de2556ce5c7a027455  chain_B/p2_0_2_checks.json
8ec5a4d19ab9b41cb321544156b273a1d54e0ace0962b298c9ac12b7052a5175  load_pair/pair_compare.json
e866ad485b72c3a440532336c5d28979774a62db8c9a6879b4a78cffc6042e88  load_pair/chain_A/d_a_prime.json
c2eeb00cb9304cbf6e61c8d0cd025ccf17adf04f519adcc72af66e049beae07b  load_pair/chain_A/d_b.json
4d284d4a6f887cfdb7fbdabc353c2935483b48c6dce1cb08400649d05917625e  load_pair/chain_A/d_b2.json
6f871ebc0e8ab2c60c1023af862176aa2b8ecd19375f00f010fbb06c43b5ebbb  load_pair/chain_A/d_c_prime.json
00643b727fa37d8e2365cdcf8ce1cfda087ab9b29d8596051f9d07569264f2a7  load_pair/chain_A/p2_0_2_checks.json
e21562ca7e76c6363eaa7b4f7687fcdab2fe0bfa0fa5a58021f1b9922a6d401e  load_pair/chain_B/d_a_prime.json
be70a7ccf255ba54a7a6df591df592f95f91fb9d125dbe45c10073c660449018  load_pair/chain_B/d_b.json
dc99bad53155701bb7cfe5385727cfaabda864be67f0bfb9eada9dcb17d19724  load_pair/chain_B/d_b2.json
2929747d2bd45aa1c2977da1d83a9964376d28a33a46dcede14d38e6d48c35d9  load_pair/chain_B/d_c_prime.json
829e83e88b5379a2d08c8c356df3b8ee012bd8fa3f1560502f93ed6a900ca649  load_pair/chain_B/p2_0_2_checks.json
```

Bağımsız doğrulama (VERIFY.md) ve PR birleştirme ajanın işi değildir.
