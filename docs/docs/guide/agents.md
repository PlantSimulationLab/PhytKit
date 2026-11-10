# Using PhytKit with an AI coding agent

Coding agents (Claude Code, Codex, Gemini CLI, Cursor and others) know little or nothing
about PhytKit from their training. The `phytkit` package ships the guidance they need, an
*agent skill*, and keeps it matched to the installed version.

## Give a project the guidance

From the project folder, after `pip install phytkit`:

```bash
pk agent-files
```

This writes two files:

- **`AGENTS.md`**: the installed version, where its skill is, and the skill's guidance:
  which tool fits which measurement, the checks to make on every fit, and the pitfalls.
  Codex, Cursor and most other agents read this file.
- **`CLAUDE.md`**: the single line `@AGENTS.md`, so Claude Code reads the same file.

Run it again after upgrading phytkit. The output lists each file as `written`,
`unchanged` or `user-owned`; `--dry-run` reports without writing, and `--stdout` prints
the `AGENTS.md` text. In Python, use `phytkit.agent_files(".")`.

**Your own edits are safe.** A file is rewritten only while its first line is the
`<!-- phytkit-managed-file ... -->` marker. Delete that line and the file is yours:
`pk agent-files` never touches it again. If you keep your own `CLAUDE.md`, add the line
`@AGENTS.md` to it, and `pk agent-files` stops reminding you.

**Turning it off.** Set `PHYTKIT_AGENT_FILES=0` (or `off`, `false`, `no`) and the command
writes nothing. This is useful in a harness that runs it unconditionally.

Helios and Phytograph use the same marker line and the same kind of switch
(`HELIOS_GENERATE_AGENT_FILES`, `PHYTOGRAPH_AGENT_FILES`), so one rule covers all three.

For Gemini CLI, which reads `GEMINI.md` by default, add `AGENTS.md` to `context.fileName`
in its settings.

## Install the skill itself

Harnesses that load skills from a folder can take the skill directly:

```bash
pk skills install                      # into .claude/skills/phytkit
pk skills install --to path/to/skills  # another harness's folder
pk skills path                         # where the installed package keeps it
```

A copy made by `pk skills install` is marked, and running the command again replaces it.
A folder it didn't make is refused unless you pass `--force`. From Python,
`phytkit.skills_dir()` returns the folder.

## Claude Code plugin

The repository is also a Claude Code plugin marketplace. From a terminal:

```bash
claude plugin marketplace add PlantSimulationLab/PhytKit
claude plugin install phytkit@plantsimulationlab-phytkit
```

The plugin's skill is a short pointer: it finds the installed `phytkit` package and
reads that version's skill. It therefore always matches what you have installed.

## Without the Python package

These commands come with the Python package, because the skill they install is part of
it. A `pk` built from source with `cargo build` reports `not_applicable` for them; use
`pip install phytkit` instead.
