"""P2-0.2 deliberate cases (TASKS X5; GOAL P2-0.2): each corruption must be
rejected, each untouched control must pass.

- controls: a rename's link update in a note whose Markdown link's label is
  the old note's name (`[roasting](roasting.md)` → `[kiln](kiln.md)`, T1) →
  200 and `canonical`; the rename of a note holding an agent's `unverified`
  paragraph, its own links rewritten in the same attestation
  (`link_update_of` = itself, `link_update_from`; T2) → 200, the human
  blocks `canonical`, the agent's paragraph `unverified`; a log line with a
  field the reader does not know → the vault opens (T4); the untouched vault
  opens;
- label + text: the label changed to something that is not the new name,
  beside a changed word → 403, nothing `canonical`;
- fake carry: a token holder's `renamed_from` from a human note to a file
  holding that note's bytes plus an agent paragraph → 403, the paragraph
  stays `unverified`; and a fake rename of an agent's `unverified` note to a
  new path → accepted, nothing raised (the carry keeps it `unverified`);
- P2-0.1's cases that still apply: a fake link update with a paragraph →
  403; `content` not hashing to `content_hash` → 400; an attest body
  carrying `partial: true` → 400; `link_update_from` without
  `link_update_of` → 400;
- log lines with an unknown value (`partial: false`, `matched: true`) → the
  vault does not open;
- MCP: `tools/list` has no attestation tool, `tools/call attest` and
  `attest_session` → JSON-RPC -32602 (`tamper_p2_0.mcp_cases`).

Writes `<out.json>`: {cases, rejected, rejected_total, controls_ok,
controls_total, pass} (the shape of P2-0.1's `tamper_p2_0_1.json`); exit 1
unless pass.

    python -m eval.tamper_p2_0_2 --rootmark <bin> --out <out.json>
"""

from __future__ import annotations

import argparse
import json
import shutil
import sys
import tempfile
from pathlib import Path

import blake3

from eval.run_p2_0_1 import Served
from eval.tamper_p2_0 import WINDOW_S, _case, _layers, _opens, _settle, mcp_cases
from eval.tamper_p2_0_1 import _body, _post

LINKS = "# Links\n\nSee [roasting](roasting.md) for the beans.\n\nAnd [[roasting]] there.\n"
LINKS2 = "# Links two\n\nAlso [roasting](roasting.md) today.\n"
SELF = "# Selfnote\n\nSee [[selfnote]] and [selfnote](selfnote.md).\n\nA second human line.\n"
AGENT = "An agent paragraph written to the file."


def _texts(sv: Served, doc: str) -> dict[str, str]:
    return {
        sv.call("read_block", {"doc_path": doc, "lineage_id": b["lineage_id"]})["block"]["block"][
            "text"
        ]: b["layer"]
        for b in sv.blame(doc) or []
    }


def _human(sv: Served, doc: str, text: str) -> None:
    """A human note saved and attested (the plugin's protocol), settled."""
    data = text.encode()
    (sv.vault / doc).write_bytes(data)
    sv.attest(doc, data)
    _settle(sv)


