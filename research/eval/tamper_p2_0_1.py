"""P2-0.1 deliberate cases (TASKS Y6; GOAL P2-0.1): each corruption must be
rejected, each untouched control must pass.

- controls: a link update Obsidian would write (only the link target of a
  renamed note changed, canonical block) attested with `link_update_of` →
  200 and `canonical`; a save attested with its bytes as `content` while an
  agent appended in the same observation → the human block `canonical`, the
  agent block `unverified`, the line `partial: true`; the vault opens;
- fake link update: the same rewrite plus a paragraph → 403, the paragraph
  stays `unverified`;
- wrong content: `content` that does not hash to `content_hash` → 400;
- partial forgery: an `attest` body carrying `partial: true` (a field only
  the server writes) → 400; a log line with `partial: false`, and one with
  `matched: true`, appended to `.rootmark/attestations.jsonl` → the vault
  does not open;
- `link_update_from` without `link_update_of` → 400;
- MCP: `tools/list` has no attestation tool, `tools/call attest` and
  `attest_session` → JSON-RPC -32602 (P2-0's check, `tamper_p2_0.mcp_cases`).

Writes `<out.json>`: {cases, rejected, rejected_total, controls_ok,
controls_total, pass} (the shape of P2-0's `tamper_p2_0.json`); exit 1
unless pass.

    python -m eval.tamper_p2_0_1 --rootmark <bin> --out <out.json>
"""

from __future__ import annotations

import argparse
import hashlib
import hmac
import json
import shutil
import sys
import tempfile
import urllib.error
import urllib.request
from pathlib import Path

from eval.run_p2_0 import content_hash
from eval.run_p2_0_1 import Served
from eval.tamper_p2_0 import WINDOW_S, _case, _layers, _opens, _settle, mcp_cases


