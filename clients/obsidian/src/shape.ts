// The fields of each result the client relies on (docs/CLIENT_CONTRACT.md
// §2 main fields and §2.5 inner fields), checked on every answer: a field
// missing or of another type is `bad_response`, never a silent half-result.
// Fields not listed here may arrive; nothing relies on them.

import type { ToolName } from "./contract.gen";

export type Spec =
  | "string"
  | "string|null"
  | "integer"
  | "number"
  | "boolean"
  | "layer"
  | "object"
  | readonly [Spec]
  | { readonly [field: string]: Spec };

// spec 2.0.0 adds `unverified` (§7.1; contract §2.5)
const LAYERS = new Set(["canonical", "proposal", "unverified", "derived"]);

/** The first violation of `spec` at `path`, or null. */
export function violation(v: unknown, spec: Spec, path: string): string | null {
  if (Array.isArray(spec)) {
    if (!Array.isArray(v)) return `${path}: not an array`;
    for (let i = 0; i < v.length; i++) {
      const e = violation(v[i], spec[0] as Spec, `${path}[${String(i)}]`);
      if (e !== null) return e;
    }
    return null;
  }
  if (typeof spec === "object") {
    if (typeof v !== "object" || v === null || Array.isArray(v)) return `${path}: not an object`;
    const o = v as Record<string, unknown>;
    for (const [k, s] of Object.entries(spec)) {
      if (!(k in o)) return `${path}.${k}: missing`;
      const e = violation(o[k], s, `${path}.${k}`);
      if (e !== null) return e;
    }
    return null;
  }
  switch (spec) {
    case "string":
      return typeof v === "string" ? null : `${path}: not a string`;
    case "string|null":
      return typeof v === "string" || v === null ? null : `${path}: not a string or null`;
    case "integer":
      return Number.isInteger(v) ? null : `${path}: not an integer`;
    case "number":
      return typeof v === "number" && Number.isFinite(v) ? null : `${path}: not a number`;
    case "boolean":
      return typeof v === "boolean" ? null : `${path}: not a boolean`;
    case "layer":
      return typeof v === "string" && LAYERS.has(v) ? null : `${path}: not a layer`;
    case "object":
      return typeof v === "object" && v !== null && !Array.isArray(v) ? null : `${path}: not an object`;
  }
}

const BLOCK = {
  lineage_id: "string",
  version_id: "string",
  layer: "layer",
  layer_source: "string",
  author_kind: "string",
  source: "string",
  seq_created: "integer",
  seq_last: "integer",
  inactive: "boolean",
} as const;
const EVENT = { seq: "integer", kind: "string", doc_path: "string", lineage_ids: ["string"], meta: "object" } as const;
const RETURNED = { doc_path: "string", lineage_id: "string", version_id: "string" } as const;
const TRACE = {
  trace_id: "string",
  checkpoint_id: "string",
  consumer: "string",
  query_hash: "string",
  at_seq: "integer",
  returned: [RETURNED],
} as const;
const CHECKPOINT = { checkpoint_id: "string", name: "string|null", event_seq: "integer", docs: "object" } as const;

/** Result specs by tool (`promote` has two shapes, checked by decision). */
export const RESULT: { readonly [T in Exclude<ToolName, "promote">]: Spec } = {
  status: {
    vault: "string",
    spec_version: "string",
    mdk_version: "string",
    level_claimed: "integer",
    embedder: "string|null",
    index: { state: "string", format: "string|null", expected_format: "string", auto_rebuild: "boolean" },
    policy: { promotion: "string", edit: "string" },
    server_policy: { server: "object", index: "object", identity: { human_id: "string|null" } },
    attestation: { unattested: "string", window_s: "number", stage: "integer", pending: "integer" },
  },
  search: {
    checkpoint_id: "string",
    mode: "string",
    include_proposal: "boolean",
    query_hash: "string",
    trace_id: "string",
    hits: [{ doc_path: "string", chunk_id: "string", lineage_ids: ["string"], layer: "layer", score: "number" }],
  },
  read_block: {
    checkpoint_id: "string",
    block: {
      doc_path: "string",
      inactive: "boolean",
      block: {
        version_id: "string",
        lineage_id: "string",
        block_type: "string",
        text: "string",
        heading_path: ["string"],
        layer: "layer",
        provenance: { author_kind: "string", source: "string" },
      },
    },
    trace: TRACE,
  },
  history: { lineage_id: "string", layer: "layer", events: [EVENT] },
  blame: { doc_path: "string", blocks: [BLOCK] },
  propose: {
    doc_path: "string",
    mode: "string",
    edit_policy: "string",
    actor: { kind: "string", id: "string" },
    blocks: [BLOCK],
  },
  checkpoint: CHECKPOINT,
  checkpoints: { checkpoints: [CHECKPOINT] },
  trace: { traces: [TRACE] },
};

const PROMOTED = {
  lineage_id: "string",
  layer: "layer",
  event: EVENT,
  policy: "string",
  policy_version: "string",
} as const;
const DENIED = {
  lineage_id: "string",
  layer: "layer",
  reason: "string",
  actor: { kind: "string", id: "string" },
  policy: "string",
  policy_version: "string",
} as const;

/** The first violation of a `tool` result, or null. */
export function resultViolation(tool: ToolName, v: unknown): string | null {
  if (tool !== "promote") return violation(v, RESULT[tool], tool);
  const d = (v as { decision?: unknown }).decision;
  if (d === "promoted") return violation(v, PROMOTED, "promote");
  if (d === "denied") return violation(v, DENIED, "promote");
  return "promote.decision: neither promoted nor denied";
}

/** `status.last_checkpoint`: a checkpoint or null (§2.5). */
export function lastCheckpointViolation(v: unknown): string | null {
  const lc = (v as { last_checkpoint?: unknown }).last_checkpoint;
  if (lc === undefined) return "status.last_checkpoint: missing";
  return lc === null ? null : violation(lc, CHECKPOINT, "status.last_checkpoint");
}