def _rewrite(text: str, old: str, new: str) -> str:
    """Obsidian's rewrite of the links to `old` (a note's stem): wikilink
    targets, Markdown destinations, and a Markdown label that is the name."""
    return (
        text.replace(f"[[{old}]]", f"[[{new}]]")
        .replace(f"[{old}]({old}.md)", f"[{new}]({new}.md)")
        .replace(f"]({old}.md)", f"]({new}.md)")
    )


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--rootmark", required=True)
    ap.add_argument("--out", required=True, type=Path)
    a = ap.parse_args(argv)
    rootmark = str(Path(a.rootmark).resolve())
    cases: list[dict] = []
    with tempfile.TemporaryDirectory(prefix="rootmark-p2-0-2-tamper-") as w:
        work = Path(w)
        sv = Served(rootmark, work / "s", WINDOW_S)
        try:
            _settle(sv)
            _human(sv, "links.md", LINKS)
            _human(sv, "links2.md", LINKS2)
            _human(sv, "selfnote.md", SELF)
            # the agent writes into the self-linking note (unattested)
            sp = sv.vault / "selfnote.md"
            sp.write_bytes(sp.read_bytes() + f"\n{AGENT}\n".encode())
            _settle(sv)
            olds = {
                d: {b["version_id"] for b in sv.blame(d) or []} for d in ["links.md", "links2.md"]
            }
            # the human renames roasting → kiln; Obsidian rewrites the links
            (sv.vault / "roasting.md").rename(sv.vault / "kiln.md")
            for d in ["links.md", "links2.md"]:
                p = sv.vault / d
                p.write_text(_rewrite(p.read_text(encoding="utf-8"), "roasting", "kiln"))
            # control: the label follows its target (T1)
            data = (sv.vault / "links.md").read_bytes()
            st = _post(
                sv,
                "attest",
                _body(
                    sv, "links.md", data, link_update_of="kiln.md", link_update_from="roasting.md"
                ),
            )
            _settle(sv)
            lay = _layers(sv, "links.md", olds["links.md"])
            good = st == 200 and lay != [] and all(x == "canonical" for x in lay)
            cases.append(
                _case(
                    "control",
                    "link update with a Markdown label that is the old name (T1)",
                    "accepted",
                    "accepted" if good else "rejected",
                    {"status": st, "layers": lay},
                )
            )
            # label + text: the label is not the new name, and a word changed
            p = sv.vault / "links2.md"
            t = p.read_text(encoding="utf-8").replace(
                "[kiln](kiln.md) today", "[the kiln](kiln.md) now"
            )
            p.write_text(t)
            st = _post(
                sv,
                "attest",
                _body(
                    sv,
                    "links2.md",
                    t.encode(),
                    link_update_of="kiln.md",
                    link_update_from="roasting.md",
                ),
            )
            _settle(sv)
            lay = _layers(sv, "links2.md", olds["links2.md"])
            cases.append(
                _case(
                    "link_update",
                    "a label changed to something that is not the new name, and a word",
                    "rejected",
                    "rejected" if st == 403 and "canonical" not in lay else "accepted",
                    {"status": st, "layers": lay},
                )
            )
            # control: the self-linking note renamed, its own links rewritten
            # in the same attestation; the carry keeps the agent unverified (T2)
            (sv.vault / "selfnote.md").rename(sv.vault / "self2.md")
            p = sv.vault / "self2.md"
            p.write_text(_rewrite(p.read_text(encoding="utf-8"), "selfnote", "self2"))
            st = _post(
                sv,
                "attest",
                _body(
                    sv,
                    "self2.md",
                    p.read_bytes(),
                    link_update_of="self2.md",
                    link_update_from="selfnote.md",
                ),
            )
            _settle(sv)
            tx = _texts(sv, "self2.md")
            humans = [v for k, v in tx.items() if AGENT not in k]
            good = (
                st == 200
                and humans != []
                and all(x == "canonical" for x in humans)
                and tx.get(AGENT) == "unverified"
            )
            cases.append(
                _case(
                    "control",
                    "a self-linking note renamed with its own links (carry: agent unverified)",
                    "accepted",
                    "accepted" if good else "rejected",
                    {"status": st, "layers": tx},
                )
            )
            # fake carry: a human note's bytes plus an agent paragraph, claimed
            # as its rename by a token holder
            src = (sv.vault / "espresso.md").read_bytes()
            fake = src.rstrip(b"\n") + b"\n\nAn agent paragraph riding a fake rename.\n"
            (sv.vault / "fake.md").write_bytes(fake)
            st = _post(sv, "attest", _body(sv, "fake.md", fake, renamed_from="espresso.md"))
            _settle(sv)
            tx = _texts(sv, "fake.md")
            cases.append(
                _case(
                    "carry",
                    "a fake rename carrying an agent paragraph",
                    "rejected",
                    "rejected"
                    if st == 403
                    and tx.get("An agent paragraph riding a fake rename.") != "canonical"
                    else "accepted",
                    {"status": st, "layers": tx},
                )
            )
            # a fake rename of the agent's own (unverified) note: accepted,
            # nothing raised
            (sv.vault / "fake.md").rename(sv.vault / "fake2.md")
            st = _post(
                sv,
                "attest",
                _body(sv, "fake2.md", (sv.vault / "fake2.md").read_bytes(), renamed_from="fake.md"),
            )
            _settle(sv)
            tx = _texts(sv, "fake2.md")
            cases.append(
                _case(
                    "carry",
                    "a rename of the agent's unverified note raises nothing",
                    "rejected",
                    "rejected" if "canonical" not in tx.values() and tx != {} else "accepted",
                    {"status": st, "layers": tx},
                )
            )
            # P2-0.1's cases that still apply
            p = sv.vault / "links.md"
            data = p.read_bytes().rstrip(b"\n") + b"\n\nAn agent paragraph riding along.\n"
            p.write_bytes(data)
            old = {b["version_id"] for b in sv.blame("links.md") or []}
            st = _post(sv, "attest", _body(sv, "links.md", data, link_update_of="kiln.md"))
            _settle(sv)
            lay = _layers(sv, "links.md", old)
            cases.append(
                _case(
                    "link_update",
                    "fake link update (a paragraph beyond the link targets)",
                    "rejected",
                    "rejected" if st == 403 and "canonical" not in lay else "accepted",
                    {"status": st, "layers": lay},
                )
            )
            doc = "espresso.md"
            st = _post(sv, "attest", _body(sv, doc, src, content="other bytes"))
            cases.append(
                _case(
                    "content",
                    "content that does not hash to content_hash",
                    "rejected",
                    "rejected" if st == 400 else "accepted",
                    {"status": st},
                )
            )
            st = _post(sv, "attest", _body(sv, doc, src, partial=True))
            cases.append(
                _case(
                    "partial",
                    "an attest body carrying partial: true",
                    "rejected",
                    "rejected" if st == 400 else "accepted",
                    {"status": st},
                )
            )
            st = _post(sv, "attest", _body(sv, doc, src, link_update_from="roasting.md"))
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
        # log lines (copies of the vault): an unknown field is read (T4), an
        # unknown value of a known field is refused
        log = sv.vault / ".rootmark/attestations.jsonl"
        last = json.loads(log.read_text().splitlines()[-1])
        for field, value, expected in [
            ("x_future", 1, "accepted"),
            ("partial", False, "rejected"),
            ("matched", True, "rejected"),
        ]:
            v = work / f"forged-{field}"
            shutil.copytree(sv.vault, v)
            # a line valid in every other respect: its own content_hash and
            # the id of the §8 formula (a copy of the last line would be
            # refused for its duplicate id, whatever the field)
            h = blake3.blake3(f"forged {field}".encode()).hexdigest()[:32]
            at = last["at_seq"]
            line = {
                "attestation_id": blake3.blake3(
                    f"{last['doc_path']}\n{h}\n{at}".encode()
                ).hexdigest()[:32],
                "doc_path": last["doc_path"],
                "content_hash": h,
                "at_seq": at,
                field: value,
            }
            with open(v / ".rootmark/attestations.jsonl", "a", encoding="utf-8") as f:
                f.write(json.dumps(line, separators=(",", ":")) + "\n")
            opened = _opens(rootmark, v)
            got = "accepted" if opened else "rejected"
            group = "control" if expected == "accepted" else "log"
            cases.append(
                _case(group, f"a log line with {field}: {json.dumps(value)}", expected, got)
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
        f"tamper_p2_0_2: rejected {res['rejected']}/{res['rejected_total']}, "
        f"controls {res['controls_ok']}/{res['controls_total']}"
    )
    return 0 if res["pass"] else 1


if __name__ == "__main__":
    sys.exit(main())
