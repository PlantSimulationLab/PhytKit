"""Keeps the agent skill shipped in the wheel honest: well formed, every reference linked,
and every command, option, model, parameter, error code and warning code it names exists in
the installed package (`pk describe`)."""
import inspect
import re
from pathlib import Path

import pytest

import phytkit as pk

SRC = Path(__file__).resolve().parents[1] / "python" / "phytkit" / "skills"
SKILL = SRC / "phytkit" / "SKILL.md"
REFS = SKILL.parent / "references"


def docs():
    return [SKILL, *sorted(REFS.glob("*.md"))]


def text(p):
    return p.read_text(encoding="utf-8")


def frontmatter(s):
    m = re.match(r"^---\n(.*?)\n---\n", s, re.S)
    assert m, "SKILL.md must start with YAML frontmatter"
    out = {}
    for line in m.group(1).splitlines():
        k, _, v = line.partition(":")
        out[k.strip()] = v.strip()
    return out


@pytest.fixture(scope="module")
def catalogue():
    d = pk.describe()
    models = {m: pk.describe(m)["model"] for m in d["models"]}
    options = {o["name"] for c in d["commands"] for o in c["options"]}
    return {
        "describe": d,
        "models": models,
        "params": {p["name"] for m in models.values() for p in m["params"]},
        "options": options,
        "commands": {c["name"] for c in d["commands"]},
        "tools": {t["name"] for t in d["tools"]},
        "errors": {c["code"] for c in d["errors"]["codes"]},
        "warnings": {c["code"] for c in d["warnings"]["codes"]},
        "constraints": {c["key"] for m in models.values() for c in m["constraints"]},
    }


def test_frontmatter_and_length():
    s = text(SKILL)
    fm = frontmatter(s)
    assert fm["name"] == SKILL.parent.name
    assert re.fullmatch(r"[a-z0-9-]{1,64}", fm["name"])
    assert 1 <= len(fm["description"]) <= 1024
    assert len(s.splitlines()) <= 150, "keep SKILL.md to the decision-making core; move detail to references/"


def test_every_reference_is_linked_and_every_link_resolves():
    linked = set()
    for d in docs():
        for target in re.findall(r"\]\(([^)#]+\.md)\)", text(d)):
            path = (d.parent / target).resolve()
            assert path.exists(), f"{d.name}: broken link {target}"
            linked.add(path)
    assert {p.resolve() for p in REFS.glob("*.md")} <= linked
    # Every reference names the skill it belongs to, so a reader who lands on it knows.
    for r in REFS.glob("*.md"):
        assert "Part of the `phytkit` skill" in text(r), r.name


def test_commands_tools_and_options_exist(catalogue):
    python_side = {"agent-files", "skills"}
    for d in docs():
        s = text(d)
        for opt in set(re.findall(r"(?<![\w-])--[a-z][a-z0-9-]*", s)):
            assert opt in catalogue["options"] | {"--help"}, f"{d.name}: unknown option {opt}"
        # Commands as written in code: at the start of a code line, or right after a backtick.
        for cmd in re.findall(r"(?:^|`)pk ((?:li600 )?[a-z][a-z0-9-]*)", s, re.M):
            first = cmd.split()[0]
            assert cmd in catalogue["commands"] or first in python_side or cmd == "li600", f"{d.name}: unknown command pk {cmd}"
        for tool in re.findall(r"(?:^|`)pk fit ([a-z]+)", s, re.M):
            assert tool in catalogue["tools"] or tool == "--help", f"{d.name}: unknown tool {tool}"


