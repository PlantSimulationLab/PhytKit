"""Warnings are {code, level, message} objects with stable codes; warning-level entries are
also raised as PhytKitWarning so a harness reading stderr sees them."""
import subprocess
import sys
import warnings
from pathlib import Path

import pytest

import phytkit as pk

RUST = Path(__file__).resolve().parents[3]
LEAF = RUST / "examples" / "aci.csv"


def codes():
    return {c["code"]: c["level"] for c in pk.describe()["warnings"]["codes"]}


def test_every_warning_is_a_coded_object():
    known = codes()
    res = pk.fit("photosynthesis", LEAF, model="fvcb")
    assert res["warnings"], "the example always reports its column mapping"
    for w in res["warnings"]:
        assert list(w) == ["code", "level", "message"]
        assert known[w["code"]] == w["level"]
    assert res["warnings"][0]["code"] == "columns_mapped"


def test_warning_level_entries_raise_phytkitwarning_and_info_entries_do_not():
    with warnings.catch_warnings(record=True) as caught:
        warnings.simplefilter("always")
        res = pk.fit("photosynthesis", LEAF, model="fvcb")
    raised = [str(w.message) for w in caught if issubclass(w.category, pk.PhytKitWarning)]
    expected = [w for w in res["warnings"] if w["level"] == "warning"]
    assert len(raised) == len(expected) > 0
    for msg, w in zip(raised, expected):
        assert msg.startswith(f"[{w['code']}] ")
        assert w["message"] in msg
    # The warning points at the caller, not at phytkit's own code.
    assert all(Path(w.filename).name == Path(__file__).name for w in caught)


def test_fit_arrays_reports_parameters_on_a_bound():
    q = [50.0 * i for i in range(1, 25)]
    y = [20 * qi / (qi + 300) - 0.0 for qi in q]  # Rd = 0, the lower bound
    with pytest.warns(pk.PhytKitWarning, match=r"^\[parameter_at_bound\] .*Rd"):
        res = pk.fit_arrays("lightResponse", {"Q": q}, y)
    assert any(w["code"] == "parameter_at_bound" for w in res["warnings"])


def test_not_converged_is_a_warning():
    with pytest.warns(pk.PhytKitWarning, match=r"^\[not_converged\]"):
        res = pk.fit("photosynthesis", LEAF, model="fvcb", max_iter=1)
    assert not res["converged"]


def test_python_warning_line_matches_the_format_harnesses_parse():
    # Heliades reads `<file>:<line>: <Class>Warning: <message>` from stderr.
    code = (
        "import phytkit as pk, sys\n"
        f"pk.fit('photosynthesis', {str(LEAF)!r}, model='fvcb', max_iter=1)\n"
    )
    err = subprocess.run([sys.executable, "-c", code], capture_output=True, text=True).stderr
    import re

    pat = re.compile(r"^(?P<loc>.+?):\d+:\s*(?P<cls>\w*Warning):\s*(?P<msg>.*)$", re.M)
    found = [m.groupdict() for m in pat.finditer(err)]
    assert any(f["cls"] == "PhytKitWarning" and f["msg"].startswith("[not_converged]") for f in found), err


def test_single_temperature_fit_of_temperature_parameters_is_flagged():
    import csv

    rows = [r for r in csv.DictReader(open(LEAF)) if r["CurveID"] == "1"]
    x = {"Ci": [float(r["Ci"]) for r in rows], "Q": [float(r["Qabs"]) for r in rows], "T": [float(r["Tleaf"]) + 273.15 for r in rows]}
    y = [float(r["A"]) for r in rows]
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        base = ["Vcmax25", "Jmax25", "TPU25", "Rd25", "alpha", "theta"]
        free = pk.fit_arrays("fvcb", x, y, init={"gm": 1e6}, fit_params=base + ["Vcmax_dHa", "Jmax_dHa"])
        held = pk.fit_arrays("fvcb", x, y, init={"gm": 1e6}, fit_params=base)
    assert "temperature_response_unconstrained" in [w["code"] for w in free["warnings"]]
    assert "temperature_response_unconstrained" not in [w["code"] for w in held["warnings"]]
