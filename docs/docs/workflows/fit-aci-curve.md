# Fit an A–Ci curve

Fit the Farquhar–von Caemmerer–Berry C3 model to A–Ci (and light and
temperature) response data.

## Inputs

A CSV, `.xlsx` or LI-6800 log with columns for **A**, **Ci**, light (**Qabs**
or **Q**) and leaf temperature (**Tleaf** or **T**). Light should be
**absorbed** light; PhytKit uses `Qabs` when both are present. Temperatures
below 200 are read as °C.

Add a **CurveID** column if the file holds several curves. PhytKit then
applies FvCB's per-curve constraints to each curve separately. LI-6800 logs
merged with [`pk concat`](merge-li6800-logs.md) have one.

## Steps

1. Look at the model's parameters and defaults:

    ```bash
    pk describe fvcb
    ```

2. Fit. This is an **apparent** fit (\(C_c = C_i\)), with mesophyll conductance
   fixed very high and `alphaG` held at zero:

    ```bash
    pk fit photosynthesis examples/aci.csv --model fvcb --ci-based
    ```

3. Check the result before you use it:

    - `warnings`: did PhytKit use the columns and units you expected?
    - `converged`: `false` means it stopped at the iteration limit (rerun
      with `--max-iter 400` or more) or stalled away from a minimum (try
      other starting values with `--set`).
    - `r2` and `rmse`: how closely the model follows the data.
    - `params`: values sitting exactly at a bound (`pk describe fvcb`) are
      poorly determined by the data.

4. Add `--predicted` to get the model's prediction for every point next to
   the measured values, for plotting:

    ```bash
    pk fit photosynthesis examples/aci.csv --model fvcb --ci-based --predicted
    ```

## Choosing what to fit

By default, every parameter not marked *fixed by default* in
[Models and parameters](../reference/models.md#fvcb) is fitted. That's 18
parameters, including the temperature responses. For a single curve at one
temperature, that's far more than the data can determine. Fit only what the
data constrains:

```bash
pk fit photosynthesis examples/aci.csv --model fvcb --ci-based --fit Vcmax25,Jmax25,TPU25,Rd25,alpha,theta
```

A single-temperature curve says nothing about the temperature parameters, so
leaving them out of `--fit` keeps them at their defaults.

## C4 leaves and the empirical model

```bash
pk fit photosynthesis examples/aci.csv --model fvcbC4
pk fit photosynthesis examples/aci.csv --model empiricalA
```

The sample data are C3, so these are for illustration only. See
[Photosynthesis models](../concepts/photosynthesis.md).

## Constraints

Most of FvCB's soft constraints are on by default; `ap_limits_high` and
`vcmax_window` are off (see
[Photosynthesis models](../concepts/photosynthesis.md#soft-constraints)). To
fit without any, to turn one on or off, or to change one's weight (`--weight`
sets the weight outright; the defaults are listed by `pk describe fvcb`, and
`vj_ratio`'s is 0.1):

```bash
pk fit photosynthesis examples/aci.csv --model fvcb --constraints none
pk fit photosynthesis examples/aci.csv --model fvcb --no-constraint vj_ratio
pk fit photosynthesis examples/aci.csv --model fvcb --constraint vcmax_window --weight vj_ratio=0.02
```

## In Python

```python
import phytkit as pk

fit = pk.fit("photosynthesis", "examples/aci.csv", model="fvcb", set={"gm": 1e6}, fix=["gm", "alphaG"], predicted=True)
print(fit["params"]["Vcmax25"], fit["params"]["Jmax25"])
```
