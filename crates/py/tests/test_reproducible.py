"""What a sandboxed harness relies on: the same inputs give bit-identical parameters run to
run and at any thread count, nothing is written outside the paths given, and the build is
identified."""
import json
import os
import subprocess
import sys
from pathlib import Path

import phytkit as pk

RUST = Path(__file__).resolve().parents[3]
LEAF = RUST / "examples" / "aci.csv"


def strip_timing(x):
    if isinstance(x, dict):
        return {k: strip_timing(v) for k, v in x.items() if k != "elapsedMs"}
    if isinstance(x, list):
        return [strip_timing(v) for v in x]
    return x


def curves():
    import csv

    rows = list(csv.DictReader(open(LEAF)))
    by = {}
    for r in rows:
        by.setdefault(r["CurveID"], []).append(r)
    out = []
    for rs in by.values():
        x = {"Ci": [float(r["Ci"]) for r in rs], "Q": [float(r["Qabs"]) for r in rs], "T": [float(r["Tleaf"]) + 273.15 for r in rs]}
        out.append((x, [float(r["A"]) for r in rs]))
    return out * 4  # 24 datasets, enough to spread over 8 threads


def test_fit_is_identical_run_to_run():
    a = strip_timing(pk.fit("photosynthesis", LEAF, model="fvcb"))
    b = strip_timing(pk.fit("photosynthesis", LEAF, model="fvcb"))
    assert json.dumps(a) == json.dumps(b)


def test_fit_many_is_bit_identical_at_any_thread_count():
    data = curves()
    opts = dict(fit_params=["Vcmax25", "Jmax25", "TPU25", "Rd25"], init={"gm": 1e6})
    runs = [pk.fit_many("fvcb", data, threads=t, **opts) for t in (1, 2, 8, 8)]
    # Compare the exact bits of every float, not approximate equality.
    as_hex = lambda rs: [{k: float(v).hex() for k, v in r["params"].items()} for r in rs]
    first = as_hex(runs[0])
    for r in runs[1:]:
        assert as_hex(r) == first
    # And the same as fitting one at a time.
    one = [pk.fit_arrays("fvcb", x, y, **opts) for x, y in data]
    assert as_hex(one) == first


def test_rayon_num_threads_is_honoured_and_changes_nothing():
    code = (
        "import json, phytkit as pk\n"
        "data=[({'Q':[50.0*i for i in range(1,25)]},[20*50*i/(50*i+300)-1.5 for i in range(1,25)])]*16\n"
        "print(json.dumps([{k: float(v).hex() for k, v in r['params'].items()} for r in pk.fit_many('lightResponse', data)]))\n"
    )
    outs = set()
    for n in ("1", "3"):
        env = dict(os.environ, RAYON_NUM_THREADS=n)
        outs.add(subprocess.run([sys.executable, "-c", code], env=env, capture_output=True, text=True, check=True).stdout)
    assert len(outs) == 1


def test_writes_nothing_outside_the_given_paths(tmp_path):
    # Run with HOME and the working directory pointed at empty folders; only --out may write.
    home, work = tmp_path / "home", tmp_path / "work"
    home.mkdir(), work.mkdir()
    out = tmp_path / "corrected.csv"
    code = (
        "import phytkit as pk, warnings\n"
        "warnings.simplefilter('ignore')\n"
        f"pk.describe(); pk.fit('photosynthesis', {str(LEAF)!r})\n"
        f"pk.li600_correct({str(RUST / 'examples' / 'li600.csv')!r}, out={str(out)!r})\n"
        "pk.build_info()\n"
    )
    env = dict(os.environ, HOME=str(home), USERPROFILE=str(home), XDG_CACHE_HOME=str(home / ".cache"))
    subprocess.run([sys.executable, "-c", code], cwd=work, env=env, check=True)
    assert list(home.iterdir()) == [] and list(work.iterdir()) == []
    assert sorted(p.name for p in tmp_path.iterdir()) == ["corrected.csv", "home", "work"]


def test_build_info_identifies_the_build():
    info = pk.build_info()
    assert list(info) == ["version", "commit", "dirty", "build", "profile", "target"]
    assert info["version"] == pk.__version__
    assert info["build"] in ("release", "local")
    assert info["commit"] == "unknown" or len(info["commit"]) == 40
