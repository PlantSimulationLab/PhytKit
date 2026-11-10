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
import warnings as _warnings
from pathlib import Path as _Path
from typing import Any, Iterable, Mapping

from . import _phytkit
from ._phytkit import PhytKitError, __version__, build_info, models, predict

__all__ = [
    "PhytKitError", "PhytKitWarning", "__version__", "agent_files", "build_info", "concat", "describe", "fit", "fit_arrays",
    "fit_many", "li600_correct", "li600_estimate", "models", "predict", "run", "skills_dir",
]


def skills_dir() -> _Path:
    """The folder of agent skills shipped with this version of phytkit.

    `skills_dir() / "phytkit" / "SKILL.md"` is the guide for AI coding agents, with its
    `references/` next to it. It always matches the installed version; copy it into a
    harness's skills folder with `pk skills install`.
    """
    return _Path(__file__).resolve().parent / "skills"


_phytkit._set_skills_dir(str(skills_dir()))


def agent_files(dir: str | os.PathLike = ".", *, dry_run: bool = False) -> dict:
    """Write AGENTS.md and CLAUDE.md with PhytKit's agent guidance into a project
    (`pk agent-files`).

    A file is rewritten only while its first line is the phytkit marker; delete that line
    to keep your own edits. PHYTKIT_AGENT_FILES=0 turns it off. Returns each file's status:
    "written", "would be written" (dry_run), "unchanged" or "user-owned".
    """
    return run(["agent-files", os.fspath(dir), *(["--dry-run"] if dry_run else [])])


class PhytKitWarning(UserWarning):
    """A result-level warning from phytkit: something that may make the result wrong (rows
    dropped, a unit guessed from the values, a parameter on its bound, no convergence).

    Raised through the `warnings` module for every entry whose level is "warning"; entries
    whose level is "info" (columns used, documented conversions) are only in the result's
    `warnings` list. The message starts with the entry's code in brackets, for example
    "[parameter_at_bound] gm ended on its upper bound (5): ...". Codes are listed in
    `describe()["warnings"]`.
    """


def _emit(result: Any, stacklevel: int = 3) -> Any:
    """Raise PhytKitWarning for each level-"warning" entry in a command's result (a fit, a
    batch of fits, or an li600/concat result). Returns the result unchanged."""
    def entries(obj: Any, where: str | None):
        if isinstance(obj, dict):
            for w in obj.get("warnings") or ():
                if isinstance(w, dict) and w.get("level") == "warning":
                    yield where or obj.get("file"), w
            for r in obj.get("results") or ():
                yield from entries(r, None)
    if isinstance(result, dict):
        for where, w in entries(result, None):
            prefix = f"{where}: " if where else ""
            _warnings.warn(f"[{w['code']}] {prefix}{w['message']}", PhytKitWarning, stacklevel=stacklevel)
    elif isinstance(result, list):
        for i, r in enumerate(result):
            for w in (r.get("warnings") or ()) if isinstance(r, dict) else ():
                if w.get("level") == "warning":
                    _warnings.warn(f"[{w['code']}] datasets[{i}]: {w['message']}", PhytKitWarning, stacklevel=stacklevel)
    return result


def run(argv: list[str]) -> dict | str:
    """Run a `pk` command (arguments as on the command line, without `pk`).

    Returns a dict for JSON output or a str for CSV and help text; raises PhytKitError on
    failure. Each level-"warning" entry in the result is also raised as a PhytKitWarning.
    """
    return _emit(_phytkit.run(list(argv)))


def fit_arrays(model_name: str, x: Mapping[str, Any], y: Any, **options: Any) -> dict:
    """Fit a model to data in memory. See `_phytkit.fit_arrays` for the arguments.

    x maps each of the model's inputs (see `describe(model)`) to a sequence of values;
    temperatures for the photosynthesis models are in kelvin. Returns params, fitted,
    predicted, rmse, r2, iterations, converged and warnings; each level-"warning" entry is
    also raised as a PhytKitWarning.
    """
    return _emit(_phytkit.fit_arrays(model_name, x, y, **options))


def fit_many(model_name: str, datasets: Any, **options: Any) -> list[dict]:
    """Fit the same model to many (x, y) datasets in parallel; results keep the input order.
    See `_phytkit.fit_many`. threads=None uses RAYON_NUM_THREADS, else every core."""
    return _emit(_phytkit.fit_many(model_name, datasets, **options))


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
                # repr(float(v)) keeps full precision; float() also turns NumPy scalars,
                # whose repr is "np.float64(90.0)", into plain floats.
                out += [flag, f"{k}={float(v)!r}" if isinstance(v, float) else f"{k}={v}"]
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
    return _emit(_phytkit.run(["fit", tool, *_paths(files), *_flags(options)]))


def li600_correct(file: str | os.PathLike, **options: Any) -> dict | str:
    """LI-600 energy-balance gsw correction (`pk li600 correct`).
    options: sidedness, thermal_conductance, rows, out, format."""
    return _emit(_phytkit.run(["li600", "correct", os.fspath(file), *_flags(options)]))


def li600_estimate(file: str | os.PathLike, **options: Any) -> dict | str:
    """A, Ci and Cc from LI-600 gsw and photosynthesis parameters (`pk li600 estimate`).
    options: model, params (a file), set={...}, ca, absorptance, min_gsw, rows, out, format."""
    return _emit(_phytkit.run(["li600", "estimate", os.fspath(file), *_flags(options)]))


def concat(*files: str | os.PathLike, **options: Any) -> dict | str:
    """Merge LI-6800 logs or CSVs under one header with a CurveID per file (`pk concat`).
    options: out, format. Returns the merged CSV text unless out or format="json" is given."""
    return _emit(_phytkit.run(["concat", *_paths(files), *_flags(options)]))
