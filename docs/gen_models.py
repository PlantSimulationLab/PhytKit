#!/usr/bin/env python3
"""Write docs/docs/reference/models.md from `pk describe`, so the parameter tables can't
drift from the code. Run from rust/ after `cargo build --release`:

    python3 docs/gen_models.py
"""
import json
import subprocess
from pathlib import Path

RUST = Path(__file__).resolve().parents[1]
PK = RUST / "target" / "release" / "pk"
OUT = RUST / "docs" / "docs" / "reference" / "models.md"

def pk(*args):
    return json.loads(subprocess.run([str(PK), *args], capture_output=True, text=True, check=True).stdout)


def num(v):
    if v is None:
        return "—"
    if isinstance(v, float) and (abs(v) >= 1e5 or (0 < abs(v) < 1e-3)):
        return f"{v:g}"
    return f"{v:g}"


def main():
    catalogue = pk("describe")
    tools = {m["name"]: (t["name"], m["label"]) for t in catalogue["tools"] for m in t["models"]}
    lines = [
        "# Models and parameters",
        "",
        "Every built-in model, with its inputs, parameters and default bounds. This page is",
        "generated from `pk describe` by `docs/gen_models.py`; `pk describe <model>` prints the",
        "same information as JSON.",
        "",
        "**Fixed by default** parameters are held at their default. To fit one, list it with `--fit`",
        "(or `fit_params` in Python), but `--fit` replaces the whole set of fitted parameters, so also",
        "list every other parameter you want fitted. Bounds are hard limits the fitter never crosses.",
        "",
        "| Model | Tool | Name in the tool |",
        "|---|---|---|",
    ]
    for name in catalogue["models"]:
        tool, label = tools.get(name, ("Python only", "—"))
        lines.append(f"| [`{name}`](#{name.lower()}) | {tool} | {label} |")
    for name in catalogue["models"]:
        m = pk("describe", name)["model"]
        tool, label = tools.get(name, ("Python only", None))
        lines += ["", f"## {name}", ""]
        if label:
            lines += [f"**{label}** — `pk fit {tool} <file> --model {name}`", ""]
        else:
            lines += ["Available from Python (`fit_arrays`, `fit_many`, `predict`); no `pk fit` tool uses it.", ""]
        if m["latex"]:
            lines += ["\\[", m["latex"], "\\]", ""]
        lines += ["**Inputs:** " + ", ".join(f"`{i['name']}` ({i['description']}, {i['units']})" for i in m["inputs"]), ""]
        r = m["response"]
        lines += [f"**Predicts:** `{r['name']}` ({r['description']}, {r['units']})", ""]
        lines += ["| Parameter | Units | Default | Lower bound | Upper bound | Fixed by default | Description |", "|---|---|---|---|---|---|---|"]
        for p in m["params"]:
            lines.append(f"| `{p['name']}` | {p['units']} | {num(p['default'])} | {num(p['lb'])} | {num(p['ub'])} | {'yes' if p['fixedByDefault'] else ''} | {p['description'] or ''} |")
        if m["constraints"]:
            lines += ["", "**Soft constraints** (penalties added to the fit; turn one off with `--no-constraint <key>`, all with `--constraints none`, turn on one that is off by default with `--constraint <key>`, reweight with `--weight <key>=<w>`):", ""]
            lines += ["| Key | Constraint | On by default | Default weight | Per curve | What it does |", "|---|---|---|---|---|---|"]
            for c in m["constraints"]:
                lines.append(f"| `{c['key']}` | {c['label']} | {'yes' if c.get('defaultOn', True) else 'no'} | {num(c['weight'])} | {'yes' if c['perCurve'] else ''} | {c['description'] or ''} |")
    OUT.write_text("\n".join(lines) + "\n")
    print(f"wrote {OUT.relative_to(RUST)}")


if __name__ == "__main__":
    main()
