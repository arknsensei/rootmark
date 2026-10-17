// Real answers of `mdk serve` (P1-E/E1 probe of every tool over HTTP),
// local paths replaced: the contract tests' fake server returns them.
import type { ToolName } from "../src/api";

export const SAMPLES: Record<ToolName, unknown> = {
  "search": {
    "checkpoint_id": "5e6669bbb36cd9558eff3bd83cfe1e45",
    "hits": [
      {
        "chunk_id": "11e8136622016d91f1188d40593d4e57",
        "doc_path": "a.md",
        "layer": "canonical",
        "lineage_ids": [
          "f4495f5ee61ea960be41bfa9de348401",
          "3a9c0a7481a3565fc10fbf3b7d0179fd",
          "4f6b5cb6f4c5549239547db8faaa7392"
        ],
        "score": 0.03278688524590164
      }
    ],
    "include_proposal": false,
    "mode": "hybrid",
    "query_hash": "a55527015d106b03e8c5f0b8a461b569",
    "trace_id": "f1df769008e368fd450c5d040deda0c4"
  },
  "status": {
    "embedder": "minilm-l6-v2/e73168f9",
    "index": {
      "auto_rebuild": true,
      "expected_format": "mdkernel-index/3",
      "format": "mdkernel-index/3",
      "state": "ok"
    },
    "last_checkpoint": {
      "checkpoint_id": "5e6669bbb36cd9558eff3bd83cfe1e45",
      "docs": {
        "a.md": "11e8136622016d91f1188d40593d4e57"
      },
      "event_seq": 4,
      "name": null
    },
    "level_claimed": 3,
    "mdk_version": "0.2.0",
    "policy": {
      "edit": "append_only",
      "edit_version": "1",
      "journal": {
        "retention_hours": 24
      },
      "promotion": "human_only",
      "promotion_version": "1",
      "source": "/vault/.mdkernel/policy.toml",
      "version_trigger": {
        "idle_ms": 60000,
        "min_sim_change": 0.15
      }
    },
    "server_policy": {
      "identity": {
        "human_id": "ada"
      },
      "index": {
        "compact_every_s": 300
      },
      "server": {
        "http_port": null,
        "max_propose_bytes": 1048576,
        "token_rotate_days": 30
      }
    },
    "spec_version": "1.0.3",
    "vault": "/vault"
  },
  "read_block": {
    "block": {
      "block": {
        "block_type": "Heading",
        "heading_level": 1,
        "heading_path": [],
        "layer": "canonical",
        "lineage_id": "f4495f5ee61ea960be41bfa9de348401",
        "normalized_text": "# A",
        "parents": [],
        "provenance": {
          "author_kind": "Human",
          "source": ""
        },
        "text": "# A",
        "version_id": "ba2f8a67da73b8981e9d95879a3df51d"
      },
      "doc_path": "a.md",
      "inactive": false
    },
    "checkpoint_id": "5e6669bbb36cd9558eff3bd83cfe1e45",
    "trace": {
      "at_seq": 1,
      "checkpoint_id": "5e6669bbb36cd9558eff3bd83cfe1e45",
      "consumer": "probe",
      "query_hash": "8d765afcb30fbf906a90b8914759b6c9",
      "returned": [
        {
          "doc_path": "a.md",
          "lineage_id": "f4495f5ee61ea960be41bfa9de348401",
          "version_id": "ba2f8a67da73b8981e9d95879a3df51d"
        }
      ],
      "trace_id": "42b45c3cff81c5595c9c221eb6704b44"
    }
  },
  "history": {
    "events": [
      {
        "doc_path": "a.md",
        "kind": "BlockCreated",
        "lineage_ids": [
          "f4495f5ee61ea960be41bfa9de348401"
        ],
        "meta": {
          "matcher_version": "v3",
          "provenance": {
            "author_kind": "Human",
            "source": ""
          }
        },
        "new_version_ids": [
          "ba2f8a67da73b8981e9d95879a3df51d"
        ],
        "old_version_ids": [],
        "seq": 0,
        "snapshot_id": null
      }
    ],
    "layer": "canonical",
    "lineage_id": "f4495f5ee61ea960be41bfa9de348401"
  },
  "blame": {
    "at": null,
    "blocks": [
      {
        "author_kind": "Human",
        "inactive": false,
        "layer": "canonical",
        "layer_source": "event",
        "lineage_id": "f4495f5ee61ea960be41bfa9de348401",
        "seq_created": 0,
        "seq_last": 0,
        "source": "",
        "version_id": "ba2f8a67da73b8981e9d95879a3df51d"
      },
      {
        "author_kind": "Human",
        "inactive": false,
        "layer": "canonical",
        "layer_source": "event",
        "lineage_id": "3a9c0a7481a3565fc10fbf3b7d0179fd",
        "seq_created": 1,
        "seq_last": 1,
        "source": "",
        "version_id": "1db96e0d9cc284b36f2fcceab496e668"
      },
      {
        "author_kind": "Human",
        "inactive": false,
        "layer": "canonical",
        "layer_source": "event",
        "lineage_id": "4f6b5cb6f4c5549239547db8faaa7392",
        "seq_created": 2,
        "seq_last": 2,
        "source": "",
        "version_id": "1da4a058824d8a2e310e117c9a5e7c7d"
      }
    ],
    "doc_path": "a.md"
  },
  "propose": {
    "actor": {
      "id": "ada",
      "kind": "human"
    },
    "blocks": [
      {
        "author_kind": "Human",
        "inactive": false,
        "layer": "canonical",
        "layer_source": "event",
        "lineage_id": "f4495f5ee61ea960be41bfa9de348401",
        "seq_created": 0,
        "seq_last": 0,
        "source": "",
        "version_id": "ba2f8a67da73b8981e9d95879a3df51d"
      },
      {
        "author_kind": "Human",
        "inactive": false,
        "layer": "canonical",
        "layer_source": "event",
        "lineage_id": "3a9c0a7481a3565fc10fbf3b7d0179fd",
        "seq_created": 1,
        "seq_last": 1,
        "source": "",
        "version_id": "1db96e0d9cc284b36f2fcceab496e668"
      },
      {
        "author_kind": "Human",
        "inactive": false,
        "layer": "canonical",
        "layer_source": "event",
        "lineage_id": "4f6b5cb6f4c5549239547db8faaa7392",
        "seq_created": 2,
        "seq_last": 2,
        "source": "",
        "version_id": "1da4a058824d8a2e310e117c9a5e7c7d"
      },
      {
        "author_kind": "Agent",
        "inactive": false,
        "layer": "proposal",
        "layer_source": "event",
        "lineage_id": "b30a9aa6ab4833385d0e4cdd94d66276",
        "seq_created": 5,
        "seq_last": 5,
        "source": "probe",
        "version_id": "792f38f0f1131ad95b49767a927d910a"
      },
      {
        "author_kind": "Agent",
        "inactive": false,
        "layer": "proposal",
        "layer_source": "event",
        "lineage_id": "f6aed2eee498ffff1d3679f32bb62e36",
        "seq_created": 6,
        "seq_last": 6,
        "source": "probe",
        "version_id": "eec1f96afb23cfd1b4ee94db45f5d9f2"
      }
    ],
    "bytes": 62,
    "chunks_embedded": 1,
    "doc_path": "a.md",
    "edit_policy": "append_only",
    "events": 3,
    "events_hash": "0a8c5e6257a74bd94e479f280971a283",
    "mode": "append",
    "snapshot_id": "8b1bfb808c054ab887a517336770c897",
    "synced_external_events": 0
  },
  "promote": {
    "decision": "promoted",
    "event": {
      "doc_path": "a.md",
      "kind": "Promoted",
      "lineage_ids": [
        "f6aed2eee498ffff1d3679f32bb62e36"
      ],
      "meta": {
        "actor": {
          "id": "ada",
          "kind": "human"
        },
        "from": "proposal",
        "policy": "human_only",
        "policy_version": "1",
        "reason": "ok",
        "to": "canonical"
      },
      "new_version_ids": [
        "eec1f96afb23cfd1b4ee94db45f5d9f2"
      ],
      "old_version_ids": [
        "eec1f96afb23cfd1b4ee94db45f5d9f2"
      ],
      "seq": 8,
      "snapshot_id": null
    },
    "layer": "canonical",
    "lineage_id": "f6aed2eee498ffff1d3679f32bb62e36",
    "policy": "human_only",
    "policy_version": "1"
  },
  "checkpoint": {
    "checkpoint_id": "5d164cc8ca85695bbf54e177bcd7a4be",
    "docs": {
      "a.md": "8b1bfb808c054ab887a517336770c897"
    },
    "event_seq": 9,
    "name": "y"
  },
  "checkpoints": {
    "checkpoints": [
      {
        "checkpoint_id": "5e6669bbb36cd9558eff3bd83cfe1e45",
        "docs": {
          "a.md": "11e8136622016d91f1188d40593d4e57"
        },
        "event_seq": 4,
        "name": null
      },
      {
        "checkpoint_id": "5d164cc8ca85695bbf54e177bcd7a4be",
        "docs": {
          "a.md": "8b1bfb808c054ab887a517336770c897"
        },
        "event_seq": 9,
        "name": "y"
      }
    ]
  },
  "trace": {
    "traces": [
      {
        "at_seq": 0,
        "checkpoint_id": "5e6669bbb36cd9558eff3bd83cfe1e45",
        "consumer": "probe",
        "query_hash": "a55527015d106b03e8c5f0b8a461b569",
        "returned": [
          {
            "doc_path": "a.md",
            "lineage_id": "f4495f5ee61ea960be41bfa9de348401",
            "version_id": "ba2f8a67da73b8981e9d95879a3df51d"
          },
          {
            "doc_path": "a.md",
            "lineage_id": "3a9c0a7481a3565fc10fbf3b7d0179fd",
            "version_id": "1db96e0d9cc284b36f2fcceab496e668"
          },
          {
            "doc_path": "a.md",
            "lineage_id": "4f6b5cb6f4c5549239547db8faaa7392",
            "version_id": "1da4a058824d8a2e310e117c9a5e7c7d"
          }
        ],
        "trace_id": "f1df769008e368fd450c5d040deda0c4"
      },
      {
        "at_seq": 1,
        "checkpoint_id": "5e6669bbb36cd9558eff3bd83cfe1e45",
        "consumer": "probe",
        "query_hash": "8d765afcb30fbf906a90b8914759b6c9",
        "returned": [
          {
            "doc_path": "a.md",
            "lineage_id": "f4495f5ee61ea960be41bfa9de348401",
            "version_id": "ba2f8a67da73b8981e9d95879a3df51d"
          }
        ],
        "trace_id": "42b45c3cff81c5595c9c221eb6704b44"
      }
    ]
  }
};
