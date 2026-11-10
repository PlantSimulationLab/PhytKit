# Estimate A from porometry

Solve net assimilation A, Ci and Cc for every LI-600 reading from its
conductance, light and leaf temperature and a photosynthesis parameter set.
See [Estimating A from conductance](../concepts/li600.md#estimating-a-from-conductance).

## Inputs

- **An LI-600 file** with `gsw`, `Qamb` and `Tleaf` (a `gsw_corrected`
  column, if present, is used in place of `gsw`).
  Correct it first ([Correct LI-600 conductance](correct-li600.md)), so the
  estimate uses the corrected conductance.
- **Photosynthesis parameters** for the species, usually from a `pk fit` of
  its A–Ci curves.

## Steps

1. Fit the species' A–Ci curves and save the parameters:

    ```bash
    pk fit photosynthesis examples/aci.csv --model fvcb --set gm=1000000 --fix gm,alphaG > fvcb_fit.json
    ```

2. Correct the porometer file:

    ```bash
    pk li600 correct examples/li600.csv --sidedness 1 --out li600_corrected.csv
    ```

3. Estimate A, writing the solved columns into a copy of the file:

    ```bash
    pk li600 estimate li600_corrected.csv --params fvcb_fit.json --out li600_with_A.csv
    ```

    The output reports:

    - `usedCorrectedGsw`: whether `gsw_corrected` was used.
    - `params`: the parameter set used, after merging defaults, the file and
      `--set`.
    - `ignoredParams`: names in the file the model doesn't use.
    - `summary`: mean and maximum A, mean Ci, and mean Ci/Ca.

    `li600_with_A.csv` adds `A_solved`, `Ci_solved`, `Cc_solved`, `gsw_used`
    and `Qabs`.

## Options

| Option | Default | Meaning |
|---|---|---|
| `--model` | the `--params` fit's model, else `fvcb` | `fvcb`, `fvcbC4` or `empiricalA` |
| `--params` | — | Parameter file: a single `pk fit` JSON result (a batch result is rejected; pass one entry of its `results`), a CSV of fitted parameters (last row used), a Parameter/Value table, or `name = value` lines |
| `--set` | — | Override one parameter, e.g. `--set Vcmax25=90` |
| `--ca` | 420 | Ambient CO₂, µmol mol⁻¹ |
| `--absorptance` | 0.85 | Leaf absorptance applied to `Qamb` |
| `--min-gsw` | 0.001 | Floor for conductance; lower readings are raised to it and counted in `clamped`. `0` leaves them unsolved. |

Without `--params`, the model's defaults are used. That's fine for trying the
command, but not for real estimates:

```bash
pk li600 estimate examples/li600.csv --set Vcmax25=90 --set Jmax25=170
```
