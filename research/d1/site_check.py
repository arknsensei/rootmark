"""D-1/R6 (D1-9): the published documentation site, read over HTTPS.

Fetches each page of the site's SUMMARY (and every page reachable from them
on the same host), checks status 200, then every same-site link and anchor
(`page.html#id`): the target page answers 200 and the id exists. Off-site
links are listed, not fetched. Writes a record with no host value.

    python3 -I research/d1/site_check.py --base https://docs.rootmark.ai/ --out <record.json>
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import urllib.error
import urllib.request
from html.parser import HTMLParser
from urllib.parse import urldefrag, urljoin, urlparse

REQUIRED = ["install.html", "guide.html", "git.html", "contract.html", "threat-model.html",
            "spec/index.html", "spec/conformance.html", "changelog.html", "security.html"]


class P(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links, self.ids = [], set()
        self.title, self._t = "", False

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if a.get("id"):
            self.ids.add(a["id"])
        if tag == "a" and a.get("name"):
            self.ids.add(a["name"])
        for k in ("href", "src"):
            if a.get(k) and tag in ("a", "link", "script", "img"):
                self.links.append((tag, a[k]))
        self._t = tag == "title"

    def handle_data(self, d):
        if self._t:
            self.title += d

    def handle_endtag(self, tag):
        self._t = False


def fetch(url):
    req = urllib.request.Request(url, headers={"User-Agent": "site-check"})
    try:
        with urllib.request.urlopen(req, timeout=30) as r:
            return r.status, r.read(), r.headers.get_content_type()
    except urllib.error.HTTPError as e:
        return e.code, b"", ""


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--base", required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    host = urlparse(a.base).netloc
    pages, queue, assets, offsite = {}, [a.base] + [urljoin(a.base, p) for p in REQUIRED], {}, set()
    while queue:
        u = urldefrag(queue.pop())[0]
        if u in pages:
            continue
        st, body, ctype = fetch(u)
        info = {"status": st}
        if st == 200 and ctype == "text/html":
            p = P()
            p.feed(body.decode("utf-8", "replace"))
            info.update(title=p.title.strip(), ids=p.ids, links=p.links)
            for tag, l in p.links:
                t = urljoin(u, l)
                pu = urlparse(t)
                if pu.scheme not in ("http", "https"):
                    continue
                if pu.netloc != host:
                    offsite.add(f"{pu.scheme}://{pu.netloc}{pu.path}")
                    continue
                base = urldefrag(t)[0]
                if tag == "a" and (base.endswith(".html") or base.endswith("/")):
                    queue.append(base)
                elif base not in assets:
                    assets[base] = fetch(base)[0]
        pages[u] = info
    broken = []
    for u, info in pages.items():
        for tag, l in info.get("links", []):
            t = urljoin(u, l)
            pu = urlparse(t)
            if pu.netloc != host or pu.scheme not in ("http", "https"):
                continue
            base, frag = urldefrag(t)
            st = pages.get(base, {}).get("status", assets.get(base))
            if st != 200:
                broken.append({"from": u.replace(a.base, "/"), "to": l, "status": st})
            elif frag and base in pages and frag not in pages[base].get("ids", set()):
                broken.append({"from": u.replace(a.base, "/"), "to": l, "missing_anchor": frag})
    rel = lambda u: "/" + u[len(a.base):] if u.startswith(a.base) else u  # noqa: E731
    rec = {
        "what": "D-1/R6 published site check (research/d1/site_check.py)",
        "base": a.base,
        "required": {p: pages.get(urljoin(a.base, p), {}).get("status") for p in REQUIRED},
        "pages": {rel(u): {"status": i["status"], "title": i.get("title")} for u, i in sorted(pages.items())},
        "assets_checked": len(assets),
        "assets_not_200": {rel(u): s for u, s in assets.items() if s != 200},
        "internal_links_broken": broken,
        "offsite_links": sorted(offsite),
    }
    rec["pass"] = (all(s == 200 for s in rec["required"].values()) and pages[a.base]["status"] == 200
                   and all(i["status"] == 200 for i in pages.values()) and not broken and not rec["assets_not_200"])
    with open(a.out, "w", encoding="utf-8") as f:
        json.dump(rec, f, indent=1)
        f.write("\n")
    print(json.dumps({k: rec[k] for k in ("required", "assets_checked", "assets_not_200", "internal_links_broken", "pass")}))
    print(len(rec["pages"]), "pages;", len(rec["offsite_links"]), "offsite")
    return 0 if rec["pass"] else 1


if __name__ == "__main__":
    sys.exit(main())
