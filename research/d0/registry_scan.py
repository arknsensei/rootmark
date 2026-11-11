#!/usr/bin/env python3
"""D-0/R1 — name x registry scan (docs/rounds/D-0/progress/DESIGN.md §1).

Read-only: every request is a GET (one function, `fetch`), plus a port-43
whois query for `.io` (no RDAP in IANA's bootstrap). Nothing is written to
any registry. Each cell becomes one evidenced row: URL, UTC time, response
code, a content-free summary (state, owner type/handle or project URL,
dates), and the sha256 of the response body (a fingerprint, not content).
No e-mail address, person name or body text is recorded.

    python3 -I research/d0/registry_scan.py --out reports/rounds/D-0
"""
from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import re
import shutil
import subprocess
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

NAMES = [
    "rootmark", "rootmark-bindings", "rootmark-core", "rootmark-index",
    "rootmark-server", "rootmark-ai", "getrootmark",
    "rhizomark", "rootmarkhq",  # backups (DESIGN §1.1)
]
TLDS = ["ai", "com", "dev", "io", "app", "org"]
COLUMNS = [
    "crates.io", "pypi", "testpypi", "npm_pkg", "npm_org", "npm_user",
    "dockerhub", "ghcr", "github", "brew_formula", "brew_cask", "brew_tap",
    "obsidian_id", "obsidian_name", "vscode_publisher", "vscode_ext",
    "openvsx_ns", "openvsx_ext",
] + [f"dom_{t}" for t in TLDS]
UA = "rootmark-d0-registry-scan/1 (read-only)"
OBS = "https://raw.githubusercontent.com/obsidianmd/obsidian-releases/master/"


def now() -> str:
    return dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


class Resp:
    def __init__(self, url, status, body, final_url, err=None):
        self.url, self.status, self.body, self.final_url, self.err = url, status, body, final_url, err
        self.at = now()

    @property
    def sha(self):
        return hashlib.sha256(self.body).hexdigest() if self.body is not None else None

    def json(self):
        try:
            return json.loads(self.body)
        except Exception:
            return None


def fetch(url: str, headers: dict | None = None, ok=(200, 404)) -> Resp:
    """The only HTTP request function: method GET, fixed. Retries non-200/404
    answers (429, 5xx, timeouts) up to three times with growing waits."""
    last = None
    for attempt in range(4):
        req = urllib.request.Request(url, headers={"User-Agent": UA, **(headers or {})}, method="GET")
        try:
            with urllib.request.urlopen(req, timeout=30) as f:
                r = Resp(url, f.status, f.read(), f.url)
        except urllib.error.HTTPError as e:
            r = Resp(url, e.code, e.read(), url)
        except Exception as e:  # timeouts, DNS, TLS
            r = Resp(url, None, None, url, err=type(e).__name__)
        last = r
        if r.status in ok:
            return r
        time.sleep(2 * (attempt + 1))
    return last


def gh_get(path: str) -> Resp:
    """GitHub REST GET via `gh api` (local identity, read-only) or urllib."""
    url = "https://api.github.com/" + path
    if shutil.which("gh"):
        for attempt in range(4):
            p = subprocess.run(["gh", "api", "-X", "GET", "-i", path], capture_output=True)
            raw = p.stdout
            head, _, body = raw.partition(b"\r\n\r\n")
            m = re.match(rb"HTTP/[\d.]+ (\d+)", head)
            status = int(m.group(1)) if m else None
            r = Resp(url, status, body, url, err=None if status else "gh_failed")
            if status in (200, 404):
                return r
            time.sleep(3 * (attempt + 1))
        return r
    return fetch(url)


def row(name, col, r: Resp | None, state, summary=None, note=None, url=None, **extra):
    d = {
        "name": name, "registry": col,
        "url": url or (r.url if r else None),
        "queried_at": r.at if r else now(),
        "http_status": r.status if r else None,
        "state": state,
        "summary": summary or {},
        "body_sha256": r.sha if r else None,
        "note": note,
    }
    if r is not None and r.err:
        d["error"] = r.err
    d.update(extra)
    return d


def st(r: Resp) -> str:
    if r.status == 200:
        return "taken"
    if r.status == 404:
        return "free"
    return "unreachable"


def why(r: Resp) -> str | None:
    if st(r) == "unreachable":
        return f"unreachable after 4 attempts: {r.status or r.err}"
    return None


def date(s):
    return s[:10] if isinstance(s, str) else None


# --- registries ---------------------------------------------------------

