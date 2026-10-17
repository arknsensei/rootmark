"""Content-referenced truth → lineage (EVAL §Gerçek referansı).

A truth unit is `{"doc_path", "version_id", "k"}`; it is resolved only through
`Vault.resolve_truth`. A single unresolvable reference aborts the run: there is
no fallback and no silent skip.
"""

from __future__ import annotations

import re

import rootmark_core

HEX32 = re.compile(r"^[0-9a-f]{32}$")


class TruthError(RuntimeError):
    pass


def check_ref(ref: dict) -> None:
    if set(ref) - {"doc_path", "version_id", "k"} or "version_id" not in ref or "k" not in ref:
        raise TruthError(f"malformed truth reference: {ref!r}")
    if not HEX32.match(str(ref["version_id"])) or not isinstance(ref["k"], int) or ref["k"] < 0:
        raise TruthError(f"malformed truth reference: {ref!r}")


def resolve(vault: rootmark_core.Vault, ref: dict, doc_path: str | None = None) -> str:
    """lineage_id of `ref`; `doc_path` overrides/provides the document."""
    check_ref(ref)
    doc = doc_path if doc_path is not None else ref.get("doc_path")
    if not doc:
        raise TruthError(f"truth reference without doc_path: {ref!r}")
    try:
        return vault.resolve_truth(doc, ref["version_id"], ref["k"])
    except RuntimeError as e:
        raise TruthError(f"unresolvable truth {doc}:{ref['version_id']}#{ref['k']}: {e}") from e


def resolve_all(
    vault: rootmark_core.Vault, refs: list[dict], doc_path: str | None = None
) -> list[str]:
    return [resolve(vault, r, doc_path) for r in refs]
