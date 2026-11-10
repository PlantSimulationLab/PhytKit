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
from typing import Any, Iterable, Mapping, Union

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


def fit_arrays(
    model_name: str,
    x: Mapping[str, Any],
    y: Any,
    *,
    fit_params: Iterable[str] | None = None,
    init: Mapping[str, float] | None = None,
    max_iter: int | None = None,
    curves: Iterable[Iterable[int]] | None = None,
    constraints: Iterable[str] | None = None,
    constraint_weights: Mapping[str, float] | None = None,
) -> dict:
    """Fit a model to data in memory.

    x maps each of the model's inputs (see `describe(model)["inputs"]`, with units) to a
    sequence of values; leaf temperature T is in kelvin. y is the measured response.
    fit_params: exactly the parameters to fit (default: those not fixed by default).
    init: start values (within bounds for fitted parameters) or the values fixed parameters
    are held at. max_iter: iteration limit (default 200). curves: 0-based row indices per
    curve, for per-curve soft constraints. constraints: exactly the soft constraints to apply.
    constraint_weights: {key: weight}.

    Returns params, fitted, predicted, rmse, r2, iterations, converged and warnings; each
    level-"warning" entry is also raised as a PhytKitWarning.
    """
    return _emit(_phytkit.fit_arrays(
        model_name, x, y, fit_params=fit_params, init=init, max_iter=max_iter, curves=curves,
        constraints=constraints, constraint_weights=constraint_weights))


def fit_many(
    model_name: str,
    datasets: Iterable[tuple[Mapping[str, Any], Any]],
    *,
    fit_params: Iterable[str] | None = None,
    init: Mapping[str, float] | None = None,
    max_iter: int | None = None,
    constraints: Iterable[str] | None = None,
    constraint_weights: Mapping[str, float] | None = None,
    threads: int | None = None,
) -> list[dict]:
    """Fit the same model to many (x, y) datasets in parallel; results keep the input order.

    Arguments as for fit_arrays. threads: worker threads; None uses RAYON_NUM_THREADS, else
    every core. Results are bit-identical at any thread count.
    """
    return _emit(_phytkit.fit_many(
        model_name, list(datasets), fit_params=fit_params, init=init, max_iter=max_iter,
        constraints=constraints, constraint_weights=constraint_weights, threads=threads))


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


# A file or folder path. typing.Union, not `|`: this runs on Python 3.9.
Path = Union[str, os.PathLike]


def _options(values: dict, extra: Mapping[str, Any]) -> list[str]:
    """Named keyword arguments (in the order of the COMMANDS table) plus any others, as flags.
    Unknown names reach the command layer, which rejects them with a suggestion."""
    out = {k: v for k, v in values.items() if v is not None and v is not False}
    for name in ("fit", "fix"):
        if isinstance(out.get(name), (list, tuple)):
            out[name] = ",".join(out[name])
    return _flags({**out, **extra})


def fit(
    tool: str,
    *files: Path,
    model: str | None = None,
    fit: str | Iterable[str] | None = None,
    fix: str | Iterable[str] | None = None,
    set: Mapping[str, float] | None = None,
    ci_based: bool = False,
    max_iter: int | None = None,
    no_constraint: Iterable[str] | None = None,
    constraint: Iterable[str] | None = None,
    constraints: str | None = None,
    weight: Mapping[str, float] | None = None,
    units: str | None = None,
    group_by: str | None = None,
    survey: Path | Iterable[Path] | None = None,
    stat: str | None = None,
    ref_ci: float | None = None,
    predicted: bool = False,
    format: str | None = None,
    **options: Any,
) -> dict | str:
    """Fit a tool's model to one or more files (`pk fit`).

    tool: "photosynthesis", "stomatal", "hydraulics" or "soil" (`describe()["tools"]`).
    One file returns that fit; several files, or group_by, return a batch summary.
    None or False leaves an option out, so the command's default applies.

    model: the model to fit (default: the tool's first model that reads the file).
    fit: exactly the parameters to fit; replaces the default set.
    fix: parameters to hold at their start values.
    set: {name: value} start values (within bounds for fitted parameters), or the values
        fixed parameters are held at.
    ci_based: fvcb only; gm at infinity (Cc = Ci) and alphaG at 0, the form Helios uses.
    max_iter: iteration limit (default 200).
    no_constraint: soft constraints to turn off. constraint: ones to turn on.
    constraints: "none" turns every soft constraint off. weight: {key: weight}.
    units: soil ψ units, "MPa" (default), "kPa", "hPa", "bar", "m" or "cm".
    group_by: column whose values split one file into separate fits.
    survey: survey file(s) to rescale photosynthesis curves to; stat: "mean" (default),
        "median" or "max"; ref_ci: reference Ci for rescaling.
    predicted: add point-by-point predicted and observed values.
    format: "json" (default) or "csv".

    Every result's "warnings" are {code, level, message}; level-"warning" entries are also
    raised as PhytKitWarning.
    """
    values = dict(locals())
    for k in ("tool", "files", "options"):
        values.pop(k)
    return _emit(_phytkit.run(["fit", tool, *_paths(files), *_options(values, options)]))


def li600_correct(
    file: Path,
    *,
    sidedness: float | None = None,
    thermal_conductance: float | None = None,
    rows: bool = False,
    out: Path | None = None,
    format: str | None = None,
    **options: Any,
) -> dict | str:
    """LI-600 energy-balance gsw correction (`pk li600 correct`).

    sidedness: 1 (default) for stomata on one side of the leaf, 2 for both.
    thermal_conductance: instrument heat exchange, W °C⁻¹ (default 0.007).
    rows: add each reading's corrected values. out: also write the input file with the
    corrected columns added. format: "json" (default) or "csv".
    """
    values = dict(locals())
    for k in ("file", "options"):
        values.pop(k)
    return _emit(_phytkit.run(["li600", "correct", os.fspath(file), *_options(values, options)]))


def li600_estimate(
    file: Path,
    *,
    model: str | None = None,
    params: Path | None = None,
    set: Mapping[str, float] | None = None,
    ca: float | None = None,
    absorptance: float | None = None,
    min_gsw: float | None = None,
    rows: bool = False,
    out: Path | None = None,
    format: str | None = None,
    **options: Any,
) -> dict | str:
    """A, Ci and Cc for each LI-600 reading from its gsw and photosynthesis parameters
    (`pk li600 estimate`).

    model: "fvcb", "fvcbC4" or "empiricalA" (default: the params fit's model, else fvcb).
    params: a parameter file (a pk fit JSON result, a CSV, a Parameter/Value table, or
    name = value lines). set: {name: value} overrides. ca: ambient CO₂ (default 420).
    absorptance: applied to Qamb (default 0.85). min_gsw: conductance floor (default 0.001;
    0 leaves lower readings unsolved). rows: add each reading's solution. out: also write
    the input file with the solved columns added. format: "json" (default) or "csv".
    """
    values = dict(locals())
    for k in ("file", "options"):
        values.pop(k)
    return _emit(_phytkit.run(["li600", "estimate", os.fspath(file), *_options(values, options)]))


def concat(*files: Path, out: Path | None = None, format: str | None = None, **options: Any) -> dict | str:
    """Merge LI-6800 logs or CSVs under one header with a CurveID per file (`pk concat`).

    out: write the merged CSV to this file and return a JSON summary. format: "csv"
    (default without out) or "json". Returns the merged CSV text unless out or
    format="json" is given.
    """
    values = dict(locals())
    for k in ("files", "options"):
        values.pop(k)
    return _emit(_phytkit.run(["concat", *_paths(files), *_options(values, options)]))
