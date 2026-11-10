# How fitting works

Every `pk fit` and `fit_arrays` call uses the same optimizer: a bounded
**Levenberg–Marquardt** least-squares fit. It finds the parameter values that
minimize the sum of squared differences between the model and your
measurements, the same objective as MATLAB's `lsqcurvefit`.

## The optimizer

At each iteration the fitter estimates how the residuals change with each
parameter (the **Jacobian**), then takes a step that blends Gauss–Newton (fast
near a minimum) with gradient descent (safe far from one). A damping factor
sets the blend. It adapts by comparing how much each step actually improved
the fit with how much it was predicted to. This is Nielsen's update, from
Madsen, Nielsen & Tingleff, *Methods for Non-Linear Least Squares Problems*.

- **Scaled parameters.** Internally, each parameter is divided by the size of
  its starting value, so parameters of very different magnitudes (a `Vcmax25`
  of 100 next to an `alpha` of 0.3) are treated evenly.
- **Bounds.** Each parameter has a lower and upper bound, listed in
  [Models and parameters](../reference/models.md). A step that would cross a
  bound is clipped to it. A parameter sitting on a bound, where the fit would
  improve by pushing it further out, is held there while the other parameters
  keep moving. A starting value you give (`--set`, `init`) outside a fitted
  parameter's bounds is an error; a fixed parameter can be held at any value.
  Starting values that PhytKit sets from the data are clipped to the bounds.
- **Units of the data.** Internally the residuals are divided by the size of
  the measurements. That doesn't change the best fit, but it means the
  fitter's tolerances work the same whether conductance is given as 0.1 or as
  1e-7.
- **Derivatives.** Most models use finite differences: each parameter is
  nudged slightly and the change measured. The `lightResponse` model has exact
  derivatives.

## Free and fixed parameters

Each model has a default set of parameters to fit. Others, such as the
Rubisco kinetic constants in FvCB, are **fixed by default** at literature
values. You can change this:

| `pk fit` | Python | Effect |
|---|---|---|
| `--fit a,b` | `fit_params=["a", "b"]` | Fit exactly these (this replaces the default set, so list every parameter you want fitted) |
| `--fix a,b` | (leave out of `fit_params`) | Hold these at their starting value |
| `--set a=1.5` | `init={"a": 1.5}` | Starting value, or the value a fixed parameter is held at |

The data sets some starting values. For example, soil models start θs and θr
at the wettest and driest measured water contents, and BTA starts `Em` at
least as high as the largest measured conductance. In `pk fit`, soil models also try
several starting values for their air-entry and shape parameters, spread over
the measured range of ψ, and keep the fit with the lowest `rmse`. Values you
give with `--set` are never changed by this.

## Soft constraints

Some models add **penalties** to the objective that pull a fit away from
results that are physically implausible but would otherwise fit the data. An
example is FvCB's requirement that TPU limitation should only appear at high
Ci. Most penalties are zero when their condition holds and grow as it's
violated. A few, such as fvcbC4's `rd_tether`, are soft priors that pull a
parameter toward a target value. Each has a weight, and you can turn penalties off or reweight them. See
[Photosynthesis models](photosynthesis.md#soft-constraints).

`r2` and `rmse` measure only how well the model fits your data; they never
include penalties.

## When the fit stops

The fitter stops when one of these happens:

1. The gradient is effectively zero: the fit is at a minimum. Parameters held
   on a bound don't count, since the fit can't move them further.
2. The step becomes negligibly small.
3. No step in any direction improves the fit (the fit has **stalled**).
4. It reaches the iteration limit: 200 by default, set with `--max-iter`.

`converged` is `true` for the first two. For a stall it's `true` only if the
gradient is small relative to the residuals, which means the fit is at a
minimum to within floating-point precision; otherwise it's `false`. It's
`false` at the iteration limit, and whenever the model gives non-finite values
for the data. A fit that stops at the limit isn't finished. Its parameters
may still be moving, so raise `--max-iter` and fit again before you use them.

!!! note "Converged doesn't mean correct"
    `converged: true` means the fitter can't improve the fit from where it
    is. It doesn't mean the result is the best possible fit, or that every
    parameter is well determined. Check `r2`, `rmse` and the parameters
    themselves, and compare fits from different starting values (`--set`)
    when it matters.

## Parameters the data can't pin down

A model with many parameters can often fit a dataset almost equally well with
quite different parameter values. When that happens:

- Small changes, such as a slightly different starting value or one extra data
  point, can move the fit to a different combination with the same `rmse`.
- Parameters can end up at their bounds.

This is a property of the model and the data, not an error. It's most common
with the empirical photosynthesis model and with small datasets. To make
parameters better determined:

- **Fix what you know.** For example, `--fix gm` with a measured or assumed
  value, or keep the kinetic constants at their defaults.
- **Fit fewer parameters** with `--fit`.
- **Pool curves** that should share parameters into one fit by putting them
  in one file (for example with `pk concat`). For photosynthesis, a CurveID
  column keeps the per-curve constraints applied to each curve separately.
- **Measure over a wider range**, such as more temperatures or more light
  levels.

A parameter reported exactly at its bound (see
[Models and parameters](../reference/models.md)) is a sign the data doesn't
constrain it.
