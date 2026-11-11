#!/usr/bin/env python3
"""D-0/R3 — licence fields of the pinned embedder models (GET only).

Reads the Hugging Face model API for each models/MANIFEST*.json source at its
pinned revision and records the card's licence field and base model.

    python3 -I research/d0/model_license.py --out reports/rounds/D-0/transfer/model_licenses.json
"""

import argparse
import datetime as dt
import hashlib
import json
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def get(url):
    req = urllib.request.Request(
        url, headers={"User-Agent": "rootmark-d0-model-license/1 (read-only)"}, method="GET"
    )
    at = dt.datetime.now(dt.UTC).strftime("%Y-%m-%dT%H:%M:%SZ")
    try:
        with urllib.request.urlopen(req, timeout=30) as f:
            return at, f.status, f.read()
    except urllib.error.HTTPError as e:
        return at, e.code, e.read()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    rows = []
    for m in sorted(ROOT.glob("models/MANIFEST*.json")):
        man = json.loads(m.read_text())
        src, rev = man["source"], man["revision"]
        url = f"https://huggingface.co/api/models/{src}/revision/{rev}"
        at, code, body = get(url)
        j = json.loads(body) if code == 200 else {}
        card = j.get("cardData") or {}
        size = sum(f["size"] for f in man["files"].values())
        rows.append(
            {
                "manifest": m.name,
                "name": man["name"],
                "source": src,
                "revision": rev,
                "url": url,
                "queried_at": at,
                "http_status": code,
                "body_sha256": hashlib.sha256(body).hexdigest(),
                "license": card.get("license"),
                "base_model": card.get("base_model"),
                "pinned_files_bytes": size,
            }
        )
    Path(a.out).write_text(json.dumps({"rows": rows}, indent=1) + "\n")
    for r in rows:
        print(r["name"], r["http_status"], r["license"], r["base_model"], r["pinned_files_bytes"])


if __name__ == "__main__":
    main()
