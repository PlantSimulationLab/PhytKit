---
name: phytkit
description: Fit plant physiology models to measured data with the phytkit Python package or its pk command - FvCB C3/C4 and empirical photosynthesis from A/Ci curves, BTA and Medlyn stomatal conductance, xylem vulnerability, pressure-volume curves, soil water retention, and the LI-600 porometer correction. Use when a task fits, checks or converts parameters such as Vcmax25, Jmax25, g1, P50 or van Genuchten alpha, including parameters meant for a Helios simulation.
---

# Fitting plant physiology data with PhytKit

PhytKit is new; your training data probably knows nothing reliable about it. The installed
package describes itself, so ask it instead of guessing a name, an option or a unit:

```bash
pk describe                 # every tool, model, command option, error code and warning code
pk describe fvcb            # one model: inputs, response, parameters with units and bounds
pk fit --help               # a command's options
```

```python
import phytkit as pk
pk.describe("fvcb")         # the same JSON as a dict
```

Every parameter and input in `pk describe <model>` has a `units` field. Use it.

## Which tool fits which measurement

| Measurement | Command | Models (first is the default) |
|---|---|---|
| A–Ci, light or temperature response curves (LI-6800 log, `.xlsx` or CSV) | `pk fit photosynthesis` | `fvcb`, `fvcbC4`, `empiricalA` |
| Stomatal conductance with light and VPD | `pk fit stomatal` | `bta` (gs, Q, VPD), `medlyn` (gs, A, Cs, VPD) |
| Xylem vulnerability (Ψ with PLC or K) | `pk fit hydraulics` | `vulnerability` |
| Pressure–volume curve (RWC, Ψ) | `pk fit hydraulics --model pvcurve` | `pvcurve` |
| Soil water retention (ψ, θ) | `pk fit soil` | `vanGenuchten`, `modifiedVanGenuchten`, `raats`, `brooksCorey`, `fractalSWRC` |
| LI-600 porometer readings | `pk li600 correct`, then `pk li600 estimate` | `fvcb`, `fvcbC4`, `empiricalA` for the estimate |
| Several LI-6800 logs into one file | `pk concat` | — |
| Data already in memory | `pk.fit_arrays`, `pk.fit_many`, `pk.predict` | any model in `pk.models()` |

Python mirrors the commands: `pk.fit("photosynthesis", "leaf.csv", model="fvcb", ci_based=True)`,
`pk.li600_correct(...)`, `pk.li600_estimate(...)`, `pk.concat(...)`. A keyword becomes the option
of the same name (`max_iter=400` is `--max-iter 400`, `set={"Vcmax25": 90}` is
`--set Vcmax25=90`).

**One fit per curve, or one fit for all?** Ask the user. Several files, or
`--group-by CurveID`, fit each file or curve separately. Without `--group-by`, the curves in a
file are fitted together as one dataset that shares parameters.

## Rules that hold for every fit

1. **Check `ok`.** A failure is `{"ok": false, "error": {"code", "message", "hint"}}` (Python
   raises `PhytKitError` with `.code` and `.hint`). The `hint` usually names the fix.
2. **Check `converged`, `r2` and `rmse`.**
3. **Read every entry in `warnings`.** Each is `{"code", "level", "message"}`.
   - `level: "info"` reports a choice PhytKit made, such as the columns it used.
   - `level: "warning"` means the result may be wrong. Tell the user about each one:
     `rows_dropped`, `unit_guessed_from_values`, `parameter_at_bound`, `not_converged`,
     `temperature_response_unconstrained`, and the rest listed in `pk describe`.
   - Python also raises each `warning` entry as `pk.PhytKitWarning`.
4. **Check `mapping`**: is each input the column you meant?
5. **Look at the fit**, not only r²: `--predicted` (`predicted=True`) adds `predicted` and
   `observed` point by point. Compare them, ideally on a plot.
