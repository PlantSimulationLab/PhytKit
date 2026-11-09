"""Tests for the Python bindings, checked against the `pk` binary, which runs the same
command layer, and against the bundled example data in rust/examples/."""
import glob
import json
import math
import os
import shlex
import subprocess
import sys
from pathlib import Path

import pytest

import phytkit as pk

RUST = Path(__file__).resolve().parents[3]
PK = RUST / "target" / "release" / "pk"
LEAF = RUST / "examples" / "aci.csv"
LI600 = RUST / "examples" / "li600.csv"


def cases():
    for raw in (Path(__file__).parent / "cli_cases.txt").read_text().splitlines():
        line = raw.split(" #")[0].strip()
        if not line or line.startswith("#"):
            continue
        args = [str(RUST / a) if a.startswith("examples/") else a for a in shlex.split(line)]
        yield pytest.param(args, id=line[:70])


def strip_timing(x):
    if isinstance(x, dict):
        return {k: strip_timing(v) for k, v in x.items() if k != "elapsedMs"}
    if isinstance(x, list):
        return [strip_timing(v) for v in x]
    return x


def drop_timing_column(text):
    lines = text.split("\n")
    head = lines[0].split(",")
    if "elapsed_ms" not in head:
        return text
    j = head.index("elapsed_ms")
    return "\n".join(",".join(c for i, c in enumerate(line.split(",")) if i != j) for line in lines)


@pytest.mark.skipif(not PK.exists(), reason="build the Rust pk first: cargo build --release")
@pytest.mark.parametrize("args", list(cases()))
def test_run_matches_the_pk_binary(args):
    proc = subprocess.run([str(PK), *args], capture_output=True, text=True)
    try:
        got = pk.run(args)
    except pk.PhytKitError as e:
        want = json.loads(proc.stdout)
        fields = {"code": e.code, "message": str(e), "usage": e.usage, "hint": e.hint, "help": e.help}
        assert proc.returncode == 1 and want["error"] == {k: v for k, v in fields.items() if v is not None}
        return
    assert proc.returncode == 0
    if isinstance(got, str):
        assert drop_timing_column(got.rstrip("\n")) == drop_timing_column(proc.stdout.rstrip("\n"))
    else:
        assert strip_timing(got) == strip_timing(json.loads(proc.stdout))


def test_fit_arrays_matches_the_file_command():
    from_file = pk.fit("photosynthesis", LEAF, model="fvcb", predicted=True)
    import csv

    rows = [r for r in csv.DictReader(open(LEAF))]
    keep = [r for r in rows if all(r[k] not in ("", None) and math.isfinite(float(r[k])) for k in ("A", "Ci", "Qabs", "Tleaf"))]
    x = {"Ci": [float(r["Ci"]) for r in keep], "Q": [float(r["Qabs"]) for r in keep], "T": [float(r["Tleaf"]) + 273.15 for r in keep]}
    y = [float(r["A"]) for r in keep]
    # The file command groups rows by CurveID for FvCB's per-curve constraints; do the same.
    groups = {}
    for i, r in enumerate(keep):
        groups.setdefault(r["CurveID"], []).append(i)
    res = pk.fit_arrays("fvcb", x, y, curves=list(groups.values()))
    assert res["iterations"] == from_file["iterations"]
    for k, v in from_file["params"].items():
        assert res["params"][k] == pytest.approx(v, abs=5e-7)
    assert res["predicted"] == pytest.approx(from_file["predicted"], abs=1e-5)


def synthetic_curves(n):
    """Light responses from the lightResponse model plus a small deterministic wobble."""
    q = [50.0 * i for i in range(1, 25)]
    out = []
    for k in range(n):
        amax, kq = 20 + k % 7, 200 + 13 * (k % 11)
        y = [amax * qi / (qi + kq) - 1.5 + 0.2 * math.sin(i + k) for i, qi in enumerate(q)]
        out.append(({"Q": q}, y))
    return out


def test_fit_many_matches_fitting_one_at_a_time():
    data = synthetic_curves(40)
    many = pk.fit_many("lightResponse", data)
    one = [pk.fit_arrays("lightResponse", x, y) for x, y in data]
    assert many == one
    assert pk.fit_many("lightResponse", data, threads=1) == one


def test_predict_uses_defaults_for_missing_parameters():
    q = {"Q": [0.0, 300.0, 1e9]}
    got = pk.predict("lightResponse", {"Amax": 20}, q)
    assert got == pytest.approx([-1.5, 20 * 300 / 600 - 1.5, 20 - 1.5], rel=1e-6)


def raises(code, fn, *args, **kwargs):
    with pytest.raises(pk.PhytKitError) as e:
        fn(*args, **kwargs)
    assert e.value.code == code, (e.value.code, str(e.value))
    return e.value


def test_bad_input_raises_clear_errors():
    assert "Did you mean lightResponse?" in raises("unknown_model", pk.fit_arrays, "lightRespons", {"Q": [1.0]}, [1.0]).hint
    assert "Rename \"q\" to \"Q\"" in raises("missing_input", pk.fit_arrays, "lightResponse", {"q": [1.0]}, [1.0]).hint
    raises("invalid_value", pk.fit_arrays, "lightResponse", {"Q": [1.0, 2.0]}, [1.0])
    assert "Did you mean Amax?" in raises("unknown_parameter", pk.fit_arrays, "lightResponse", {"Q": [1.0]}, [1.0], init={"amax": 5.0}).hint
    raises("unknown_parameter", pk.fit_many, "lightResponse", [({"Q": [1.0]}, [1.0])], fit_params=["Kq", "nope"])
    raises("unknown_parameter", pk.predict, "lightResponse", {"Vcmax25": 1.0}, {"Q": [1.0]})
    raises("missing_input", pk.predict, "lightResponse", {}, {"Ci": [1.0]})
    e = raises("unknown_model", pk.describe, "nope")
    assert e.usage


def test_unknown_keyword_options_are_errors():
    e = raises("unknown_option", pk.li600_correct, LI600, sidednes=2)
    assert e.hint.startswith("Did you mean --sidedness?") and e.help.startswith("pk li600 correct")
    raises("invalid_value", pk.li600_estimate, LI600, ca=-10)
    raises("unknown_option", pk.fit, "photosynthesis", LEAF, rows=True)


def test_module_entry_point_prints_what_the_binary_prints():
    for args in (["li600", "correct", str(LI600), "--sidednes", "2"], ["describe", "fvcb"], ["fit", "--help"]):
        binary = subprocess.run([str(PK), *args], capture_output=True, text=True)
        module = subprocess.run([sys.executable, "-m", "phytkit", *args], capture_output=True, text=True)
        assert (module.stdout, module.returncode) == (binary.stdout, binary.returncode)


def test_keyword_options_become_pk_flags():
    from phytkit import _flags

    assert _flags({"max_iter": 400, "set": {"gm": 1e6, "Rd25": 1}, "survey": ["a.csv", "b.csv"], "predicted": True, "model": None, "rows": False}) == [
        "--max-iter", "400", "--set", "gm=1000000.0", "--set", "Rd25=1", "--survey", "a.csv", "--survey", "b.csv", "--predicted"]


def test_keyword_options_reach_the_fit():
    res = pk.fit("photosynthesis", LEAF, model="fvcb", fix=["gm", "alphaG"], set={"Vcmax25": 90.0}, max_iter=3)
    assert res["iterations"] <= 3 and "gm" not in res["fitted"] and "alphaG" not in res["fitted"]