def crates(n):
    r = fetch(f"https://crates.io/api/v1/crates/{n}")
    s = {}
    if r.status == 200 and r.json():
        c = r.json()["crate"]
        s = {"canonical_name": c["name"], "created": date(c["created_at"]), "updated": date(c["updated_at"]),
             "max_version": c.get("max_version"), "repository": c.get("repository")}
    return row(n, "crates.io", r, st(r), s, why(r))


def pypi(n, col, host):
    r = fetch(f"https://{host}/pypi/{n}/json")
    s = {}
    if r.status == 200 and r.json():
        j = r.json()
        info = j["info"]
        rel = sorted((f["upload_time_iso_8601"] for v in j.get("releases", {}).values() for f in v), key=str)
        s = {"canonical_name": info.get("name"), "summary": (info.get("summary") or "")[:120],
             "latest_version": info.get("version"), "first_upload": date(rel[0]) if rel else None,
             "last_upload": date(rel[-1]) if rel else None, "release_count": len(j.get("releases", {})),
             "project_urls": sorted(set((info.get("project_urls") or {}).values()) | ({info["home_page"]} if info.get("home_page") else set()))}
    return row(n, col, r, st(r), s, why(r))


def npm_pkg(n):
    r = fetch(f"https://registry.npmjs.org/{n}")
    s = {}
    if r.status == 200 and r.json():
        j = r.json()
        t = j.get("time", {})
        s = {"latest": (j.get("dist-tags") or {}).get("latest"), "created": date(t.get("created")),
             "modified": date(t.get("modified")), "description": (j.get("description") or "")[:120]}
    variant = re.sub(r"[-_.]", "", n)
    extra = {}
    if variant != n:
        rv = fetch(f"https://registry.npmjs.org/{variant}")
        extra["punct_variant"] = {"name": variant, "url": rv.url, "queried_at": rv.at,
                                  "http_status": rv.status, "state": st(rv), "body_sha256": rv.sha}
    note = why(r)
    if extra and extra["punct_variant"]["state"] == "taken":
        note = (note or "") + f"npm rejects names equal after removing punctuation: '{variant}' is taken"
    return row(n, "npm_pkg", r, st(r), s, note, **extra)


def npm_org(n):
    r = fetch(f"https://registry.npmjs.org/-/org/{n}/package")
    s = {}
    if r.status == 200 and r.json() is not None:
        s = {"packages_in_scope": len(r.json())}
    return row(n, "npm_org", r, st(r), s, why(r))


def npm_user(n):
    r = fetch(f"https://www.npmjs.com/~{n}")
    rs = fetch(f"https://registry.npmjs.org/-/v1/search?text=maintainer:{n}&size=1")
    total = (rs.json() or {}).get("total") if rs.status == 200 else None
    weak = {"url": rs.url, "queried_at": rs.at, "http_status": rs.status, "packages_maintained": total,
            "body_sha256": rs.sha}
    if r.status in (200, 404):
        return row(n, "npm_user", r, st(r), {}, None, weak_evidence=weak)
    return row(n, "npm_user", r, "unreachable", {},
               f"www.npmjs.com answered {r.status or r.err} (browser check; also for control names in R0); "
               "user existence has no public API -> HUMAN_STEP.md", weak_evidence=weak)


def dockerhub(n):
    r = fetch(f"https://hub.docker.com/v2/users/{n}/")
    s = {}
    if r.status == 200 and r.json():
        j = r.json()
        kind = "organization" if "/v2/orgs/" in r.final_url else "user"
        s = {"kind": kind, "joined": date(j.get("date_joined"))}
    valid = bool(re.fullmatch(r"[a-z0-9]{4,30}", n))
    note = why(r)
    if not valid:
        note = ((note + "; ") if note else "") + "invalid_name: Docker ID is 4-30 of [a-z0-9]"
    return row(n, "dockerhub", r, st(r), s, note, invalid_name=not valid)


def github(n):
    r = gh_get(f"users/{n}")
    s = {}
    if r.status == 200 and r.json():
        j = r.json()
        s = {"login": j.get("login"), "type": j.get("type"), "created": date(j.get("created_at")),
             "public_repos": j.get("public_repos")}
    return row(n, "github", r, st(r), s, why(r))


