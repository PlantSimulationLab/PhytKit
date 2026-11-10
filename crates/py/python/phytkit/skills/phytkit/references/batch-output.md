# Many fits at once

Part of the `phytkit` skill.

## Several files, or one file split by a column

```bash
pk fit photosynthesis a.csv b.csv c.csv --ci-based
pk fit photosynthesis curves.csv --ci-based --group-by CurveID --fit Vcmax25,Jmax25,TPU25,Rd25,alpha,theta
```

Each file or group is fitted separately. With one file and no `--group-by`, or a
`--group-by` that finds one group, the output is a single fit. Otherwise it is a batch:

| Field | Meaning |
|---|---|
| `tool`, `files`, `groupBy` | What was run; `files` counts files or groups |
| `fitted`, `failed` | How many fits succeeded and failed |
| `results` | One full fit object per success, each with its own `file`, `params`, `converged` and `warnings` |
| `errors` | `{file, code, error, hint}` for each failure; failures don't stop the others |

Read `converged` and `warnings` in **every** entry of `results`, and report every entry of
`errors`. In Python, `PhytKitWarning` messages name the file or group (`[code] CurveID 3: ...`).
`--format csv` prints one row per fit, with any error in the last column.

For photosynthesis, `--group-by CurveID` fits each curve separately. Without it, the curves in
a file are fitted together, sharing parameters, and CurveID only scopes the per-curve
constraints. Ask the user which they want.

## Data in memory: `fit_many`

```python
results = pk.fit_many("fvcb", [(x1, y1), (x2, y2)], init={"gm": 1e6},
                      fit_params=["Vcmax25", "Jmax25", "TPU25", "Rd25", "alpha", "theta"])
```

- Results come back in the order of the datasets. Each is a `fit_arrays` result with `params`,
  `fitted`, `predicted`, `rmse`, `r2`, `iterations`, `converged` and `warnings`.
- `threads=None` uses `RAYON_NUM_THREADS`, else every core. On a shared machine, set one of
  them. Results are bit-identical at any thread count.
- Inputs are in model units: `T` in kelvin, light absorbed.
