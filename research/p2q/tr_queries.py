"""P2-Q Turkish query pass (K-P2Q.3; scenario_tr.json): a fresh copy of a
personal corpus, the `tr` recipe, an HTTP-only server, the registered
queries, and the oracle — the documents that contain a query's folded token,
computed locally from the copy by the documented `uni_fold_tr` rule.
Records counts, equalities and keyed path hashes only.

    python3 -m p2q.tr_queries --corpus obsidian|qorbit --snapshot <dir> --work <dir> --out <dir>
        [--rootmark rootmark]
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import unicodedata
from pathlib import Path

from p2q.common import dump, fresh_copy, k2, list_md, load, path_key
from p2q.driver import Server

HERE = Path(__file__).resolve().parent


def fold_tokens(text: str) -> list[str]:
    """uni_fold_tr (index/src/tokenize.rs): NFC; İ→i, I→ı; lower case;
    maximal runs of alphabetic/numeric characters."""
    out, cur = [], []
    for c in unicodedata.normalize("NFC", text):
        c = "i" if c == "İ" else "ı" if c == "I" else c
        for lc in c.lower():
            if lc.isalpha() or lc.isnumeric():
                cur.append(lc)
            elif cur:
                out.append("".join(cur))
                cur = []
    if cur:
        out.append("".join(cur))
    return out


def body(text: str) -> str:
    """The indexed text: front matter (a leading --- block) left out."""
    if text.startswith("---\n"):
        end = text.find("\n---", 4)
        if end >= 0:
            return text[end + 4 :]
    return text


def run(a) -> int:
    scen = load(HERE / "scenario_tr.json")
    main = load(HERE / "scenario.json")
    key = path_key()
    work, out = a.work.resolve(), a.out.resolve()
    work.mkdir(parents=True, exist_ok=True)
    out.mkdir(parents=True, exist_ok=True)
    vault = work / "vault"
    fresh_copy(a.snapshot, vault)
    paths = list_md(vault)
    tokens = {
        p: set(fold_tokens(body((vault / p).read_text(encoding="utf-8", errors="replace"))))
        for p in paths
    }
    subprocess.run(
        [a.rootmark, "ingest-dir", str(vault), str(vault)], check=True, capture_output=True
    )
    (vault / ".rootmark/policy.toml").write_text(main["recipes"]["tr"], encoding="utf-8")
    subprocess.run([a.rootmark, "index", "build", str(vault)], check=True, capture_output=True)
    srv = Server(
        a.rootmark,
        vault,
        work / "serve.log",
        work / "attest.log",
        {"name": "tr-queries", "version": "1.0"},
    )
    res, rows = {}, []
    for q in scen["queries"]:
        k = scen["k"][q["mode"]]
        r = srv.http("search", {"query": q["query"], "mode": q["mode"], "k": k}, consumer="p2q-tr")
        hits = [(k2(h["doc_path"], key), h["chunk_id"]) for h in r.get("hits", [])]
        res[q["id"]] = {
            "hits": hits,
            "docs": sorted({d for d, _ in hits}),
            "error": r.get("_error"),
        }
    for q in scen["queries"]:
        row = {
            "id": q["id"],
            "group": q["group"],
            "hits": len(res[q["id"]]["hits"]),
            "docs": len(res[q["id"]]["docs"]),
            "error": res[q["id"]]["error"],
        }
        if q["mode"] == "lexical":
            toks = fold_tokens(q["query"])
            oracle = sorted({k2(p, key) for p, t in tokens.items() if all(x in t for x in toks)})
            row["oracle_docs"] = len(oracle)
            row["TR-2"] = res[q["id"]]["docs"] == oracle
            row["only_search"] = len(set(res[q["id"]]["docs"]) - set(oracle))
            row["only_oracle"] = len(set(oracle) - set(res[q["id"]]["docs"]))
        if q["group"] == "fold" and q["query"] != q["base"]:
            base = next(x for x in scen["queries"] if x["query"] == q["base"])
            row["TR-1"] = res[q["id"]]["hits"] == res[base["id"]]["hits"]
        if q["group"] in ("accentless", "accentless_hybrid"):
            acc = [
                x
                for x in scen["queries"]
                if x.get("query") == q["accented"] and x["mode"] == q["mode"]
            ]
            if q["mode"] == "lexical":
                acc_docs = {
                    p for p, t in tokens.items() if all(x in t for x in fold_tokens(q["accented"]))
                }
                row["accented_docs_lost"] = len(
                    {k2(p, key) for p in acc_docs} - set(res[q["id"]]["docs"])
                )
            else:
                r2 = srv.http(
                    "search", {"query": q["accented"], "mode": "hybrid", "k": 10}, consumer="p2q-tr"
                )
                top = [k2(h["doc_path"], key) for h in r2.get("hits", [])]
                row["top10_overlap_with_accented"] = len(set(top) & set(res[q["id"]]["docs"]))
            row["registered_pair"] = bool(acc) or q["mode"] == "lexical"
        rows.append(row)
    stop = srv.close()
    tr1 = [r for r in rows if "TR-1" in r]
    tr2 = [r for r in rows if "TR-2" in r]
    summary = {
        "corpus": a.corpus,
        "queries": len(rows),
        "TR-1": {"passed": sum(r["TR-1"] for r in tr1), "of": len(tr1)},
        "TR-2": {"passed": sum(r["TR-2"] for r in tr2), "of": len(tr2)},
        "serve_rc": stop["rc"],
    }
    dump({"summary": summary, "rows": rows}, out / "tr_queries.json")
    print(json.dumps(summary))
    return (
        0 if summary["TR-1"]["passed"] == len(tr1) and summary["TR-2"]["passed"] == len(tr2) else 1
    )


def main(argv=None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--corpus", required=True, choices=["obsidian", "qorbit", "tooltest"])
    ap.add_argument("--snapshot", required=True, type=Path)
    ap.add_argument("--work", required=True, type=Path)
    ap.add_argument("--out", required=True, type=Path)
    ap.add_argument("--rootmark", default="rootmark")
    a = ap.parse_args(argv)
    if not os.environ.get("ROOTMARK_MODELS"):
        raise SystemExit("ROOTMARK_MODELS is not set")
    return run(a)


if __name__ == "__main__":
    sys.exit(main())
