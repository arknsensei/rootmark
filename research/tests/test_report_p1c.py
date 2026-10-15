"""Decision rule of P1-C (GOAL "Karar kuralı") as derived by eval.report_p1c."""

from eval.report_p1c import decision

IDS = [f"C-{i}" for i in range(1, 14)]


def rows(fail: set[str]) -> list[dict]:
    return [{"id": i, "pass": i not in fail} for i in IDS]


def test_all_pass_validates_section_9():
    out = " ".join(decision(rows(set())))
    assert "Validated" in out and "P1-D başlar" in out


def test_c3_failure_starts_p1d_but_holds_release():
    out = " ".join(decision(rows({"C-3"})))
    assert "P1-D başlar" in out and "0.1.0 yayımı bekler" in out
    assert "Draft" in out


def test_determinism_failure_stops_p1d():
    for c in ("C-4", "C-5", "C-6"):
        out = " ".join(decision(rows({c})))
        assert "P1-D başlamaz" in out and "başlar**" not in out.replace("başlamaz**", "")


def test_governance_failure_stops_p1d_even_with_c3():
    out = " ".join(decision(rows({"C-7", "C-3"})))
    assert "yönetişim" in out and "**başlamaz**" in out


def test_unnamed_failure_assumes_no_branch():
    out = " ".join(decision(rows({"C-8"})))
    assert "adlandırmaz" in out and "P1-D başlar" not in out