def ghcr(n, gh_row):
    rt = fetch(f"https://ghcr.io/token?scope=repository:{n}/rootmark:pull", ok=(200, 401, 403, 404))
    tok = (rt.json() or {}).get("token") if rt.status == 200 else None
    r = fetch(f"https://ghcr.io/v2/{n}/rootmark/tags/list",
              {"Authorization": f"Bearer {tok}"} if tok else None, ok=(200, 401, 403, 404))
    if r.status == 200:
        state = "taken"
    elif r.status in (401, 403, 404):
        state = "taken" if gh_row["state"] == "taken" else "free"
    else:
        state = "unreachable"
    s = {"public_image_rootmark": r.status == 200, "token_status": rt.status,
         "namespace_owner": gh_row["summary"] or None, "namespace_owner_from": "github row"}
    note = ("namespace belongs to the GitHub account of the same name"
            + ("; no public image <ns>/rootmark" if r.status != 200 else ""))
    return row(n, "ghcr", r, state, s, note if state != "unreachable" else why(r))


def brew(n, kind):
    r = fetch(f"https://formulae.brew.sh/api/{kind}/{n}.json")
    s = {}
    if r.status == 200 and r.json():
        j = r.json()
        s = {"desc": (j.get("desc") or "")[:120], "homepage": j.get("homepage"),
             "version": (j.get("versions") or {}).get("stable") or j.get("version")}
    return row(n, "brew_" + kind, r, st(r), s, why(r))


def brew_tap(n):
    a = gh_get(f"repos/{n}/homebrew-tap")
    b = gh_get(f"repos/{n}/homebrew-rootmark")
    states = {st(a), st(b)}
    state = "taken" if "taken" in states else ("unreachable" if "unreachable" in states else "free")
    s = {"homebrew-tap": st(a), "homebrew-rootmark": st(b)}
    return row(n, "brew_tap", a, state, s,
               "tap = GitHub repo <owner>/homebrew-<tap>; owner account: github row",
               url=f"{a.url} ; {b.url}", second={"url": b.url, "queried_at": b.at, "http_status": b.status,
                                                 "body_sha256": b.sha})


def obsidian(names):
    rp = fetch(OBS + "community-plugins.json")
    rr = fetch(OBS + "community-plugins-removed.json")
    plugins = rp.json() or [] if rp.status == 200 else None
    removed = rr.json() or [] if rr.status == 200 else None
    out = []
    for n in names:
        for col, key in (("obsidian_id", "id"), ("obsidian_name", "name")):
            if plugins is None or removed is None:
                out.append(row(n, col, rp, "unreachable", {}, f"community-plugins: {rp.status}, removed: {rr.status}"))
                continue
            f = (lambda x: x.lower()) if key == "name" else (lambda x: x)
            hit = [p for p in plugins if f(str(p.get(key, ""))) == f(n)]
            hit_r = [p for p in removed if f(str(p.get(key, ""))) == f(n)]
            s = {"matches": [{"id": p.get("id"), "name": p.get("name"), "repo": p.get("repo")} for p in hit],
                 "removed_matches": [{"id": p.get("id"), "name": p.get("name")} for p in hit_r],
                 "plugins_listed": len(plugins), "removed_listed": len(removed),
                 "removed_file": {"url": rr.url, "queried_at": rr.at, "http_status": rr.status, "body_sha256": rr.sha}}
            out.append(row(n, col, rp, "taken" if hit or hit_r else "free", s,
                           f"{key} equality ({'case-insensitive' if key == 'name' else 'exact'}) in community-plugins.json and -removed.json"))
    return out


def vscode(n):
    a = fetch(f"https://marketplace.visualstudio.com/publishers/{n}")
    b = fetch(f"https://marketplace.visualstudio.com/items?itemName={n}.rootmark")
    return [row(n, "vscode_publisher", a, st(a), {}, why(a)),
            row(n, "vscode_ext", b, st(b), {"extension_id": f"{n}.rootmark"}, why(b))]


def openvsx(n):
    a = fetch(f"https://open-vsx.org/api/{n}")
    b = fetch(f"https://open-vsx.org/api/{n}/rootmark")
    sa = {}
    if a.status == 200 and a.json():
        sa = {"extensions": len((a.json() or {}).get("extensions") or {})}
    return [row(n, "openvsx_ns", a, st(a), sa, why(a)),
            row(n, "openvsx_ext", b, st(b), {"extension_id": f"{n}.rootmark"}, why(b))]


_BOOT = None


def rdap_base(tld):
    global _BOOT
    if _BOOT is None:
        r = fetch("https://data.iana.org/rdap/dns.json")
        _BOOT = {t: s[1][0] for s in (r.json() or {}).get("services", []) for t in s[0]}
    return _BOOT.get(tld)


