# Photosynthesis fits

Part of the `phytkit` skill. `pk describe fvcb` (or `fvcbC4`, `empiricalA`) lists every
parameter with its units, bounds and default; this file says how to use them.

## Input

CSV, `.xlsx` or an LI-6800 log with `A`, `Ci`, `Qabs` or `Q` (absorbed light), `Tleaf` or
`T`, and optionally `CurveID`. Names are matched ignoring case. Leaf temperatures below 200 are
read as °C and converted (`temperature_celsius`). Rows missing a value are dropped
(`rows_dropped`). Merge several LI-6800 logs first with `pk concat`; see
[li600.md](li600.md) for porometer data.

## FvCB C3 (`fvcb`, the default)

- **Fitted by default:** `Vcmax25`, `Jmax25`, `TPU25`, `Rd25`, `gm`, `alpha`, `theta`,
  `alphaG`, and the temperature responses `Vcmax_dHa`, `Vcmax_dHd`, `Vcmax_Topt`, `Jmax_*`,
  `TPU_*` and `Rd_dHa`.
- **Fixed by default:** Γ\*, Kc, Ko and O (`GammaS25`, `Kc25`, `Ko25`, `O` and their `_dHa`).
- **Apparent fit:** `--ci-based` holds `gm` at infinity (Cc = Ci) and `alphaG` at 0. Use it
  unless the user has a reason to estimate gm; on ordinary A–Ci data a fitted gm often ends
  on a bound (`parameter_at_bound`).
- **Temperature responses** need curves at several leaf temperatures fitted together. For one
  curve, or curves at one temperature, fit only
  `--fit Vcmax25,Jmax25,TPU25,Rd25,alpha,theta` (add `--ci-based`) and report the temperature
  parameters as held at their defaults.
- **Soft constraints** steer the fit toward sensible limitation patterns. `pk describe fvcb`
  lists their keys. `--no-constraint <key>` turns one off, `--constraints none` turns all off,
  `--constraint <key>` turns on one that is off by default, and `--weight key=w` reweights one.
- **Fit quality.** `predicted`, `rmse` and `r2` come from the model's own solution, not from
  the measured A used to compute Cc during fitting.

## FvCB C4 (`fvcbC4`)

von Caemmerer (2021). Fits `Vpmax25`, `Vcmax25`, `Jmax25`, `Rd25` and their temperature
parameters by default, and holds `gm`, `gbs`, `Vpr` and the kinetic constants fixed. It has no
TPU limitation, and its Kc, Ko, Kp and Om are in µbar.

## Empirical (`empiricalA`)

A light, CO₂ and temperature response with `Amax25`, `Kq25`, `Kco25`, `GammaS25`, `Rd25` and
activation energies. A `gm` of 99 or more means no mesophyll resistance. It is not Helios's
empirical model; see [helios.md](helios.md).

## Rescaling curves to survey points

`--survey survey.csv` (one or more files of A, Ci, Qabs and Tleaf) scales the curves' amplitude
to the survey's level of A at the survey's mean light and temperature, then refits. The result's
`rescale` section gives the `factor`, the reference condition and the first-pass fit. `--stat`
chooses `mean`, `median` or `max`; `--ref-ci` sets the reference Ci.

## Arrays in Python

```python
x = {"Ci": ci, "Q": q_absorbed, "T": [t + 273.15 for t in tleaf_c]}   # T in kelvin
res = pk.fit_arrays("fvcb", x, a, init={"gm": 1e6},
                    fit_params=["Vcmax25", "Jmax25", "TPU25", "Rd25", "alpha", "theta"])
```

`fit_arrays` has no `ci_based` keyword. Leave `gm` and `alphaG` out of `fit_params` and hold
`gm` large with `init`, as above. To match `pk fit` exactly, pass `curves`: one list of
0-based row indices per curve, for the per-curve constraints.
