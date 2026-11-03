"""P2-Q numbers (TASKS Q4): reports/rounds/P2-Q/numbers.json and numbers.md
from the round's records only — every number names the record it comes from.

    python -m p2q.numbers
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
R = REPO / "reports/rounds/P2-Q"

KEYS = [
    ("ingest_s", "ingest-dir (s)"),
    ("index_build_s", "index build (s)"),
    ("index_embedded", "chunks embedded"),
    ("index_chunks_per_s", "chunks/s"),
    ("propose_ms", "MCP propose (ms)"),
    ("propose_to_searchable_ms", "propose → searchable (ms)"),
    ("direct_write_to_ingested_ms", "direct write → ingested (ms)"),
    ("obsidian_save_to_layer_ms", "Obsidian save → layer (ms)"),
    ("search_ms", "search (ms, HTTP, client side)"),
    ("plugin_proposals_view_ms", "plugin proposals view open + refresh (ms)"),
    ("serve_peak_rss_kib", "serve peak RSS (KiB)"),
    ("rootmark_dir_bytes", ".rootmark/ (bytes)"),
]


def runs() -> list[tuple[str, str, Path]]:
    out = []
    for env in ("mac", "docker", "ci"):
        base = R / env
        if not base.is_dir():
            continue
        for f in sorted(base.rglob("numbers.json")):
            out.append((env, str(f.parent.relative_to(R)), f))
    return out


def main() -> int:
    rec = {
        "source": "reports/rounds/P2-Q/<env>/<corpus>[/<run>]/numbers.json",
        "note": "no thresholds (GOAL P2-Q); docker = emulated linux/amd64 on the Mac: information",
        "runs": {},
    }
    for env, rel, f in runs():
        n = json.loads(f.read_text())
        row = {
            "env": env,
            "record": f"reports/rounds/P2-Q/{rel}/numbers.json",
            "cpu": n.get("cpu"),
            "platform": n.get("platform"),
        }
        for k, _ in KEYS:
            if k in n:
                row[k] = n[k]
        rec["runs"][rel] = row
    install = R / "mac/install.json"
    if install.exists():
        rec["install"] = json.loads(install.read_text())
    (R / "numbers.json").write_text(
        json.dumps(rec, indent=1, sort_keys=True, ensure_ascii=False) + "\n"
    )
    lines = [
        "# P2-Q — Ölçülen sayılar (eşiksiz; kaynak: her satırın kaydı)",
        "",
        "| Kayıt | Ortam | İşlemci | " + " | ".join(lbl for _, lbl in KEYS) + " |",
        "|" + "---|" * (3 + len(KEYS)),
    ]

    def fmt(v):
        if isinstance(v, dict):
            return "/".join(str(v.get(x)) for x in ("p50", "p95") if x in v) or str(v.get("max"))
        return "—" if v is None else str(v)

    for rel, row in rec["runs"].items():
        lines.append(
            f"| `{rel}` | {row['env']} | {row.get('cpu') or '—'} | "
            + " | ".join(fmt(row.get(k)) for k, _ in KEYS)
            + " |"
        )
    lines += ["", "Gecikmeler p50/p95. Docker satırları emülasyondur (bilgi, hız sayısı değil)."]
    (R / "numbers.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"numbers: {len(rec['runs'])} runs")
    return 0


if __name__ == "__main__":
    sys.exit(main())