def test_models_parameters_and_codes_exist(catalogue):
    for d in docs():
        s = text(d)
        for m in re.findall(r"--model ([A-Za-z0-9]+)|model=\"([A-Za-z0-9]+)\"|pk\.(?:fit_arrays|fit_many|describe|predict)\(\"([A-Za-z0-9]+)\"", s):
            name = next(x for x in m if x)
            assert name in catalogue["models"], f"{d.name}: unknown model {name}"
        for names in re.findall(r"--(?:fit|fix) ([A-Za-z0-9_,]+)", s):
            for n in names.split(","):
                assert n in catalogue["params"], f"{d.name}: unknown parameter {n}"
        for n in re.findall(r"--set ([A-Za-z0-9_]+)=", s):
            assert n in catalogue["params"], f"{d.name}: unknown parameter {n}"
        for lst in re.findall(r"fit_params=\[([^\]]*)\]", s):
            for n in re.findall(r"\"([^\"]+)\"", lst):
                assert n in catalogue["params"], f"{d.name}: unknown parameter {n}"
        for n in re.findall(r"init=\{\"([A-Za-z0-9_]+)\"", s):
            assert n in catalogue["params"], f"{d.name}: unknown parameter {n}"


# Backticked names with an underscore must be something the package knows: an error or
# warning code, a parameter, a Python name or keyword, an input column, or an output field.
COLUMNS = {
    "gsw_corrected", "E_apparent", "rh_r", "rh_s", "P_atm", "E_corrected", "T_in_corrected",
    "Ta_chamb_corrected", "T_out_corrected", "W_chamb_corrected", "stomatal_sidedness",
    "A_solved", "Ci_solved", "Cc_solved", "gsw_used", "psi_tlp", "h_cm", "BMFcoefficients",
    "MOPTcoefficients",
}
ENV = {"RAYON_NUM_THREADS", "PHYTKIT_AGENT_FILES"}
PY_KEYWORDS = {"fit_params", "max_iter", "group_by", "ci_based", "constraint_weights"}


def test_backticked_names_exist(catalogue):
    py = set(dir(pk))
    known = (catalogue["errors"] | catalogue["warnings"] | catalogue["params"] | catalogue["constraints"]
             | py | PY_KEYWORDS | COLUMNS | ENV)
    for d in docs():
        for tok in re.findall(r"`([A-Za-z][A-Za-z0-9]*_[A-Za-z0-9_*]+)`", text(d)):
            if "*" in tok:  # a pattern such as Jmax_* or *_dHa
                continue
            assert tok in known, f"{d.name}: `{tok}` is not a code, parameter or name phytkit knows"


def test_python_calls_bind_to_real_functions(catalogue):
    fit_opts = {o["name"][2:].replace("-", "_") for c in catalogue["describe"]["commands"] if c["name"] == "fit" for o in c["options"]}
    for d in docs():
        s = text(d)
        for fn in re.findall(r"\bpk\.([a-z_]+)\(", s):
            assert hasattr(pk, fn), f"{d.name}: pk.{fn} does not exist"
        for call in re.findall(r"pk\.fit\(([^)]*)\)", s):
            for kw in re.findall(r"\b([a-z_]+)=", call):
                assert kw in fit_opts, f"{d.name}: pk.fit has no keyword {kw}"
        for fn in ("fit_arrays", "fit_many"):
            params = set(inspect.signature(getattr(pk._phytkit, fn)).parameters) if hasattr(getattr(pk._phytkit, fn), "__text_signature__") else None
            if not params:
                continue
            for call in re.findall(rf"pk\.{fn}\((.*?)\)\n", s, re.S):
                for kw in re.findall(r"\b([a-z_]+)=", call):
                    assert kw in params, f"{d.name}: pk.{fn} has no keyword {kw}"


def test_installed_skill_matches_the_source():
    # In CI the package is installed from the built wheel, so this checks the wheel ships every
    # skill file unchanged. With `maturin develop` it compares the source with itself.
    installed = pk.skills_dir()
    for f in SRC.rglob("*"):
        if f.is_file() and "__pycache__" not in f.parts and not f.name.startswith("."):
            twin = installed / f.relative_to(SRC)
            assert twin.is_file(), f"{twin} is missing from the installed package"
            assert twin.read_bytes() == f.read_bytes(), f"{twin} differs from the source"
