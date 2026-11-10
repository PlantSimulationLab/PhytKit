# PhytKit: start here (AI coding agents and people)

PhytKit fits plant physiology models to measured data:
- FvCB C3 and C4 and empirical photosynthesis;
- BTA and Medlyn stomatal conductance;
- xylem vulnerability, pressure–volume curves and soil water retention;
- the LI-600 porometer correction.

It is a Python package, `phytkit` (`pip install phytkit`), with a command-line tool, `pk`.
Both return the same JSON.

**Fitting data?** Use the package, as below. **Changing PhytKit itself?** Skip to the
last section.

## Using PhytKit

1. **Read the skill before writing any code.** It covers which tool fits which measurement,
   the checks to make on every fit, the pitfalls, and how to carry parameters into Helios.
   - Installed package: `pk skills path`, or `phytkit.skills_dir() / "phytkit" / "SKILL.md"`.
   - In this repository: `crates/py/python/phytkit/skills/phytkit/SKILL.md`, with its
     `references/`.
2. **Ask the installed package, not your memory.**
   - `pk describe` lists every tool, model, parameter (with units and bounds), option,
     error code and warning code.
   - `pk describe <model>` and `pk <command> --help` give the detail.
3. **Read `ok`, `converged` and every entry in `warnings`** of each result. An entry with
   `level: "warning"` may make the result wrong; tell the user about it.
4. **In the user's own project**, run `pk agent-files`. It writes this guidance, matched to
   the installed version, into their `AGENTS.md` and `CLAUDE.md`. `pk skills install`
   copies the skill into `.claude/skills/` instead.

```python
import phytkit as pk
res = pk.fit("photosynthesis", "examples/aci.csv", model="fvcb", ci_based=True)
res["params"]["Vcmax25"], res["converged"], res["warnings"]
```

Documentation: <https://plantsimulationlab.github.io/PhytKit/>. Its sources are in
`docs/docs/`. When the docs and the installed package disagree, the installed package
(`pk describe`, the shipped skill) is right for that version.

If you were tripped up while using PhytKit, end your response with a one-line proposed
addition to the skill. Don't edit the skill yourself.

## Changing PhytKit

Read [CONTRIBUTING.md](CONTRIBUTING.md): the workspace layout, build and test commands,
the rules that are easy to break, and how the agent skill is kept in step with the code.
The longer developer documentation is in `docs/docs/developers/`.
