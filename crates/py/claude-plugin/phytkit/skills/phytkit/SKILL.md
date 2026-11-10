---
name: phytkit
description: Fit plant physiology models to measured data with the phytkit Python package or its pk command - FvCB C3/C4 and empirical photosynthesis from A/Ci curves, BTA and Medlyn stomatal conductance, xylem vulnerability, pressure-volume curves, soil water retention, and the LI-600 porometer correction. Use when a task fits, checks or converts parameters such as Vcmax25, Jmax25, g1, P50 or van Genuchten alpha, including parameters meant for a Helios simulation.
---

# PhytKit from Python (plugin entry)

The instructions for the `phytkit` package ship inside the package, so that they always match
the installed version. This plugin follows the repository's main branch, not the user's install,
so it holds no instructions of its own: it finds the installed ones.

1. **Find the package.** Use the Python the user's project uses (its virtual environment if it
   has one) and run
   `python -c "import phytkit; print(phytkit.__version__, phytkit.skills_dir())"`.
   If the import fails, ask the user before installing: `pip install phytkit`.
2. **Read the installed skill in full and follow it**: `<skills_dir>/phytkit/SKILL.md`. It
   names reference files beside it (`references/*.md`); read the one for the measurement you
   fit before reporting results, and `references/helios.md` before handing values to Helios.
3. **Offer to put the guidance in the project** so later sessions start with it:
   `pk agent-files` (writes `AGENTS.md` and a `CLAUDE.md` that imports it). If the project
   already has its own `CLAUDE.md` without the line `@AGENTS.md`, tell the user.

If `phytkit.skills_dir` does not exist, the installed package predates the shipped skill. Tell
the user to upgrade (`pip install -U phytkit`); until then, work from `pk describe`,
`pk <command> --help` and https://plantsimulationlab.github.io/PhytKit/.
