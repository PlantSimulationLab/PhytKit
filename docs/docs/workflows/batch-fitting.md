# Fit many curves at once

## Several files

Give several files to one `pk fit`. Each is fitted on its own:

```bash
pk fit photosynthesis examples/aci.csv examples/messy_aci.csv --model empiricalA
```

The result lists each fit under `results`. Files that couldn't be read or
fitted are listed under `errors` and don't stop the others. Add
`--format csv` for one row per file, with the error in the last column.

## One file, split by a column

`--group-by <column>` splits one file into a fit per value of that column, in
the order the values first appear. Rows with a blank value are dropped.

```bash
pk fit hydraulics examples/pv.csv --model pvcurve --group-by Leaf
pk fit soil examples/soil.csv --units kPa --group-by sample --format csv
```

!!! note "Group-by or CurveID?"
    For photosynthesis, `--group-by CurveID` fits each curve **separately**.
    Without it, the curves in a file are fitted **together** as one dataset
    that shares parameters, with the CurveID column used only for the
    per-curve constraints. Choose based on whether the curves should share
    parameters.

!!! warning "A curve at one temperature can't fit a temperature response"
    Each A–Ci curve is usually measured at one leaf temperature. Fitted alone,
    it cannot determine `Vcmax_dHa`, `Vcmax_Topt` and the other temperature
    parameters, and Vcmax25 then depends on where they settle. PhytKit warns
    with `temperature_response_unconstrained`. List the parameters to fit, so
    the temperature parameters stay at their defaults, and report those defaults
    alongside the 25 °C values:

    ```bash
    pk fit photosynthesis examples/aci.csv --ci-based --group-by CurveID --fit Vcmax25,Jmax25,TPU25,Rd25,alpha,theta
    ```

## Thousands of datasets in Python

`fit_many` fits a list of datasets in parallel on all CPU cores. Use it when
the data are already in memory, for example from pandas:

```python
import phytkit as pk

datasets = []
for ci, q, t_kelvin, a in curves:   # one entry per curve
    datasets.append(({"Ci": ci, "Q": q, "T": t_kelvin}, a))

results = pk.fit_many("fvcb", datasets, init={"gm": 1e6},
                      fit_params=["Vcmax25", "Jmax25", "TPU25", "Rd25", "alpha", "theta"], threads=8)
for r in results:
    print(r["params"]["Vcmax25"], r["rmse"], r["converged"])
```

Results come back in the same order as `datasets`. `threads` defaults to every
core. The fits run outside Python's global lock, so other Python threads keep
running while they do.
