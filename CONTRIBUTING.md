# Contributing to PhytKit

Guidance for people and AI coding agents changing PhytKit itself. To **use** PhytKit to fit
data, read [AGENTS.md](AGENTS.md) and the skill it names instead.

The longer developer documentation is in [`docs/docs/developers/`](docs/docs/developers/):
architecture, building, and adding a model.

## The workspace

A Rust workspace (Rust 1.88+, Python 3.9+):

| Crate | What it is |
|---|---|
| `crates/core` (`phytkit-core`) | Models, fitter, file parsers, LI-600 tools, warning codes |
| `crates/cli` (`phytkit-cli`) | The `pk` command and the command layer the Python package calls |
| `crates/py` (`phytkit` in Python) | PyO3 bindings, built with maturin; ships the agent skill |

```bash
cargo build --release                 # target/release/pk; release builds are slow (fat LTO)
cargo fmt --all                       # CI checks formatting
cargo clippy --release --all-targets -- -D warnings
cargo test --release                  # unit tests + model reference tests

python -m venv .venv && .venv/bin/pip install maturin pytest
VIRTUAL_ENV=$PWD/.venv .venv/bin/maturin develop --release -m crates/py/Cargo.toml
.venv/bin/python -m pytest crates/py/tests   # needs target/release/pk

sh docs/check_examples.sh             # runs every `pk` example in the docs
python3 docs/gen_models.py            # regenerates the model reference page
```

CI runs the equivalent on pushes to `main` and on every pull request.

## Rules that are easy to break

- **Command behavior lives in `crates/cli`.** `phytkit_cli::run(argv)` is the whole `pk`
  implementation, and the Python package calls the same function. Don't reimplement a command
  in Python. `crates/py/tests/cli_cases.txt` lists command lines whose output must match between
  `phytkit.run` and the `pk` binary.
- **Options are declared in the `COMMANDS` table** in `crates/cli/src/options.rs`, which drives
  parsing, validation, `--help`, `pk describe` and the Python keyword signatures
  (`test_signatures.py` holds them equal). Every new error code goes in `args::ERROR_CODES`.
- **Warnings have stable codes.** Create them with `phytkit_core::warn::Warning::new(code, msg)`.
  Each code is listed in `WARNING_CODES` with its level, and documented in
  `docs/docs/reference/output.md`. Never rename or remove a code.
- **Every parameter has units and a description** (`.units(...)`, `.describe(...)`); every model
  input and response is listed in `crates/core/src/units.rs`. A unit test enforces both.
- **New models** implement `phytkit_core::Model` and are registered in
  `crates/core/src/models/mod.rs`. To fit them from files, also add a `ToolModel` to `TOOLS` in
  `crates/cli/src/tools.rs`. Follow `docs/docs/developers/adding-a-model.md`.
- **Model outputs are pinned** by `crates/core/tests/data/model_reference.json`. If a change
  alters them on purpose, regenerate with
  `UPDATE_MODEL_REFERENCE=1 cargo test --release --test model_reference` and explain the diff in
  the pull request. Never regenerate it to make an unexplained failure go away.
- **JSON key order is part of the output contract** (`IndexMap` + `serde_json` `preserve_order`).
- **Use `numbers::max`/`numbers::min`, not `f64::max`/`min`**, and keep comparisons written as
  `!(x >= y)`. Both propagate NaN on purpose, so the fitter rejects invalid parameter regions.
- **Error messages are user-facing.** Parser errors go straight into the JSON output; write them
  for the person holding the data file.
- **Keep the docs in step.** `docs/docs/developers/` describes the architecture.

## Guidance for agents that use PhytKit

Users' agents learn PhytKit from the skill shipped in the wheel:
`crates/py/python/phytkit/skills/phytkit/SKILL.md` and its `references/`. It is the single
source; nothing else is a hand-kept copy.

- `pk agent-files` writes it into user projects (`crates/cli/src/agent_files.rs`).
- The Claude Code plugin in `crates/py/claude-plugin/` is a pointer to the installed copy.
- The docs site appends it to `llms-full.txt` (`docs/hooks/llms_txt.py`).

`crates/py/tests/test_skill.py` keeps every option, command, model, parameter, error code and
warning code the skill names bound to `pk describe`. When you change a command, update the
skill too.

**Report snags as candidate additions to the skill.** Whenever you are tripped up while *using*
the package (a misread unit or convention, a wrong call order, a parameter you had to guess, a
silent bad result, an error that didn't say what to do), tell the user at the end of your
response, as a **potential addition to the phytkit skill**. Give a one-line proposed wording and
the file it belongs in. Don't edit the skill yourself unless asked; the user decides what goes in.

## Releases

The version is `workspace.package.version` in the root `Cargo.toml`; the Python package takes it
from there. The output contract (JSON fields, key order, error and warning codes) is versioned
with it: a breaking change bumps the minor version while PhytKit is below 1.0. Pushing a tag
`vX.Y.Z` that matches the version builds wheels for Linux, macOS and Windows, tests them,
publishes them to PyPI, and creates a GitHub release.
