# Your first fit

This page fits the sample A–Ci curves that come with PhytKit, then walks
through the result. Run the commands from the top folder of the PhytKit source (the one with
`Cargo.toml` and `examples/` in it).

## The data

`examples/aci.csv` holds six synthetic A–Ci curves, at two light levels
and three leaf temperatures:

```text
A,Ci,Qabs,Tleaf,CurveID
1.8521,59.26,1500,20,1
5.5849,97.87,1500,20,1
...
```

PhytKit needs four columns for photosynthesis: **A**, **Ci**, light (**Qabs**,
or **Q**) and leaf temperature (**Tleaf**, or **T**). Other columns are kept but
ignored. A **CurveID** column tells PhytKit which rows belong to the same
curve. See [Input files](../reference/input-files.md) for the full rules.

## Fit

```bash
pk fit photosynthesis examples/aci.csv --model fvcb --set gm=1000000 --fix gm,alphaG
```

This fits the Farquhar–von Caemmerer–Berry C3 model. The two options:

- `--set gm=1000000` starts mesophyll conductance at a very large value.
- `--fix gm,alphaG` holds it there, together with `alphaG`.

The result is an **apparent** fit, where the chloroplast CO₂ is taken to equal
Ci. That's the common choice when you have no independent estimate of gm.
See [Photosynthesis models](../concepts/photosynthesis.md#mesophyll-conductance).

## Read the result

`pk` prints one JSON object. The important parts:

```json
{
  "ok": true,
  "command": "fit",
  "file": "aci.csv",
  "tool": "photosynthesis",
  "model": "fvcb",
  "n": 66,
  "mapping": { "A": "A", "Ci": "Ci", "Q": "Qabs", "T": "Tleaf" },
  "r2": 0.998906,
  "rmse": 0.431796,
  "iterations": 63,
  "converged": true,
  "fitted": ["Vcmax25", "Jmax25", "TPU25", "Rd25", "alpha", "theta", "..."],
  "params": { "Vcmax25": 95.231833, "Jmax25": 181.472849, "...": "..." },
  "warnings": [
    "Using A=\"A\", Ci=\"Ci\", light=\"Qabs\", T=\"Tleaf\".",
    "Treated \"Tleaf\" as °C and converted 66 value(s) to Kelvin.",
    "Per-curve constraints: 6 curve(s) via \"CurveID\"."
  ]
}
```

| Field | Meaning |
|---|---|
| `n` | Number of rows fitted, after dropping incomplete ones |
| `mapping` | Which file column was used for each input |
| `r2`, `rmse` | Fit quality, on the measured A only |
| `iterations`, `converged` | How the fitter finished; see [How fitting works](../concepts/fitting.md) |
| `fitted` | The parameters that were optimized |
| `params` | Every parameter value, fitted or fixed |
| `warnings` | What PhytKit assumed or changed. Read these. |

The warnings here confirm that PhytKit:

- used the columns you expected,
- read `Tleaf` as °C and converted it to Kelvin, and
- found 6 curves through `CurveID`.

The full list of output fields is in [Output](../reference/output.md).

## Get a table instead

`--format csv` prints one row per fit, which is handy for a spreadsheet:

```bash
pk fit photosynthesis examples/aci.csv --model fvcb --set gm=1000000 --fix gm,alphaG --format csv
```

## Next steps

- **[Fit an A–Ci curve](../workflows/fit-aci-curve.md)**: choosing the model,
  freeing and fixing parameters, and checking the fit.
- **[Scripting with Python](python.md)**: the same fit from Python.
