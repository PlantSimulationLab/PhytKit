# PhytKit

Plant physiology model fitting, written in Rust: photosynthesis (FvCB C3 and C4, empirical), stomatal conductance (BTA, Medlyn), xylem vulnerability, pressure–volume curves, soil water retention, and LI-600 porometer tools. It comes as a command-line tool, `pk`, and a Python package, `phytkit`.

| Crate | What it is |
|---|---|
| `crates/core` (`phytkit-core`) | Models, fitter, file parsers, LI-600 tools |
| `crates/cli` (`phytkit-cli`) | The `pk` command, plus the command layer the Python package calls |
| `crates/py` (`phytkit` in Python) | Python bindings, built with maturin |

Documentation: <https://plantsimulationlab.github.io/PhytKit/>. AI agents and contributors: see [AGENTS.md](AGENTS.md).

## Install

```bash
pip install phytkit      # the Python package and the pk command
```

## Building from source: requirements

- Rust 1.88 or newer.
- For Python: Python 3.9 or newer, and [maturin](https://www.maturin.rs).

## Command line

```bash
cargo build --release          # builds target/release/pk
cargo test --release

./target/release/pk --help
./target/release/pk describe fvcb
./target/release/pk fit photosynthesis leaf.csv --model fvcb
./target/release/pk fit soil examples/soil.csv --group-by sample --units kPa
./target/release/pk li600 correct porometer.csv --sidedness 1.5
```

`cargo install --path crates/cli` puts `pk` on your PATH. Output is JSON (or CSV with `--format csv`); `pk concat` prints the merged CSV unless `--out` or `--format json` is given. Errors print `{"ok": false, "error": {"code", "message", "hint", ...}}` and exit with code 1; unknown options and out-of-range values are errors, never ignored.

## Python

```bash
python -m venv .venv && .venv/bin/pip install maturin pytest
VIRTUAL_ENV=$PWD/.venv .venv/bin/maturin develop --release -m crates/py/Cargo.toml
.venv/bin/python -m pytest crates/py/tests
```

If Conda is active, add `env -u CONDA_PREFIX` before `maturin`. Otherwise it may install into the Conda environment instead of `.venv`.

```python
import phytkit as pk

pk.fit("photosynthesis", "leaf.csv", model="fvcb", fix=["gm"], max_iter=400)   # same result as `pk fit`
pk.li600_estimate("poro.csv", params="fit.csv", rows=True)
pk.fit_arrays("fvcb", {"Ci": ci, "Q": q, "T": t_kelvin}, a)                    # data in memory
pk.fit_many("fvcb", [(x1, y1), (x2, y2)], threads=8)                         # parallel batch
pk.predict("fvcb", {"Vcmax25": 90}, {"Ci": ci, "Q": q, "T": t_kelvin})
```

Commands return the same structure as `pk`'s JSON, and take the same options as `pk` (`max_iter=400` for `--max-iter 400`). Failures raise `pk.PhytKitError`, with the same `.code`, `.hint` and `.usage` as `pk`'s error object.

## Tests

- **`cargo test --release`:** unit tests, plus model tests that check every model's predictions, derivatives and fits against values stored in `crates/core/tests/data/model_reference.json`. After changing a model on purpose, run `UPDATE_MODEL_REFERENCE=1 cargo test --release --test model_reference` to rewrite them, and review the difference.
- **`pytest crates/py/tests`:** the Python package against the `pk` binary, on the example files in `examples/`. Build `pk` first.

The example files are synthetic and seeded; `python3 examples/make_examples.py` regenerates them.

## Known limitations

- Date cells are read as text (ISO 8601), so a date column can't be used as a numeric input.
- Only `.xlsx` workbooks are read; convert older `.xls` files first.

## Documentation

The documentation site lives in `docs/` and is published at <https://plantsimulationlab.github.io/PhytKit/>. To read it locally:

```bash
python3 -m venv .docs-venv && .docs-venv/bin/pip install -r docs/requirements.txt
cd docs && ../.docs-venv/bin/mkdocs serve      # then open http://127.0.0.1:8000
```

## Releasing

Bump `workspace.package.version` in `Cargo.toml`, commit, and push a matching tag (`git tag v0.2.0 && git push origin v0.2.0`). The `Release` workflow builds wheels for Linux (x86_64, aarch64), macOS (arm64, x86_64) and Windows, tests them (except macOS x86_64, which is built but not tested), publishes them and an sdist to PyPI, and creates a GitHub release.

## Credits

The original version of PhytKit was developed primarily by Kyle Rizzo ([@ktrizzo](https://github.com/ktrizzo)). It is maintained by the [PlantSimulationLab](https://github.com/PlantSimulationLab).

## License

MIT. See [LICENSE](LICENSE).