def domain(n, tld):
    d = f"{n}.{tld}"
    base = rdap_base(tld)
    if base:
        r = fetch(base.rstrip("/") + f"/domain/{d}")
        s = {}
        if r.status == 200 and r.json():
            j = r.json()
            ev = {e.get("eventAction"): date(e.get("eventDate")) for e in j.get("events", [])}
            reg = [e for e in j.get("entities", []) if "registrar" in (e.get("roles") or [])]
            rname = None
            if reg:
                for v in (reg[0].get("vcardArray") or [None, []])[1]:
                    if v[0] == "fn":
                        rname = v[3]
            s = {"registered": ev.get("registration"), "expires": ev.get("expiration"),
                 "registrar": rname, "status": j.get("status")}
        return row(n, f"dom_{tld}", r, st(r), s, why(r), domain=d)
    # no RDAP service in IANA's bootstrap (.io): whois, port 43
    at = now()
    try:
        p = subprocess.run(["whois", "-h", f"whois.nic.{tld}", d], capture_output=True, timeout=60)
        out = p.stdout.decode("utf-8", "replace")
    except Exception as e:
        return row(n, f"dom_{tld}", None, "unreachable", {}, f"whois failed: {type(e).__name__}",
                   url=f"whois:whois.nic.{tld} {d}", domain=d)
    if "Domain not found" in out or "NOT FOUND" in out.upper().split("\n")[0]:
        state, s = "free", {}
    elif re.search(r"^Domain Name:", out, re.M | re.I):
        g = lambda k: (re.search(rf"^{k}:\s*(\S+)", out, re.M | re.I) or [None, None])[1]
        state = "taken"
        s = {"registered": date(g("Creation Date")), "expires": date(g("Registry Expiry Date")),
             "registrar": (re.search(r"^Registrar:\s*(.+)$", out, re.M) or [None, None])[1]}
    else:
        state, s = "unreachable", {}
    r = {"name": n, "registry": f"dom_{tld}", "url": f"whois:whois.nic.{tld} {d}", "queried_at": at,
         "http_status": None, "whois_status": "found" if state == "taken" else ("not_found" if state == "free" else "unparsed"),
         "state": state, "summary": s, "body_sha256": hashlib.sha256(out.encode()).hexdigest(),
         "note": "no RDAP service for this TLD in IANA's bootstrap; whois (port 43, read-only)", "domain": d}
    return r


# --- near field ---------------------------------------------------------