6. **A parameter on its bound was set by the bound, not the data.** Report it as unconstrained,
   or hold it with `--fix` at a value the user chooses.
7. **Check values against physical sense** and the literature range for the species.

## Pitfalls

- **Temperatures.** The array functions (`fit_arrays`, `fit_many`, `predict`) take leaf
  temperature `T` in **kelvin** as given; only the file commands convert °C. Temperature optima
  (`Vcmax_Topt`, `Jmax_Topt`, `TPU_Topt`) are kelvin too. Helios takes them in °C.
- **One temperature per curve.** A curve measured at one leaf temperature cannot determine
  `*_dHa`, `*_dHd` or `*_Topt`, and the 25 °C values (Vcmax25, Jmax25, ...) then depend on where
  they settle. Fit only the rest, for example
  `--fit Vcmax25,Jmax25,TPU25,Rd25,alpha,theta`, and report the temperature parameters that were
  held. Curves at several temperatures, fitted together, can determine them.
- **Mesophyll conductance.** `fvcb` fits `gm` by default (bounds 0.01–5), and on typical A–Ci
  data it often ends on a bound. For an apparent, Ci-based fit (Cc = Ci), use `--ci-based`
  (`ci_based=True`): it holds `gm` at infinity (written as `null`) and `alphaG` at 0. That is the
  form Helios uses by default.
- **`--fit` replaces the fitted set**; it does not add to it. `--fix` removes names from the
  default set. A `--set` value for a fitted parameter must lie within its bounds; a fixed
  parameter can be held at any value.
- **Columns are matched by name** against lists of accepted names (photosynthesis: `A`, `Ci`,
  `Qabs` or `Q`, `Tleaf` or `T`, optional `CurveID`). Unit suffixes are not stripped in general.
  Check `mapping`.
- **Soil ψ units** come from `--units`, else from the column name (`Psi (kPa)`), else MPa (with
  the `soil_units_assumed` warning). Fitted α and ψb are in MPa-based units.
- **Light.** Photosynthesis uses absorbed light (`Qabs` before `Q`); `pk fit stomatal` prefers
  incident `Qin`.
- **Unknown options and out-of-range values are errors.** Fix the value or ask the user; never
  drop the option to make the error go away.
- **Threads.** Only `fit_many` runs in parallel. `threads=None` uses `RAYON_NUM_THREADS`, else
  every core. Results are bit-identical at any thread count.

## Using the parameters in Helios

Read [references/helios.md](references/helios.md) before handing any fitted value to Helios. In
short: fit FvCB with `--ci-based`, convert `*_Topt` from kelvin to °C, pass `theta`, `alpha`,
`dHa`, `dHd` and `Topt` explicitly, and use BTA's `Em`, `i0`, `k` and `b` unchanged for Helios's
BMF model.

## References: load the one you need

- [references/photosynthesis.md](references/photosynthesis.md): FvCB, C4 and empirical fits,
  soft constraints, rescaling to survey points.
- [references/stomatal.md](references/stomatal.md): BTA and Medlyn inputs and units.
- [references/hydraulics-and-soil.md](references/hydraulics-and-soil.md): vulnerability, PV
  curves, soil retention.
- [references/batch-output.md](references/batch-output.md): several files, `--group-by`,
  `fit_many`, and reading batch results.
- [references/li600.md](references/li600.md): the LI-600 correction and estimating A from gsw.
- [references/helios.md](references/helios.md): carrying fitted parameters into Helios, and
  what does not transfer.

## Setting up a project

`pk agent-files` writes an `AGENTS.md` and a `CLAUDE.md` with this guidance into the current
project. `pk skills install` copies this skill into `.claude/skills/`. `pk skills path` prints
where it is installed.

## When something here is wrong

If you were tripped up while using the package (a misread unit, an option you had to guess, a
silent bad result, an error that did not say what to do), end your response to the user with a
**candidate addition to the phytkit skill**: one line, worded as it would appear here. Do not
edit the skill yourself; the user decides what goes in.
