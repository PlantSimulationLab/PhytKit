"""phytkit: plant physiology model fitting.

The commands run the same Rust code as the `pk` command line and return the same
structure as its JSON output, as dicts::

    import phytkit as pk
    fit = pk.fit("photosynthesis", "leaf1.csv", model="fvcb")
    fit["params"]["Vcmax25"]

For data already in memory, use `fit_arrays`, `fit_many` (parallel) and `predict`.
"""
from __future__ import annotations

import os
from typing import Any, Iterable, Mapping

from ._phytkit import PhytkitError, __version__, fit_arrays, fit_many, models, predict, run

__all__ = [
    "PhytkitError", "__version__", "concat", "describe", "fit", "fit_arrays", "fit_many",
    "li600_correct", "li600_estimate", "models", "predict", "run",
]


def _flags(options: Mapping[str, Any]) -> list[str]:
    """Keyword arguments to `pk` flags: max_iter=400 -> --max-iter 400, fix=["gm"] -> --fix gm,
    set={"gm": 1e6} -> --set gm=1000000, predicted=True -> --predicted. None is skipped."""
    out: list[str] = []
    for key, value in options.items():
        if value is None or value is False:
            continue
        flag = "--" + key.rstrip("_").replace("_", "-")
        if value is True:
            out.append(flag)
        elif isinstance(value, Mapping):
            for k, v in value.items():
                out += [flag, f"{k}={v!r}" if isinstance(v, float) else f"{k}={v}"]
        elif isinstance(value, (list, tuple)):
            # Repeatable flags (--survey, --no-constraint) take one value each.
            for v in value:
                out += [flag, os.fspath(v) if isinstance(v, os.PathLike) else str(v)]
        else:
            out += [flag, os.fspath(value) if isinstance(value, os.PathLike) else str(value)]
    return out


def _paths(files: Iterable[str | os.PathLike]) -> list[str]:
    return [os.fspath(f) for f in files]


def describe(model: str | None = None) -> dict:
    """Tools, models, parameters and constraints; or one model's details."""
    return run(["describe"] + ([model] if model else []))


def fit(tool: str, *files: str | os.PathLike, **options: Any) -> dict | str:
    """Fit a tool's model to one or more files (`pk fit`).

    tool: "photosynthesis", "stomatal", "hydraulics" or "soil".
    options: model, fit=[...], fix=[...], set={...}, no_constraint=[...], constraints="none",
    weight={...}, units, max_iter, group_by, survey=[...], stat, ref_ci, predicted, format.
    One file returns that fit; several (or group_by) return a batch summary.
    """
    for name in ("fit", "fix"):
        if isinstance(options.get(name), (list, tuple)):
            options[name] = ",".join(options[name])
    return run(["fit", tool, *_paths(files), *_flags(options)])


def li600_correct(file: str | os.PathLike, **options: Any) -> dict | str:
    """LI-600 energy-balance gsw correction (`pk li600 correct`).
    options: sidedness, thermal_conductance, rows, out, format."""
    return run(["li600", "correct", os.fspath(file), *_flags(options)])


def li600_estimate(file: str | os.PathLike, **options: Any) -> dict | str:
    """A, Ci and Cc from LI-600 gsw and photosynthesis parameters (`pk li600 estimate`).
    options: model, params (a file), set={...}, ca, absorptance, min_gsw, rows, out, format."""
    return run(["li600", "estimate", os.fspath(file), *_flags(options)])


def concat(*files: str | os.PathLike, **options: Any) -> dict | str:
    """Merge LI-6800 logs or CSVs under one header with a CurveID per file (`pk concat`).
    options: out, format. Returns the merged CSV text unless out or format="json" is given."""
    return run(["concat", *_paths(files), *_flags(options)])