def near_field():
    out = {}
    r = gh_get("search/repositories?q=rootmark+in:name&sort=stars&per_page=30")
    items = (r.json() or {}).get("items", []) if r.status == 200 else []
    out["github_repos_named_rootmark"] = {
        "url": r.url, "queried_at": r.at, "http_status": r.status, "body_sha256": r.sha,
        "total_count": (r.json() or {}).get("total_count") if r.status == 200 else None,
        "repos": [{"full_name": i["full_name"], "description": (i.get("description") or "")[:120],
                   "created": date(i["created_at"]), "pushed": date(i["pushed_at"]),
                   "stars": i["stargazers_count"], "archived": i.get("archived"), "fork": i.get("fork")}
                  for i in items]}
    sites = []
    for d in ("rootmark.com", "rootmark.dev", "rootmark.ai"):
        rr = fetch(f"https://{d}/", ok=(200, 301, 302, 403, 404))
        sites.append({"url": rr.url, "queried_at": rr.at, "http_status": rr.status, "error": rr.err,
                      "final_url": rr.final_url if rr.final_url != rr.url else None})
    out["websites"] = sites
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    root = Path(__file__).resolve().parents[2]
    commit = subprocess.run(["git", "-C", str(root), "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip()
    started = now()
    rows = []
    for n in NAMES:
        rows.append(crates(n))
        rows.append(pypi(n, "pypi", "pypi.org"))
        rows.append(pypi(n, "testpypi", "test.pypi.org"))
        rows.append(npm_pkg(n))
        rows.append(npm_org(n))
        rows.append(npm_user(n))
        rows.append(dockerhub(n))
        g = github(n)
        rows.append(ghcr(n, g))
        rows.append(g)
        rows.append(brew(n, "formula"))
        rows.append(brew(n, "cask"))
        rows.append(brew_tap(n))
        rows.extend(vscode(n))
        rows.extend(openvsx(n))
        for t in TLDS:
            rows.append(domain(n, t))
        print(f"{n}: done", file=sys.stderr)
    rows.extend(obsidian(NAMES))
    order = {c: i for i, c in enumerate(COLUMNS)}
    rows.sort(key=lambda x: (NAMES.index(x["name"]), order[x["registry"]]))
    cells = {(x["name"], x["registry"]) for x in rows}
    missing = [f"{n}×{c}" for n in NAMES for c in COLUMNS if (n, c) not in cells]
    unevidenced = [f"{x['name']}×{x['registry']}" for x in rows
                   if not x.get("url") or not x.get("queried_at")
                   or (x["state"] != "unreachable" and x.get("http_status") is None and x.get("whois_status") is None)
                   or (x["state"] == "unreachable" and not x.get("note"))]
    doc = {
        "scan": "D-0/R1 registry scan (docs/rounds/D-0/progress/DESIGN.md §1)",
        "script": "research/d0/registry_scan.py", "script_commit": commit,
        "started_at": started, "finished_at": now(), "python": sys.version.split()[0], "user_agent": UA,
        "method": "GET only (one request function); whois port 43 for TLDs without RDAP",
        "names": NAMES, "columns": COLUMNS,
        "cells_expected": len(NAMES) * len(COLUMNS), "cells": len(rows),
        "missing_cells": missing, "unevidenced_rows": unevidenced,
        "counts": {s: sum(1 for x in rows if x["state"] == s) for s in ("taken", "free", "unreachable")},
        "rows": rows, "near_field": near_field(),
    }
    (out / "registry_scan.json").write_text(json.dumps(doc, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
    write_md(doc, out / "registry_scan.md")
    print(json.dumps({k: doc[k] for k in ("cells_expected", "cells", "counts")}, ensure_ascii=False))
    print(f"missing={len(missing)} unevidenced={len(unevidenced)}")
    return 0 if not missing and not unevidenced else 1


SYM = {"taken": "dolu", "free": "boş", "unreachable": "erişilemedi"}


def write_md(doc, path):
    rows = doc["rows"]
    by = {(r["name"], r["registry"]): r for r in rows}
    L = ["# D-0 — Kayıt defteri taraması (okunur tablo)", "",
         f"Kaynak: `registry_scan.json` (betik `{doc['script']}` @ `{doc['script_commit'][:7]}`; "
         f"koşu {doc['started_at']} → {doc['finished_at']} UTC). Her hücrenin URL'si, zamanı, yanıt kodu ve "
         "gövde sağlaması JSON'dadır. Bu tablo JSON'dan üretilir; elle düzenlenmez.", "",
         f"Hücre: {doc['cells']}/{doc['cells_expected']}; eksik {len(doc['missing_cells'])}; kanıtsız {len(doc['unevidenced_rows'])}; "
         f"dolu {doc['counts']['taken']}, boş {doc['counts']['free']}, erişilemedi {doc['counts']['unreachable']}.", "",
         "## Matris", "", "| sütun | " + " | ".join(f"`{n}`" for n in doc["names"]) + " |",
         "|---|" + "---|" * len(doc["names"])]
    for c in doc["columns"]:
        cells = []
        for n in doc["names"]:
            r = by[(n, c)]
            s = SYM[r["state"]]
            if r.get("invalid_name"):
                s += " (geçersiz ad)"
            cells.append(s)
        L.append(f"| `{c}` | " + " | ".join(cells) + " |")
    L += ["", "## Dolu ve erişilemeyen hücreler", "", "| ad | sütun | UTC | kod | özet | not |", "|---|---|---|---|---|---|"]
    for r in rows:
        if r["state"] == "free":
            continue
        code = r["http_status"] if r["http_status"] is not None else r.get("whois_status")
        summ = json.dumps(r["summary"], ensure_ascii=False)
        if len(summ) > 260:
            summ = summ[:257] + "…"
        L.append(f"| `{r['name']}` | `{r['registry']}` | {r['queried_at']} | {code} | {summ.replace('|', '/')} | {(r.get('note') or '').replace('|', '/')} |")
    nf = doc["near_field"]
    g = nf["github_repos_named_rootmark"]
    L += ["", "## Yakın alan: GitHub'da adında `rootmark` geçen depolar", "",
          f"`{g['url']}` · {g['queried_at']} · {g['http_status']} · toplam {g['total_count']}", "",
          "| depo | tanıtım (ilk 120) | oluşturma | son itme | yıldız |", "|---|---|---|---|---|"]
    for i in g["repos"]:
        L.append(f"| `{i['full_name']}` | {i['description'].replace('|', '/')} | {i['created']} | {i['pushed']} | {i['stars']} |")
    L += ["", "## Yakın alan: web siteleri", "", "| URL | UTC | kod / hata | yönlenme |", "|---|---|---|---|"]
    for s in nf["websites"]:
        L.append(f"| {s['url']} | {s['queried_at']} | {s['http_status'] or s['error']} | {s['final_url'] or ''} |")
    path.write_text("\n".join(L) + "\n", encoding="utf-8")


if __name__ == "__main__":
    sys.exit(main())
