# Architecture

```text
PhytKit/       (the repository root)
├── crates/
│   ├── core/   phytkit-core: models, fitter, parsers, LI-600 tools
│   ├── cli/    phytkit-cli: the pk command and its command layer
│   └── py/     phytkit: Python bindings (PyO3, built with maturin)
├── examples/   example data files, used by the docs and tests
└── docs/       this documentation (MkDocs Material)
```

## `phytkit-core`

| Module | What it does |
|---|---|
| `model` | The `Model` trait, `ParamSpec`, `ConstraintSpec`, and the `Params` and `Columns` types (insertion-ordered maps) |
| `fit` | The bounded Levenberg–Marquardt fitter |
| `models/` | One file per model; `models/mod.rs` registers them in `builtin_models()` |
| `linalg`, `dual`, `tresp` | Linear solver, forward-mode autodiff, temperature response |
| `table` | CSV reading and writing (on the `csv` crate), with delimiter detection |
| `parse` | Photosynthesis CSVs (A, Ci, Q, T, CurveID) |
| `hydraulics` | Stomatal, vulnerability, pressure–volume and soil parsers |
| `licor`, `xlsx` | LI-6800 logs, and `.xlsx` workbooks (on `calamine`) |
| `group_csv`, `concat` | `--group-by` splitting and `pk concat` |
| `li600`, `poro` | The LI-600 correction and the A estimate |
| `rescale` | Rescale mode |
| `numbers` | NaN-propagating `max`/`min`, number parsing, and rounding for output |

## `phytkit-cli`

`options.rs` holds a table of every command: its arguments, and each
option's name, type, default and description. That one table drives parsing
and validation, `--help`, and the `commands` section of `pk describe`, so a
new option is added in one place. Errors are `CliError`s with a stable `code`
listed in `args::ERROR_CODES`; a unit test fails if code uses one that isn't
listed.

`lib.rs` holds the command layer: `run(argv)` parses arguments and returns
an `Output`, either JSON or raw text. `render` turns that into what `pk`
prints and its exit code. The `pk` binary in `main.rs` is just `render(run(...))`.
The Python bindings call `run` too, so the CLI and Python can't drift apart.

| Module | Command |
|---|---|
| `describe` | `pk describe` |
| `fit`, `tools` | `pk fit`, and the table of tools, models, parsers and seeds |
| `li600` | `pk li600 correct` and `pk li600 estimate` |
| `concat` | `pk concat` |
| `options` | The command table, argument parsing and validation, help text |
| `args` | Parsed-argument accessors, `CliError` and `ERROR_CODES` |
| `io` | File loading and `--out` writing |

## `phytkit` (Python)

`crates/py/src/lib.rs` exposes `run`, `fit_arrays`, `fit_many`, `predict`,
`models` and `build_info` from Rust. `python/phytkit/__init__.py` adds:

- the keyword-argument wrappers (`fit`, `li600_correct`, ...), whose keywords
  are the command's options from the `COMMANDS` table (`test_signatures.py`
  keeps them equal);
- `PhytKitWarning`, raised for each `warning`-level entry in a result.

`fit_many` releases Python's global lock and fits datasets in parallel in its
own Rayon pool, the only place PhytKit starts threads. `crates/py/build.rs`
records the git commit and build kind for `build_info()`.

## Guidance for agents

The agent skill in `python/phytkit/skills/phytkit/` (`SKILL.md` and
`references/`) is the single source of usage guidance for AI agents. It ships in
the wheel (`phytkit.skills_dir()`). Everything else is generated from it or
points at it:

- `pk agent-files` writes it into a user's `AGENTS.md`
  (`crates/cli/src/agent_files.rs`; the Python package passes the skill's
  location to the command layer on import);
- the Claude Code plugin in `crates/py/claude-plugin/` points at the installed copy;
- `docs/hooks/llms_txt.py` appends it to the site's `llms-full.txt`.

`test_skill.py` checks every name the skill uses against `pk describe`. The
repository's own entry points are `AGENTS.md` (for people using PhytKit),
`CLAUDE.md` (which imports `AGENTS.md`), and `CONTRIBUTING.md` (for developers).

## Conventions

- **Output** is built with `serde_json`. Non-finite numbers are written as
  `null`, and fitted values are rounded to six decimals, or six significant
  figures when that keeps more digits (`numbers::round_fit`).
- **NaN propagates** through `numbers::max` and `numbers::min` (unlike
  `f64::max`), so a parameter set outside a model's valid region gives a NaN
  residual that the fitter rejects, rather than a finite but wrong value.
- **Error messages** from the parsers reach the JSON output, so they're written
  for the person reading them.
- **Warnings** have a stable `code` and a `level` (`phytkit_core::warn`). Add
  a code to `WARNING_CODES` and to the table in `reference/output.md`; never
  rename or remove one.
- **Units** are part of every `ParamSpec` and of `crates/core/src/units.rs`, and
  `pk describe` reports them.
