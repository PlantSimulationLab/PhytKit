# Photosynthesis models

The `photosynthesis` tool fits net CO₂ assimilation **A** as a function of
intercellular CO₂ **Ci**, absorbed light **Q** and leaf temperature **T**. Three
models are available:

| Model | `--model` | Use it for |
|---|---|---|
| Farquhar–von Caemmerer–Berry C3 | `fvcb` (default) | C3 leaves: A–Ci, light and temperature responses |
| von Caemmerer C4 | `fvcbC4` | C4 leaves |
| Empirical | `empiricalA` | A smooth response surface without a biochemical interpretation |

All three take **Q as absorbed light** (PPFD × leaf absorptance) and **T in
Kelvin**. When you fit a file, PhytKit prefers a `Qabs` column over `Q`, and
converts temperatures below 200 from °C to Kelvin.

## FvCB C3

Assimilation is the least of three carboxylation rates, less photorespiration
and day respiration:

\[
A = \left(1 - \frac{\Gamma^*}{C_c}\right) \min\{w_c,\ w_j,\ w_p\} - R_d
\]

| Rate | Limited by | Expression |
|---|---|---|
| \(w_c\) | Rubisco | \(\dfrac{V_{cmax}\,C_c}{C_c + K_c(1 + O/K_o)}\) |
| \(w_j\) | RuBP regeneration (electron transport) | \(\dfrac{J\,C_c}{4C_c + 8\Gamma^*}\) |
| \(w_p\) | Triose-phosphate utilization (TPU) | \(\dfrac{3\,TPU\,C_c}{C_c - (1 + 3\alpha_G)\Gamma^*}\) |

The minimum is smoothed, so the fitter's derivatives stay continuous where
the limitation changes: it is a hyperbolic minimum with curvature 0.999, the
co-limitation form of Collatz et al. (1991). Where two rates are equal, A is
about 3% below them; away from the transitions it equals the smaller rate.
The curvature is a fixed internal constant, not a parameter.

**Light.** Electron transport \(J\) follows a non-rectangular hyperbola in
absorbed light, with quantum yield `alpha` and curvature `theta`.

**Temperature.** `Vcmax`, `Jmax` and `TPU` follow a peaked Arrhenius response.
Each has a value at 25 °C (`Vcmax25`), an activation energy (`_dHa`), a
deactivation energy (`_dHd`) and an optimum temperature (`_Topt`, in Kelvin).
Give `_Topt` values in Kelvin. The model reads a value below 200 as °C, but
that only works when the parameter is held fixed: a free starting value below
200 is outside the 288–333 K bounds and is rejected. Γ*, Kc, Ko and Rd follow a plain Arrhenius
response.

The formulation matches PhoTorch. Rubisco kinetic constants (`GammaS25`,
`Kc25`, `Ko25` and their activation energies) and `O` are fixed by default.

### Mesophyll conductance

Rubisco sees the chloroplast CO₂ \(C_c\), not \(C_i\). During fitting PhytKit
uses the measured A to set

\[
C_c = C_i - A / g_m
\]

The reported `predicted` values, `rmse` and `r2` don't use the measured A:
they come from the model's own solution of A and \(C_c\) together, exactly as
`predict` computes them.

`gm` is a fitted parameter by default (bounds 0.01–5 mol m⁻² s⁻¹ bar⁻¹).
Mesophyll conductance is hard to estimate from gas exchange alone, so many
analyses fit an **apparent** model with \(C_c = C_i\) instead. `--ci-based`
does that: it holds gm at infinity, so \(C_c = C_i\) exactly, and holds
`alphaG` at 0, so the TPU limit is \(3\,TPU - R_d\):

```bash
pk fit photosynthesis examples/aci.csv --model fvcb --ci-based
```

The output then has `"ciBased": true`, and `gm` is written as `null` (JSON has no
infinity). This is also the form Helios's photosynthesis plugin uses unless you set
its mesophyll conductance. In Python, pass `ci_based=True`. Holding gm at a large
finite value instead (`--set gm=1000000 --fix gm,alphaG`) gives the same fit to
about one part in a million.

### Soft constraints

FvCB adds penalties that steer the fit toward biologically sensible
limitation patterns (from PhoTorch). They matter most when a curve doesn't
clearly show every limitation.

| Key | Encourages |
|---|---|
| `ap_nonneg` | A non-negative TPU-limited rate at every point |
| `aj_le_ac_mid` | Light limitation at or below Rubisco limitation at Ci 500–700 |
| `rubisco_region` | A Rubisco-limited stretch at low Ci |
| `ap_above_cross` | No TPU limitation where Rubisco and light limitation cross |
| `ap_limits_high` | TPU limitation at the highest Ci (off by default) |
| `vcmax_window` | `Vcmax25` between 20 and 130 (off by default) |
| `alphag_zero` | `alphaG` near zero |
| `vj_ratio` | `Jmax25`/`Vcmax25` between 1.6 and 2.0 |

The four marked **per curve** in [Models and parameters](../reference/models.md#fvcb)
are applied to each curve separately when the file has a `CurveID` column, and
to all rows together otherwise. Turn one off with `--no-constraint <key>`,
all of them with `--constraints none`, or change a weight with
`--weight <key>=<w>`.

`ap_limits_high` and `vcmax_window` are off by default: on curves without a
TPU plateau, `ap_limits_high` forces a TPU limit anyway, and `vcmax_window`
pulls `Vcmax25` of high-capacity leaves down toward 130. Turn one on with
`--constraint <key>` (in Python, list it in `constraints`).

## von Caemmerer C4

The C4 model follows von Caemmerer (2021), as implemented in PhoTorch and
Helios. Defaults are for *Setaria viridis*.

- **Mesophyll CO₂:** \(C_m = C_i - A/g_m\).
- **PEP carboxylation:** \(V_p = \min\big(C_m V_{pmax}/(C_m + K_p),\ V_{pr}\big)\).
- **Enzyme-limited rate \(A_c\)** and **electron-transport-limited rate \(A_j\):** each the lower root of a quadratic.
- **Assimilation:** \(A = \min(A_c, A_j)\). The model has no TPU limitation.

## Empirical model

\[
A = A_{max}(T)\,\frac{Q}{Q + K_q(T)}\cdot\frac{C_c - \Gamma^*(T)}{C_c + K_{co}(T)} - R_d(T),
\qquad C_c = C_i - A/g_m
\]

A light-saturation term times a CO₂-saturation term, with temperature
responses on each coefficient. The CO₂ term uses the chloroplast CO₂ \(C_c\),
solved together with A through the mesophyll conductance `gm`, which is
fitted by default (0.01–99). With `gm` at 99 or more, \(C_c = C_i\). It has many parameters for what A–Ci data alone
can determine, so fits often reach equally good results with different
parameter values. See
[How fitting works](fitting.md#parameters-the-data-cant-pin-down).

## Next

- [Fit an A–Ci curve](../workflows/fit-aci-curve.md)
- [Rescale curves to survey points](../workflows/rescale-to-survey.md)
- [Models and parameters](../reference/models.md#fvcb)
