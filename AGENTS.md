# AGENTS.md

Guidance for AI coding agents, and for people, working with Phytkit, either to fit data with it or
to change it.

## What this project is

Phytkit fits plant physiology models to measured data: photosynthesis (FvCB C3 and C4, empirical
light response), stomatal conductance (BTA, Medlyn), xylem vulnerability curves, pressure–volume
curves, soil water retention, and the LI-600 porometer energy-balance correction. It is written in
Rust and comes as a command-line tool, `pk`, and a Python package, `phytkit`
(`pip install phytkit`, which also installs `pk`). Both run the same command layer and return the
same JSON.

Documentation: <https://plantsimulationlab.github.io/PhytKit/>, with sources in `docs/docs/`.

## Fitting data with Phytkit

1. **Ask the tool, not your memory.** Phytkit is new and your training data likely knows nothing
   about it. `pk describe` lists every tool, model, parameter (with bounds and defaults), soft
   constraint and command option, as JSON. `pk describe <model>` gives one model in full, and
   `pk <command> --help` gives a command's options. Check every model, parameter and option name
   against these before using it.
2. **Pin down the question with the user first:** which measurement (A/Ci curves, porometer
   readings, PV curves, ...), which model, which parameters are fixed and at what values, and
   whether curves are fitted one by one (`--group-by`, several files) or together.
3. **Read the whole output, every time.** Check `ok`, then `converged`, `r2`/`rmse`, and
   `warnings`. `warnings` reports the columns Phytkit chose, unit conversions it made, and rows it
   dropped. A fit can finish with `ok: true` and still be wrong because the wrong column was used
   or half the rows were dropped. Do not report a result as good until you have read them.
4. **Look at the fit, not only the statistics.** Use `--predicted` (`predicted=True` in Python) to
   get model and measured values point by point, and compare them, ideally on a plot. A high r²
   can hide a systematic misfit in one limitation region.
5. **Check fitted values against physical sense** and the literature range for the species. A
   parameter sitting on its bound means the bound, not the data, set it: say so.

### Output and errors

Output is a JSON envelope: `{"ok": true, "command": ..., ...}` on success. On failure it is
`{"ok": false, "error": {"code", "message", "usage", "hint"?, "help"?}}` and `pk` exits with
status 1; Python raises `phytkit.PhytkitError` with the same `.code`, `.hint` and `.usage`. Error
codes are stable and listed in `docs/docs/reference/commands.md`. Read `hint`: it often names the
fix (`Did you mean lightResponse?`). `NaN` and infinite values are written as `null`, and fitted
values are rounded to six decimals.

### Python

```python
import phytkit as pk

pk.describe("fvcb")                                                   # same as `pk describe fvcb`
pk.fit("photosynthesis", "leaf.csv", model="fvcb", fix=["gm"], max_iter=400)
pk.fit_arrays("fvcb", {"Ci": ci, "Q": q, "T": t_kelvin}, a)           # data already in memory
pk.fit_many("fvcb", [(x1, y1), (x2, y2)], threads=8)                  # many independent fits
pk.predict("fvcb", {"Vcmax25": 90}, {"Ci": ci, "Q": q, "T": t_kelvin})
pk.run(["fit", "soil", "soil.csv", "--group-by", "sample"])           # any pk command line
```

Keyword arguments become `pk` flags (`max_iter=400` is `--max-iter 400`; a list repeats the flag;
a dict becomes repeated `--flag key=value`).

## Pitfalls

- **Temperatures in `fit_arrays`, `fit_many` and `predict` are in Kelvin.** The file commands
  convert any temperature below 200 from °C to Kelvin and say so in `warnings`; the array functions
  take your numbers as they are.
- **Column names are matched loosely** (ignoring case, spaces, underscores and unit suffixes), and
  `Qabs` is preferred over `Q` when both are present. Check `mapping` and `warnings` to see which
  columns were actually used.
- **Soil water potential is in MPa** unless `--units` says otherwise.
- **Unknown options and out-of-range values are errors**, never silently ignored. Do not work
  around one by dropping the option; fix the value or ask the user.
- **Only `.xlsx` workbooks are read**, not `.xls`. Date cells are read as ISO 8601 text, so a date
  column cannot be a numeric input.

## Changing Phytkit

The repository is a Rust workspace (Rust 1.88+, Python 3.9+):

| Crate | What it is |
|---|---|
| `crates/core` (`phytkit-core`) | Models, fitter, file parsers, LI-600 tools |
| `crates/cli` (`phytkit-cli`) | The `pk` command and the command layer the Python package calls |
| `crates/py` (`phytkit` in Python) | PyO3 bindings, built with maturin |

```bash
cargo build --release                 # target/release/pk; release builds are slow (fat LTO)
cargo clippy --release -- -D warnings
cargo test --release                  # unit tests + model reference tests

python -m venv .venv && .venv/bin/pip install maturin pytest
VIRTUAL_ENV=$PWD/.venv .venv/bin/maturin develop --release -m crates/py/Cargo.toml
.venv/bin/python -m pytest crates/py/tests   # needs target/release/pk

sh docs/check_examples.sh             # runs every `pk` example in the docs
```

CI runs all of the above on every push and pull request.

Rules that are easy to break:

- **Command behavior lives in `crates/cli`.** `phytkit_cli::run(argv)` is the whole `pk`
  implementation, and the Python package calls the same function. Do not reimplement a command
  in Python. `crates/py/tests/cli_cases.txt` lists command lines whose output must match between
  `phytkit.run` and the `pk` binary.
- **Options are declared in the `COMMANDS` table in `crates/cli/src/options.rs`**, which drives
  parsing, validation, `--help` and `pk describe`. Every new error code must be added to
  `args::ERROR_CODES`.
- **New models** implement `phytkit_core::Model` and are registered in
  `crates/core/src/models/mod.rs`; to fit them from files, also add a `ToolModel` to `TOOLS` in
  `crates/cli/src/tools.rs`. Follow `docs/docs/developers/adding-a-model.md`.
- **Model outputs are pinned** by `crates/core/tests/data/model_reference.json`. If a change
  alters them on purpose, regenerate with
  `UPDATE_MODEL_REFERENCE=1 cargo test --release --test model_reference` and explain the diff in
  the pull request. Never regenerate it to make an unexplained failure go away.
- **JSON key order is part of the output contract** (`IndexMap` + `serde_json` `preserve_order`).
- **Use `numbers::max`/`numbers::min`, not `f64::max`/`min`**, and keep comparisons written as
  `!(x >= y)`: both propagate NaN on purpose so the fitter rejects invalid parameter regions.
- **Error messages are user-facing.** Parser errors go straight into the JSON output; write them
  for the person holding the data file.
- **Keep the docs in step.** `python3 docs/gen_models.py` regenerates the model reference page
  from `pk describe`; `docs/docs/developers/` describes the architecture.

## Releases

The version is `workspace.package.version` in the root `Cargo.toml` (the Python package takes it
from there). Pushing a tag `vX.Y.Z` that matches it builds wheels for Linux, macOS and Windows,
tests them, publishes them to PyPI and creates a GitHub release.
