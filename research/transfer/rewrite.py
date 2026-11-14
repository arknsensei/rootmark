"""D-1 — the M (mechanical) rewrites of the transfer (DESIGN §1.3).

Every lab path whose transform type is M has a rule list here: each rule is
(old, new, count) on the file's bytes — `old` must occur exactly `count`
times, else the transfer stops (the source changed: update the rule; no
silent failure). The result is the same bytes on every run.
"""

from __future__ import annotations

LAB = b"arknsensei/rootmark"
PRODUCT = b"rootmark-ai/rootmark"

RULES: dict[str, list[tuple[bytes, bytes, int]]] = {
    # the product tests read their reference data from testdata/ (K moves)
    "index/tests/lexical.rs": [(b"research/data", b"testdata", 2)],
    "index/tests/embed.rs": [(b"research/data", b"testdata", 2)],
    "index/tests/p2_l_tokenize.rs": [(b"research/data", b"testdata", 2)],
    "server/tests/p2_b_b0.rs": [(b"research/data", b"testdata", 1)],
    "server/tests/p2_b_b1.rs": [(b"research/data", b"testdata", 1)],
    "server/tests/p2_p_channel.rs": [(b"research/data", b"testdata", 1)],
    "server/tests/p2_p_p0.rs": [(b"research/data", b"testdata", 2)],
    "server/tests/promotion_rules.rs": [(b"research/data", b"testdata", 1)],
    # the index_ort fixture's store is dot-rootmark/ in the product (no dot
    # directory in the product tree, D0-K14): the copy gets its name back
    "server/tests/p2v1_migrate.rs": [
        (
            b'        .success());\n    root.join("vault")\n}',
            (
                b"        .success());\n"
                b"    // product repository (D0-K14): the fixture's store is kept as\n"
                b"    // dot-rootmark/; the copy gets the name the server reads\n"
                b'    std::fs::rename(root.join("vault/dot-rootmark"), root.join("vault/.rootmark")).unwrap();\n'
                b'    root.join("vault")\n}'
            ),
            1,
        )
    ],
    "server/Cargo.toml": [
        (
            b'repository = "https://github.com/arknsensei/rootmark"',
            b'repository = "https://github.com/rootmark-ai/rootmark"',
            1,
        )
    ],
    "packaging/install.sh": [(LAB, PRODUCT, 2)],
    "packaging/install.ps1": [(LAB, PRODUCT, 2)],
    ".github/dependabot.yml": [
        (
            (
                b"  - package-ecosystem: uv\n"
                b"    directory: /research\n"
                b"    schedule:\n"
                b"      interval: weekly\n"
                b"    open-pull-requests-limit: 0\n"
                b"    groups:\n"
                b"      security:\n"
                b"        applies-to: security-updates\n"
                b'        patterns: ["*"]\n'
            ),
            b"",
            1,
        )
    ],
    "scripts/scene.sh": [(b"reports/rounds/P1-E/scene", b"out/scene", 2)],
    "scripts/scene_agent.sh": [(b"reports/rounds/P1-D", b"out/scene_agent", 2)],
    "scripts/plugin_e2e.sh": [(b"reports/rounds/P1-E/plugin_e2e", b"out/plugin_e2e", 2)],
}


class RewriteError(Exception):
    pass


def rewrite(path: str, data: bytes) -> bytes:
    rules = RULES.get(path)
    if rules is None:
        raise RewriteError(f"{path}: transform M but no rule in research/transfer/rewrite.py")
    for old, new, count in rules:
        n = data.count(old)
        if n != count:
            raise RewriteError(
                f"{path}: rule {old[:60]!r} matches {n} times, expected {count} "
                "(the lab file changed: update the rule)"
            )
        data = data.replace(old, new)
    return data
