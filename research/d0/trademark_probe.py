#!/usr/bin/env python3
"""D-0/R2 — can the four trademark sources be queried with a public GET?

(docs/rounds/D-0/progress/DESIGN.md §2.) For each source one documented
search URL is fetched with GET (no POST, no login, no captcha). The row says
whether the answer carries a machine-readable result for the query or only a
JavaScript application shell; in the latter case the query is a human step
(HUMAN_STEP.md). Only the status, content type, size, body sha256 and
whether the query word occurs in the body are recorded — no body text.

    python3 -I research/d0/trademark_probe.py --out reports/rounds/D-0/trademark_probe.json
"""
from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import urllib.error
import urllib.request
from pathlib import Path

UA = "rootmark-d0-trademark-probe/1 (read-only)"
Q = "rootmark"
SOURCES = [
    ("TÜRKPATENT", "https://www.turkpatent.gov.tr/arastirma-yap?form=trademark"),
    ("EUIPO TMview", f"https://www.tmdn.org/tmview/#/tmview/results?page=1&pageSize=30&criteria=C&basicSearch={Q}"),
    ("EUIPO TMview (API, GET)", f"https://www.tmdn.org/tmview/api/search/results?basicSearch={Q}&page=1&pageSize=30"),
    ("USPTO Trademark Search", f"https://tmsearch.uspto.gov/search/search-results?query={Q}&section=default"),
    ("WIPO Global Brand Database", f"https://branddb.wipo.int/en/quicksearch/brand?by=brandName&v={Q}"),
]


def now():
    return dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def fetch(url):
    req = urllib.request.Request(url, headers={"User-Agent": UA}, method="GET")
    at = now()
    try:
        with urllib.request.urlopen(req, timeout=30) as f:
            return at, f.status, f.headers.get("Content-Type"), f.read(), None
    except urllib.error.HTTPError as e:
        return at, e.code, e.headers.get("Content-Type"), e.read(), None
    except Exception as e:
        return at, None, None, None, type(e).__name__


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    rows = []
    for src, url in SOURCES:
        at, code, ctype, body, err = fetch(url)
        is_json = bool(ctype and "json" in ctype)
        hit = (Q.encode() in body.lower()) if body else False
        machine = None
        if is_json and body:
            try:
                json.loads(body)
                machine = True
            except Exception:
                machine = False
        rows.append({"source": src, "url": url, "queried_at": at, "http_status": code, "error": err,
                     "content_type": ctype, "bytes": len(body) if body else None,
                     "body_sha256": hashlib.sha256(body).hexdigest() if body else None,
                     "query_word_in_body": hit, "machine_readable_result": bool(machine),
                     "verdict": "agent_result" if machine else "human_step (no machine-readable result for a GET)"})
    doc = {"probe": "D-0/R2 trademark sources, GET only", "query": Q, "user_agent": UA, "rows": rows}
    Path(a.out).write_text(json.dumps(doc, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    for r in rows:
        print(r["source"], r["http_status"] or r["error"], r["content_type"], r["bytes"], r["query_word_in_body"], r["verdict"])


if __name__ == "__main__":
    main()
