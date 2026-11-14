# Third-party data in testdata/

| File | Contents | Source | Licence |
|---|---|---|---|
| `tok_ref.json` | tokenizer reference: text fragments and their expected tokens | the documentation of [dendronhq/dendron-site](https://github.com/dendronhq/dendron-site) at commit `fbc0776` | MIT, Copyright (c) 2020 Thence LLC — full text in [NOTICE](../NOTICE) |
| `embed_ref.json` | embedder reference vectors of text fragments | the same | MIT, as above |
| `p2_0_old_vaults/`, `p2_0_1_old_vaults/`, `p2_0_2_old_vaults/` | synthetic vaults written by older rootmark versions (migration and event-log hash tests) | written by this project | Apache-2.0 |

The data is kept as it was recorded (byte for byte): the tests compare against it.