def _post(sv: Served, tool: str, body: dict) -> int:
    req = urllib.request.Request(
        f"http://127.0.0.1:{sv.port}/v1/{tool}",
        data=json.dumps(body).encode(),
        headers={
            "Authorization": f"Bearer {sv.token}",
            "X-Consumer": "p2-0-1-tamper",
            "Content-Type": "application/json",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=60) as r:
            return r.status
    except urllib.error.HTTPError as e:
        return e.code


def _body(sv: Served, doc: str, data: bytes, **extra) -> dict:
    if sv.session is None:
        s = sv.call("attest_session", {})
        sv.session = (s["session_id"], bytes.fromhex(s["key"]))
    sid, key = sv.session
    h = content_hash(data)
    mac = hmac.new(key, f"rootmark-attest/1\n{doc}\n{h}\n".encode(), hashlib.sha256).hexdigest()
    return {"doc_path": doc, "content_hash": h, "session_id": sid, "mac": mac, **extra}


def _rename_with_links(sv: Served, target: str, to: str) -> list[str]:
    """Moves `target` to `to` and rewrites, in every note linking to it, the
    wikilink targets naming it (what Obsidian's link update writes); the
    notes rewritten."""
    old_stem, new_stem = Path(target).stem, Path(to).stem
    (sv.vault / target).rename(sv.vault / to)
    out = []
    for p in sorted(sv.vault.glob("*.md")):
        t = p.read_text(encoding="utf-8")
        n = (
            t.replace(f"[[{old_stem}]]", f"[[{new_stem}]]")
            .replace(f"[[{old_stem}|", f"[[{new_stem}|")
            .replace(f"[[{old_stem}#", f"[[{new_stem}#")
        )
        if n != t:
            p.write_text(n, encoding="utf-8")
            out.append(p.name)
    return out


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--rootmark", required=True)
    ap.add_argument("--out", required=True, type=Path)
    a = ap.parse_args(argv)
    rootmark = str(Path(a.rootmark).resolve())
    cases: list[dict] = []
    with tempfile.TemporaryDirectory(prefix="rootmark-p2-0-1-tamper-") as w:
        work = Path(w)
        sv = Served(rootmark, work / "s", WINDOW_S)
        try:
            # the mini vault: roasting.md is linked from several notes
            _settle(sv)
            linking = [
                p.name
                for p in sorted(sv.vault.glob("*.md"))
                if "[[roasting" in p.read_text(encoding="utf-8")
            ]
            if not linking:
                raise RuntimeError("no note links to roasting.md in the mini vault")
            olds = {d: {b["version_id"] for b in sv.blame(d) or []} for d in linking}
            rewritten = _rename_with_links(sv, "roasting.md", "kiln.md")
            # control: the first rewritten note, attested as a link update
            ok_doc = rewritten[0]
            data = (sv.vault / ok_doc).read_bytes()
            st = _post(sv, "attest", _body(sv, ok_doc, data, link_update_of="kiln.md"))
            _settle(sv)
            lay = _layers(sv, ok_doc, olds[ok_doc])
            good = st == 200 and lay != [] and all(x == "canonical" for x in lay)
            cases.append(
                _case(
                    "control",
                    "link update of a rename (link target only)",
                    "accepted",
                    "accepted" if good else "rejected",
                    {"status": st, "layers": lay},
                )
            )
            # fake link update: another rewritten note plus a paragraph
            if len(rewritten) < 2:
                raise RuntimeError("the rename rewrote fewer than two notes")
            bad_doc = rewritten[1]
            p = sv.vault / bad_doc
            p.write_bytes(p.read_bytes().rstrip(b"\n") + b"\n\nAn agent paragraph riding along.\n")
            data = p.read_bytes()
            st = _post(sv, "attest", _body(sv, bad_doc, data, link_update_of="kiln.md"))
            _settle(sv)
            lay = _layers(sv, bad_doc, olds[bad_doc])
            cases.append(
                _case(
                    "link_update",
                    "fake link update (a paragraph beyond the link targets)",
                    "rejected",
                    "rejected" if st == 403 and "canonical" not in lay else "accepted",
                    {"status": st, "layers": lay},
                )
            )
            # control: a partial match (human save + agent append, one
            # observation)
            doc = "espresso.md"
            old = {b["version_id"] for b in sv.blame(doc) or []}
            hp = sv.vault / doc
            human = hp.read_bytes().rstrip(b"\n") + b"\n\nA human paragraph saved now.\n"
            hp.write_bytes(human)
            sv.attest(doc, human)
            hp.write_bytes(human + b"\nAn agent paragraph in the same observation.\n")
            _settle(sv)
            blocks = {
                sv.call("read_block", {"doc_path": doc, "lineage_id": b["lineage_id"]})["block"][
                    "block"
                ]["text"]: b["layer"]
                for b in sv.blame(doc) or []
                if b["version_id"] not in old
            }
            lines = (sv.vault / ".rootmark/attestations.jsonl").read_text().splitlines()
            good = (
                blocks.get("A human paragraph saved now.") == "canonical"
                and blocks.get("An agent paragraph in the same observation.") == "unverified"
                and any('"partial":true' in x for x in lines)
            )
            cases.append(
                _case(
                    "control",
                    "partial match (human canonical, agent unverified, partial line)",
                    "accepted",
                    "accepted" if good else "rejected",
                    {"layers": blocks},
                )
            )
            # wrong content
            st = _post(sv, "attest", _body(sv, doc, human, content="other bytes"))
            cases.append(
                _case(
                    "content",
                    "content that does not hash to content_hash",
                    "rejected",
                    "rejected" if st == 400 else "accepted",
                    {"status": st},
                )
            )
            # partial forgery in the request
            st = _post(sv, "attest", _body(sv, doc, human, partial=True))
            cases.append(
                _case(
                    "partial",
                    "an attest body carrying partial: true",
                    "rejected",
                    "rejected" if st == 400 else "accepted",
                    {"status": st},
                )
            )
            st = _post(sv, "attest", _body(sv, doc, human, link_update_from="roasting.md"))
            cases.append(
                _case(
                    "link_update",
                    "link_update_from without link_update_of",
                    "rejected",
                    "rejected" if st == 400 else "accepted",
                    {"status": st},
                )
            )
        finally:
            sv.stop()
        cases.append(
            _case(
                "control",
                "the untouched vault opens",
                "accepted",
                "accepted" if _opens(rootmark, sv.vault) else "rejected",
            )
        )
        # forged log lines (copies of the vault)
        log = sv.vault / ".rootmark/attestations.jsonl"
        last = json.loads(log.read_text().splitlines()[-1])
        for field, value in [("partial", False), ("matched", True)]:
            v = work / f"forged-{field}"
            shutil.copytree(sv.vault, v)
            line = {**last, field: value}
            with open(v / ".rootmark/attestations.jsonl", "a", encoding="utf-8") as f:
                f.write(json.dumps(line, separators=(",", ":")) + "\n")
            cases.append(
                _case(
                    "log",
                    f"a log line with {field}: {json.dumps(value)}",
                    "rejected",
                    "accepted" if _opens(rootmark, v) else "rejected",
                )
            )
        cases += mcp_cases(rootmark, sv.vault)
    rej = [c for c in cases if c["expected"] == "rejected"]
    ctl = [c for c in cases if c["expected"] == "accepted"]
    res = {
        "cases": cases,
        "rejected": sum(c["as_expected"] for c in rej),
        "rejected_total": len(rej),
        "controls_ok": sum(c["as_expected"] for c in ctl),
        "controls_total": len(ctl),
    }
    res["pass"] = res["rejected"] == res["rejected_total"] and res["controls_ok"] == len(ctl)
    a.out.parent.mkdir(parents=True, exist_ok=True)
    a.out.write_text(json.dumps(res, indent=1) + "\n", encoding="utf-8")
    print(
        f"tamper_p2_0_1: rejected {res['rejected']}/{res['rejected_total']}, "
        f"controls {res['controls_ok']}/{res['controls_total']}"
    )
    return 0 if res["pass"] else 1


if __name__ == "__main__":
    sys.exit(main())
