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

# Per-model input descriptions that differ from INPUTS.
INPUT_OVERRIDES = {
    ("bta", "Q"): "light (PPFD), µmol m⁻² s⁻¹; `pk fit stomatal` prefers `Qin` (incident) over `Qabs`, see Input files",
}

INPUTS = {
    "Ci": "intercellular CO₂, µmol mol⁻¹",
    "Q": "absorbed light (PPFD), µmol m⁻² s⁻¹",
    "T": "leaf temperature, K",
    "Ds": "leaf-to-air vapour pressure deficit as a mole fraction, mmol mol⁻¹",
    "A": "net CO₂ assimilation, µmol m⁻² s⁻¹",
    "Cs": "CO₂ at the leaf surface, µmol mol⁻¹",
    "D": "leaf-to-air vapour pressure deficit, kPa",
    "P": "water potential, MPa (negative)",
    "w": "relative water content, 0–1",
    "psi": "matric potential, MPa (negative)",
}


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
        lines += ["**Inputs:** " + ", ".join(f"`{i}` ({INPUT_OVERRIDES.get((m['name'], i), INPUTS.get(i, ''))})" for i in m["inputs"]), ""]
        lines += ["| Parameter | Default | Lower bound | Upper bound | Fixed by default | Description |", "|---|---|---|---|---|---|"]
        for p in m["params"]:
            lines.append(f"| `{p['name']}` | {num(p['default'])} | {num(p['lb'])} | {num(p['ub'])} | {'yes' if p['fixedByDefault'] else ''} | {p['description'] or ''} |")
        if m["constraints"]:
            lines += ["", "**Soft constraints** (penalties added to the fit; turn one off with `--no-constraint <key>`, all with `--constraints none`, turn on one that is off by default with `--constraint <key>`, reweight with `--weight <key>=<w>`):", ""]
            lines += ["| Key | Constraint | On by default | Default weight | Per curve | What it does |", "|---|---|---|---|---|---|"]
            for c in m["constraints"]:
                lines.append(f"| `{c['key']}` | {c['label']} | {'yes' if c.get('defaultOn', True) else 'no'} | {num(c['weight'])} | {'yes' if c['perCurve'] else ''} | {c['description'] or ''} |")
    OUT.write_text("\n".join(lines) + "\n")
    print(f"wrote {OUT.relative_to(RUST)}")


if __name__ == "__main__":
    main()
